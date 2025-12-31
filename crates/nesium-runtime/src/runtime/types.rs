use core::ffi::c_void;
use std::{any::Any, path::PathBuf, time::Duration};

use nesium_core::ppu::{
    SCREEN_HEIGHT, SCREEN_WIDTH,
    buffer::{ColorFormat, SwapchainLockCallback, SwapchainUnlockCallback},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioMode {
    Auto,
    Disabled,
}

#[derive(Debug, Clone, Copy)]
pub struct VideoExternalConfig {
    pub color_format: ColorFormat,
    /// Bytes per scanline for each plane.
    ///
    /// This can be larger than `SCREEN_WIDTH * bytes_per_pixel` (e.g. padded/strided buffers).
    pub pitch_bytes: usize,
    pub plane0: *mut u8,
    pub plane1: *mut u8,
}

impl VideoExternalConfig {
    #[inline]
    pub fn len_bytes(self) -> usize {
        self.pitch_bytes * SCREEN_HEIGHT
    }

    #[inline]
    pub fn expected_pitch_bytes(self) -> usize {
        SCREEN_WIDTH * self.color_format.bytes_per_pixel()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VideoSwapchainConfig {
    pub color_format: ColorFormat,
    pub lock: SwapchainLockCallback,
    pub unlock: SwapchainUnlockCallback,
    pub user_data: *mut c_void,
}

#[derive(Debug, Clone, Copy)]
pub enum VideoConfig {
    External(VideoExternalConfig),
    Swapchain(VideoSwapchainConfig),
}

#[derive(Debug, Clone, Copy)]
pub struct RuntimeConfig {
    pub video: VideoConfig,
    pub audio: AudioMode,
}

pub trait Event: Any + Send + Sync + std::fmt::Debug {}

#[derive(Debug, Clone)]
pub enum NotificationEvent {
    /// Out-of-band notification emitted by the runtime thread (not a direct response
    /// to a control command).
    AudioInitFailed { error: String },
}

impl Event for NotificationEvent {}

/// CPU register snapshot for debugging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CpuDebugState {
    pub pc: u16,
    pub a: u8,
    pub x: u8,
    pub y: u8,
    pub sp: u8,
    pub status: u8,
    pub cycle: u64,
}

/// PPU state snapshot for debugging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PpuDebugState {
    pub scanline: i16,
    pub cycle: u16,
    pub frame: u32,
    pub ctrl: u8,
    pub mask: u8,
    pub status: u8,
    pub oam_addr: u8,
    pub vram_addr: u16,
    pub temp_addr: u16,
    pub fine_x: u8,
}

/// Complete debug state (sent per-frame when subscribed).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DebugState {
    pub cpu: CpuDebugState,
    pub ppu: PpuDebugState,
}

impl Event for DebugState {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventTopic {
    Notification,
    DebugState,
}

impl NotificationEvent {
    pub fn topic(&self) -> EventTopic {
        match self {
            NotificationEvent::AudioInitFailed { .. } => EventTopic::Notification,
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum RuntimeError {
    #[error("video buffer length is zero")]
    VideoBufferLenZero,
    #[error("runtime control channel disconnected")]
    ControlChannelDisconnected,
    #[error("runtime did not respond in time for {op}")]
    ControlTimeout { op: &'static str },
    #[error("PAL is not supported yet")]
    PalNotSupported,
    #[error("unsupported integer FPS target: {fps}")]
    UnsupportedIntegerFpsTarget { fps: u32 },
    #[error("failed to load ROM: {path}: {error}")]
    LoadRomFailed { path: PathBuf, error: String },
    #[error("palette blobs must be 192 or 256 bytes (got {actual})")]
    InvalidPaletteSize { actual: usize },
    #[error("invalid palette data: {error}")]
    InvalidPaletteData { error: String },
    #[error("failed to load palette: {path}: {error}")]
    LoadPaletteFailed { path: PathBuf, error: String },
    #[error("failed to save state: {path}: {error}")]
    SaveStateFailed { path: PathBuf, error: String },
    #[error("failed to load state: {path}: {error}")]
    LoadStateFailed { path: PathBuf, error: String },
}

pub(crate) const NTSC_FPS_EXACT: f64 = 60.098_811_862_348_4;
pub(crate) const CONTROL_REPLY_TIMEOUT: Duration = Duration::from_secs(2);
pub(crate) const LOAD_ROM_REPLY_TIMEOUT: Duration = Duration::from_secs(10);
pub(crate) const SAVE_STATE_REPLY_TIMEOUT: Duration = Duration::from_secs(5);

pub trait RuntimeEventSender: Send + Sync + 'static {
    fn send(&self, event: Box<dyn Event>) -> bool;
}

impl<T: RuntimeEventSender + ?Sized> RuntimeEventSender for Box<T> {
    fn send(&self, event: Box<dyn Event>) -> bool {
        (**self).send(event)
    }
}

impl<T: RuntimeEventSender + ?Sized> RuntimeEventSender for std::sync::Arc<T> {
    fn send(&self, event: Box<dyn Event>) -> bool {
        (**self).send(event)
    }
}
