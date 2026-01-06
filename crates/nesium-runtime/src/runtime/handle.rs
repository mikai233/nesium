use core::ffi::c_void;
use std::{
    path::PathBuf,
    sync::{Arc, atomic::Ordering},
    thread::{self, JoinHandle},
    time::Duration,
};

use crossbeam_channel::{Sender, bounded, unbounded};
use nesium_core::{
    audio::bus::AudioBusConfig,
    controller::Button,
    interceptor::{
        palette_interceptor::CapturePoint as PaletteCapturePoint,
        sprite_interceptor::CapturePoint as SpriteCapturePoint,
        tile_viewer_interceptor::CapturePoint as TileViewerCapturePoint,
        tilemap_interceptor::CapturePoint as TilemapCapturePoint,
    },
    ppu::buffer::FrameReadyCallback,
    ppu::buffer::{ExternalFrameHandle, FrameBuffer},
    ppu::palette::{Palette, PaletteKind},
    reset_kind::ResetKind,
};

use super::{
    control::{ControlMessage, ControlReplySender},
    pubsub::RuntimePubSub,
    runner::Runner,
    state::RuntimeState,
    types::{
        CONTROL_REPLY_TIMEOUT, EventTopic, LOAD_ROM_REPLY_TIMEOUT, RuntimeConfig, RuntimeError,
        RuntimeEventSender, SAVE_STATE_REPLY_TIMEOUT, TileViewerBackground, TileViewerLayout,
        TileViewerSource, VideoConfig,
    },
    util::button_bit,
};

struct RuntimeInner {
    ctrl_tx: Sender<ControlMessage>,
    frame_handle: Option<Arc<ExternalFrameHandle>>,
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
        Self::start_internal(config, None)
    }
}

impl Runtime {
    pub fn start_with_sender(
        config: RuntimeConfig,
        sender: Box<dyn RuntimeEventSender>,
    ) -> Result<Self, RuntimeError> {
        Self::start_internal(config, Some(sender))
    }

    pub fn start_pending(config: RuntimeConfig) -> Result<Self, RuntimeError> {
        Self::start_internal(config, None)
    }

