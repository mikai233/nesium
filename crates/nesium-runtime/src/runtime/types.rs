use std::{path::PathBuf, time::Duration};

use nesium_core::{
    ppu::{SCREEN_HEIGHT, SCREEN_WIDTH, buffer::ColorFormat},
    reset_kind::ResetKind,
};

pub use nesium_core::ppu::buffer::FrameReadyCallback;
pub use nesium_core::ppu::palette::PaletteKind;

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
    RomLoaded { path: PathBuf },
    RomLoadFailed { path: PathBuf, error: String },
    Reset { kind: ResetKind },
    Ejected,
    AudioInitFailed { error: String },
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
}

pub(crate) const NTSC_FPS_EXACT: f64 = 60.098_811_862_348_4;
pub(crate) const CONTROL_REPLY_TIMEOUT: Duration = Duration::from_secs(2);
pub(crate) const LOAD_ROM_REPLY_TIMEOUT: Duration = Duration::from_secs(10);
