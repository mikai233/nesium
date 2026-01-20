use flutter_rust_bridge::frb;
use nesium_core::interceptor::{
    palette_interceptor::CapturePoint as PaletteCapturePoint,
    sprite_interceptor::CapturePoint as SpriteCapturePoint,
    tile_viewer_interceptor::CapturePoint as TileViewerCapturePoint,
    tilemap_interceptor::CapturePoint as TilemapCapturePoint,
};
use nesium_runtime::runtime::EventTopic;

use nesium_runtime::{TileViewerBackground, TileViewerLayout};

use crate::frb_generated::StreamSink;
use crate::runtime_handle;
use crate::senders::debug::FlutterDebugEventSender;
use crate::senders::emulation_status::EmulationStatusSender;
use crate::senders::replay::ReplayEventNotification;
use crate::senders::runtime::FlutterRuntimeEventSender;
use crate::senders::tile::TileTextureAndStateSender;
use crate::senders::tilemap::{self, TilemapTextureAndStateSender, TilemapTextureSender};

// =============================================================================
// Auxiliary Texture IDs
// =============================================================================

#[frb]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AuxTextureIds {
    pub tilemap: u32,
    pub tile: u32,
    pub sprite: u32,
    pub sprite_screen: u32,
}

/// Returns all auxiliary texture IDs defined on the Rust side.
///
/// Flutter should treat these as the single source of truth and avoid hard-coding IDs.
#[frb]
pub async fn aux_texture_ids() -> AuxTextureIds {
    AuxTextureIds {
        tilemap: crate::senders::tilemap::TILEMAP_TEXTURE_ID,
        tile: crate::senders::tile::TILE_VIEWER_TEXTURE_ID,
        sprite: crate::senders::sprite::SPRITE_TEXTURE_ID,
        sprite_screen: crate::senders::sprite::SPRITE_SCREEN_TEXTURE_ID,
    }
}

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
// Emulation Status Stream (paused/rewind/fast-forward)
// =============================================================================

#[frb]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EmulationStatusNotification {
    pub paused: bool,
    pub rewinding: bool,
    pub fast_forwarding: bool,
}

