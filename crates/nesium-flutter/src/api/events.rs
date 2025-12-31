use flutter_rust_bridge::frb;
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
