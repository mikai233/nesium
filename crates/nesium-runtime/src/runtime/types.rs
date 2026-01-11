use core::ffi::c_void;
use std::{any::Any, path::PathBuf, time::Duration};

use nesium_core::cartridge::header::Mirroring;
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TilemapState {
    /// Character Internal RAM (CIRAM) - 2 KiB nametable data (0-0x7FF).
    /// Nametable 0 is at offset 0x000, nametable 1 at 0x400.
    pub ciram: Vec<u8>,
    pub palette: [u8; 32],
    pub chr: Vec<u8>,
    pub mirroring: Mirroring,
    /// 64-entry BGRA palette for aux texture rendering (matches CVPixelBuffer format).
    pub bgra_palette: [[u8; 4]; 64],
    /// Background pattern table base address ($0000 or $1000).
    pub bg_pattern_base: u16,
    /// PPU internal VRAM address (`v` register, 15 bits).
    pub vram_addr: u16,
    /// PPU temporary VRAM address (`t` register, 15 bits).
    pub temp_addr: u16,
    /// Fine X scroll (`x` register, 0..7).
    pub fine_x: u8,
}

impl Default for TilemapState {
    fn default() -> Self {
        Self {
            ciram: Vec::new(),
            palette: [0; 32],
            chr: Vec::new(),
            mirroring: Mirroring::Horizontal,
            bgra_palette: [[0; 4]; 64],
            bg_pattern_base: 0,
            vram_addr: 0,
            temp_addr: 0,
            fine_x: 0,
        }
    }
}

