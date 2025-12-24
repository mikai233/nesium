use std::net::SocketAddr;
use tokio::sync::mpsc;

use super::framing::PacketOwned;

/// Unique connection identifier assigned by the server.
pub type ConnId = u64;

/// Transport type for a connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportKind {
    Tcp,
}

/// Sender used by upper layers to write bytes to this connection.
/// The payload is already framed (for TCP) and ready to write.
pub type OutboundTx = mpsc::Sender<bytes::Bytes>;

/// Inbound events produced by the network layer.
///
/// Current behavior:
/// - `Connected` is emitted once per accepted connection, with an `OutboundTx`
///   that upper layers can use to send bytes back.
/// - `Packet` is emitted for every decoded packet.
/// - `Disconnected` is emitted when the connection handler exits.
#[derive(Debug)]
pub enum InboundEvent {
    Connected {
        conn_id: ConnId,
        peer: SocketAddr,
        transport: TransportKind,
        outbound: OutboundTx,
    },

    Packet {
        conn_id: ConnId,
        peer: SocketAddr,
        transport: TransportKind,
        packet: PacketOwned,
    },

    Disconnected {
        conn_id: ConnId,
        peer: SocketAddr,
        transport: TransportKind,
        /// Best-effort human-readable reason (logging/debug).
        reason: String,
    },
}
