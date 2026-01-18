//! Message ID definitions and protocol registry.
//!
//! This module uses the `define_protocol!` macro to generate:
//! - `MsgId` enum with auto-assigned values
//! - `Message` trait implementations for all message types
//! - `MessageKind` enum for type-erased dispatch
//! - `decode_message` function for deserializing by MsgId

use nesium_netproto_derive::define_protocol;

define_protocol! {
    // Session messages
    session: {
        Hello,
        Welcome,
        AttachChannel,
        JoinRoom,
        JoinAck,
        Leave,
        ErrorMsg,
        SwitchRole,
        RoleChanged,
        PlayerLeft,
        PlayerJoined,
        LoadRom,
        RomLoaded,
        StartGame,
        PauseGame,
        PauseSync,
        ResetGame,
        ResetSync,
        SetSyncMode,
        SyncModeChanged,
        RejoinReady,
        ActivatePort,
        QueryRoom,
        RoomInfo,
        RequestState,
        ProvideState,
        SyncState,
        BeginCatchUp,
        P2PCreateRoom,
        P2PRoomCreated,
        P2PJoinRoom,
        P2PJoinAck,
        P2PRequestFallback,
        P2PFallbackNotice,
        P2PHostDisconnected,
        RequestFallbackRelay,
        FallbackToRelay,
    },
    // Input messages
    input: {
        InputBatch,
        RelayInputs,
        InputAck,
    },
    // Sync messages
    sync: {
        Ping,
        Pong,
        SyncHint,
    },
    // Resync messages
    resync: {
        ResyncReq,
        SnapshotFrag,
    },
}