/// Emulation status stream.
///
/// Used to notify Flutter UI about pause/rewind/fast-forward changes that are triggered on the Rust
/// side (e.g. desktop gamepad polling thread).
#[frb]
pub async fn emulation_status_stream(
    sink: StreamSink<EmulationStatusNotification>,
) -> Result<(), String> {
    let handle = crate::runtime_handle();
    let sender = Box::new(EmulationStatusSender::new(sink));

    handle
        .subscribe_event(EventTopic::EmulationStatus, sender)
        .map_err(|e| format!("Failed to subscribe to EmulationStatus events: {}", e))?;

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
    let sender = Box::new(TilemapTextureSender {});

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

// =============================================================================
// Tile Viewer Capture Point
// =============================================================================

/// Use the PPU frame start (scanline 0, cycle 0) as the Tile Viewer capture point.
#[frb]
pub async fn set_tile_viewer_capture_frame_start() -> Result<(), String> {
    runtime_handle()
        .set_tile_viewer_capture_point(TileViewerCapturePoint::FrameStart)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Use the PPU VBlank start (scanline 241, cycle 1) as the Tile Viewer capture point.
#[frb]
pub async fn set_tile_viewer_capture_vblank_start() -> Result<(), String> {
    runtime_handle()
        .set_tile_viewer_capture_point(TileViewerCapturePoint::VblankStart)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Use a specific scanline and dot as the Tile Viewer capture point.
#[frb]
pub async fn set_tile_viewer_capture_scanline(scanline: i32, dot: i32) -> Result<(), String> {
    if !(-1..=260).contains(&scanline) {
        return Err(format!("Invalid scanline: {}", scanline));
    }
    if !(0..=340).contains(&dot) {
        return Err(format!("Invalid dot: {}", dot));
    }
    runtime_handle()
        .set_tile_viewer_capture_point(TileViewerCapturePoint::ScanlineDot {
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
// Tile Stream
// =============================================================================

/// Tile snapshot for UI inspection (tile preview, palette selection, etc).
///
/// Note: `rgba_palette` is ALWAYS RGBA regardless of platform, so Flutter can render it easily.
#[frb]
#[derive(Debug, Clone)]
pub struct TileSnapshot {
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

/// Subscribes to Tile state updates.
///
/// This refreshes the Tile auxiliary texture, so the UI can use a single subscription.
#[frb]
pub async fn tile_state_stream(sink: StreamSink<TileSnapshot>) -> Result<(), String> {
    let handle = runtime_handle();
    let sender = Box::new(TileTextureAndStateSender::new(sink));

    handle
        .subscribe_event(EventTopic::Tile, sender)
        .map_err(|e| format!("Failed to subscribe to Tile events: {}", e))?;

    Ok(())
}

/// Unsubscribes from Tile state updates.
#[frb]
pub async fn unsubscribe_tile_state() -> Result<(), String> {
    let handle = runtime_handle();

    handle
        .unsubscribe_event(EventTopic::Tile)
        .map_err(|e| format!("Failed to unsubscribe from Tile events: {}", e))?;

    Ok(())
}

/// Sets the palette index for Tile Viewer rendering.
///
/// - `0-3`: Background palettes
/// - `4-7`: Sprite palettes
#[frb]
pub async fn set_tile_viewer_palette(palette_index: u8) -> Result<(), String> {
    if palette_index > 7 {
        return Err(format!("Invalid palette index: {}", palette_index));
    }
    runtime_handle()
        .set_tile_viewer_palette(palette_index)
        .map_err(|e| e.to_string())
}

/// Sets the display mode for Tile Viewer auxiliary texture.
///
/// - `0`: Default (use selected palette)
/// - `1`: Grayscale
#[frb]
pub async fn set_tile_viewer_display_mode(mode: u8) -> Result<(), String> {
    if mode > 1 {
        return Err(format!("Invalid Tile Viewer display mode: {}", mode));
    }
    runtime_handle()
        .set_tile_viewer_use_grayscale_palette(mode == 1)
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

    let handle = runtime_handle();
    let sender = Box::new(SpriteTextureAndStateSender::new(sink));

    handle
        .subscribe_event(EventTopic::Sprite, sender)
        .map_err(|e| format!("Failed to subscribe to Sprite events: {}", e))?;

    Ok(())
}

// =============================================================================
// Sprite Viewer Capture Point
// =============================================================================

/// Use the PPU frame start (scanline 0, cycle 0) as the sprite capture point.
#[frb]
pub async fn set_sprite_capture_frame_start() -> Result<(), String> {
    runtime_handle()
        .set_sprite_capture_point(SpriteCapturePoint::FrameStart)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Use the PPU VBlank start (scanline 241, cycle 1) as the sprite capture point.
#[frb]
pub async fn set_sprite_capture_vblank_start() -> Result<(), String> {
    runtime_handle()
        .set_sprite_capture_point(SpriteCapturePoint::VblankStart)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Use a specific scanline and dot as the sprite capture point.
#[frb]
pub async fn set_sprite_capture_scanline(scanline: i32, dot: i32) -> Result<(), String> {
    if !(-1..=260).contains(&scanline) {
        return Err(format!("Invalid scanline: {}", scanline));
    }
    if !(0..=340).contains(&dot) {
        return Err(format!("Invalid dot: {}", dot));
    }
    runtime_handle()
        .set_sprite_capture_point(SpriteCapturePoint::ScanlineDot {
            scanline: scanline as i16,
            dot: dot as u16,
        })
        .map_err(|e| e.to_string())?;
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

// =============================================================================
// Palette Viewer Stream
// =============================================================================

/// Palette snapshot for UI inspection.
///
/// Contains the 32-byte palette RAM and the 64-entry BGRA palette for rendering.
#[frb]
#[derive(Debug, Clone)]
pub struct PaletteSnapshot {
    /// 32-byte palette RAM (NES internal palette indices $00-$1F).
    pub palette: Vec<u8>,
    /// 64-entry BGRA palette flattened (256 bytes = 64 colors × 4 bytes per color).
    /// Format: [B0, G0, R0, A0, B1, G1, R1, A1, ...] for each color index 0-63.
    pub bgra_palette: Vec<u8>,
}

/// Subscribes to Palette state updates.
///
/// This streams the current palette to Flutter every frame.
#[frb]
pub async fn palette_state_stream(sink: StreamSink<PaletteSnapshot>) -> Result<(), String> {
    use crate::senders::palette::PaletteStateSender;

    let handle = runtime_handle();
    let sender = Box::new(PaletteStateSender::new(sink));

    handle
        .subscribe_event(EventTopic::Palette, sender)
        .map_err(|e| format!("Failed to subscribe to Palette events: {}", e))?;

    Ok(())
}

/// Unsubscribes from Palette state updates.
#[frb]
pub async fn unsubscribe_palette_state() -> Result<(), String> {
    let handle = runtime_handle();

    handle
        .unsubscribe_event(EventTopic::Palette)
        .map_err(|e| format!("Failed to unsubscribe from Palette events: {}", e))?;

    Ok(())
}

// =============================================================================
// Palette Viewer Capture Point
// =============================================================================

/// Use the PPU frame start (scanline 0, cycle 0) as the palette capture point.
#[frb]
pub async fn set_palette_capture_frame_start() -> Result<(), String> {
    runtime_handle()
        .set_palette_capture_point(PaletteCapturePoint::FrameStart)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Use the PPU VBlank start (scanline 241, cycle 1) as the palette capture point.
#[frb]
pub async fn set_palette_capture_vblank_start() -> Result<(), String> {
    runtime_handle()
        .set_palette_capture_point(PaletteCapturePoint::VblankStart)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Use a specific scanline and dot as the palette capture point.
#[frb]
pub async fn set_palette_capture_scanline(scanline: i32, dot: i32) -> Result<(), String> {
    if !(-1..=260).contains(&scanline) {
        return Err(format!("Invalid scanline: {}", scanline));
    }
    if !(0..=340).contains(&dot) {
        return Err(format!("Invalid dot: {}", dot));
    }
    runtime_handle()
        .set_palette_capture_point(PaletteCapturePoint::ScanlineDot {
            scanline: scanline as i16,
            dot: dot as u16,
        })
        .map_err(|e| e.to_string())?;
    Ok(())
}

// =============================================================================
// Replay Event Stream (QuickSave/QuickLoad)
// =============================================================================

#[frb]
#[frb]
pub async fn replay_event_stream(sink: StreamSink<ReplayEventNotification>) -> Result<(), String> {
    crate::senders::replay::set_replay_sink(sink);
    Ok(())
}
