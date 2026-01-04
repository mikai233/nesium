use flutter_rust_bridge::frb;
use nesium_core::interceptor::tilemap_capture_interceptor::TilemapCapturePoint;
use nesium_runtime::runtime::EventTopic;
use nesium_runtime::{TileViewerBackground, TileViewerLayout, TileViewerSource};

use crate::frb_generated::StreamSink;
use crate::runtime_handle;
use crate::senders::chr::ChrTextureAndStateSender;
use crate::senders::debug::FlutterDebugEventSender;
use crate::senders::runtime::FlutterRuntimeEventSender;
use crate::senders::tilemap::{self, TilemapTextureAndStateSender, TilemapTextureSender};

// =============================================================================
// Runtime Notification (for general events like audio init failure)
// =============================================================================
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeNotificationKind {
    AudioInitFailed,
}

#[derive(Debug, Clone)]
pub struct RuntimeNotification {
    pub kind: RuntimeNotificationKind,
    pub error: Option<String>,
}

/// Runtime notification stream.
///
/// This registers the sink directly with the runtime.
#[frb]
pub async fn runtime_notifications(sink: StreamSink<RuntimeNotification>) -> Result<(), String> {
    let handle = crate::runtime_handle();
    let sender = FlutterRuntimeEventSender::new(sink);

    handle
        .subscribe_event(EventTopic::Notification, Box::new(sender))
        .map_err(|e| format!("Failed to subscribe to Notification events: {}", e))?;

    Ok(())
}

// =============================================================================
// Debug State Stream
// =============================================================================

/// Debug state notification sent per-frame when subscribed.
#[frb]
#[derive(Debug, Clone)]
pub struct DebugStateNotification {
    pub cpu_pc: u16,
    pub cpu_a: u8,
    pub cpu_x: u8,
    pub cpu_y: u8,
    pub cpu_sp: u8,
    pub cpu_status: u8,
    pub cpu_cycle: u64,
    pub ppu_scanline: i16,
    pub ppu_cycle: u16,
    pub ppu_frame: u32,
    pub ppu_ctrl: u8,
    pub ppu_mask: u8,
    pub ppu_status: u8,
}

/// Subscribes to debug state updates (CPU/PPU registers per frame).
///
/// Call this when opening the debug panel. The stream will receive updates
/// every frame until cancelled.
#[frb]
pub async fn debug_state_stream(sink: StreamSink<DebugStateNotification>) -> Result<(), String> {
    let handle = runtime_handle();
    let sender = Box::new(FlutterDebugEventSender::new(sink));

    handle
        .subscribe_event(EventTopic::DebugState, sender)
        .map_err(|e| format!("Failed to subscribe to DebugState events: {}", e))?;

    Ok(())
}

/// Unsubscribes from debug state updates.
///
/// Call this when closing the debug panel to stop unnecessary computation.
#[frb]
pub async fn unsubscribe_debug_state() -> Result<(), String> {
    let handle = runtime_handle();

    handle
        .unsubscribe_event(EventTopic::DebugState)
        .map_err(|e| format!("Failed to unsubscribe from DebugState events: {}", e))?;

    Ok(())
}

// =============================================================================
// Tilemap Texture Subscription
// =============================================================================

/// Subscribes to tilemap texture updates.
///
/// This enables per-frame rendering of the tilemap to the auxiliary texture.
/// The actual pixel data is written directly to the texture buffer, not sent via stream.
#[frb]
pub async fn subscribe_tilemap_texture() -> Result<(), String> {
    let handle = runtime_handle();
    let sender = Box::new(TilemapTextureSender);

    handle
        .subscribe_event(EventTopic::Tilemap, sender)
        .map_err(|e| format!("Failed to subscribe to Tilemap events: {}", e))?;

    Ok(())
}

