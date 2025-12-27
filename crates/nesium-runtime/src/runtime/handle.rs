use core::ffi::c_void;
use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};

use crossbeam_channel::{Receiver, Sender, TryRecvError, bounded, unbounded};
use nesium_core::{
    audio::bus::AudioBusConfig,
    controller::Button,
    ppu::buffer::{BufferMode, ExternalFrameHandle, FrameBuffer},
    reset_kind::ResetKind,
};

use super::{
    control::{ControlMessage, ControlReplySender},
    runner::Runner,
    state::RuntimeState,
    types::{
        CONTROL_REPLY_TIMEOUT, FrameReadyCallback, LOAD_ROM_REPLY_TIMEOUT, RuntimeConfig,
        RuntimeError, RuntimeEvent,
    },
    util::button_bit,
};

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
        let (ctrl_tx, ctrl_rx) = unbounded::<ControlMessage>();
        let (event_tx, event_rx) = unbounded::<RuntimeEvent>();

        let len = config.video.len_bytes();
        if len == 0 {
            return Err(RuntimeError::VideoBufferLenZero);
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

        let state = Arc::new(RuntimeState::new());
        let thread_state = Arc::clone(&state);
        let audio_mode = config.audio;

        let join = thread::spawn(move || {
            let mut runner = Runner::new(audio_mode, ctrl_rx, event_tx, framebuffer, thread_state);
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
    fn send_with_reply(
        &self,
        op: &'static str,
        timeout: Duration,
        build: impl FnOnce(ControlReplySender) -> ControlMessage,
    ) -> Result<(), RuntimeError> {
        let (reply_tx, reply_rx) = bounded::<Result<(), RuntimeError>>(1);
        self.inner
            .ctrl_tx
            .send(build(reply_tx))
            .map_err(|_| RuntimeError::ControlChannelDisconnected)?;
        match reply_rx.recv_timeout(timeout) {
            Ok(res) => res,
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                Err(RuntimeError::ControlTimeout { op })
            }
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                Err(RuntimeError::ControlChannelDisconnected)
            }
        }
    }

    pub fn frame_handle(&self) -> &Arc<ExternalFrameHandle> {
        &self.inner.frame_handle
    }

    pub fn frame_seq(&self) -> u64 {
        self.inner
            .state
            .frame_seq
            .load(std::sync::atomic::Ordering::Relaxed)
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
        self.inner
            .state
            .paused
            .store(paused, std::sync::atomic::Ordering::Release);
    }

    pub fn paused(&self) -> bool {
        self.inner
            .state
            .paused
            .load(std::sync::atomic::Ordering::Acquire)
    }

    pub fn set_pad_mask(&self, pad: usize, mask: u8) {
        if let Some(slot) = self.inner.state.pad_masks.get(pad) {
            slot.store(mask, std::sync::atomic::Ordering::Release);
        }
    }

    pub fn set_turbo_mask(&self, pad: usize, mask: u8) {
        if let Some(slot) = self.inner.state.turbo_masks.get(pad) {
            slot.store(mask, std::sync::atomic::Ordering::Release);
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
            .store(frames.max(1), std::sync::atomic::Ordering::Release);
    }

    pub fn set_button(&self, pad: usize, button: Button, pressed: bool) {
        let Some(slot) = self.inner.state.pad_masks.get(pad) else {
            return;
        };
        let bit = button_bit(button);
        let mask = 1u8 << bit;

        if pressed {
            slot.fetch_or(mask, std::sync::atomic::Ordering::AcqRel);
        } else {
            slot.fetch_and(!mask, std::sync::atomic::Ordering::AcqRel);
        }
    }

    pub fn load_rom(&self, path: impl Into<PathBuf>) -> Result<(), RuntimeError> {
        let path = path.into();
        self.send_with_reply("load_rom", LOAD_ROM_REPLY_TIMEOUT, |reply| {
            ControlMessage::LoadRom(path, reply)
        })
    }

    pub fn reset(&self, kind: ResetKind) -> Result<(), RuntimeError> {
        self.send_with_reply("reset", CONTROL_REPLY_TIMEOUT, |reply| {
            ControlMessage::Reset(kind, reply)
        })
    }

    pub fn eject(&self) -> Result<(), RuntimeError> {
        self.send_with_reply("eject", CONTROL_REPLY_TIMEOUT, ControlMessage::Eject)
    }

    pub fn set_audio_config(&self, cfg: AudioBusConfig) -> Result<(), RuntimeError> {
        self.send_with_reply("set_audio_config", CONTROL_REPLY_TIMEOUT, |reply| {
            ControlMessage::SetAudioConfig(cfg, reply)
        })
    }

    pub fn set_frame_ready_callback(
        &self,
        cb: Option<FrameReadyCallback>,
        user_data: *mut c_void,
    ) -> Result<(), RuntimeError> {
        self.send_with_reply("set_frame_ready_callback", CONTROL_REPLY_TIMEOUT, |reply| {
            ControlMessage::SetFrameReadyCallback(cb, user_data, reply)
        })
    }

    /// Enables an integer FPS pacing mode.
    ///
    /// - `None`: run at the NES's exact NTSC FPS (~60.0988Hz)
    /// - `Some(60)`: pace frames at 60Hz to match common displays (reduces judder)
    ///
    /// PAL (`Some(50)`) is reserved for future support.
    pub fn set_integer_fps_target(&self, fps: Option<u32>) -> Result<(), RuntimeError> {
        if let Some(fps) = fps {
            match fps {
                60 => {}
                50 => return Err(RuntimeError::PalNotSupported),
                _ => {
                    return Err(RuntimeError::UnsupportedIntegerFpsTarget { fps });
                }
            }
        }

        self.send_with_reply("set_integer_fps_target", CONTROL_REPLY_TIMEOUT, |reply| {
            ControlMessage::SetIntegerFpsTarget(fps, reply)
        })
    }
}
