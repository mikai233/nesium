use nesium_support::rewind::RewindState;
use nesium_support::tas::{FrameFlags, InputFrame};
use sha1::{Digest, Sha1};
use std::sync::atomic::Ordering;
use std::{
    path::PathBuf,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use super::{
    control::{ControlMessage, ControlReplySender},
    pubsub::RuntimePubSub,
    state::RuntimeState,
    types::{
        AudioMode, CpuDebugState, DebugState, EventTopic, NTSC_FPS_EXACT, NotificationEvent,
        PpuDebugState, RuntimeError,
    },
    util::button_bit,
};
use crossbeam_channel::{Receiver, RecvTimeoutError, Sender, TryRecvError};
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use nesium_core::state::SnapshotMeta;
use nesium_core::state::nes::NesSnapshot;
use nesium_core::{
    Nes,
    audio::bus::AudioBusConfig,
    controller::Button,
    ppu::buffer::{FrameBuffer, FrameReadyCallback, SCREEN_SIZE},
    ppu::palette::{Palette, PaletteKind},
    reset_kind::ResetKind,
};
use std::ffi::c_void;

use crate::audio::NesAudioPlayer;

enum WaitOutcome {
    /// Runtime thread should exit (channel disconnected or Stop received).
    Exit,
    /// A control message was received and handled; caller should re-check state/deadlines.
    ControlHandled,
    /// The target deadline has been reached.
    DeadlineReached,
}

// NTSC: ~60.0988 Hz
const FRAME_DURATION_NTSC: Duration = Duration::from_nanos(16_639_263);
const FRAME_DURATION_60HZ: Duration = Duration::from_nanos(16_666_667);
// Hybrid wait tuning:
// - Sleep in small chunks until we're close to the deadline.
// - Spin for the final window for tighter frame pacing.
const MAX_SLEEP_CHUNK: Duration = Duration::from_millis(4);
const SPIN_THRESHOLD: Duration = Duration::from_micros(300);
const SPIN_YIELD_EVERY: u32 = 512;
// Allow frames to start slightly early to reduce the chance of missing the deadline.
const FRAME_LEAD: Duration = Duration::from_micros(50);

/// The canonical NES controller button order used by the runtime.
///
/// The bit positions are defined by `button_bit()` and must stay in sync with the
/// TAS/movie parsers and the UI input layer.
const BUTTONS: [Button; 8] = [
    Button::A,
    Button::B,
    Button::Select,
    Button::Start,
    Button::Up,
    Button::Down,
    Button::Left,
    Button::Right,
];

pub(crate) struct Runner {
    nes: Nes,
    audio: Option<NesAudioPlayer>,
    ctrl_rx: Receiver<ControlMessage>,
    pubsub: RuntimePubSub,
    state: Arc<RuntimeState>,
    next_frame_deadline: Instant,
    frame_duration: Duration,
    integer_fps_target: Option<u32>,
    turbo_prev_masks: [u8; 4],
    turbo_start_frame: [[u64; 8]; 4],
    rewind: RewindState,
    movie: Option<nesium_support::tas::Movie>,
    movie_frame: usize,
}

impl Runner {
    pub(crate) fn new(
        audio_mode: AudioMode,
        ctrl_rx: Receiver<ControlMessage>,
        mut pubsub: RuntimePubSub,
        framebuffer: FrameBuffer,
        state: Arc<RuntimeState>,
    ) -> Self {
        let (audio, runtime_sample_rate) = match audio_mode {
            AudioMode::Disabled => (None, 48_000),
            AudioMode::Auto => match NesAudioPlayer::new() {
                Ok(player) => {
                    let sr = player.sample_rate();
                    (Some(player), sr)
                }
                Err(e) => {
                    pubsub.broadcast(
                        EventTopic::Notification,
                        Box::new(NotificationEvent::AudioInitFailed {
                            error: e.to_string(),
                        }),
                    );
                    (None, 48_000)
                }
            },
        };

        let nes = Nes::builder()
            .framebuffer(framebuffer)
            .sample_rate(runtime_sample_rate)
            .build();

        Self {
            nes,
            audio,
            ctrl_rx,
            pubsub,
            state,
            next_frame_deadline: Instant::now(),
            frame_duration: FRAME_DURATION_NTSC,
            integer_fps_target: None,
            turbo_prev_masks: [0; 4],
            turbo_start_frame: [[0; 8]; 4],
            rewind: RewindState::new(),
            movie: None,
            movie_frame: 0,
        }
    }

    pub(crate) fn run(&mut self) {
        let mut last_paused = self.state.paused.load(Ordering::Acquire);
        let mut last_rewinding = self.state.rewinding.load(Ordering::Acquire);

        loop {
            while let Ok(msg) = self.ctrl_rx.try_recv() {
                if self.handle_control(msg) {
                    return;
                }
            }

            let paused = self.state.paused.load(Ordering::Acquire);
            let rewinding = self.state.rewinding.load(Ordering::Acquire);

            if (paused != last_paused && !paused) || (rewinding != last_rewinding) {
                self.next_frame_deadline = Instant::now();
            }
            last_paused = paused;
            last_rewinding = rewinding;

            if self.nes.get_cartridge().is_none() || (paused && !rewinding) {
                match self.ctrl_rx.recv_timeout(Duration::from_millis(10)) {
                    Ok(msg) => {
                        if self.handle_control(msg) {
                            return;
                        }
                    }
                    Err(RecvTimeoutError::Timeout) => {}
                    Err(RecvTimeoutError::Disconnected) => return,
                }
                continue;
            }

            match self.wait_until_next_deadline() {
                WaitOutcome::Exit => return,
                WaitOutcome::ControlHandled => continue,
                WaitOutcome::DeadlineReached => {}
            }

            let mut frames_run: u32 = 0;
            while (rewinding || !paused)
                && Instant::now() + FRAME_LEAD >= self.next_frame_deadline
                && frames_run < 3
            {
                if self.state.rewinding.load(Ordering::Acquire) {
                    self.rewind_frame();
                } else {
                    self.step_frame();
                }
                self.next_frame_deadline += self.frame_duration;
                frames_run += 1;
            }

            let now = Instant::now();
            if now > self.next_frame_deadline
                && now.duration_since(self.next_frame_deadline) > self.frame_duration * 2
            {
                self.next_frame_deadline = now;
            }
        }
    }

    fn wait_until_next_deadline(&mut self) -> WaitOutcome {
        loop {
            let target = self
                .next_frame_deadline
                .checked_sub(FRAME_LEAD)
                .unwrap_or(self.next_frame_deadline);

            let now = Instant::now();
            if now >= target {
                return WaitOutcome::DeadlineReached;
            }

            let remaining = target - now;

            // Coarse phase: sleep in chunks while still far from the deadline,
            // but always keep a final spin window.
            if remaining > SPIN_THRESHOLD {
                let sleep_for = (remaining - SPIN_THRESHOLD).min(MAX_SLEEP_CHUNK);
                match self.ctrl_rx.recv_timeout(sleep_for) {
                    Ok(msg) => {
                        if self.handle_control(msg) {
                            return WaitOutcome::Exit;
                        }
                        return WaitOutcome::ControlHandled;
                    }
                    Err(RecvTimeoutError::Timeout) => continue,
                    Err(RecvTimeoutError::Disconnected) => return WaitOutcome::Exit,
                }
            }

            // Fine phase: spin until the deadline. We still poll control messages
            // to keep the runtime responsive.
            let mut spins: u32 = 0;
            while Instant::now() < target {
                match self.ctrl_rx.try_recv() {
                    Ok(msg) => {
                        if self.handle_control(msg) {
                            return WaitOutcome::Exit;
                        }
                        return WaitOutcome::ControlHandled;
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => return WaitOutcome::Exit,
                }

                std::hint::spin_loop();
                spins = spins.wrapping_add(1);
                if spins.is_multiple_of(SPIN_YIELD_EVERY) {
                    thread::yield_now();
                }
            }

            return WaitOutcome::DeadlineReached;
        }
    }

    fn handle_control(&mut self, msg: ControlMessage) -> bool {
        match msg {
            ControlMessage::Stop => return true,
            ControlMessage::LoadRom(path, reply) => self.handle_load_rom(path, reply),
            ControlMessage::Reset(kind, reply) => self.handle_reset(kind, reply),
            ControlMessage::Eject(reply) => self.handle_eject(reply),
            ControlMessage::SetAudioConfig(cfg, reply) => self.handle_set_audio_config(cfg, reply),
            ControlMessage::SetFrameReadyCallback(cb, user_data, reply) => {
                self.handle_set_frame_ready_callback(cb, user_data, reply)
            }
            ControlMessage::SetIntegerFpsTarget(fps, reply) => {
                self.handle_set_integer_fps_target(fps, reply)
            }
            ControlMessage::SetPaletteKind(kind, reply) => {
                self.handle_set_palette_kind(kind, reply)
            }
            ControlMessage::SetPalette(palette, reply) => self.handle_set_palette(palette, reply),
            ControlMessage::SaveState(path, reply) => self.handle_save_state(path, reply),
            ControlMessage::LoadState(path, reply) => self.handle_load_state(path, reply),
            ControlMessage::SaveStateToMemory(reply) => self.handle_save_state_to_memory(reply),
            ControlMessage::LoadStateFromMemory(bytes, reply) => {
                self.handle_load_state_from_memory(bytes, reply)
            }
            ControlMessage::SetRewinding(rewinding, reply) => {
                self.handle_set_rewinding(rewinding, reply)
            }
            ControlMessage::LoadMovie(movie, reply) => self.handle_load_movie(movie, reply),
            ControlMessage::SubscribeEvent(topic, sender, reply) => {
                self.pubsub.subscribe(topic, sender);
                let _ = reply.send(Ok(()));
            }
            ControlMessage::UnsubscribeEvent(topic, reply) => {
                self.pubsub.unsubscribe(topic);
                let _ = reply.send(Ok(()));
            }
        }

        false
    }

    fn rewind_frame(&mut self) {
        if let Some((snapshot, indices)) = self.rewind.rewind_frame()
            && self.nes.load_snapshot(&snapshot).is_ok()
        {
            // Copy the palette array to avoid borrow checker issues.
            let palette = *self.nes.ppu.palette().as_colors();
            // Refresh the framebuffer with the saved palette indices.
            //
            // The rewind history stores the canonical index plane (one byte per pixel).
            // We write it into the back index plane and present it as a full frame.
            let fb = self.nes.ppu.framebuffer_mut();
            let back_indices = fb.write();

            if back_indices.len() != indices.len() {
                // A size mismatch indicates an incompatible framebuffer configuration.
                // Keep the current frame rather than presenting corrupted data.
                return;
            }

            back_indices.copy_from_slice(&indices);

            // Increment `frame_seq` BEFORE present so the Android signal pipe carries the new sequence.
            self.state.frame_seq.fetch_add(1, Ordering::Release);

            // Present converts indices to packed pixels once and swaps the presented plane.
            fb.present(&palette);

            if let Some(audio) = &self.audio {
                audio.clear();
            }
        }
    }

    fn step_frame(&mut self) {
        self.maybe_capture_rewind_history();

        // If a TAS movie is active, we treat it as the sole input source.
        // User input and turbo are ignored during playback.
        let movie_frame = self.current_movie_frame();
        if let Some(frame) = movie_frame.as_ref() {
            // TAS playback and interactive rewind are mutually exclusive.
            self.state.rewinding.store(false, Ordering::Release);
            // Apply reset markers before sampling inputs for the frame.
            self.apply_movie_resets(frame);
        }

        let frame_seq = self.state.frame_seq.load(Ordering::Relaxed);
        let turbo_on_frames = self.state.turbo_on_frames.load(Ordering::Acquire).max(1) as u64;
        let turbo_off_frames = self.state.turbo_off_frames.load(Ordering::Acquire).max(1) as u64;
        let turbo_cycle = turbo_on_frames + turbo_off_frames;

        let mut any_input = false;

        // Sync inputs (atomic bitmasks, no channel).
        for pad in 0..4 {
            let mask = if let Some(frame) = movie_frame.as_ref() {
                // TAS: feed the recorded button mask as-is.
                frame.ports[pad]
            } else {
                // Live input: base mask plus optional turbo overlay.
                let base = self.state.pad_masks[pad].load(Ordering::Acquire);
                self.apply_turbo_to_mask(pad, base, frame_seq, turbo_on_frames, turbo_cycle)
            };

            if mask != 0 {
                any_input = true;
            }

            self.apply_pad_mask(pad, mask);
        }

        // Any input (including TAS input) should cancel rewind.
        // This prevents the runtime from getting stuck in a rewind-only loop.
        if any_input {
            self.state.rewinding.store(false, Ordering::Release);
        }

        let samples = self.nes.run_frame(self.audio.is_some());
        if let Some(audio) = &mut self.audio
            && !samples.is_empty()
        {
            audio.push_samples(&samples);
        }

        self.state.frame_seq.fetch_add(1, Ordering::Relaxed);

        // Advance the TAS timeline AFTER the frame has been executed.
        if movie_frame.is_some() {
            self.advance_movie_after_frame();
        }

        // Broadcast debug state if there's a subscriber.
        self.maybe_broadcast_debug_state();
        self.maybe_broadcast_tilemap_state();
    }

    /// Captures rewind history (snapshot + render index plane) when enabled.
    fn maybe_capture_rewind_history(&mut self) {
        if !self.state.rewind_enabled.load(Ordering::Acquire) {
            return;
        }

        let meta = SnapshotMeta {
            tick: self.nes.master_clock(),
            ..Default::default()
        };

        if let Ok(snap) = self.nes.save_snapshot(meta) {
            let mut indices = vec![0u8; SCREEN_SIZE];
            self.nes.copy_render_index_buffer(&mut indices);
            let cap = self.state.rewind_capacity.load(Ordering::Acquire) as usize;
            self.rewind.push_frame(&snap, indices, cap);
        }
    }

    /// Returns the current TAS frame if playback is active.
    ///
    /// If the movie is exhausted (or otherwise inconsistent), playback is stopped.
    fn current_movie_frame(&mut self) -> Option<InputFrame> {
        let movie = self.movie.as_ref()?;
        let len = movie.frames.len();
        if self.movie_frame >= len {
            // Do not silently fall back to live input when the movie ends.
            // A finished movie is an explicit state transition.
            self.movie = None;
            println!("[Runner] TAS playback finished");
            return None;
        }

        // Safe: `movie_frame < len`.
        self.movie
            .as_ref()
            .and_then(|m| m.frames.get(self.movie_frame))
            .copied()
    }

    /// Applies reset flags embedded in a TAS frame.
    ///
    /// These markers are processed before input is sampled for the frame.
    fn apply_movie_resets(&mut self, frame: &InputFrame) {
        let flags = frame.commands;
        if flags.contains(FrameFlags::POWER) {
            self.nes.reset(ResetKind::PowerOn);
        } else if flags.contains(FrameFlags::RESET) {
            self.nes.reset(ResetKind::Soft);
        }
    }

    /// Overlays turbo behavior onto a live input mask.
    fn apply_turbo_to_mask(
        &mut self,
        pad: usize,
        base_mask: u8,
        frame_seq: u64,
        turbo_on_frames: u64,
        turbo_cycle: u64,
    ) -> u8 {
        let turbo_mask = self.state.turbo_masks[pad].load(Ordering::Acquire);
        let prev_turbo_mask = self.turbo_prev_masks[pad];
        let rising = turbo_mask & !prev_turbo_mask;

        if rising != 0 {
            for button in BUTTONS {
                let bit_idx = button_bit(button) as usize;
                let flag = 1u8 << bit_idx;
                if (rising & flag) != 0 {
                    // Anchor turbo to the moment the turbo bit is first enabled so the first
                    // press is immediate instead of depending on a global frame phase.
                    self.turbo_start_frame[pad][bit_idx] = frame_seq;
                }
            }
        }

        self.turbo_prev_masks[pad] = turbo_mask;

        let mut mask = base_mask;
        for button in BUTTONS {
            let bit = 1u8 << button_bit(button);
            if (turbo_mask & bit) == 0 {
                continue;
            }

            let bit_idx = button_bit(button) as usize;
            let start = self.turbo_start_frame[pad][bit_idx];
            let rel = frame_seq.wrapping_sub(start);
            let phase = rel % turbo_cycle;
            if phase < turbo_on_frames {
                mask |= bit;
            }
        }

        mask
    }

    /// Applies a resolved button mask to a controller port.
    fn apply_pad_mask(&mut self, pad: usize, mask: u8) {
        for button in BUTTONS {
            let bit = 1u8 << button_bit(button);
            let pressed = (mask & bit) != 0;
            self.nes.set_button(pad, button, pressed);
        }
    }

    /// Advances TAS playback by one frame and stops playback at end-of-movie.
    fn advance_movie_after_frame(&mut self) {
        let Some(movie) = self.movie.as_ref() else {
            return;
        };

        let len = movie.frames.len();
        self.movie_frame = self.movie_frame.saturating_add(1);
        if self.movie_frame >= len {
            self.movie = None;
            println!("[Runner] TAS playback finished");
        }
    }

    /// Broadcasts debug state to subscribers if someone is listening.
    fn maybe_broadcast_debug_state(&mut self) {
        if !self.pubsub.has_subscriber(EventTopic::DebugState) {
            return;
        }

        let cpu_snap = self.nes.debug_state();
        let (scanline, cycle, frame, ctrl, mask, status, oam_addr, vram_addr, temp_addr, fine_x) =
            self.nes.ppu_debug_state();

        let debug = DebugState {
            cpu: CpuDebugState {
                pc: cpu_snap.pc,
                a: cpu_snap.a,
                x: cpu_snap.x,
                y: cpu_snap.y,
                sp: cpu_snap.s,
                status: cpu_snap.p,
                cycle: self.nes.master_clock() / 3, // Approximate CPU cycles (NTSC)
            },
            ppu: PpuDebugState {
                scanline,
                cycle,
                frame,
                ctrl,
                mask,
                status,
                oam_addr,
                vram_addr,
                temp_addr,
                fine_x,
            },
        };

        self.pubsub
            .broadcast(EventTopic::DebugState, Box::new(debug));
    }

    /// Broadcasts tilemap state to subscribers if someone is listening.
    fn maybe_broadcast_tilemap_state(&mut self) {
        if self.pubsub.has_subscriber(EventTopic::Tilemap) {
            let (vram, palette, chr, mirroring, bg_pattern_base) = self.nes.debug_tilemap_data();

            // Convert current NES palette (RGB) to BGRA for aux texture rendering.
            let nes_palette = self.nes.palette();
            let mut bgra_palette = [[0u8; 4]; 64];
            for (i, color) in nes_palette.as_colors().iter().enumerate() {
                bgra_palette[i] = [color.b, color.g, color.r, 0xFF];
            }

            let tilemap = crate::runtime::types::TilemapState {
                vram,
                palette,
                chr,
                mirroring,
                bgra_palette,
                bg_pattern_base,
            };
            self.pubsub
                .broadcast(EventTopic::Tilemap, Box::new(tilemap));
        }
    }

    /// Loads a ROM from the specified path, calculates its SHA-1 hash, and initializes the NES state.
    fn handle_load_rom(&mut self, path: PathBuf, reply: ControlReplySender) {
        match std::fs::read(&path) {
            Ok(bytes) => {
                let mut hasher = Sha1::new();
                hasher.update(&bytes);
                let hash: [u8; 20] = hasher.finalize().into();
                // Pad to 32 bytes for the internal representation.
                let mut full_hash = [0u8; 32];
                full_hash[..20].copy_from_slice(&hash);

                match self.nes.load_cartridge_from_file(&path) {
                    Ok(_) => {
                        *self.state.rom_hash.lock().unwrap() = Some(full_hash);
                        self.state.paused.store(false, Ordering::Release);
                        self.next_frame_deadline = Instant::now();
                        if let Some(audio) = &self.audio {
                            audio.clear();
                        }
                        self.rewind.clear();
                        self.state.rewinding.store(false, Ordering::Release);
                        let _ = reply.send(Ok(()));
                    }
                    Err(e) => {
                        *self.state.rom_hash.lock().unwrap() = None;
                        let error = e.to_string();
                        let _ = reply.send(Err(RuntimeError::LoadRomFailed { path, error }));
                    }
                }
            }
            Err(e) => {
                *self.state.rom_hash.lock().unwrap() = None;
                let error = e.to_string();
                let _ = reply.send(Err(RuntimeError::LoadRomFailed { path, error }));
            }
        }
    }

    /// Resets the NES console (warm or cold reset) and clears transient states.
    fn handle_reset(&mut self, kind: ResetKind, reply: ControlReplySender) {
        if self.nes.get_cartridge().is_some() {
            self.nes.reset(kind);
            for mask in &self.state.pad_masks {
                mask.store(0, Ordering::Release);
            }
            for mask in &self.state.turbo_masks {
                mask.store(0, Ordering::Release);
            }
            if let Some(audio) = &self.audio {
                audio.clear();
            }
            self.rewind.clear();
            self.state.rewinding.store(false, Ordering::Release);
            self.state.paused.store(false, Ordering::Release);
            self.next_frame_deadline = Instant::now();
        }
        let _ = reply.send(Ok(()));
    }

    /// Ejects the current cartridge and clears associated runtime states.
    fn handle_eject(&mut self, reply: ControlReplySender) {
        self.nes.eject_cartridge();
        for mask in &self.state.pad_masks {
            mask.store(0, Ordering::Release);
        }
        for mask in &self.state.turbo_masks {
            mask.store(0, Ordering::Release);
        }
        if let Some(audio) = &self.audio {
            audio.clear();
        }
        self.rewind.clear();
        self.state.rewinding.store(false, Ordering::Release);
        let _ = reply.send(Ok(()));
    }

    /// Updates the audio bus configuration.
    fn handle_set_audio_config(&mut self, cfg: AudioBusConfig, reply: ControlReplySender) {
        self.nes.set_audio_bus_config(cfg);
        let _ = reply.send(Ok(()));
    }

    /// Configures the frame-ready callback for video presentation.
    fn handle_set_frame_ready_callback(
        &mut self,
        cb: Option<FrameReadyCallback>,
        user_data: *mut c_void,
        reply: ControlReplySender,
    ) {
        self.nes.set_frame_ready_callback(cb, user_data);
        let _ = reply.send(Ok(()));
    }

    /// Configures the target FPS, allowing for integer 60Hz mode to reduce judder.
    fn handle_set_integer_fps_target(&mut self, fps: Option<u32>, reply: ControlReplySender) {
        self.integer_fps_target = fps;
        if fps == Some(60) {
            self.frame_duration = FRAME_DURATION_60HZ;
            self.nes.set_audio_integer_fps_scale(60.0 / NTSC_FPS_EXACT);
        } else {
            self.frame_duration = FRAME_DURATION_NTSC;
            self.nes.reset_audio_integer_fps_scale();
        }

        // Re-anchor to avoid a big catch-up burst right after toggling.
        self.next_frame_deadline = Instant::now();
        let _ = reply.send(Ok(()));
    }

    /// Sets the color palette kind.
    fn handle_set_palette_kind(&mut self, kind: PaletteKind, reply: ControlReplySender) {
        self.nes.set_palette(kind.palette());
        let _ = reply.send(Ok(()));
    }

    /// Sets a custom color palette.
    fn handle_set_palette(&mut self, palette: Palette, reply: ControlReplySender) {
        self.nes.set_palette(palette);
        let _ = reply.send(Ok(()));
    }

    /// Captures the NES state, compresses it with LZ4, and saves it to a file.
    fn handle_save_state(&mut self, path: PathBuf, reply: ControlReplySender) {
        match self.capture_compressed_snapshot() {
            Ok(compressed) => match std::fs::write(&path, compressed) {
                Ok(_) => {
                    let _ = reply.send(Ok(()));
                }
                Err(e) => {
                    let _ = reply.send(Err(RuntimeError::SaveStateFailed {
                        path,
                        error: e.to_string(),
                    }));
                }
            },
            Err(error) => {
                let _ = reply.send(Err(RuntimeError::SaveStateFailed { path, error }));
            }
        }
    }

    /// Reads a save state from a file, decompresses it, and restores the machine state.
    fn handle_load_state(&mut self, path: PathBuf, reply: ControlReplySender) {
        match std::fs::read(&path) {
            Ok(bytes) => match self.apply_compressed_snapshot(bytes) {
                Ok(_) => {
                    let _ = reply.send(Ok(()));
                }
                Err(error) => {
                    let _ = reply.send(Err(RuntimeError::LoadStateFailed { path, error }));
                }
            },
            Err(e) => {
                let _ = reply.send(Err(RuntimeError::LoadStateFailed {
                    path,
                    error: e.to_string(),
                }));
            }
        }
    }

    /// Captures the NES state, compresses it, and sends the bytes via the reply channel.
    fn handle_save_state_to_memory(&mut self, reply: Sender<Result<Vec<u8>, RuntimeError>>) {
        match self.capture_compressed_snapshot() {
            Ok(bytes) => {
                let _ = reply.send(Ok(bytes));
            }
            Err(error) => {
                let _ = reply.send(Err(RuntimeError::SaveStateFailed {
                    path: PathBuf::from("memory"),
                    error,
                }));
            }
        }
    }

    /// Restores the NES state from a byte buffer, with decompression.
    fn handle_load_state_from_memory(&mut self, bytes: Vec<u8>, reply: ControlReplySender) {
        match self.apply_compressed_snapshot(bytes) {
            Ok(_) => {
                let _ = reply.send(Ok(()));
            }
            Err(error) => {
                let _ = reply.send(Err(RuntimeError::LoadStateFailed {
                    path: PathBuf::from("memory"),
                    error,
                }));
            }
        }
    }

    /// Enables or disables real-time rewind support.
    fn handle_set_rewinding(&mut self, rewinding: bool, reply: ControlReplySender) {
        self.state.rewinding.store(rewinding, Ordering::Release);
        let _ = reply.send(Ok(()));
    }

    /// Loads a TAS movie and initializes playback by resetting the console.
    fn handle_load_movie(&mut self, movie: nesium_support::tas::Movie, reply: ControlReplySender) {
        if self.nes.get_cartridge().is_some() {
            // FCEUX logic: fully reload the game/power cycle before playing any movie.
            self.nes.reset(nesium_core::reset_kind::ResetKind::PowerOn);

            if let Some(savestate) = &movie.savestate {
                // If the movie contains an embedded savestate, apply it to initialize the system state.
                if let Err(e) = self.apply_compressed_snapshot(savestate.clone()) {
                    let _ = reply.send(Err(RuntimeError::LoadStateFailed {
                        path: PathBuf::from("movie_embedded"),
                        error: e,
                    }));
                    return;
                }
            }

            // Clear transient states.
            for mask in &self.state.pad_masks {
                mask.store(0, Ordering::Release);
            }
            if let Some(audio) = &self.audio {
                audio.clear();
            }
            self.rewind.clear();
            self.state.rewinding.store(false, Ordering::Release);
            self.state.paused.store(false, Ordering::Release);
            self.next_frame_deadline = Instant::now();

            self.movie = Some(movie);
            self.movie_frame = 0;
        }
        let _ = reply.send(Ok(()));
    }

    /// Captures the current NES state as a postcard-serialized, LZ4-compressed byte buffer.
    fn capture_compressed_snapshot(&mut self) -> Result<Vec<u8>, String> {
        let cart = self.nes.get_cartridge().ok_or("no cartridge loaded")?;
        let rom_hash = *self.state.rom_hash.lock().unwrap();
        let header = cart.header();
        let mapper = Some((header.mapper(), header.submapper()));

        let meta = SnapshotMeta {
            tick: self.nes.master_clock(),
            rom_hash,
            mapper,
            ..Default::default()
        };

        let snap = self
            .nes
            .save_snapshot(meta)
            .map_err(|e| format!("{:?}", e))?;
        let bytes = snap.to_postcard_bytes().map_err(|e| e.to_string())?;
        Ok(compress_prepend_size(&bytes))
    }

    /// Decompresses and applies a snapshot buffer to the NES instance, validating ROM compatibility.
    fn apply_compressed_snapshot(&mut self, bytes: Vec<u8>) -> Result<(), String> {
        let cartridge = self.nes.get_cartridge().ok_or("no cartridge loaded")?;
        let decoded = decompress_size_prepended(&bytes).unwrap_or(bytes);
        let snap = NesSnapshot::from_postcard_bytes(&decoded).map_err(|e| e.to_string())?;

        // Validate ROM Mapper.
        if let Some((mapper, submapper)) = snap.meta.mapper
            && (mapper != cartridge.header().mapper()
                || submapper != cartridge.header().submapper())
        {
            return Err("ROM mapper mismatch: this save belongs to a different game".to_string());
        }
        // Validate ROM Hash.
        if let Some(expected_hash) = snap.meta.rom_hash {
            let current_hash = *self.state.rom_hash.lock().unwrap();
            if Some(expected_hash) != current_hash {
                return Err("ROM hash mismatch: this save belongs to a different game".to_string());
            }
        }

        self.nes
            .load_snapshot(&snap)
            .map_err(|e| format!("{:?}", e))?;
        Ok(())
    }
}
