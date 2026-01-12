use serde::{Deserialize, Serialize};

use crate::channel::ChannelKind;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportKind {
    Tcp,
    Quic,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Hello {
    pub client_nonce: u32,
    pub transport: TransportKind,
    pub proto_min: u8,
    pub proto_max: u8,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Welcome {
    pub server_nonce: u32,
    /// Token used to attach additional channels (e.g., input/bulk) for the same session.
    pub session_token: u64,
    pub assigned_client_id: u32,
    pub room_id: u32,
    pub tick_hz: u16,
    pub input_delay_frames: u8,
    pub max_payload: u16,
    pub rewind_capacity: u32,
}

/// Attach a secondary channel connection to an existing session.
///
/// For TCP fallback, clients may open multiple TCP connections and send this message
/// on the new connection as the first packet.
#[derive(Serialize, Deserialize, Debug)]
pub struct AttachChannel {
    pub session_token: u64,
    pub channel: ChannelKind,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JoinRoom {
    pub room_code: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JoinAck {
    pub ok: bool,
    pub player_index: u8,
    pub start_frame: u32,
    pub room_id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Leave {
    pub reason_code: u8,
}

/// Server error codes sent to clients.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCode {
    /// Unspecified error
    Unknown = 0,
    /// Message parsing/decoding failed
    BadMessage = 1,
    /// Room with given code does not exist
    RoomNotFound = 2,
    /// Room is at maximum player capacity
    RoomFull = 3,
    /// Client is already in a room
    AlreadyInRoom = 4,
    /// Client is not in any room
    NotInRoom = 5,
    /// Operation requires different permissions (e.g., host-only)
    PermissionDenied = 6,
    /// Cannot perform action while game is running
    GameAlreadyStarted = 7,
    /// Invalid game or protocol state for this operation
    InvalidState = 8,
}

/// Server sends an error response to the client.
#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorMsg {
    pub code: ErrorCode,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SwitchRole {
    /// 0-3 for player index, or `SPECTATOR_PLAYER_INDEX` for spectator.
    pub new_role: u8,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoleChanged {
    pub client_id: u32,
    /// 0-3 for player index, or `SPECTATOR_PLAYER_INDEX` for spectator.
    pub new_role: u8,
}

/// Server notifies clients that a player has left the room.
#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerLeft {
    pub client_id: u32,
    /// The player's previous index (0-3), or `SPECTATOR_PLAYER_INDEX` if spectator.
    pub player_index: u8,
}

/// Server notifies clients that a new player has joined the room.
#[derive(Serialize, Deserialize, Debug)]
pub struct PlayerJoined {
    pub client_id: u32,
    /// The player's index (0-3), or `SPECTATOR_PLAYER_INDEX` if spectator.
    pub player_index: u8,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoadRom {
    pub data: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RomLoaded;

#[derive(Serialize, Deserialize, Debug)]
pub struct StartGame {
    /// Bitmask of active player ports (bit N = player index N is present)
    pub active_ports_mask: u8,
}

/// Client requests to pause/resume the game for all players.
#[derive(Serialize, Deserialize, Debug)]
pub struct PauseGame {
    pub paused: bool,
}

/// Server broadcasts pause state to all players.
#[derive(Serialize, Deserialize, Debug)]
pub struct PauseSync {
    pub paused: bool,
}

/// Client requests to reset the game for all players.
#[derive(Serialize, Deserialize, Debug)]
pub struct ResetGame {
    /// 0 = Soft, 1 = Hard/Power
    pub kind: u8,
}

/// Server broadcasts reset to all players.
#[derive(Serialize, Deserialize, Debug)]
pub struct ResetSync {
    pub kind: u8,
}

/// Client requests current game state (for reconnection/late join).
#[derive(Serialize, Deserialize, Debug)]
pub struct RequestState;

/// Client provides game state snapshot to server.
#[derive(Serialize, Deserialize, Debug)]
pub struct ProvideState {
    pub frame: u32,
    pub data: Vec<u8>,
}

/// Server sends game state snapshot to a client.
#[derive(Serialize, Deserialize, Debug)]
pub struct SyncState {
    pub frame: u32,
    pub data: Vec<u8>,
}

/// Server tells a late joiner to begin catch-up playback.
///
/// The client is expected to:
/// - Ensure the ROM is loaded
/// - Apply the `SyncState` snapshot for `snapshot_frame`
/// - Replay inputs from `snapshot_frame` onward until it reaches `target_frame`
#[derive(Serialize, Deserialize, Debug)]
pub struct BeginCatchUp {
    /// The frame number corresponding to the snapshot that the client should start from.
    pub snapshot_frame: u32,
    /// The server's current target frame to catch up to (best-effort).
    pub target_frame: u32,
    /// Bitmask of controller ports that must be treated as active (bit 0..3).
    pub active_ports_mask: u8,
}