/// Unsubscribes from tilemap texture updates.
#[frb]
pub async fn unsubscribe_tilemap_texture() -> Result<(), String> {
    let handle = runtime_handle();

    handle
        .unsubscribe_event(EventTopic::Tilemap)
        .map_err(|e| format!("Failed to unsubscribe from Tilemap events: {}", e))?;

    Ok(())
}

// =============================================================================
// Tilemap State Stream (for inspection/hover info)
// =============================================================================

#[frb]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TilemapMirroring {
    Horizontal,
    Vertical,
    FourScreen,
    SingleScreenLower,
    SingleScreenUpper,
    MapperControlled,
}

/// Tilemap snapshot for UI inspection (hover/selection, tile preview, etc).
///
/// Note: `rgba_palette` is ALWAYS RGBA regardless of platform, so Flutter can render it easily.
#[frb]
#[derive(Debug, Clone)]
pub struct TilemapSnapshot {
    pub ciram: Vec<u8>,
    pub palette: Vec<u8>,
    pub chr: Vec<u8>,
    pub mirroring: TilemapMirroring,
    pub bg_pattern_base: u16,
    pub rgba_palette: Vec<u8>,
    pub vram_addr: u16,
    pub temp_addr: u16,
    pub fine_x: u8,
}

/// Subscribes to tilemap state updates.
///
/// This also refreshes the tilemap auxiliary texture, so the UI can use a single subscription.
#[frb]
pub async fn tilemap_state_stream(sink: StreamSink<TilemapSnapshot>) -> Result<(), String> {
    let handle = runtime_handle();
    let sender = Box::new(TilemapTextureAndStateSender::new(sink));

    handle
        .subscribe_event(EventTopic::Tilemap, sender)
        .map_err(|e| format!("Failed to subscribe to Tilemap events: {}", e))?;

    Ok(())
}

