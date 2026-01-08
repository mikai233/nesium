//! Unified message size limits for the netplay protocol.
//!
//! This module provides a single source of truth for all size limits,
//! eliminating scattered constants and ensuring consistency across crates.

use crate::msg_id::MsgId;

// ============================================================================
// Core Size Limits
// ============================================================================

/// Maximum payload size for control messages (4 KB).
///
/// Control messages include: handshake, session management, input batches,
/// ping/pong, and other small protocol messages.
pub const MAX_CONTROL_PAYLOAD: usize = 4 * 1024;

/// Maximum payload size for data messages (2 MB).
///
/// Data messages include: ROM data, state snapshots, and other large transfers.
pub const MAX_DATA_PAYLOAD: usize = 2 * 1024 * 1024;

/// Maximum payload size for UDP packets (1200 bytes).
///
/// Kept below typical path MTU to reduce fragmentation risk.
pub const MAX_UDP_PAYLOAD: usize = 1200;

// ============================================================================
// Derived Limits
// ============================================================================

/// Maximum TCP frame size (header + payload).
///
/// This is the absolute maximum size of a single framed TCP packet.
pub const MAX_TCP_FRAME: usize = crate::constants::HEADER_LEN + MAX_DATA_PAYLOAD;

/// TCP receive buffer size.
///
/// Set to accommodate one maximum-size data frame plus some margin
/// for partial frames and protocol overhead.
pub const TCP_RX_BUFFER_SIZE: usize = MAX_DATA_PAYLOAD + 64 * 1024;

// ============================================================================
// Message Classification
// ============================================================================

/// Returns `true` if the given message ID represents a "data" message
/// that may carry large payloads (ROM, state snapshots, etc.).
///
/// Data messages use [`MAX_DATA_PAYLOAD`] as their size limit.
/// All other messages are considered "control" messages and use
/// [`MAX_CONTROL_PAYLOAD`].
#[inline]
pub const fn is_data_message(msg_id: MsgId) -> bool {
    matches!(
        msg_id,
        MsgId::LoadRom
            | MsgId::RomLoaded
            | MsgId::SyncState
            | MsgId::ProvideState
            | MsgId::SnapshotFrag
            | MsgId::BeginCatchUp
    )
}

/// Returns the maximum payload size allowed for the given message ID.
///
/// - Data messages (ROM, snapshots, etc.): [`MAX_DATA_PAYLOAD`] (2 MB)
/// - Control messages (handshake, inputs, etc.): [`MAX_CONTROL_PAYLOAD`] (4 KB)
#[inline]
pub const fn max_payload_for(msg_id: MsgId) -> usize {
    if is_data_message(msg_id) {
        MAX_DATA_PAYLOAD
    } else {
        MAX_CONTROL_PAYLOAD
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_messages_use_large_limit() {
        assert!(is_data_message(MsgId::LoadRom));
        assert!(is_data_message(MsgId::RomLoaded));
        assert!(is_data_message(MsgId::SyncState));
        assert!(is_data_message(MsgId::ProvideState));
        assert!(is_data_message(MsgId::SnapshotFrag));
        assert!(is_data_message(MsgId::BeginCatchUp));

        assert_eq!(max_payload_for(MsgId::LoadRom), MAX_DATA_PAYLOAD);
    }

    #[test]
    fn control_messages_use_small_limit() {
        assert!(!is_data_message(MsgId::Hello));
        assert!(!is_data_message(MsgId::Ping));
        assert!(!is_data_message(MsgId::InputBatch));
        assert!(!is_data_message(MsgId::JoinRoom));

        assert_eq!(max_payload_for(MsgId::Ping), MAX_CONTROL_PAYLOAD);
    }

    #[test]
    fn rx_buffer_larger_than_max_frame() {
        assert!(TCP_RX_BUFFER_SIZE > MAX_TCP_FRAME);
    }
}
