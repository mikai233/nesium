use std::{
    os::raw::{c_uint, c_void},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU8, Ordering},
        mpsc::{Receiver, RecvTimeoutError, Sender, TryRecvError, channel},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use nesium_core::{
    Nes,
    audio::bus::AudioBusConfig,
    controller::Button,
    ppu::{
        SCREEN_HEIGHT, SCREEN_WIDTH,
        buffer::{BufferMode, ColorFormat, ExternalFrameHandle, FrameBuffer},
    },
    reset_kind::ResetKind,
};

use crate::audio::NesAudioPlayer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioMode {
    Auto,
    Disabled,
}

#[derive(Debug, Clone, Copy)]
pub struct VideoConfig {
    pub color_format: ColorFormat,
    pub plane0: *mut u8,
    pub plane1: *mut u8,
}

impl VideoConfig {
    #[inline]
    pub fn len_bytes(self) -> usize {
        SCREEN_WIDTH * SCREEN_HEIGHT * self.color_format.bytes_per_pixel()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RuntimeConfig {
    pub video: VideoConfig,
    pub audio: AudioMode,
}

#[derive(Debug, Clone)]
pub enum RuntimeEvent {
    StatusInfo(String),
    Error(String),
}

pub type FrameReadyCallback = extern "C" fn(
    buffer_index: c_uint,
    width: c_uint,
    height: c_uint,
    pitch: c_uint,
    user_data: *mut c_void,
);

enum ControlMessage {
    Stop,
    LoadRom(PathBuf),
    Reset(ResetKind),
    Eject,
    SetAudioConfig(AudioBusConfig),
    SetFrameReadyCallback(Option<FrameReadyCallback>, *mut c_void),
}

// SAFETY: raw pointers and function pointers are forwarded to the runtime thread without
// dereferencing on the sending thread; the receiver owns and uses them.
unsafe impl Send for ControlMessage {}

enum WaitOutcome {
    /// Runtime thread should exit (channel disconnected or Stop received).
    Exit,
    /// A control message was received and handled; caller should re-check state/deadlines.
    ControlHandled,
    /// The target deadline has been reached.
    DeadlineReached,
}

#[derive(Debug, Clone)]
pub struct RuntimeError(String);

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for RuntimeError {}

struct RuntimeState {
    paused: AtomicBool,
    pad_masks: [AtomicU8; 4],
    turbo_masks: [AtomicU8; 4],
    turbo_frames_per_toggle: AtomicU8,
    frame_seq: std::sync::atomic::AtomicU64,
}

impl RuntimeState {
    fn new() -> Self {
        Self {
            paused: AtomicBool::new(false),
            pad_masks: std::array::from_fn(|_| AtomicU8::new(0)),
            turbo_masks: std::array::from_fn(|_| AtomicU8::new(0)),
            turbo_frames_per_toggle: AtomicU8::new(TURBO_FRAMES_PER_TOGGLE_DEFAULT),
            frame_seq: std::sync::atomic::AtomicU64::new(0),
        }
    }
}

struct RuntimeInner {
    ctrl_tx: Sender<ControlMessage>,
    events_rx: Mutex<Receiver<RuntimeEvent>>,
    frame_handle: Arc<ExternalFrameHandle>,
    state: Arc<RuntimeState>,
}

pub struct Runtime {
    inner: Arc<RuntimeInner>,
    join: Option<JoinHandle<()>>,
}

#[derive(Clone)]
pub struct RuntimeHandle {
    inner: Arc<RuntimeInner>,
}

impl Runtime {
    pub fn start(config: RuntimeConfig) -> Result<Self, RuntimeError> {
        let (ctrl_tx, ctrl_rx) = channel::<ControlMessage>();
        let (event_tx, event_rx) = channel::<RuntimeEvent>();

        let len = config.video.len_bytes();
        if len == 0 {
            return Err(RuntimeError("video buffer length is zero".to_string()));
        }

        let (framebuffer, frame_handle) = unsafe {
            FrameBuffer::new_external(
                BufferMode::Color {
                    format: config.video.color_format,
                },
                len,
                config.video.plane0,
                config.video.plane1,
            )
        };

        let runner_config = RunnerConfig {
            color_format: config.video.color_format,
            audio: config.audio,
        };

        let state = Arc::new(RuntimeState::new());
        let thread_state = Arc::clone(&state);
        let thread_frame_handle = Arc::clone(&frame_handle);

        let join = thread::spawn(move || {
            let mut runner = Runner::new(
                runner_config,
                ctrl_rx,
                event_tx,
                framebuffer,
                thread_frame_handle,
                thread_state,
            );
            runner.run();
        });

        let inner = Arc::new(RuntimeInner {
            ctrl_tx,
            events_rx: Mutex::new(event_rx),
            frame_handle,
            state,
        });

        Ok(Self {
            inner,
            join: Some(join),
        })
    }

    pub fn handle(&self) -> RuntimeHandle {
        RuntimeHandle {
            inner: Arc::clone(&self.inner),
        }
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        let _ = self.inner.ctrl_tx.send(ControlMessage::Stop);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

impl RuntimeHandle {
    pub fn frame_handle(&self) -> Arc<ExternalFrameHandle> {
        Arc::clone(&self.inner.frame_handle)
    }

    pub fn frame_seq(&self) -> u64 {
        self.inner.state.frame_seq.load(Ordering::Relaxed)
    }

    pub fn try_recv_event(&self) -> Option<RuntimeEvent> {
        let rx = self.inner.events_rx.lock().ok()?;
        match rx.try_recv() {
            Ok(ev) => Some(ev),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => None,
        }
    }

    pub fn set_paused(&self, paused: bool) {
        self.inner.state.paused.store(paused, Ordering::Release);
    }

    pub fn paused(&self) -> bool {
        self.inner.state.paused.load(Ordering::Acquire)
    }

    pub fn set_pad_mask(&self, pad: usize, mask: u8) {
        if let Some(slot) = self.inner.state.pad_masks.get(pad) {
            slot.store(mask, Ordering::Release);
        }
    }

    pub fn set_turbo_mask(&self, pad: usize, mask: u8) {
        if let Some(slot) = self.inner.state.turbo_masks.get(pad) {
            slot.store(mask, Ordering::Release);
        }
    }

    /// Set how many frames each turbo phase lasts (ON then OFF).
    ///
    /// - `1` toggles every frame (~30Hz on NTSC)
    /// - `2` toggles every 2 frames (~15Hz on NTSC)
    pub fn set_turbo_frames_per_toggle(&self, frames: u8) {
        self.inner
            .state
            .turbo_frames_per_toggle
            .store(frames.max(1), Ordering::Release);
    }

    pub fn set_button(&self, pad: usize, button: Button, pressed: bool) {
        let Some(slot) = self.inner.state.pad_masks.get(pad) else {
            return;
        };
        let bit = button_bit(button);
        let mask = 1u8 << bit;

        if pressed {
            slot.fetch_or(mask, Ordering::AcqRel);
        } else {
            slot.fetch_and(!mask, Ordering::AcqRel);
        }
    }

    pub fn load_rom(&self, path: impl Into<PathBuf>) -> Result<(), RuntimeError> {
        self.inner
            .ctrl_tx
            .send(ControlMessage::LoadRom(path.into()))
            .map_err(|e| RuntimeError(e.to_string()))
    }

    pub fn reset(&self, kind: ResetKind) -> Result<(), RuntimeError> {
        self.inner
            .ctrl_tx
            .send(ControlMessage::Reset(kind))
            .map_err(|e| RuntimeError(e.to_string()))
    }

    pub fn eject(&self) -> Result<(), RuntimeError> {
        self.inner
            .ctrl_tx
            .send(ControlMessage::Eject)
            .map_err(|e| RuntimeError(e.to_string()))
    }

    pub fn set_audio_config(&self, cfg: AudioBusConfig) -> Result<(), RuntimeError> {
        self.inner
            .ctrl_tx
            .send(ControlMessage::SetAudioConfig(cfg))
            .map_err(|e| RuntimeError(e.to_string()))
    }

    pub fn set_frame_ready_callback(
        &self,
        cb: Option<FrameReadyCallback>,
        user_data: *mut c_void,
    ) -> Result<(), RuntimeError> {
        self.inner
            .ctrl_tx
            .send(ControlMessage::SetFrameReadyCallback(cb, user_data))
            .map_err(|e| RuntimeError(e.to_string()))
    }
}

fn button_bit(button: Button) -> u8 {
    match button {
        Button::A => 0,
        Button::B => 1,
        Button::Select => 2,
        Button::Start => 3,
        Button::Up => 4,
        Button::Down => 5,
        Button::Left => 6,
        Button::Right => 7,
    }
}

// NTSC: ~60.0988 Hz
const FRAME_DURATION: Duration = Duration::from_nanos(16_639_263);
// Hybrid wait tuning:
// - Sleep in small chunks until we're close to the deadline.
// - Spin for the final window for tighter frame pacing.
const MAX_SLEEP_CHUNK: Duration = Duration::from_millis(4);
const SPIN_THRESHOLD: Duration = Duration::from_micros(300);
const SPIN_YIELD_EVERY: u32 = 512;
const TURBO_FRAMES_PER_TOGGLE_DEFAULT: u8 = 2;

#[derive(Debug, Clone, Copy)]
struct RunnerConfig {
    color_format: ColorFormat,
    audio: AudioMode,
}

struct Runner {
    config: RunnerConfig,
    nes: Nes,
    audio: Option<NesAudioPlayer>,
    runtime_sample_rate: u32,

    ctrl_rx: Receiver<ControlMessage>,
    event_tx: Sender<RuntimeEvent>,
    callback: Option<(FrameReadyCallback, *mut c_void)>,

    frame_handle: Arc<ExternalFrameHandle>,
    state: Arc<RuntimeState>,

    has_cartridge: bool,
    next_frame_deadline: Instant,
}

impl Runner {
    fn new(
        config: RunnerConfig,
        ctrl_rx: Receiver<ControlMessage>,
        event_tx: Sender<RuntimeEvent>,
        framebuffer: FrameBuffer,
        frame_handle: Arc<ExternalFrameHandle>,
        state: Arc<RuntimeState>,
    ) -> Self {
        let (audio, runtime_sample_rate) = match config.audio {
            AudioMode::Disabled => (None, 48_000),
            AudioMode::Auto => match NesAudioPlayer::new() {
                Ok(player) => {
                    let sr = player.sample_rate();
                    (Some(player), sr)
                }
                Err(e) => {
                    let _ = event_tx.send(RuntimeEvent::Error(format!("Audio init failed: {e}")));
                    (None, 48_000)
                }
            },
        };

        let nes = Nes::new_with_framebuffer_and_sample_rate(framebuffer, runtime_sample_rate);

        Self {
            config,
            nes,
            audio,
            runtime_sample_rate,
            ctrl_rx,
            event_tx,
            callback: None,
            frame_handle,
            state,
            has_cartridge: false,
            next_frame_deadline: Instant::now(),
        }
    }

    fn run(&mut self) {
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
                && Instant::now() >= self.next_frame_deadline
                && frames_run < 3
            {
                self.step_frame();
                self.next_frame_deadline += FRAME_DURATION;
                frames_run += 1;
            }

            let now = Instant::now();
            if now > self.next_frame_deadline
                && now.duration_since(self.next_frame_deadline) > FRAME_DURATION * 2
            {
                self.next_frame_deadline = now;
            }
        }
    }

    fn wait_until_next_deadline(&mut self) -> WaitOutcome {
        loop {
            let now = Instant::now();
            if now >= self.next_frame_deadline {
                return WaitOutcome::DeadlineReached;
            }

            let remaining = self.next_frame_deadline - now;

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
            while Instant::now() < self.next_frame_deadline {
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
        match msg {
            ControlMessage::Stop => return true,
            ControlMessage::LoadRom(path) => match self.nes.load_cartridge_from_file(&path) {
                Ok(_) => {
                    self.has_cartridge = true;
                    self.state.paused.store(false, Ordering::Release);
                    self.next_frame_deadline = Instant::now();
                    if let Some(audio) = &self.audio {
                        audio.clear();
                    }
                    let _ = self.event_tx.send(RuntimeEvent::StatusInfo(format!(
                        "Loaded {}",
                        path.display()
                    )));
                }
                Err(e) => {
                    self.has_cartridge = false;
                    let _ = self
                        .event_tx
                        .send(RuntimeEvent::Error(format!("Failed to load ROM: {e}")));
                }
            },
            ControlMessage::Reset(kind) => {
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
                    let _ = self
                        .event_tx
                        .send(RuntimeEvent::StatusInfo("Reset".to_string()));
                }
            }
            ControlMessage::Eject => {
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
                let _ = self
                    .event_tx
                    .send(RuntimeEvent::StatusInfo("Ejected".to_string()));
            }
            ControlMessage::SetAudioConfig(cfg) => {
                self.nes.set_audio_bus_config(cfg);
            }
            ControlMessage::SetFrameReadyCallback(cb, user_data) => {
                self.callback = cb.map(|f| (f, user_data));
            }
        }

        false
    }

    fn step_frame(&mut self) {
        let frame = self.state.frame_seq.load(Ordering::Relaxed);
        let turbo_frames_per_toggle = self
            .state
            .turbo_frames_per_toggle
            .load(Ordering::Acquire)
            .max(1) as u64;
        let turbo_on = (frame / turbo_frames_per_toggle) % 2 == 0;
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

        if let Some((cb, user_data)) = self.callback {
            // NOTE: `buffer_index` is informational. Readers should use a safe copy API
            // (e.g. begin/end front copy) to avoid races.
            let buffer_index = self.frame_handle.front_index() as c_uint;
            cb(
                buffer_index,
                SCREEN_WIDTH as c_uint,
                SCREEN_HEIGHT as c_uint,
                (SCREEN_WIDTH * self.config.color_format.bytes_per_pixel()) as c_uint,
                user_data,
            );
        }
    }
}
