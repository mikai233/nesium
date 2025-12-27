use std::{
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

use crossbeam_channel::{Receiver, RecvTimeoutError, Sender, TryRecvError};
use nesium_core::{Nes, controller::Button, ppu::buffer::FrameBuffer};

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
    has_cartridge: bool,
    next_frame_deadline: Instant,
    frame_duration: Duration,
    integer_fps_target: Option<u32>,
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
            has_cartridge: false,
            next_frame_deadline: Instant::now(),
            frame_duration: FRAME_DURATION_NTSC,
            integer_fps_target: None,
        }
    }

    pub(crate) fn run(&mut self) {
        use std::sync::atomic::Ordering;
        let mut last_paused = self.state.paused.load(Ordering::Acquire);

        loop {
            while let Ok(msg) = self.ctrl_rx.try_recv() {
                if self.handle_control(msg) {
                    return;
                }
            }

            let paused = self.state.paused.load(Ordering::Acquire);
            if paused != last_paused && !paused {
                self.next_frame_deadline = Instant::now();
            }
            last_paused = paused;

            if !self.has_cartridge || paused {
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
            while !self.state.paused.load(Ordering::Acquire)
                && Instant::now() + FRAME_LEAD >= self.next_frame_deadline
                && frames_run < 3
            {
                self.step_frame();
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
                if spins % SPIN_YIELD_EVERY == 0 {
                    thread::yield_now();
                }
            }

            return WaitOutcome::DeadlineReached;
        }
    }

    fn handle_control(&mut self, msg: ControlMessage) -> bool {
        use std::sync::atomic::Ordering;
        match msg {
            ControlMessage::Stop => return true,
            ControlMessage::LoadRom(path, reply) => {
                match self.nes.load_cartridge_from_file(&path) {
                    Ok(_) => {
                        self.has_cartridge = true;
                        self.state.paused.store(false, Ordering::Release);
                        self.next_frame_deadline = Instant::now();
                        if let Some(audio) = &self.audio {
                            audio.clear();
                        }
                        let _ = reply.send(Ok(()));
                    }
                    Err(e) => {
                        self.has_cartridge = false;
                        let error = e.to_string();
                        let _ = reply.send(Err(RuntimeError::LoadRomFailed { path, error }));
                    }
                }
            }
            ControlMessage::Reset(kind, reply) => {
                if self.has_cartridge {
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
                    self.state.paused.store(false, Ordering::Release);
                    self.next_frame_deadline = Instant::now();
                }
                let _ = reply.send(Ok(()));
            }
            ControlMessage::Eject(reply) => {
                self.nes.eject_cartridge();
                self.has_cartridge = false;
                for mask in &self.state.pad_masks {
                    mask.store(0, Ordering::Release);
                }
                for mask in &self.state.turbo_masks {
                    mask.store(0, Ordering::Release);
                }
                if let Some(audio) = &self.audio {
                    audio.clear();
                }
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
        }

        false
    }

    fn step_frame(&mut self) {
        use std::sync::atomic::Ordering;
        let frame = self.state.frame_seq.load(Ordering::Relaxed);
        let turbo_on_frames = self.state.turbo_on_frames.load(Ordering::Acquire).max(1) as u64;
        let turbo_off_frames = self.state.turbo_off_frames.load(Ordering::Acquire).max(1) as u64;
        let turbo_cycle = turbo_on_frames + turbo_off_frames;
        let turbo_on = (frame % turbo_cycle) < turbo_on_frames;
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
            let turbo_mask = self.state.turbo_masks[pad].load(Ordering::Acquire);

            for button in buttons {
                let bit = 1u8 << button_bit(button);
                let normal_pressed = (mask & bit) != 0;
                let turbo_pressed = (turbo_mask & bit) != 0;
                let pressed = normal_pressed || (turbo_pressed && turbo_on);
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
