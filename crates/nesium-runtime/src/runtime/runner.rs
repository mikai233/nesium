use nesium_rewind::RewindState;
use sha1::{Digest, Sha1};
use std::sync::atomic::Ordering;
use std::{
    path::PathBuf,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender, TryRecvError};
use nesium_core::state::SnapshotMeta;
use nesium_core::state::nes::NesSnapshot;
use nesium_core::{
    Nes,
    controller::Button,
    ppu::buffer::{FrameBuffer, SCREEN_SIZE},
};

use crate::audio::NesAudioPlayer;

use super::{
    control::ControlMessage,
    state::RuntimeState,
    types::{AudioMode, NTSC_FPS_EXACT, RuntimeError, RuntimeNotification},
    util::button_bit,
};

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

pub(crate) struct Runner {
    nes: Nes,
    audio: Option<NesAudioPlayer>,
    ctrl_rx: Receiver<ControlMessage>,
    state: Arc<RuntimeState>,
    next_frame_deadline: Instant,
    frame_duration: Duration,
    integer_fps_target: Option<u32>,
    turbo_prev_masks: [u8; 4],
    turbo_start_frame: [[u64; 8]; 4],
    rewind: RewindState,
}

impl Runner {
    pub(crate) fn new(
        audio_mode: AudioMode,
        ctrl_rx: Receiver<ControlMessage>,
        event_tx: Sender<RuntimeNotification>,
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
                    let _ = event_tx.send(RuntimeNotification::AudioInitFailed {
                        error: e.to_string(),
                    });
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
            state,
            next_frame_deadline: Instant::now(),
            frame_duration: FRAME_DURATION_NTSC,
            integer_fps_target: None,
            turbo_prev_masks: [0; 4],
            turbo_start_frame: [[0; 8]; 4],
            rewind: RewindState::new(),
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
                    println!("rewinding");
                    self.rewind_one_frame();
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
            ControlMessage::LoadRom(path, reply) => {
                match std::fs::read(&path) {
                    Ok(bytes) => {
                        let mut hasher = Sha1::new();
                        hasher.update(&bytes);
                        let hash: [u8; 20] = hasher.finalize().into();
                        // Pad to 32 bytes for the internal representation
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
                                let _ =
                                    reply.send(Err(RuntimeError::LoadRomFailed { path, error }));
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
            ControlMessage::Reset(kind, reply) => {
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
            ControlMessage::Eject(reply) => {
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
            ControlMessage::SetAudioConfig(cfg, reply) => {
                self.nes.set_audio_bus_config(cfg);
                let _ = reply.send(Ok(()));
            }
            ControlMessage::SetFrameReadyCallback(cb, user_data, reply) => {
                self.nes.set_frame_ready_callback(cb, user_data);
                let _ = reply.send(Ok(()));
            }
            ControlMessage::SetIntegerFpsTarget(fps, reply) => {
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
            ControlMessage::SetPaletteKind(kind, reply) => {
                self.nes.set_palette(kind.palette());
                let _ = reply.send(Ok(()));
            }
            ControlMessage::SetPalette(palette, reply) => {
                self.nes.set_palette(palette);
                let _ = reply.send(Ok(()));
            }
            ControlMessage::SaveState(path, reply) => match self.nes.get_cartridge() {
                None => {
                    let _ = reply.send(Err(RuntimeError::SaveStateFailed {
                        path,
                        error: "no cartridge loaded".to_string(),
                    }));
                }
                Some(cart) => {
                    let rom_hash = *self.state.rom_hash.lock().unwrap();
                    let header = cart.header();
                    let mapper = Some((header.mapper(), header.submapper()));

                    let meta = SnapshotMeta {
                        tick: self.nes.master_clock(),
                        rom_hash,
                        mapper,
                        ..Default::default()
                    };

                    match self.nes.save_snapshot(meta) {
                        Ok(snap) => match snap.to_postcard_bytes() {
                            Ok(bytes) => match std::fs::write(&path, bytes) {
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
                            Err(e) => {
                                let _ = reply.send(Err(RuntimeError::SaveStateFailed {
                                    path,
                                    error: e.to_string(),
                                }));
                            }
                        },
                        Err(e) => {
                            let _ = reply.send(Err(RuntimeError::SaveStateFailed {
                                path,
                                error: format!("{:?}", e),
                            }));
                        }
                    }
                }
            },
            ControlMessage::LoadState(path, reply) => {
                match self.nes.get_cartridge() {
                    Some(cartridge) => {
                        match std::fs::read(&path) {
                            Ok(bytes) => {
                                match NesSnapshot::from_postcard_bytes(&bytes) {
                                    Ok(snap) => {
                                        // Validate ROM Mapper
                                        if let Some((mapper, submapper)) = snap.meta.mapper
                                            && (mapper != cartridge.header().mapper()
                                                || submapper != cartridge.header().submapper())
                                        {
                                            let _ = reply.send(Err(RuntimeError::LoadStateFailed {
                                                path,
                                                error: "ROM mappper mismatch: this save belongs to a different game".to_string(),
                                            }));
                                            return false;
                                        }
                                        // Validate ROM Hash
                                        if let Some(expected_hash) = snap.meta.rom_hash {
                                            let current_hash = *self.state.rom_hash.lock().unwrap();
                                            if Some(expected_hash) != current_hash {
                                                let _ = reply.send(Err(RuntimeError::LoadStateFailed {
                                                path,
                                                error: "ROM hash mismatch: this save belongs to a different game".to_string(),
                                            }));
                                                return false;
                                            }
                                        }

                                        match self.nes.load_snapshot(&snap) {
                                            Ok(_) => {
                                                let _ = reply.send(Ok(()));
                                            }
                                            Err(e) => {
                                                let _ = reply.send(Err(
                                                    RuntimeError::LoadStateFailed {
                                                        path,
                                                        error: format!("{:?}", e),
                                                    },
                                                ));
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        let _ = reply.send(Err(RuntimeError::LoadStateFailed {
                                            path,
                                            error: e.to_string(),
                                        }));
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = reply.send(Err(RuntimeError::LoadStateFailed {
                                    path,
                                    error: e.to_string(),
                                }));
                            }
                        }
                    }
                    None => {
                        let _ = reply.send(Err(RuntimeError::LoadStateFailed {
                            path,
                            error: "no cartridge loaded".to_string(),
                        }));
                    }
                }
            }
            ControlMessage::SaveStateToMemory(reply) => match self.nes.get_cartridge() {
                None => {
                    let _ = reply.send(Err(RuntimeError::SaveStateFailed {
                        path: PathBuf::from("memory"),
                        error: "no cartridge loaded".to_string(),
                    }));
                }
                Some(cart) => {
                    let rom_hash = *self.state.rom_hash.lock().unwrap();
                    let header = cart.header();
                    let mapper = Some((header.mapper(), header.submapper()));

                    let meta = SnapshotMeta {
                        tick: self.nes.master_clock(),
                        rom_hash,
                        mapper,
                        ..Default::default()
                    };

                    match self.nes.save_snapshot(meta) {
                        Ok(snap) => match snap.to_postcard_bytes() {
                            Ok(bytes) => {
                                let _ = reply.send(Ok(bytes));
                            }
                            Err(e) => {
                                let _ = reply.send(Err(RuntimeError::SaveStateFailed {
                                    path: PathBuf::from("memory"),
                                    error: e.to_string(),
                                }));
                            }
                        },
                        Err(e) => {
                            let _ = reply.send(Err(RuntimeError::SaveStateFailed {
                                path: PathBuf::from("memory"),
                                error: format!("{:?}", e),
                            }));
                        }
                    }
                }
            },
            ControlMessage::LoadStateFromMemory(bytes, reply) => {
                match self.nes.get_cartridge() {
                    Some(cartridge) => {
                        match NesSnapshot::from_postcard_bytes(&bytes) {
                            Ok(snap) => {
                                // Validate ROM Mapper
                                if let Some((mapper, submapper)) = snap.meta.mapper
                                    && (mapper != cartridge.header().mapper()
                                        || submapper != cartridge.header().submapper())
                                {
                                    let _ = reply.send(Err(RuntimeError::LoadStateFailed {
                                        path: PathBuf::from("memory"),
                                        error: "ROM mappper mismatch: this save belongs to a different game".to_string(),
                                    }));
                                    return false;
                                }
                                // Validate ROM Hash
                                if let Some(expected_hash) = snap.meta.rom_hash {
                                    let current_hash = *self.state.rom_hash.lock().unwrap();
                                    if Some(expected_hash) != current_hash {
                                        let _ = reply.send(Err(RuntimeError::LoadStateFailed {
                                            path: PathBuf::from("memory"),
                                            error: "ROM hash mismatch".to_string(),
                                        }));
                                        return false;
                                    }
                                }

                                match self.nes.load_snapshot(&snap) {
                                    Ok(_) => {
                                        let _ = reply.send(Ok(()));
                                    }
                                    Err(e) => {
                                        let _ = reply.send(Err(RuntimeError::LoadStateFailed {
                                            path: PathBuf::from("memory"),
                                            error: format!("{:?}", e),
                                        }));
                                    }
                                }
                            }
                            Err(e) => {
                                let _ = reply.send(Err(RuntimeError::LoadStateFailed {
                                    path: PathBuf::from("memory"),
                                    error: e.to_string(),
                                }));
                            }
                        }
                    }
                    None => {
                        let _ = reply.send(Err(RuntimeError::LoadStateFailed {
                            path: PathBuf::from("memory"),
                            error: "no cartridge loaded".to_string(),
                        }));
                    }
                }
            }
            ControlMessage::SetRewinding(rewinding, reply) => {
                self.state.rewinding.store(rewinding, Ordering::Release);
                let _ = reply.send(Ok(()));
            }
        }

        false
    }

    fn rewind_one_frame(&mut self) {
        if let Some((snapshot, indices)) = self.rewind.rewind_one_frame()
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
        if self.state.rewind_enabled.load(Ordering::Acquire) {
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

        let frame = self.state.frame_seq.load(Ordering::Relaxed);
        let turbo_on_frames = self.state.turbo_on_frames.load(Ordering::Acquire).max(1) as u64;
        let turbo_off_frames = self.state.turbo_off_frames.load(Ordering::Acquire).max(1) as u64;
        let turbo_cycle = turbo_on_frames + turbo_off_frames;
        let buttons = [
            Button::A,
            Button::B,
            Button::Select,
            Button::Start,
            Button::Up,
            Button::Down,
            Button::Left,
            Button::Right,
        ];

        // Sync Inputs (atomic bitmasks, no channel).
        for pad in 0..4 {
            let mask = self.state.pad_masks[pad].load(Ordering::Acquire);
            if mask != 0 {
                self.state.rewinding.store(false, Ordering::Release);
            }
            let turbo_mask = self.state.turbo_masks[pad].load(Ordering::Acquire);
            let prev_turbo_mask = self.turbo_prev_masks[pad];
            let rising = turbo_mask & !prev_turbo_mask;
            if rising != 0 {
                for button in buttons {
                    let bit = button_bit(button) as usize;
                    let flag = 1u8 << bit;
                    if (rising & flag) != 0 {
                        // Anchor turbo to the moment the turbo bit is first enabled so the first
                        // press is immediate instead of depending on a global frame phase.
                        self.turbo_start_frame[pad][bit] = frame;
                    }
                }
            }
            self.turbo_prev_masks[pad] = turbo_mask;

            for button in buttons {
                let bit = 1u8 << button_bit(button);
                let normal_pressed = (mask & bit) != 0;
                let turbo_pressed = (turbo_mask & bit) != 0;
                let turbo_on = if turbo_pressed {
                    let bit_idx = button_bit(button) as usize;
                    let start = self.turbo_start_frame[pad][bit_idx];
                    let rel = frame.wrapping_sub(start);
                    let phase = rel % turbo_cycle;
                    phase < turbo_on_frames
                } else {
                    false
                };
                let pressed = normal_pressed || turbo_on;
                self.nes.set_button(pad, button, pressed);
            }
        }

        let samples = self.nes.run_frame(self.audio.is_some());
        if let Some(audio) = &mut self.audio
            && !samples.is_empty()
        {
            audio.push_samples(&samples);
        }

        self.state.frame_seq.fetch_add(1, Ordering::Relaxed);
    }
}