impl Event for TilemapState {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileViewerSource {
    /// PPU-visible pattern table bytes ($0000-$1FFF after mapper banking).
    Ppu,
    /// Cartridge CHR ROM bytes (unbanked).
    ChrRom,
    /// Cartridge CHR RAM bytes (unbanked).
    ChrRam,
    /// Cartridge PRG ROM bytes.
    PrgRom,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileViewerLayout {
    Normal,
    SingleLine8x16,
    SingleLine16x16,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileViewerBackground {
    Default,
    Transparent,
    PaletteColor,
    Black,
    White,
    Magenta,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileViewerConfig {
    pub source: TileViewerSource,
    pub start_address: u32,
    pub column_count: u16,
    pub row_count: u16,
    pub layout: TileViewerLayout,
    pub background: TileViewerBackground,
    pub selected_palette: u8,
    pub use_grayscale_palette: bool,
}

impl Default for TileViewerConfig {
    fn default() -> Self {
        Self {
            source: TileViewerSource::Ppu,
            start_address: 0,
            column_count: 16,
            row_count: 32,
            layout: TileViewerLayout::Normal,
            background: TileViewerBackground::Default,
            selected_palette: 0,
            use_grayscale_palette: false,
        }
    }
}

/// Tile Viewer state for Flutter inspection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TileState {
    /// Rendered tile view (platform-native RGBA/BGRA) matching the aux texture.
    /// May be empty if rendering is deferred to worker thread.
    pub rgba: Vec<u8>,
    /// Raw source bytes for tile rendering (passed to worker for deferred rendering).
    pub source_bytes: Vec<u8>,
    /// Tile view width (pixels).
    pub width: u16,
    /// Tile view height (pixels).
    pub height: u16,
    /// Selected source kind for the current view.
    pub source: TileViewerSource,
    /// Total size of the selected source memory, in bytes.
    pub source_size: u32,
    /// Start address within the selected source memory.
    pub start_address: u32,
    /// Number of tiles (8×8) per row in the view.
    pub column_count: u16,
    /// Number of tile rows (8×8) in the view.
    pub row_count: u16,
    pub layout: TileViewerLayout,
    pub background: TileViewerBackground,
    /// 32-byte palette RAM (NES internal palette).
    pub palette: [u8; 32],
    /// 64-entry BGRA palette for aux texture rendering.
    pub bgra_palette: [[u8; 4]; 64],
    /// Currently selected palette index (0-7: 0-3 for BG, 4-7 for sprites).
    pub selected_palette: u8,
    /// Whether to display using a grayscale palette.
    pub use_grayscale_palette: bool,
    /// Background pattern table base ($0000 or $1000).
    pub bg_pattern_base: u16,
    /// Sprite pattern table base ($0000 or $1000).
    pub sprite_pattern_base: u16,
    /// Whether sprites use 8×16 mode.
    pub large_sprites: bool,
}

impl Default for TileState {
    fn default() -> Self {
        Self {
            rgba: Vec::new(),
            source_bytes: Vec::new(),
            width: 0,
            height: 0,
            source: TileViewerSource::Ppu,
            source_size: 0,
            start_address: 0,
            column_count: 16,
            row_count: 32,
            layout: TileViewerLayout::Normal,
            background: TileViewerBackground::Default,
            palette: [0; 32],
            bgra_palette: [[0; 4]; 64],
            selected_palette: 0,
            use_grayscale_palette: false,
            bg_pattern_base: 0,
            sprite_pattern_base: 0,
            large_sprites: false,
        }
    }
}

impl Event for TileState {}

// =====================
// Sprite Viewer Types
// =====================

/// Information about a single OAM sprite.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpriteInfo {
    /// Sprite index in OAM (0-63).
    pub index: u8,
    /// X position on screen (0-255, but can render partially offscreen).
    pub x: u8,
    /// Y position on screen (0-239 visible, 0xEF-0xFF = offscreen top).
    pub y: u8,
    /// Tile index in pattern table.
    pub tile_index: u8,
    /// Palette index (0-3 for sprites, which maps to $3F10-$3F1F).
    pub palette: u8,
    /// Horizontal flip.
    pub flip_h: bool,
    /// Vertical flip.
    pub flip_v: bool,
    /// Priority: false = in front of background, true = behind background.
    pub behind_bg: bool,
    /// Whether sprite is visible on screen (Y not in hidden range).
    pub visible: bool,
}

/// Sprite Viewer state for Flutter inspection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteState {
    /// Information about all 64 sprites.
    pub sprites: Vec<SpriteInfo>,
    /// Rendered screen preview with sprites at their positions (BGRA, 256x256).
    /// The visible picture is the top 256x240 portion; the extra 16 pixels are below.
    /// May be empty if rendering is deferred to worker thread.
    pub screen_rgba: Vec<u8>,
    /// Screen preview width.
    pub screen_width: u16,
    /// Screen preview height.
    pub screen_height: u16,
    /// Rendered sprite thumbnails (BGRA, 64 sprites × 8x8 or 8x16 each).
    /// May be empty if rendering is deferred to worker thread.
    pub thumbnails_rgba: Vec<u8>,
    /// Thumbnail width per sprite.
    pub thumbnail_width: u8,
    /// Thumbnail height per sprite.
    pub thumbnail_height: u8,
    /// Whether 8x16 sprite mode is active.
    pub large_sprites: bool,
    /// Sprite pattern table base ($0000 or $1000, only for 8x8 mode).
    pub pattern_base: u16,
    /// 64-entry BGRA palette.
    pub bgra_palette: [[u8; 4]; 64],
    /// Raw OAM data (256 bytes, for deferred rendering).
    pub oam: Vec<u8>,
    /// Raw CHR data (for deferred rendering).
    pub chr: Vec<u8>,
    /// 32-byte palette RAM (for deferred rendering).
    pub palette: [u8; 32],
}

impl Default for SpriteState {
    fn default() -> Self {
        Self {
            sprites: Vec::new(),
            screen_rgba: Vec::new(),
            screen_width: 256,
            screen_height: 240,
            thumbnails_rgba: Vec::new(),
            thumbnail_width: 8,
            thumbnail_height: 8,
            large_sprites: false,
            pattern_base: 0,
            bgra_palette: [[0; 4]; 64],
            oam: Vec::new(),
            chr: Vec::new(),
            palette: [0; 32],
        }
    }
}

impl Event for SpriteState {}

// =====================
// Palette Viewer Types
// =====================

/// Palette Viewer state for Flutter inspection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteState {
    /// 32-byte palette RAM (NES internal palette).
    pub palette: [u8; 32],
    /// 64-entry BGRA palette for rendering.
    pub bgra_palette: [[u8; 4]; 64],
}

impl Default for PaletteState {
    fn default() -> Self {
        Self {
            palette: [0; 32],
            bgra_palette: [[0; 4]; 64],
        }
    }
}

impl Event for PaletteState {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmulationStatus {
    pub paused: bool,
    pub rewinding: bool,
    pub fast_forwarding: bool,
}

impl Event for EmulationStatus {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayEvent {
    QuickSave,
    QuickLoad,
}

impl Event for ReplayEvent {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventTopic {
    Notification,
    DebugState,
    Tilemap,
    Tile,
    Sprite,
    Palette,
    EmulationStatus,
    Replay,
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