/// Use the PPU frame start (scanline 0, cycle 0) as the tilemap capture point.
#[frb]
pub async fn set_tilemap_capture_frame_start() -> Result<(), String> {
    runtime_handle()
        .set_tilemap_capture_point(TilemapCapturePoint::FrameStart)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Use the PPU VBlank start (scanline 241, cycle 1) as the tilemap capture point.
#[frb]
pub async fn set_tilemap_capture_vblank_start() -> Result<(), String> {
    runtime_handle()
        .set_tilemap_capture_point(TilemapCapturePoint::VblankStart)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Use a specific scanline and dot as the tilemap capture point.
#[frb]
pub async fn set_tilemap_capture_scanline(scanline: i32, dot: i32) -> Result<(), String> {
    if !(-1..=260).contains(&scanline) {
        return Err(format!("Invalid scanline: {}", scanline));
    }
    if !(0..=340).contains(&dot) {
        return Err(format!("Invalid dot: {}", dot));
    }
    runtime_handle()
        .set_tilemap_capture_point(TilemapCapturePoint::ScanlineDot {
            scanline: scanline as i16,
            dot: dot as u16,
        })
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Sets the render mode for the tilemap auxiliary texture.
///
/// - `0`: Default
/// - `1`: Grayscale
/// - `2`: Attribute view
#[frb]
pub async fn set_tilemap_display_mode(mode: u8) -> Result<(), String> {
    if mode > 2 {
        return Err(format!("Invalid tilemap display mode: {}", mode));
    }
    tilemap::set_tilemap_display_mode(mode);
    Ok(())
}

// =============================================================================
// CHR (Tile) Viewer Stream
// =============================================================================

/// CHR snapshot for UI inspection (tile preview, palette selection, etc).
///
/// Note: `rgba_palette` is ALWAYS RGBA regardless of platform, so Flutter can render it easily.
#[frb]
#[derive(Debug, Clone)]
pub struct ChrSnapshot {
    pub palette: Vec<u8>,
    pub rgba_palette: Vec<u8>,
    pub selected_palette: u8,
    pub width: u16,
    pub height: u16,
    /// `0..=3` as per `set_tile_viewer_source`.
    pub source: u8,
    pub source_size: u32,
    pub start_address: u32,
    pub column_count: u16,
    pub row_count: u16,
    /// `0..=2` as per `set_tile_viewer_layout`.
    pub layout: u8,
    /// `0..=5` as per `set_tile_viewer_background`.
    pub background: u8,
    pub use_grayscale_palette: bool,
    pub bg_pattern_base: u16,
    pub sprite_pattern_base: u16,
    pub large_sprites: bool,
}

/// Subscribes to CHR state updates.
///
/// This refreshes the CHR auxiliary texture, so the UI can use a single subscription.
#[frb]
pub async fn chr_state_stream(sink: StreamSink<ChrSnapshot>) -> Result<(), String> {
    let handle = runtime_handle();
    let sender = Box::new(ChrTextureAndStateSender::new(sink));

    handle
        .subscribe_event(EventTopic::Chr, sender)
        .map_err(|e| format!("Failed to subscribe to Chr events: {}", e))?;

    Ok(())
}

/// Unsubscribes from CHR state updates.
#[frb]
pub async fn unsubscribe_chr_state() -> Result<(), String> {
    let handle = runtime_handle();

    handle
        .unsubscribe_event(EventTopic::Chr)
        .map_err(|e| format!("Failed to unsubscribe from Chr events: {}", e))?;

    Ok(())
}

/// Sets the palette index for CHR rendering.
///
/// - `0-3`: Background palettes
/// - `4-7`: Sprite palettes
#[frb]
pub async fn set_chr_palette(palette_index: u8) -> Result<(), String> {
    if palette_index > 7 {
        return Err(format!("Invalid palette index: {}", palette_index));
    }
    runtime_handle()
        .set_tile_viewer_palette(palette_index)
        .map_err(|e| e.to_string())
}

/// Sets the display mode for CHR auxiliary texture.
///
/// - `0`: Default (use selected palette)
/// - `1`: Grayscale
#[frb]
pub async fn set_chr_display_mode(mode: u8) -> Result<(), String> {
    if mode > 1 {
        return Err(format!("Invalid CHR display mode: {}", mode));
    }
    runtime_handle()
        .set_tile_viewer_use_grayscale_palette(mode == 1)
        .map_err(|e| e.to_string())
}

/// Sets the CHR preset source for the Tile Viewer.
///
/// - `0`: PPU (current PPU-visible CHR at $0000-$1FFF)
/// - `1`: CHR (cartridge CHR ROM/RAM, first 8 KiB)
/// - `2`: ROM (cartridge PRG ROM, first 8 KiB)
#[frb]
pub async fn set_chr_source(source: u8) -> Result<(), String> {
    if source > 2 {
        return Err(format!("Invalid CHR source: {}", source));
    }

    let handle = runtime_handle();
    let src = match source {
        0 => TileViewerSource::Ppu,
        1 => TileViewerSource::ChrRam,
        2 => TileViewerSource::PrgRom,
        _ => TileViewerSource::Ppu,
    };

    handle
        .set_tile_viewer_source(src)
        .map_err(|e| e.to_string())
}

/// Sets the tile viewer source.
///
/// - `0`: PPU
/// - `1`: CHR ROM
/// - `2`: CHR RAM
/// - `3`: PRG ROM
#[frb]
pub async fn set_tile_viewer_source(source: u8) -> Result<(), String> {
    use nesium_runtime::TileViewerSource;
    let src = match source {
        0 => TileViewerSource::Ppu,
        1 => TileViewerSource::ChrRom,
        2 => TileViewerSource::ChrRam,
        3 => TileViewerSource::PrgRom,
        _ => return Err(format!("Invalid tile viewer source: {}", source)),
    };
    runtime_handle()
        .set_tile_viewer_source(src)
        .map_err(|e| e.to_string())
}

#[frb]
pub async fn set_tile_viewer_start_address(start_address: u32) -> Result<(), String> {
    runtime_handle()
        .set_tile_viewer_start_address(start_address)
        .map_err(|e| e.to_string())
}

#[frb]
pub async fn set_tile_viewer_size(columns: u16, rows: u16) -> Result<(), String> {
    runtime_handle()
        .set_tile_viewer_size(columns, rows)
        .map_err(|e| e.to_string())
}

/// Sets the tile layout.
///
/// - `0`: Normal
/// - `1`: SingleLine8x16
/// - `2`: SingleLine16x16
#[frb]
pub async fn set_tile_viewer_layout(layout: u8) -> Result<(), String> {
    let layout = match layout {
        0 => TileViewerLayout::Normal,
        1 => TileViewerLayout::SingleLine8x16,
        2 => TileViewerLayout::SingleLine16x16,
        _ => return Err(format!("Invalid tile layout: {}", layout)),
    };
    runtime_handle()
        .set_tile_viewer_layout(layout)
        .map_err(|e| e.to_string())
}

/// Sets the tile background.
///
/// - `0`: Default
/// - `1`: Transparent
/// - `2`: PaletteColor
/// - `3`: Black
/// - `4`: White
/// - `5`: Magenta
#[frb]
pub async fn set_tile_viewer_background(background: u8) -> Result<(), String> {
    let bg = match background {
        0 => TileViewerBackground::Default,
        1 => TileViewerBackground::Transparent,
        2 => TileViewerBackground::PaletteColor,
        3 => TileViewerBackground::Black,
        4 => TileViewerBackground::White,
        5 => TileViewerBackground::Magenta,
        _ => return Err(format!("Invalid tile background: {}", background)),
    };
    runtime_handle()
        .set_tile_viewer_background(bg)
        .map_err(|e| e.to_string())
}

// =============================================================================
// Sprite Viewer Stream
// =============================================================================

/// Information about a single OAM sprite.
#[frb]
#[derive(Debug, Clone)]
pub struct SpriteInfo {
    pub index: u8,
    pub x: u8,
    pub y: u8,
    pub tile_index: u8,
    pub palette: u8,
    pub flip_h: bool,
    pub flip_v: bool,
    pub behind_bg: bool,
    pub visible: bool,
}

/// Sprite snapshot for UI inspection.
///
/// Note: `rgba_palette` is ALWAYS RGBA regardless of platform, so Flutter can render it easily.
#[frb]
#[derive(Debug, Clone)]
pub struct SpriteSnapshot {
    pub sprites: Vec<SpriteInfo>,
    pub thumbnail_width: u8,
    pub thumbnail_height: u8,
    pub large_sprites: bool,
    pub pattern_base: u16,
    pub rgba_palette: Vec<u8>,
}

/// Subscribes to Sprite state updates.
///
/// This refreshes the Sprite auxiliary texture, so the UI can use a single subscription.
#[frb]
pub async fn sprite_state_stream(sink: StreamSink<SpriteSnapshot>) -> Result<(), String> {
    use crate::senders::sprite::SpriteTextureAndStateSender;
    use nesium_core::interceptor::tilemap_capture_interceptor::TilemapCapturePoint;

    let handle = runtime_handle();
    let sender = Box::new(SpriteTextureAndStateSender::new(sink));

    // Ensure tilemap capture is enabled at VBlank so we get OAM data each frame
    handle
        .set_tilemap_capture_point(TilemapCapturePoint::VblankStart)
        .map_err(|e| format!("Failed to set capture point: {}", e))?;

    handle
        .subscribe_event(EventTopic::Sprite, sender)
        .map_err(|e| format!("Failed to subscribe to Sprite events: {}", e))?;

    Ok(())
}

/// Unsubscribes from Sprite state updates.
#[frb]
pub async fn unsubscribe_sprite_state() -> Result<(), String> {
    let handle = runtime_handle();

    handle
        .unsubscribe_event(EventTopic::Sprite)
        .map_err(|e| format!("Failed to unsubscribe from Sprite events: {}", e))?;

    Ok(())
}
