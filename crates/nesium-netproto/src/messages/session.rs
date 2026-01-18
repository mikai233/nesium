use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

use crate::channel::ChannelKind;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportKind {
    Tcp,
    Quic,
}

/// Synchronization mode for netplay sessions.
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SyncMode {
    /// Wait for all players' confirmed inputs before advancing each frame.
    /// Best for low-latency networks (LAN, same region).
    #[default]
    Lockstep = 0,
    /// Predict remote inputs and rollback/resimulate on misprediction.
    /// Best for high-latency networks (cross-region, internet).
    Rollback = 1,
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
    pub room_id: u32,
    /// Preferred sync mode (if None, server decides based on room settings).
    pub preferred_sync_mode: Option<SyncMode>,
    /// Desired role at join time.
    ///
    /// 0-3 for a player slot, `0xFE` for auto-assign, or `0xFF` for spectator.
    pub desired_role: u8,
    /// True if the client already has the ROM loaded and does not need `LoadRom`.
    pub has_rom: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JoinAck {
    pub ok: bool,
    pub player_index: u8,
    pub start_frame: u32,
    pub room_id: u32,
    /// The sync mode that will be used for this room.
    pub sync_mode: SyncMode,
    /// If true, the client must keep its local port inactive and wait for `ActivatePort`
    /// before sending non-zero inputs (lockstep reconnect).
    pub pending_activation: bool,
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
    /// P2P host is not available (disconnected or never set)
    HostNotAvailable = 9,
    /// Server is at maximum capacity (no room IDs available)
    ServerFull = 10,
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

/// Host requests to change the room's synchronization mode.
/// Only valid before the game starts.
#[derive(Serialize, Deserialize, Debug)]
pub struct SetSyncMode {
    pub mode: SyncMode,
}

/// Server broadcasts sync mode change to all players.
#[derive(Serialize, Deserialize, Debug)]
pub struct SyncModeChanged {
    pub mode: SyncMode,
}

/// Client informs the server it finished catch-up and is ready for port activation.
#[derive(Serialize, Deserialize, Debug)]
pub struct RejoinReady {
    /// Best-effort: the frame the client believes it has caught up to.
    pub caught_up_to_frame: u32,
}

/// Server schedules a port to become active from a given frame.
#[derive(Serialize, Deserialize, Debug)]
pub struct ActivatePort {
    pub player_index: u8,
    pub active_from_frame: u32,
}

/// Query room info by join code (before joining).
#[derive(Serialize, Deserialize, Debug)]
pub struct QueryRoom {
    pub request_id: u32,
    pub room_id: u32,
}

/// Room info response for a `QueryRoom` request.
#[derive(Serialize, Deserialize, Debug)]
pub struct RoomInfo {
    pub request_id: u32,
    pub ok: bool,
    pub room_id: u32,
    pub started: bool,
    pub sync_mode: SyncMode,
    /// Bitmask: bit N set if player slot N is occupied (0..3).
    pub occupied_mask: u8,
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

// ---- P2P signaling (netd as signaling server) ----

/// Host asks the signaling server to create a new relay room ID and publish direct-connect info.
#[derive(Serialize, Deserialize, Debug)]
pub struct P2PCreateRoom {
    /// Addresses peers should try when connecting to the host server directly.
    ///
    /// This is an ordered list; clients should try QUIC first (if fingerprint is present),
    /// then fall back to TCP, iterating addresses in order.
    pub host_addrs: Vec<SocketAddr>,
    /// Host server room ID to join when connecting directly to the host server.
    pub host_room_id: u32,
    /// Host QUIC certificate leaf SHA-256 fingerprint (base64url/hex/colon-hex are all accepted by netplay).
    pub host_quic_cert_sha256_fingerprint: Option<String>,
    /// SNI/server_name to use for QUIC connections (pinning mode does not rely on SAN validation).
    pub host_quic_server_name: Option<String>,
}

/// Signaling server response: allocated relay room ID.
#[derive(Serialize, Deserialize, Debug)]
pub struct P2PRoomCreated {
    /// Room ID clients should use for both signaling and (if needed) relay fallback on this server.
    pub room_id: u32,
}

/// Join a P2P room on the signaling server and obtain direct-connect info for the host.
#[derive(Serialize, Deserialize, Debug)]
pub struct P2PJoinRoom {
    pub room_id: u32,
}

/// Signaling server response containing host direct-connect information.
#[derive(Serialize, Deserialize, Debug)]
pub struct P2PJoinAck {
    pub ok: bool,
    pub room_id: u32,
    pub host_addrs: Vec<SocketAddr>,
    pub host_room_id: u32,
    pub host_quic_cert_sha256_fingerprint: Option<String>,
    pub host_quic_server_name: Option<String>,
    /// If true, clients should skip direct connect and immediately use relay mode on this server.
    pub fallback_required: bool,
    pub fallback_reason: Option<String>,
}

/// Request switching this room to relay fallback mode on the signaling server.
#[derive(Serialize, Deserialize, Debug)]
pub struct P2PRequestFallback {
    pub room_id: u32,
    pub reason: String,
}

/// Signaling server broadcast indicating relay fallback is required.
#[derive(Serialize, Deserialize, Debug)]
pub struct P2PFallbackNotice {
    pub room_id: u32,
    pub reason: String,
    pub requested_by_client_id: u32,
}

/// Signaling server notifies watchers that the P2P host has disconnected.
#[derive(Serialize, Deserialize, Debug)]
pub struct P2PHostDisconnected {
    pub room_id: u32,
}

/// Maximum number of host addresses allowed in P2PCreateRoom.
pub const P2P_MAX_HOST_ADDRS: usize = 8;

/// Maximum length of reason strings in P2P messages (bytes).
pub const P2P_MAX_REASON_LEN: usize = 256;

// ---- Direct-session control (host server -> clients) ----

/// Host asks its own server to broadcast a relay fallback instruction to all connected clients.
#[derive(Serialize, Deserialize, Debug)]
pub struct RequestFallbackRelay {
    pub relay_addr: SocketAddr,
    pub relay_room_id: u32,
    pub reason: String,
}

/// Server instructs clients to disconnect and reconnect to the relay server/room.
#[derive(Serialize, Deserialize, Debug)]
pub struct FallbackToRelay {
    pub relay_addr: SocketAddr,
    pub relay_room_id: u32,
    pub reason: String,
}
