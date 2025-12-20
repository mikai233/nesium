use std::{
    path::PathBuf,
    sync::{
        Arc,
        mpsc::{Receiver, RecvTimeoutError, Sender, TryRecvError, channel},
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use anyhow::Result;
use nesium_audio::NesAudioPlayer;
use nesium_core::{
    Nes,
    audio::bus::AudioBusConfig,
    ppu::{
        SCREEN_HEIGHT, SCREEN_WIDTH,
        buffer::{BufferMode, ColorFormat, ExternalFrameHandle, FrameBuffer},
    },
    reset_kind::ResetKind,
};

use crate::app::controller::ControllerInput;

/// Commands sent from the UI thread to the Emulator thread.
pub enum Command {
    LoadRom(PathBuf),
    Reset(ResetKind),
    Eject,
    UpdateInput([ControllerInput; 4]),
    SetPaused(bool),
    SetAudioConfig(AudioBusConfig),
}

/// Events sent from the Emulator thread to the UI thread.
pub enum Event {
    FrameReady, // Notification only, data is in shared memory
    StatusInfo(String),
    Error(String),
}

pub struct EmulatorThread {
    tx: Sender<Command>,
    rx: Receiver<Event>,
    _handle: JoinHandle<()>,
    pub frame_handle: Arc<ExternalFrameHandle>,
}

struct BackingStore {
    _plane0: Box<[u8]>,
    _plane1: Box<[u8]>,
}

impl EmulatorThread {
    pub fn new() -> Self {
        let (cmd_tx, cmd_rx) = channel();
        let (event_tx, event_rx) = channel();

        // Allocate backing memory for the framebuffer
        let len = SCREEN_WIDTH * SCREEN_HEIGHT * 4;
        let mut plane0 = vec![0u8; len].into_boxed_slice();
        let mut plane1 = vec![0u8; len].into_boxed_slice();

        // Create the external framebuffer handle
        // Safety: The backing store (owned by Runner) ensures the pointers remain valid
        // as long as Runner is alive. Runner owns Nes, which owns FrameBuffer.
        let (fb, handle) = unsafe {
            FrameBuffer::new_external(
                BufferMode::Color {
                    format: ColorFormat::Rgba8888,
                },
                len,
                plane0.as_mut_ptr(),
                plane1.as_mut_ptr(),
            )
        };

        let backing = BackingStore {
            _plane0: plane0,
            _plane1: plane1,
        };

        let thread_handle = Arc::clone(&handle);
        let handle = thread::spawn(move || {
            let mut runner = Runner::new(cmd_rx, event_tx, fb, backing);
            runner.run();
        });

        Self {
            tx: cmd_tx,
            rx: event_rx,
            _handle: handle,
            frame_handle: thread_handle,
        }
    }

    pub fn send(&self, cmd: Command) {
        let _ = self.tx.send(cmd);
    }

    pub fn try_recv(&self) -> Option<Event> {
        match self.rx.try_recv() {
            Ok(event) => Some(event),
            Err(TryRecvError::Empty) => None,
            Err(TryRecvError::Disconnected) => None,
        }
    }
}

// NTSC: ~60.0988 Hz
const FRAME_DURATION: Duration = Duration::from_nanos(16_639_263);

struct Runner {
    nes: Nes,
    audio: Option<NesAudioPlayer>,
    cmd_rx: Receiver<Command>,
    event_tx: Sender<Event>,
    // Keep backing store alive so pointers in Nes.ppu.framebuffer remain valid
    _backing: BackingStore,

    // State
    paused: bool,
    has_cartridge: bool,
    input_buffer: [ControllerInput; 4],

    // Timing
    next_frame: Instant,
}

impl Runner {
    fn new(
        cmd_rx: Receiver<Command>,
        event_tx: Sender<Event>,
        framebuffer: FrameBuffer,
        backing: BackingStore,
    ) -> Self {
        // Initialize Audio
        let (audio, sample_rate) = match NesAudioPlayer::new() {
            Ok(player) => {
                let sr = player.sample_rate();
                (Some(player), sr)
            }
            Err(e) => {
                let _ = event_tx.send(Event::Error(format!("Audio init failed: {e}")));
                (None, 48000)
            }
        };

        let nes = Nes::new_with_framebuffer_and_sample_rate(framebuffer, sample_rate);

        Self {
            nes,
            audio,
            cmd_rx,
            event_tx,
            _backing: backing,
            paused: false,
            has_cartridge: false,
            input_buffer: std::array::from_fn(|_| ControllerInput::new_with_defaults()),
            next_frame: Instant::now(),
        }
    }

    fn run(&mut self) {
        loop {
            // 1. Process commands
            while let Ok(cmd) = self.cmd_rx.try_recv() {
                self.handle_command(cmd);
            }

            // 2. Run Emulation with a sleep-based fixed-step scheduler.
            //
            // Compared to the previous "sleep then spin" approach, this tends to produce
            // smoother overall frame pacing because it reduces CPU contention with the
            // renderer and the OS compositor.
            if !self.has_cartridge || self.paused {
                match self.cmd_rx.recv_timeout(Duration::from_millis(10)) {
                    Ok(cmd) => self.handle_command(cmd),
                    Err(RecvTimeoutError::Timeout) => {}
                    Err(RecvTimeoutError::Disconnected) => return,
                }
                // Reset timing while idle so we don't try to "catch up" after a long pause.
                self.next_frame = Instant::now();
                continue;
            }

            let now = Instant::now();
            if now < self.next_frame {
                // Wait until the next frame deadline, but wake up early if a command arrives.
                match self.cmd_rx.recv_timeout(self.next_frame - now) {
                    Ok(cmd) => {
                        self.handle_command(cmd);
                        continue;
                    }
                    Err(RecvTimeoutError::Timeout) => {}
                    Err(RecvTimeoutError::Disconnected) => return,
                }
            }

            // Run up to a few frames to catch up if we overslept.
            let mut frames_run: u32 = 0;
            while Instant::now() >= self.next_frame && frames_run < 3 {
                self.step_frame();
                self.next_frame += FRAME_DURATION;
                frames_run += 1;
            }

            // Drift correction: if we fell behind too much (>2 frames), snap back to now.
            let now = Instant::now();
            if now > self.next_frame && now.duration_since(self.next_frame) > FRAME_DURATION * 2 {
                self.next_frame = now;
            }
        }
    }

    fn handle_command(&mut self, cmd: Command) {
        match cmd {
            Command::LoadRom(path) => match self.nes.load_cartridge_from_file(&path) {
                Ok(_) => {
                    self.has_cartridge = true;
                    self.paused = false;
                    self.next_frame = Instant::now(); // Reset timing on load
                    let _ = self
                        .event_tx
                        .send(Event::StatusInfo(format!("Loaded {}", path.display())));
                }
                Err(e) => {
                    self.has_cartridge = false;
                    let _ = self
                        .event_tx
                        .send(Event::Error(format!("Failed to load ROM: {e}")));
                }
            },
            Command::Reset(kind) => {
                if self.has_cartridge {
                    self.nes.reset(kind);
                    for ctrl in &mut self.input_buffer {
                        ctrl.release_all();
                    }
                    if let Some(audio) = &self.audio {
                        audio.clear();
                    }
                    self.next_frame = Instant::now(); // Reset timing on reset
                    let _ = self.event_tx.send(Event::StatusInfo("Reset".to_string()));
                }
            }
            Command::Eject => {
                self.nes.eject_cartridge();
                self.has_cartridge = false;
                if let Some(audio) = &self.audio {
                    audio.clear();
                }
                let _ = self.event_tx.send(Event::StatusInfo("Ejected".to_string()));
            }
            Command::UpdateInput(inputs) => {
                self.input_buffer = inputs;
            }
            Command::SetPaused(paused) => {
                self.paused = paused;
                if !paused {
                    self.next_frame = Instant::now(); // Reset timing on resume
                }
            }
            Command::SetAudioConfig(cfg) => {
                self.nes.set_audio_bus_config(cfg);
            }
        }
    }

    fn step_frame(&mut self) {
        // Sync Inputs
        for (i, ctrl) in self.input_buffer.iter().enumerate() {
            ctrl.apply_to_nes(&mut self.nes, i);
        }

        // Run the core (Audio is generated but we don't block on it)
        let samples = self.nes.run_frame(true);
        if let Some(audio) = &mut self.audio {
            if !samples.is_empty() {
                // If buffer is full, this should ideally discard or overwrite oldest.
                // Assuming NesAudioPlayer or cpal backend handles non-blocking or dropping.
                // If this BLOCKS, then sleep logic is redundant/conflicting.
                // User said "buffer full -> discard", so this is safe.
                audio.push_samples(&samples);
            }
        }

        // Send Notification (Data is already in shared memory via ExternalFrameHandle)
        let _ = self.event_tx.send(Event::FrameReady);
    }
}
