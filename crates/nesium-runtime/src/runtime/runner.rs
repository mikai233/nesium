use nesium_core::ppu::buffer::ColorFormat;
use nesium_netplay::{NetplayInputProvider, SnapshotBuffer, SyncMode};
use nesium_support::rewind::RewindState;
use nesium_support::tas::{self, FrameFlags, InputFrame};
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
        AudioMode, CpuDebugState, DebugState, EmulationStatus, EventTopic, NTSC_FPS_EXACT,
        NotificationEvent, PaletteState, PpuDebugState, RuntimeError, SpriteState, TileState,
        TileViewerLayout, TileViewerSource, TilemapState,
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
    ctrl_tx: Sender<ControlMessage>,
    pubsub: RuntimePubSub,
    state: Arc<RuntimeState>,
    next_frame_deadline: Instant,
    frame_duration: Duration,
    integer_fps_target: Option<u32>,
    turbo_prev_masks: [u8; 4],
    turbo_start_frame: [[u64; 8]; 4],
    rewind: RewindState,
    movie: Option<tas::Movie>,
    movie_frame: usize,
    netplay_input: Option<Arc<dyn NetplayInputProvider>>,
    netplay_active: bool,
    /// Snapshot buffer for rollback resimulation (Rollback mode only)
    netplay_snapshots: SnapshotBuffer,
}