    fn start_internal(
        config: RuntimeConfig,
        event_sender: Option<Box<dyn RuntimeEventSender>>,
    ) -> Result<Self, RuntimeError> {
        let (ctrl_tx, ctrl_rx) = unbounded::<ControlMessage>();

        let (framebuffer, frame_handle) = match config.video {
            VideoConfig::External(video) => {
                let len = video.len_bytes();
                if len == 0 {
                    return Err(RuntimeError::VideoBufferLenZero);
                }
                assert!(
                    video.pitch_bytes >= video.expected_pitch_bytes(),
                    "video pitch_bytes is smaller than the expected minimum pitch"
                );
                let (fb, handle) = unsafe {
                    FrameBuffer::new_external(
                        video.color_format,
                        video.pitch_bytes,
                        video.plane0,
                        video.plane1,
                    )
                };
                (fb, Some(handle))
            }
            VideoConfig::Swapchain(video) => {
                let fb = FrameBuffer::new_swapchain(
                    video.color_format,
                    video.lock,
                    video.unlock,
                    video.user_data,
                );
                (fb, None)
            }
        };

        let state = Arc::new(RuntimeState::new());
        let thread_state = Arc::clone(&state);
        let audio_mode = config.audio;

        let mut pubsub = RuntimePubSub::new();
        if let Some(sender) = event_sender {
            // By default, a monolithic sender subscribes to everything we know about.
            pubsub.subscribe(EventTopic::Notification, sender);
        }

        let ctrl_tx_clone = ctrl_tx.clone();
        let join = thread::spawn(move || {
            let mut runner = Runner::new(
                audio_mode,
                ctrl_rx,
                ctrl_tx_clone,
                pubsub,
                framebuffer,
                thread_state,
            );
            runner.run();
        });

        let inner = Arc::new(RuntimeInner {
            ctrl_tx,
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

    pub fn subscribe_event(
        &self,
        topic: EventTopic,
        sender: Box<dyn RuntimeEventSender>,
    ) -> Result<(), RuntimeError> {
        self.send_with_reply("subscribe_event", CONTROL_REPLY_TIMEOUT, |reply| {
            ControlMessage::SubscribeEvent(topic, sender, reply)
        })
    }

    pub fn unsubscribe_event(&self, topic: EventTopic) -> Result<(), RuntimeError> {
        self.send_with_reply("unsubscribe_event", CONTROL_REPLY_TIMEOUT, |reply| {
            ControlMessage::UnsubscribeEvent(topic, reply)
        })
    }

    pub fn set_tilemap_capture_point(
        &self,
        point: TilemapCapturePoint,
    ) -> Result<(), RuntimeError> {
        self.send_with_reply(
            "set_tilemap_capture_point",
            CONTROL_REPLY_TIMEOUT,
            |reply| ControlMessage::SetTilemapCapturePoint(point, reply),
        )
    }

    pub fn set_tile_viewer_capture_point(
        &self,
        point: TileViewerCapturePoint,
    ) -> Result<(), RuntimeError> {
        self.send_with_reply(
            "set_tile_viewer_capture_point",
            CONTROL_REPLY_TIMEOUT,
            |reply| ControlMessage::SetTileViewerCapturePoint(point, reply),
        )
    }

    pub fn set_sprite_capture_point(&self, point: SpriteCapturePoint) -> Result<(), RuntimeError> {
        self.send_with_reply("set_sprite_capture_point", CONTROL_REPLY_TIMEOUT, |reply| {
            ControlMessage::SetSpriteCapturePoint(point, reply)
        })
    }

    pub fn set_palette_capture_point(
        &self,
        point: PaletteCapturePoint,
    ) -> Result<(), RuntimeError> {
        self.send_with_reply(
            "set_palette_capture_point",
            CONTROL_REPLY_TIMEOUT,
            |reply| ControlMessage::SetPaletteCapturePoint(point, reply),
        )
    }

    pub fn set_tile_viewer_source(&self, source: TileViewerSource) -> Result<(), RuntimeError> {
        self.send_with_reply("set_tile_viewer_source", CONTROL_REPLY_TIMEOUT, |reply| {
            ControlMessage::SetTileViewerSource(source, reply)
        })
    }

    pub fn set_tile_viewer_start_address(&self, start_address: u32) -> Result<(), RuntimeError> {
        self.send_with_reply(
            "set_tile_viewer_start_address",
            CONTROL_REPLY_TIMEOUT,
            |reply| ControlMessage::SetTileViewerStartAddress(start_address, reply),
        )
    }

    pub fn set_tile_viewer_size(&self, columns: u16, rows: u16) -> Result<(), RuntimeError> {
        self.send_with_reply("set_tile_viewer_size", CONTROL_REPLY_TIMEOUT, |reply| {
            ControlMessage::SetTileViewerSize {
                columns,
                rows,
                reply,
            }
        })
    }

    pub fn set_tile_viewer_layout(&self, layout: TileViewerLayout) -> Result<(), RuntimeError> {
        self.send_with_reply("set_tile_viewer_layout", CONTROL_REPLY_TIMEOUT, |reply| {
            ControlMessage::SetTileViewerLayout(layout, reply)
        })
    }

    pub fn set_tile_viewer_background(
        &self,
        background: TileViewerBackground,
    ) -> Result<(), RuntimeError> {
        self.send_with_reply(
            "set_tile_viewer_background",
            CONTROL_REPLY_TIMEOUT,
            |reply| ControlMessage::SetTileViewerBackground(background, reply),
        )
    }

    pub fn set_tile_viewer_palette(&self, palette: u8) -> Result<(), RuntimeError> {
        self.send_with_reply("set_tile_viewer_palette", CONTROL_REPLY_TIMEOUT, |reply| {
            ControlMessage::SetTileViewerPalette(palette, reply)
        })
    }

    pub fn set_tile_viewer_use_grayscale_palette(&self, enabled: bool) -> Result<(), RuntimeError> {
        self.send_with_reply(
            "set_tile_viewer_use_grayscale_palette",
            CONTROL_REPLY_TIMEOUT,
            |reply| ControlMessage::SetTileViewerUseGrayscalePalette(enabled, reply),
        )
    }

    pub fn frame_handle(&self) -> Option<&Arc<ExternalFrameHandle>> {
        self.inner.frame_handle.as_ref()
    }

    pub fn frame_seq(&self) -> u64 {
        self.inner
            .state
            .frame_seq
            .load(std::sync::atomic::Ordering::Relaxed)
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

    pub fn rom_hash(&self) -> Option<[u8; 32]> {
        *self.inner.state.rom_hash.lock().unwrap()
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
        self.set_turbo_timing(frames, frames);
    }

    /// Configure turbo as an ON/OFF cycle.
    ///
    /// Example: `on_frames=2, off_frames=1` means press for 2 frames, release for 1 frame.
    pub fn set_turbo_timing(&self, on_frames: u8, off_frames: u8) {
        use std::sync::atomic::Ordering;
        self.inner
            .state
            .turbo_on_frames
            .store(on_frames.max(1), Ordering::Release);
        self.inner
            .state
            .turbo_off_frames
            .store(off_frames.max(1), Ordering::Release);
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

    pub fn set_palette_kind(&self, kind: PaletteKind) -> Result<(), RuntimeError> {
        self.send_with_reply("set_palette_kind", CONTROL_REPLY_TIMEOUT, |reply| {
            ControlMessage::SetPaletteKind(kind, reply)
        })
    }

    pub fn set_palette_from_pal_data(&self, data: &[u8]) -> Result<(), RuntimeError> {
        let palette = Palette::from_pal_data(data).map_err(|e| match e {
            nesium_core::error::Error::InvalidPaletteSize { actual } => {
                RuntimeError::InvalidPaletteSize { actual }
            }
            _ => RuntimeError::InvalidPaletteData {
                error: e.to_string(),
            },
        })?;

        self.send_with_reply("set_palette", CONTROL_REPLY_TIMEOUT, |reply| {
            ControlMessage::SetPalette(palette, reply)
        })
    }

    pub fn set_palette_from_pal_file(&self, path: impl Into<PathBuf>) -> Result<(), RuntimeError> {
        let path = path.into();
        let data = std::fs::read(&path).map_err(|e| RuntimeError::LoadPaletteFailed {
            path: path.clone(),
            error: e.to_string(),
        })?;

        let palette = Palette::from_pal_data(&data).map_err(|e| match e {
            nesium_core::error::Error::InvalidPaletteSize { actual } => {
                RuntimeError::InvalidPaletteSize { actual }
            }
            _ => RuntimeError::LoadPaletteFailed {
                path: path.clone(),
                error: e.to_string(),
            },
        })?;

        self.send_with_reply("set_palette", CONTROL_REPLY_TIMEOUT, |reply| {
            ControlMessage::SetPalette(palette, reply)
        })
    }

    pub fn save_state(&self, path: impl Into<PathBuf>) -> Result<(), RuntimeError> {
        let path = path.into();
        self.send_with_reply("save_state", SAVE_STATE_REPLY_TIMEOUT, |reply| {
            ControlMessage::SaveState(path, reply)
        })
    }

    pub fn load_state(&self, path: impl Into<PathBuf>) -> Result<(), RuntimeError> {
        let path = path.into();
        self.send_with_reply("load_state", SAVE_STATE_REPLY_TIMEOUT, |reply| {
            ControlMessage::LoadState(path, reply)
        })
    }

    pub fn save_state_to_memory(&self) -> Result<Vec<u8>, RuntimeError> {
        let (reply_tx, reply_rx) = bounded::<Result<Vec<u8>, RuntimeError>>(1);
        self.inner
            .ctrl_tx
            .send(ControlMessage::SaveStateToMemory(reply_tx))
            .map_err(|_| RuntimeError::ControlChannelDisconnected)?;
        match reply_rx.recv_timeout(SAVE_STATE_REPLY_TIMEOUT) {
            Ok(res) => res,
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                Err(RuntimeError::ControlTimeout {
                    op: "save_state_to_memory",
                })
            }
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                Err(RuntimeError::ControlChannelDisconnected)
            }
        }
    }

    pub fn load_state_from_memory(&self, data: Vec<u8>) -> Result<(), RuntimeError> {
        self.send_with_reply(
            "load_state_from_memory",
            SAVE_STATE_REPLY_TIMEOUT,
            |reply| ControlMessage::LoadStateFromMemory(data, reply),
        )
    }

    pub fn set_rewind_config(&self, enabled: bool, capacity: u64) {
        self.inner
            .state
            .rewind_enabled
            .store(enabled, Ordering::Release);
        self.inner
            .state
            .rewind_capacity
            .store(capacity, Ordering::Release);
    }

    pub fn set_rewinding(&self, rewinding: bool) -> Result<(), RuntimeError> {
        self.send_with_reply("set_rewinding", CONTROL_REPLY_TIMEOUT, |reply| {
            ControlMessage::SetRewinding(rewinding, reply)
        })
    }

    pub fn load_movie(&self, movie: nesium_support::tas::Movie) -> Result<(), RuntimeError> {
        self.send_with_reply("load_movie", CONTROL_REPLY_TIMEOUT, |reply| {
            ControlMessage::LoadMovie(movie, reply)
        })
    }

    /// Enables the debugger with the given debug channels.
    ///
    /// Returns the receiver for debug events that the UI should monitor.
    /// The caller should hold onto the `Sender<DebugCommand>` to send commands.
    pub fn enable_debugger(
        &self,
        debug_rx: crossbeam_channel::Receiver<super::debug::DebugCommand>,
        debug_tx: crossbeam_channel::Sender<super::debug::DebugEvent>,
    ) -> Result<(), RuntimeError> {
        let (reply_tx, reply_rx) = bounded::<Result<(), RuntimeError>>(1);
        self.inner
            .ctrl_tx
            .send(ControlMessage::EnableDebugger {
                debug_rx,
                debug_tx,
                reply: reply_tx,
            })
            .map_err(|_| RuntimeError::ControlChannelDisconnected)?;
        match reply_rx.recv_timeout(CONTROL_REPLY_TIMEOUT) {
            Ok(res) => res,
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                Err(RuntimeError::ControlTimeout {
                    op: "enable_debugger",
                })
            }
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                Err(RuntimeError::ControlChannelDisconnected)
            }
        }
    }

    /// Disables the debugger and removes it from the interceptor stack.
    pub fn disable_debugger(&self) -> Result<(), RuntimeError> {
        self.send_with_reply("disable_debugger", CONTROL_REPLY_TIMEOUT, |reply| {
            ControlMessage::DisableDebugger(reply)
        })
    }
}
