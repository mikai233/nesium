use serde::{Deserialize, Serialize};

use crate::msg_id::MsgId;

/// Logical channels multiplexed over a transport.
///
/// - `Control`: handshake/session control and small messages.
/// - `Input`: time-sensitive inputs (minimize head-of-line blocking).
/// - `Bulk`: large transfers (ROM/state/snapshots).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChannelKind {
    Control = 0,
    Input = 1,
    Bulk = 2,
}

/// Map a message ID to its preferred logical channel.
///
/// Transports that do not support multiple channels should send everything on `Control`.
pub const fn channel_for_msg(msg_id: MsgId) -> ChannelKind {
    match msg_id {
        MsgId::InputBatch | MsgId::RelayInputs | MsgId::InputAck => ChannelKind::Input,

        // Large payloads / bulk sync.
        MsgId::LoadRom
        | MsgId::SyncState
        | MsgId::ProvideState
        | MsgId::SnapshotFrag
        | MsgId::BeginCatchUp => ChannelKind::Bulk,

        _ => ChannelKind::Control,
    }
}
