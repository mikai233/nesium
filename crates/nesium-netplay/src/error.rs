//! Netplay error types.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum NetplayError {
    #[error("not connected to server")]
    NotConnected,

    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    #[error("connection lost: {0}")]
    ConnectionLost(String),

    #[error("protocol error: {0}")]
    Protocol(#[from] nesium_netproto::error::ProtoError),

    #[error("handshake failed: {0}")]
    HandshakeFailed(String),

    #[error("room join failed: {0}")]
    RoomJoinFailed(String),

    #[error("ROM hash mismatch")]
    RomHashMismatch,

    #[error("state sync failed: {0}")]
    SyncFailed(String),

    #[error("input queue exhausted (waiting for remote)")]
    InputQueueEmpty,

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("channel send error")]
    ChannelSend,

    #[error("channel receive error")]
    ChannelRecv,

    #[error("session already active")]
    AlreadyConnected,
}