impl Runner {
    pub(crate) fn new(
        audio_mode: AudioMode,
        ctrl_rx: Receiver<ControlMessage>,
        ctrl_tx: Sender<ControlMessage>,
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
            ctrl_tx,
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
            netplay_input: None,
            netplay_active: false,
            netplay_snapshots: SnapshotBuffer::default(),
        }
    }

    pub(crate) fn run(&mut self) {
        let mut last_paused = self.state.paused.load(Ordering::Acquire);
        let mut last_rewinding = self.state.rewinding.load(Ordering::Acquire);
        let mut last_fast_forwarding = self.state.fast_forwarding.load(Ordering::Acquire);
        let mut last_fast_forward_speed = self
            .state
            .fast_forward_speed_percent
            .load(Ordering::Acquire);

        loop {
            while let Ok(msg) = self.ctrl_rx.try_recv() {
                if self.handle_control(msg) {
                    return;
                }
            }

            let paused = self.state.paused.load(Ordering::Acquire);
            let rewinding = self.state.rewinding.load(Ordering::Acquire);
            let fast_forwarding = self.state.fast_forwarding.load(Ordering::Acquire);
            let fast_forward_speed = self
                .state
                .fast_forward_speed_percent
                .load(Ordering::Acquire);

            if (paused != last_paused && !paused)
                || (rewinding != last_rewinding)
                || (fast_forwarding != last_fast_forwarding)
                || (fast_forward_speed != last_fast_forward_speed)
            {
                self.next_frame_deadline = Instant::now();
            }

            if paused != last_paused
                || rewinding != last_rewinding
                || fast_forwarding != last_fast_forwarding
            {
                self.pubsub.broadcast(
                    EventTopic::EmulationStatus,
                    Box::new(EmulationStatus {
                        paused,
                        rewinding,
                        fast_forwarding,
                    }),
                );
            }

            last_paused = paused;
            last_rewinding = rewinding;
            last_fast_forwarding = fast_forwarding;
            last_fast_forward_speed = fast_forward_speed;

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

            // Run exactly one frame per iteration to avoid jitter from catch-up frames.
            if rewinding || !paused {
                self.step_frame();
                self.next_frame_deadline += self.current_frame_duration();
            }

            // If we've fallen behind, reset the deadline instead of trying to catch up.
            // EXCEPTION: If Netplay inputs are ready, we WANT to catch up.
            let now = Instant::now();
            let mut allow_catchup = false;

            if let Some(np) = &self.netplay_input {
                if np.should_fast_forward(self.state.frame_seq.load(Ordering::Acquire) as u32) {
                    allow_catchup = true;
                }
            }

            if now > self.next_frame_deadline && !allow_catchup {
                self.next_frame_deadline = now;
            }
        }
    }

    fn wait_until_next_deadline(&mut self) -> WaitOutcome {
        loop {
            // Netplay Fast-Forward: If we have inputs ready for the current frame, don't wait.
            // This allows catching up quickly after a state load or lag spike.
            if let Some(np) = &self.netplay_input {
                // frame_seq is incremented *after* run_frame, so we are checking the frame we are ABOUT to run.
                if np.should_fast_forward(self.state.frame_seq.load(Ordering::Acquire) as u32) {
                    return WaitOutcome::DeadlineReached;
                }
            }

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

    fn current_frame_duration(&self) -> Duration {
        let is_fast_forwarding = self.state.fast_forwarding.load(Ordering::Acquire);
        let is_rewinding = self.state.rewinding.load(Ordering::Acquire);

        if !is_fast_forwarding && !is_rewinding {
            return self.frame_duration;
        }

        let speed = if is_fast_forwarding {
            self.state
                .fast_forward_speed_percent
                .load(Ordering::Acquire)
        } else {
            self.state.rewind_speed_percent.load(Ordering::Acquire)
        }
        .clamp(100, 1000) as u128;

        let base = self.frame_duration.as_nanos();
        let scaled = (base.saturating_mul(100) / speed).max(1);
        Duration::from_nanos(scaled.min(u128::from(u64::MAX)) as u64)
    }

    fn handle_control(&mut self, msg: ControlMessage) -> bool {
        match msg {
            ControlMessage::Stop => return true,
            ControlMessage::LoadRom(path, reply) => self.handle_load_rom(path, reply),
            ControlMessage::LoadRomFromMemory(bytes, reply) => {
                self.handle_load_rom_from_memory(bytes, reply)
            }
            ControlMessage::Reset(kind, reply) => self.handle_reset(kind, reply),
            ControlMessage::PowerOff(reply) => self.handle_power_off(reply),
            ControlMessage::SetAudioConfig(cfg, reply) => self.handle_set_audio_config(cfg, reply),
            ControlMessage::SetFrameReadyCallback(cb, user_data, reply) => {
                self.handle_set_frame_ready_callback(cb, user_data, reply)
            }
            ControlMessage::SetColorFormat(format, reply) => {
                self.handle_set_color_format(format, reply)
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
            ControlMessage::SetFastForwarding(fast_forwarding, reply) => {
                self.state
                    .fast_forwarding
                    .store(fast_forwarding, Ordering::Release);
                let _ = reply.send(Ok(()));
            }
            ControlMessage::SetFastForwardSpeed(speed, reply) => {
                self.state
                    .fast_forward_speed_percent
                    .store(speed.clamp(100, 1000), Ordering::Release);
                let _ = reply.send(Ok(()));
            }
            ControlMessage::SetRewindSpeed(speed, reply) => {
                self.state
                    .rewind_speed_percent
                    .store(speed.clamp(100, 1000), Ordering::Release);
                let _ = reply.send(Ok(()));
            }
            ControlMessage::LoadMovie(movie, reply) => self.handle_load_movie(movie, reply),
            ControlMessage::SubscribeEvent(topic, sender, reply) => {
                self.pubsub.subscribe(topic, sender);
                self.after_subscribe_event(topic);
                let _ = reply.send(Ok(()));
            }
            ControlMessage::UnsubscribeEvent(topic, reply) => {
                self.pubsub.unsubscribe(topic);
                self.after_unsubscribe_event(topic);
                let _ = reply.send(Ok(()));
            }
            ControlMessage::SetTilemapCapturePoint(point, reply) => {
                self.nes.set_tilemap_capture_point(point);
                let _ = reply.send(Ok(()));
            }
            ControlMessage::SetTileViewerCapturePoint(point, reply) => {
                self.nes.set_tile_viewer_capture_point(point);
                let _ = reply.send(Ok(()));
            }
            ControlMessage::SetSpriteCapturePoint(point, reply) => {
                self.nes.set_sprite_capture_point(point);
                let _ = reply.send(Ok(()));
            }
            ControlMessage::SetTileViewerSource(source, reply) => {
                {
                    let mut cfg = self.state.tile_viewer.lock();
                    cfg.source = source;
                }
                let _ = reply.send(Ok(()));
            }
            ControlMessage::SetTileViewerStartAddress(start_address, reply) => {
                {
                    let mut cfg = self.state.tile_viewer.lock();
                    cfg.start_address = start_address;
                }
                let _ = reply.send(Ok(()));
            }
            ControlMessage::SetTileViewerSize {
                columns,
                rows,
                reply,
            } => {
                {
                    let mut cfg = self.state.tile_viewer.lock();
                    cfg.column_count = columns;
                    cfg.row_count = rows;
                    // Match Mesen2: enforce multiple-of-2 counts when using non-normal layouts.
                    if !matches!(cfg.layout, TileViewerLayout::Normal) {
                        cfg.column_count &= !1;
                        cfg.row_count &= !1;
                    }
                }
                let _ = reply.send(Ok(()));
            }
            ControlMessage::SetTileViewerLayout(layout, reply) => {
                {
                    let mut cfg = self.state.tile_viewer.lock();
                    cfg.layout = layout;
                    if !matches!(cfg.layout, TileViewerLayout::Normal) {
                        cfg.column_count &= !1;
                        cfg.row_count &= !1;
                    }
                }
                let _ = reply.send(Ok(()));
            }
            ControlMessage::SetTileViewerBackground(background, reply) => {
                {
                    let mut cfg = self.state.tile_viewer.lock();
                    cfg.background = background;
                }
                let _ = reply.send(Ok(()));
            }
            ControlMessage::SetTileViewerPalette(palette, reply) => {
                {
                    let mut cfg = self.state.tile_viewer.lock();
                    cfg.selected_palette = palette.min(7);
                }
                let _ = reply.send(Ok(()));
            }
            ControlMessage::SetTileViewerUseGrayscalePalette(enabled, reply) => {
                {
                    let mut cfg = self.state.tile_viewer.lock();
                    cfg.use_grayscale_palette = enabled;
                }
                let _ = reply.send(Ok(()));
            }
            ControlMessage::SetPaletteCapturePoint(point, reply) => {
                self.nes.set_palette_capture_point(point);
                let _ = reply.send(Ok(()));
            }
            ControlMessage::EnableDebugger {
                debug_rx,
                debug_tx,
                reply,
            } => {
                use super::debug_interceptor::DebugInterceptor;
                let interceptor = DebugInterceptor::new(
                    self.ctrl_rx.clone(),
                    self.ctrl_tx.clone(),
                    debug_rx,
                    debug_tx,
                );
                self.nes.interceptor.add(interceptor);
                let _ = reply.send(Ok(()));
            }
            ControlMessage::DisableDebugger(reply) => {
                use super::debug_interceptor::DebugInterceptor;
                self.nes.interceptor.remove::<DebugInterceptor>();
                let _ = reply.send(Ok(()));
            }
            ControlMessage::EnableNetplay {
                input_provider,
                reply,
            } => {
                self.netplay_input = Some(input_provider);
                let _ = reply.send(Ok(()));
            }
            ControlMessage::DisableNetplay(reply) => {
                self.netplay_input = None;
                let _ = reply.send(Ok(()));
            }
        }

        false
    }

    fn after_subscribe_event(&mut self, topic: EventTopic) {
        match topic {
            EventTopic::Tilemap => self.nes.enable_tilemap_interceptor(),
            EventTopic::Tile => self.nes.enable_tile_viewer_interceptor(),
            EventTopic::Sprite => self.nes.enable_sprite_interceptor(),
            EventTopic::Palette => self.nes.enable_palette_interceptor(),
            _ => {}
        }
    }

    fn after_unsubscribe_event(&mut self, topic: EventTopic) {
        match topic {
            EventTopic::Tilemap => {
                if !self.pubsub.has_subscriber(EventTopic::Tilemap) {
                    self.nes.disable_tilemap_interceptor();
                }
            }
            EventTopic::Tile => {
                if !self.pubsub.has_subscriber(EventTopic::Tile) {
                    self.nes.disable_tile_viewer_interceptor();
                }
            }
            EventTopic::Sprite => {
                if !self.pubsub.has_subscriber(EventTopic::Sprite) {
                    self.nes.disable_sprite_interceptor();
                }
            }
            EventTopic::Palette => {
                if !self.pubsub.has_subscriber(EventTopic::Palette) {
                    self.nes.disable_palette_interceptor();
                }
            }
            _ => {}
        }
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

            // Present converts indices to packed pixels once and swaps the presented plane.
            fb.present(&palette);

            if let Some(audio) = &self.audio {
                audio.clear();
            }
        }
    }

    fn step_frame(&mut self) {
        let frame_seq = self.state.frame_seq.load(Ordering::Relaxed);

        // Check for pending rollback (Rollback mode only)
        // This must happen BEFORE advancing the frame to restore correct state
        let pending_rollback = self
            .netplay_input
            .as_ref()
            .and_then(|np| np.pending_rollback());

        if let Some(rollback) = pending_rollback {
            self.handle_netplay_rollback(rollback.target_frame, rollback.current_frame);
            if let Some(np) = &self.netplay_input {
                np.clear_rollback();
            }
        }

        let movie_frame = self.movie_frame_for_seq(frame_seq);
        self.maybe_apply_movie_frame(&movie_frame);

        let (turbo_on_frames, period) = self.turbo_params();

        let netplay_inputs = self.netplay_inputs_for_frame(frame_seq, turbo_on_frames, period);
        let (any_input, netplay_rewind) = self.apply_inputs_for_frame(
            frame_seq,
            turbo_on_frames,
            period,
            movie_frame.as_ref(),
            netplay_inputs.as_ref(),
        );

        // Check if we should rewind
        // Priority: Netplay Rewind > Offline Rewind
        let should_rewind = netplay_rewind
            || (netplay_inputs.is_none() && self.state.rewinding.load(Ordering::Acquire));

        if should_rewind {
            self.rewind_frame();
            // Skip run_frame
        } else {
            self.maybe_send_periodic_netplay_state(frame_seq);

            // Any regular input (including TAS input) should cancel offline rewind.
            // This prevents the runtime from getting stuck in a rewind-only loop.
            if any_input && netplay_inputs.is_none() {
                self.state.rewinding.store(false, Ordering::Release);
            }

            let samples = self.nes.run_frame(self.audio.is_some());
            if let Some(audio) = &mut self.audio
                && !samples.is_empty()
            {
                audio.push_samples(&samples);
            }

            // Capture history for future rewind
            self.maybe_capture_rewind_history();

            // Capture snapshot for netplay rollback (Rollback mode only)
            self.maybe_capture_netplay_snapshot(frame_seq);
        }

        self.state.frame_seq.fetch_add(1, Ordering::Relaxed);

        // Advance the TAS timeline AFTER the frame has been executed.
        if movie_frame.is_some() {
            self.advance_movie_after_frame();
        }

        // Broadcast debug state if there's a subscriber.
        self.maybe_broadcast_debug_state();

        // Broadcast viewer state (each viewer has its own interceptor snapshot)
        self.maybe_broadcast_tilemap_and_chr_state();
    }

    fn movie_frame_for_seq(&mut self, frame_seq: u64) -> Option<InputFrame> {
        if let Some(movie) = &mut self.movie {
            movie.frames.get(frame_seq as usize).copied()
        } else {
            None
        }
    }

    fn maybe_apply_movie_frame(&mut self, movie_frame: &Option<InputFrame>) {
        if let Some(frame) = movie_frame {
            self.state.rewinding.store(false, Ordering::Release);
            self.apply_movie_resets(frame);
        }
    }

    fn turbo_params(&self) -> (u64, u64) {
        let turbo_on_frames = self.state.turbo_on_frames.load(Ordering::Relaxed) as u64;
        let turbo_off_frames = self.state.turbo_off_frames.load(Ordering::Relaxed) as u64;
        let period = (turbo_on_frames + turbo_off_frames).max(1);
        (turbo_on_frames, period)
    }

    fn netplay_inputs_for_frame(
        &mut self,
        frame_seq: u64,
        turbo_on_frames: u64,
        period: u64,
    ) -> Option<[u16; 4]> {
        let np = self.netplay_input.as_ref()?;
        if !np.is_active() {
            self.netplay_active = false;
            return None;
        }
        let np = np.clone();

        // 1) Send local input to server (if we are a player).
        if let Some(_local_idx) = np.local_player() {
            let cap = np.rewind_capacity();
            self.state
                .rewind_capacity
                .store(cap as u64, Ordering::Release);

            let delay = np.input_delay();
            // In single-machine netplay, the local user is physically holding "Controller 1".
            // So we read from physical port 0, but send it as our assigned netplay index.
            let physical_pad = 0;

            if physical_pad < 4 {
                // At start of session OR upon late activation, fill the delay buffer with empty inputs
                // to prime the input pipeline.
                let was_inactive = !self.netplay_active;
                if frame_seq == 0 || was_inactive {
                    for i in 0..delay {
                        np.send_input_to_server(frame_seq as u32 + i, 0);
                    }
                }
                self.netplay_active = true;

                // Immediate State Sync on Activation (Host only)
                if was_inactive && np.local_player() == Some(0) {
                    match self.capture_compressed_snapshot() {
                        Ok(bytes) => {
                            np.send_state(frame_seq as u32, &bytes);
                            eprintln!("[Netplay] Sent immediate state sync at frame {}", frame_seq);
                        }
                        Err(e) => {
                            eprintln!("[Netplay] Failed to capture immediate state: {}", e);
                        }
                    }
                }

                let base = self.state.pad_masks[physical_pad].load(Ordering::Acquire);
                let mut input = self.apply_turbo_to_mask(
                    physical_pad,
                    base as u8,
                    frame_seq,
                    turbo_on_frames,
                    period,
                ) as u16;

                // Inject Rewind bit if requested locally.
                if self.state.rewinding.load(Ordering::Acquire) {
                    input |= 0x0100;
                }

                np.send_input_to_server(frame_seq as u32 + delay, input);
            }
        }

        // 2) Wait for confirmed inputs (lockstep).
        let mut inputs = None;
        while inputs.is_none() {
            inputs = np.poll_inputs(frame_seq as u32);
            if inputs.is_none() {
                // IMPORTANT: Don't fall back to offline inputs when netplay is active.
                // A temporary input stall must pause the emulation instead of desyncing.
                if !np.is_active() {
                    break;
                }
                thread::sleep(Duration::from_millis(1));
            }
        }
        inputs
    }

    fn apply_inputs_for_frame(
        &mut self,
        frame_seq: u64,
        turbo_on_frames: u64,
        period: u64,
        movie_frame: Option<&InputFrame>,
        netplay_inputs: Option<&[u16; 4]>,
    ) -> (bool, bool) {
        let mut any_input = false;
        let mut netplay_rewind = false;

        // Sync inputs: priority is netplay > TAS movie > live input.
        for pad in 0..4 {
            let mask: u16 = if let Some(inputs) = netplay_inputs {
                let inp = inputs[pad];
                if (inp & 0x0100) != 0 {
                    netplay_rewind = true;
                }
                inp
            } else if let Some(frame) = movie_frame {
                frame.ports[pad] as u16
            } else {
                let base = self.state.pad_masks[pad].load(Ordering::Acquire);
                self.apply_turbo_to_mask(pad, base as u8, frame_seq, turbo_on_frames, period) as u16
            };

            if (mask & 0xFF) != 0 {
                any_input = true;
            }

            self.apply_pad_mask(pad, (mask & 0xFF) as u8);
        }

        (any_input, netplay_rewind)
    }

    fn maybe_send_periodic_netplay_state(&mut self, frame_seq: u64) {
        // Clone the Arc to avoid borrowing self while mutating self for capture.
        let Some(np) = self.netplay_input.clone() else {
            return;
        };
        // Only the host (P1) is responsible for providing state.
        if np.local_player() != Some(0) {
            return;
        }

        // On-demand state sync (requested by server) for late joiners/reconnects.
        let requested = np.take_state_sync_request();

        // Periodic Netplay State Sync (Host Only)
        // Every second (60 frames), the host sends a compressed state snapshot to the server.
        // This allows late joiners (and spectators) to catch up quickly without replaying the entire history.
        let periodic_due = self.netplay_active && frame_seq != 0 && frame_seq % 60 == 0;

        if !requested && !periodic_due {
            return;
        }

        match self.capture_compressed_snapshot() {
            Ok(bytes) => {
                np.send_state(frame_seq as u32, &bytes);
            }
            Err(e) => {
                eprintln!("[Netplay] Failed to capture periodic state: {}", e);
            }
        }
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

    /// Captures a snapshot for netplay rollback resimulation (Rollback mode only).
    fn maybe_capture_netplay_snapshot(&mut self, frame_seq: u64) {
        let Some(np) = &self.netplay_input else {
            return;
        };

        // Only capture snapshots in Rollback mode
        if np.sync_mode() != SyncMode::Rollback {
            return;
        }

        let effective_frame = np.to_effective_frame(frame_seq as u32);

        // Check if we should save this frame (respects save interval)
        if !self.netplay_snapshots.should_save(effective_frame) {
            return;
        }

        // Use compressed snapshot for memory efficiency
        let meta = SnapshotMeta {
            tick: self.nes.master_clock(),
            ..Default::default()
        };

        if let Ok(snap) = self.nes.save_snapshot(meta) {
            // Serialize and compress the snapshot
            if let Ok(bytes) = postcard::to_allocvec(&snap) {
                let compressed = compress_prepend_size(&bytes);
                self.netplay_snapshots.push(effective_frame, compressed);
            }
        }
    }

    /// Handles rollback: restores state and resimulates frames with confirmed inputs.
    fn handle_netplay_rollback(&mut self, target_frame: u32, current_frame: u32) {
        eprintln!(
            "[Netplay] Rollback requested: {} -> {}",
            current_frame, target_frame
        );

        // 1. Find snapshot strictly before the target frame.
        // Our snapshots are captured after each executed frame, so to re-simulate `target_frame`
        // we need a snapshot at or before `target_frame - 1`.
        let snapshot_search_frame = target_frame.saturating_sub(1);
        let Some(snapshot) = self.netplay_snapshots.find_before(snapshot_search_frame) else {
            eprintln!(
                "[Netplay] No snapshot found for rollback to frame {}",
                target_frame
            );
            return;
        };

        let snapshot_frame = snapshot.frame;
        let snapshot_data = snapshot.data.clone();

        // 2. Decompress and restore the snapshot
        let Ok(decompressed) = decompress_size_prepended(&snapshot_data) else {
            eprintln!(
                "[Netplay] Failed to decompress snapshot for frame {}",
                snapshot_frame
            );
            return;
        };

        let Ok(nes_snapshot) = postcard::from_bytes::<NesSnapshot>(&decompressed) else {
            eprintln!(
                "[Netplay] Failed to deserialize snapshot for frame {}",
                snapshot_frame
            );
            return;
        };

        if self.nes.load_snapshot(&nes_snapshot).is_err() {
            eprintln!(
                "[Netplay] Failed to load snapshot for frame {}",
                snapshot_frame
            );
            return;
        }

        // 3. Clear audio to prevent glitches during resimulation
        if let Some(audio) = &self.audio {
            audio.clear();
        }

        // 4. Resimulate frames from snapshot_frame to current_frame using confirmed inputs
        // Clone the Arc to avoid borrow conflict with mutable self methods
        let np = match self.netplay_input.clone() {
            Some(np) => np,
            None => return,
        };

        for effective_frame in (snapshot_frame + 1)..=current_frame {
            // `rollback.*_frame` are network/effective frames; `poll_inputs` expects local frames.
            let local_frame = np.to_local_frame(effective_frame);

            // Get inputs for this frame (Rollback mode predicts when missing).
            if let Some(inputs) = np.poll_inputs(local_frame) {
                // Apply inputs to all pads
                for (pad, &buttons) in inputs.iter().enumerate() {
                    self.apply_pad_mask(pad, (buttons & 0xFF) as u8);
                }
            }

            // Run the frame without audio during resimulation
            let _ = self.nes.run_frame(false);
        }

        eprintln!(
            "[Netplay] Rollback complete: resimulated {} frames",
            current_frame - snapshot_frame
        );
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
                cycle: self.nes.cpu_cycles(),
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

    /// Broadcasts tilemap, CHR, sprite, and/or palette state to subscribers if someone is listening.
    fn maybe_broadcast_tilemap_and_chr_state(&mut self) {
        let has_tilemap = self.pubsub.has_subscriber(EventTopic::Tilemap);
        let has_chr = self.pubsub.has_subscriber(EventTopic::Tile);
        let has_sprite = self.pubsub.has_subscriber(EventTopic::Sprite);
        let has_palette = self.pubsub.has_subscriber(EventTopic::Palette);

        if !has_tilemap && !has_chr && !has_sprite && !has_palette {
            return;
        }

        // Convert current NES palette to platform-specific format for aux texture rendering.
        let nes_palette = self.nes.palette();
        let mut bgra_palette = [[0u8; 4]; 64];

        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            for (i, color) in nes_palette.as_colors().iter().enumerate() {
                bgra_palette[i] = [color.b, color.g, color.r, 0xFF]; // BGRA
            }
        }
        #[cfg(not(any(target_os = "macos", target_os = "ios")))]
        {
            for (i, color) in nes_palette.as_colors().iter().enumerate() {
                bgra_palette[i] = [color.r, color.g, color.b, 0xFF]; // RGBA
            }
        }

        // Broadcast to Tilemap subscribers
        if has_tilemap {
            if let Some(snap) = self.nes.take_tilemap_snapshot() {
                let tilemap = TilemapState {
                    ciram: snap.ciram,
                    palette: snap.palette,
                    chr: snap.chr,
                    mirroring: snap.mirroring,
                    bgra_palette,
                    bg_pattern_base: snap.bg_pattern_base,
                    vram_addr: snap.vram_addr,
                    temp_addr: snap.temp_addr,
                    fine_x: snap.fine_x,
                };
                self.pubsub
                    .broadcast(EventTopic::Tilemap, Box::new(tilemap));
            }
        }

        // Broadcast to Tile subscribers
        if has_chr {
            if let Some(snap) = self.nes.take_tile_viewer_snapshot() {
                let cfg = self.state.tile_viewer.lock().clone();

                let mut column_count = cfg.column_count.clamp(4, 256);
                let mut row_count = cfg.row_count.clamp(4, 256);
                if !matches!(cfg.layout, TileViewerLayout::Normal) {
                    column_count &= !1;
                    row_count &= !1;
                }

                let width = column_count.saturating_mul(8);
                let height = row_count.saturating_mul(8);
                let tile_count = column_count as usize * row_count as usize;
                let bytes_per_tile = 16usize; // NES 2bpp planar 8×8
                let total_size = tile_count.saturating_mul(bytes_per_tile);

                let (source_size, source_bytes) = self.tile_viewer_read_source_bytes(
                    cfg.source,
                    cfg.start_address,
                    total_size,
                    &snap.chr,
                );

                // Pass raw source_bytes to worker - rendering will happen there, not in NES thread.
                let chr_state = TileState {
                    rgba: Vec::new(), // Will be rendered by worker
                    source_bytes,
                    width,
                    height,
                    source: cfg.source,
                    source_size,
                    start_address: cfg.start_address,
                    column_count,
                    row_count,
                    layout: cfg.layout,
                    background: cfg.background,
                    palette: snap.palette,
                    bgra_palette,
                    selected_palette: cfg.selected_palette.min(7),
                    use_grayscale_palette: cfg.use_grayscale_palette,
                    bg_pattern_base: snap.bg_pattern_base,
                    sprite_pattern_base: snap.sprite_pattern_base,
                    large_sprites: snap.large_sprites,
                };

                self.pubsub.broadcast(EventTopic::Tile, Box::new(chr_state));
            }
        }

        // Broadcast to Sprite subscribers
        if has_sprite {
            if let Some(snap) = self.nes.take_sprite_snapshot() {
                let large_sprites = snap.large_sprites;
                let pattern_base = snap.sprite_pattern_base;

                // Pass raw data to worker - rendering will happen there, not in NES thread.
                let sprite_state = SpriteState {
                    sprites: Vec::new(),     // Will be built by worker
                    screen_rgba: Vec::new(), // Will be rendered by worker
                    screen_width: 256,
                    screen_height: 256,
                    thumbnails_rgba: Vec::new(), // Will be rendered by worker
                    thumbnail_width: 8,
                    thumbnail_height: if large_sprites { 16 } else { 8 },
                    large_sprites,
                    pattern_base,
                    bgra_palette,
                    oam: snap.oam,
                    chr: snap.chr,
                    palette: snap.palette,
                };

                self.pubsub
                    .broadcast(EventTopic::Sprite, Box::new(sprite_state));
            }
        }

        // Broadcast to Palette subscribers
        if has_palette {
            if let Some(snap) = self.nes.take_palette_snapshot() {
                let palette_state = PaletteState {
                    palette: snap.palette,
                    bgra_palette,
                };

                self.pubsub
                    .broadcast(EventTopic::Palette, Box::new(palette_state));
            }
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
                        *self.state.rom_hash.lock() = Some(full_hash);
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
                        *self.state.rom_hash.lock() = None;
                        let error = e.to_string();
                        let _ = reply.send(Err(RuntimeError::LoadRomFailed { path, error }));
                    }
                }
            }
            Err(e) => {
                *self.state.rom_hash.lock() = None;
                let error = e.to_string();
                let _ = reply.send(Err(RuntimeError::LoadRomFailed { path, error }));
            }
        }
    }

    fn handle_load_rom_from_memory(&mut self, bytes: Vec<u8>, reply: ControlReplySender) {
        let mut hasher = Sha1::new();
        hasher.update(&bytes);
        let hash: [u8; 20] = hasher.finalize().into();
        let mut full_hash = [0u8; 32];
        full_hash[..20].copy_from_slice(&hash);

        match nesium_core::cartridge::load_cartridge(bytes) {
            Ok(cart) => {
                self.nes.insert_cartridge(cart);
                *self.state.rom_hash.lock() = Some(full_hash);
                self.state.paused.store(false, Ordering::Release);
                self.next_frame_deadline = Instant::now();
                if let Some(audio) = &self.audio {
                    audio.clear();
                }
                self.rewind.clear();
                self.state.rewinding.store(false, Ordering::Release);
                // Reset frame sequence when netplay is active to prevent frame mismatch.
                // Without this, after power off and reload, frame_seq stays high but
                // netplay expects frames starting from 0, causing a deadlock.
                if self.netplay_input.is_some() {
                    self.state.frame_seq.store(0, Ordering::Release);
                }
                let _ = reply.send(Ok(()));
            }
            Err(e) => {
                *self.state.rom_hash.lock() = None;
                let error = e.to_string();
                let _ = reply.send(Err(RuntimeError::LoadRomFailed {
                    path: PathBuf::from("memory"),
                    error,
                }));
            }
        }
    }

    fn tile_viewer_read_source_bytes(
        &self,
        source: TileViewerSource,
        start_address: u32,
        len: usize,
        ppu_chr: &[u8],
    ) -> (u32, Vec<u8>) {
        let mut out = vec![0u8; len];

        match source {
            TileViewerSource::Ppu => {
                let size = 0x2000usize;
                let src = if ppu_chr.is_empty() {
                    &[0u8; 0]
                } else {
                    ppu_chr
                };
                for (i, b) in out.iter_mut().enumerate() {
                    if src.is_empty() {
                        *b = 0;
                    } else {
                        *b = src[(start_address as usize + i) % size % src.len()];
                    }
                }
                (size as u32, out)
            }
            TileViewerSource::ChrRom => {
                let src = self
                    .nes
                    .get_cartridge()
                    .and_then(|c| {
                        let m = c.mapper();
                        m.chr_rom().or_else(|| m.chr_ram())
                    })
                    .unwrap_or(&[]);
                let size = src.len();
                if size == 0 {
                    return (0, out);
                }
                for (i, b) in out.iter_mut().enumerate() {
                    *b = src[(start_address as usize + i) % size];
                }
                (size as u32, out)
            }
            TileViewerSource::ChrRam => {
                let src = self
                    .nes
                    .get_cartridge()
                    .and_then(|c| {
                        let m = c.mapper();
                        m.chr_ram().or_else(|| m.chr_rom())
                    })
                    .unwrap_or(&[]);
                let size = src.len();
                if size == 0 {
                    return (0, out);
                }
                for (i, b) in out.iter_mut().enumerate() {
                    *b = src[(start_address as usize + i) % size];
                }
                (size as u32, out)
            }
            TileViewerSource::PrgRom => {
                let src = self
                    .nes
                    .get_cartridge()
                    .and_then(|c| c.mapper().prg_rom())
                    .unwrap_or(&[]);
                let size = src.len();
                if size == 0 {
                    return (0, out);
                }
                for (i, b) in out.iter_mut().enumerate() {
                    *b = src[(start_address as usize + i) % size];
                }
                (size as u32, out)
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

    /// Powers off the console and performs a full shutdown.
    ///
    /// This clears all NES state and displays a black screen.
    fn handle_power_off(&mut self, reply: ControlReplySender) {
        self.nes.power_off();
        // Clear input state
        for mask in &self.state.pad_masks {
            mask.store(0, Ordering::Release);
        }
        for mask in &self.state.turbo_masks {
            mask.store(0, Ordering::Release);
        }
        // Clear audio
        if let Some(audio) = &self.audio {
            audio.clear();
        }
        // Clear rewind history
        self.rewind.clear();
        self.state.rewinding.store(false, Ordering::Release);
        // Reset frame sequence (important for netplay)
        self.state.frame_seq.store(0, Ordering::Release);
        // Clear ROM hash
        *self.state.rom_hash.lock() = None;

        // Clear framebuffer to display black screen and notify frontend.
        // clear_and_present() fills buffers with 0 bytes (black) and triggers the callback,
        // without re-rendering using the palette (which would produce gray from palette[0]).
        let fb = self.nes.ppu.framebuffer_mut();
        fb.clear_and_present();

        // Deactivate netplay to prevent lockstep issues on next ROM load
        if let Some(np) = &self.netplay_input {
            np.set_active(false);
        }
        self.netplay_active = false;

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

    /// Changes the color format for frame rendering at runtime.
    fn handle_set_color_format(&mut self, format: ColorFormat, reply: ControlReplySender) {
        self.nes.set_color_format(format);
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
                // Netplay uses a frame-relative timeline (starting at 0) mapped onto the network
                // `start_frame`. If the runtime advanced a few frames between ROM load and sync,
                // resetting here prevents 1–2 frame drift for late joiners.
                if self.netplay_input.is_some() {
                    self.state.frame_seq.store(0, Ordering::Release);
                    self.next_frame_deadline = Instant::now();
                    self.netplay_snapshots.clear();
                }
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
        let rom_hash = *self.state.rom_hash.lock();
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
            let current_hash = *self.state.rom_hash.lock();
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
