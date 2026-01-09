//! History Viewer API for Flutter.

use crate::runtime_handle;
use flutter_rust_bridge::frb;

/// Seeks to a specific frame in the rewind history for the History Viewer.
///
/// `position` is 0-indexed, where 0 is the oldest frame and `frame_count - 1` is the newest.
/// After calling this, subscribe to `history_state_stream` in `events.rs` to receive updates.
#[frb]
pub async fn history_seek(position: usize) -> Result<(), String> {
    runtime_handle()
        .history_seek(position)
        .map_err(|e| e.to_string())
}

#[frb]
pub async fn history_apply(position: usize) -> Result<(), String> {
    runtime_handle()
        .history_apply(position)
        .map_err(|e| e.to_string())
}
