use flutter_rust_bridge::frb;
use nesium_core::interceptor::tilemap_capture_interceptor::TilemapCapturePoint;
use nesium_runtime::runtime::EventTopic;

use crate::frb_generated::StreamSink;
use crate::senders::{FlutterDebugEventSender, FlutterRuntimeEventSender};

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
    let handle = crate::runtime_handle();
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
    let handle = crate::runtime_handle();

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
    let handle = crate::runtime_handle();
    let sender = Box::new(crate::senders::TilemapTextureSender);

    handle
        .subscribe_event(EventTopic::Tilemap, sender)
        .map_err(|e| format!("Failed to subscribe to Tilemap events: {}", e))?;

    Ok(())
}

/// Unsubscribes from tilemap texture updates.
#[frb]
pub async fn unsubscribe_tilemap_texture() -> Result<(), String> {
    let handle = crate::runtime_handle();

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
    pub fine_x: u8,
}

/// Subscribes to tilemap state updates.
///
/// This also refreshes the tilemap auxiliary texture, so the UI can use a single subscription.
#[frb]
pub async fn tilemap_state_stream(sink: StreamSink<TilemapSnapshot>) -> Result<(), String> {
    let handle = crate::runtime_handle();
    let sender = Box::new(crate::senders::TilemapTextureAndStateSender::new(sink));

    handle
        .subscribe_event(EventTopic::Tilemap, sender)
        .map_err(|e| format!("Failed to subscribe to Tilemap events: {}", e))?;

    Ok(())
}

/// Use the PPU frame start (scanline 0, cycle 0) as the tilemap capture point.
#[frb]
pub async fn set_tilemap_capture_frame_start() -> Result<(), String> {
    crate::runtime_handle()
        .set_tilemap_capture_point(TilemapCapturePoint::FrameStart)
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Use the PPU VBlank start (scanline 241, cycle 1) as the tilemap capture point.
#[frb]
pub async fn set_tilemap_capture_vblank_start() -> Result<(), String> {
    crate::runtime_handle()
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
    crate::runtime_handle()
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
    crate::senders::set_tilemap_display_mode(mode);
    Ok(())
}
