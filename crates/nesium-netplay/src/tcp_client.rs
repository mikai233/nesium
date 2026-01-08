//! Async TCP client for netplay.
//!
//! This module provides a tokio-based TCP client that handles:
//! - Connection to server
//! - Message framing and encoding/decoding
//! - Async send/receive loops

use std::net::SocketAddr;

use bytes::{Buf, BytesMut};
use nesium_netproto::{
    codec_tcp::{encode_tcp_frame_auto, try_decode_tcp_frames},
    header::Header,
    msg_id::MsgId,
    packet::PacketView,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpStream, tcp::OwnedWriteHalf},
    sync::mpsc,
};
use tracing::{debug, error, info, trace, warn};

use crate::error::NetplayError;

/// A decoded packet with owned payload data.
#[derive(Debug, Clone)]
pub struct PacketOwned {
    pub header: Header,
    pub msg_id: MsgId,
    pub payload: bytes::Bytes,
}

impl<'a> From<PacketView<'a>> for PacketOwned {
    fn from(view: PacketView<'a>) -> Self {
        Self {
            header: view.header,
            msg_id: view.msg_id,
            payload: bytes::Bytes::copy_from_slice(view.payload),
        }
    }
}

/// Events sent from the TCP client to the session handler.
#[derive(Debug)]
pub enum TcpClientEvent {
    /// Successfully connected to server.
    Connected,
    /// Disconnected from server.
    Disconnected { reason: String },
    /// Received a packet from server.
    Packet(PacketOwned),
    /// Connection error.
    Error(String),
}

/// Commands sent to the TCP client from the session handler.
#[derive(Debug)]
pub enum TcpClientCommand {
    /// Send a raw packet (already encoded).
    SendRaw(bytes::Bytes),
    /// Disconnect and shut down.
    Disconnect,
}

/// Handle for sending commands to a running TCP client.
#[derive(Clone)]
pub struct TcpClientHandle {
    cmd_tx: mpsc::Sender<TcpClientCommand>,
}

impl TcpClientHandle {
    /// Send a message to the server.
    ///
    /// The payload size limit is automatically selected based on the message type.
    pub async fn send_message<T: serde::Serialize>(
        &self,
        header: Header,
        msg_id: MsgId,
        payload: &T,
    ) -> Result<(), NetplayError> {
        let bytes = encode_tcp_frame_auto(header, msg_id, payload)?;
        self.cmd_tx
            .send(TcpClientCommand::SendRaw(bytes::Bytes::from(bytes)))
            .await
            .map_err(|_| NetplayError::ChannelSend)?;
        Ok(())
    }

    /// Request disconnect.
    pub async fn disconnect(&self) -> Result<(), NetplayError> {
        self.cmd_tx
            .send(TcpClientCommand::Disconnect)
            .await
            .map_err(|_| NetplayError::ChannelSend)?;
        Ok(())
    }
}

/// Start a TCP client connection to the given address.
///
/// Returns a handle for sending commands and spawns background tasks
/// for reading/writing.
pub async fn connect(
    addr: SocketAddr,
    event_tx: mpsc::Sender<TcpClientEvent>,
) -> Result<TcpClientHandle, NetplayError> {
    info!("Connecting to netplay server at {}", addr);

    let stream = TcpStream::connect(addr).await.map_err(|e| {
        NetplayError::ConnectionFailed(format!("Failed to connect to {}: {}", addr, e))
    })?;

    let _ = stream.set_nodelay(true);
    let (read_half, write_half) = stream.into_split();

    // Command channel for sending to the writer task.
    let (cmd_tx, cmd_rx) = mpsc::channel::<TcpClientCommand>(256);

    // Spawn writer task
    let event_tx_writer = event_tx.clone();
    tokio::spawn(async move {
        writer_loop(write_half, cmd_rx, event_tx_writer).await;
    });

    // Spawn reader task
    tokio::spawn(async move {
        reader_loop(read_half, event_tx).await;
    });

    Ok(TcpClientHandle { cmd_tx })
}

/// Writer task: receives commands and writes to socket.
async fn writer_loop(
    mut write: OwnedWriteHalf,
    mut cmd_rx: mpsc::Receiver<TcpClientCommand>,
    event_tx: mpsc::Sender<TcpClientEvent>,
) {
    loop {
        match cmd_rx.recv().await {
            Some(TcpClientCommand::SendRaw(bytes)) => {
                trace!("Sending {} bytes to server", bytes.len());
                if let Err(e) = write.write_all(&bytes).await {
                    error!("Write error: {}", e);
                    let _ = event_tx.send(TcpClientEvent::Error(e.to_string())).await;
                    break;
                }
            }
            Some(TcpClientCommand::Disconnect) => {
                debug!("Disconnect command received");
                break;
            }
            None => {
                // Channel closed
                debug!("Command channel closed");
                break;
            }
        }
    }

    // Attempt graceful shutdown
    let _ = write.shutdown().await;
}

/// Reader task: reads from socket and sends events.
async fn reader_loop(
    mut read: tokio::net::tcp::OwnedReadHalf,
    event_tx: mpsc::Sender<TcpClientEvent>,
) {
    let mut buf = BytesMut::with_capacity(8 * 1024);
    let mut connected_sent = false;

    loop {
        // Send connected event on first successful read attempt
        if !connected_sent {
            let _ = event_tx.send(TcpClientEvent::Connected).await;
            connected_sent = true;
        }

        buf.reserve(4096);
        match read.read_buf(&mut buf).await {
            Ok(0) => {
                // EOF
                info!("Server closed connection");
                let _ = event_tx
                    .send(TcpClientEvent::Disconnected {
                        reason: "server closed connection".to_string(),
                    })
                    .await;
                break;
            }
            Ok(n) => {
                trace!("Received {} bytes from server", n);

                // Try to decode frames
                match try_decode_tcp_frames(&buf) {
                    Ok((views, consumed)) => {
                        for view in views {
                            let packet = PacketOwned::from(view);
                            debug!("Received {:?} from server", packet.msg_id);
                            if event_tx.send(TcpClientEvent::Packet(packet)).await.is_err() {
                                warn!("Event channel closed");
                                return;
                            }
                        }
                        buf.advance(consumed);
                    }
                    Err(e) => {
                        error!("Protocol decode error: {}", e);
                        let _ = event_tx
                            .send(TcpClientEvent::Error(format!("Protocol error: {}", e)))
                            .await;
                        let _ = event_tx
                            .send(TcpClientEvent::Disconnected {
                                reason: format!("protocol error: {}", e),
                            })
                            .await;
                        break;
                    }
                }
            }
            Err(e) => {
                error!("Read error: {}", e);
                let _ = event_tx
                    .send(TcpClientEvent::Disconnected {
                        reason: e.to_string(),
                    })
                    .await;
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packet_owned_conversion() {
        let header = Header::new(MsgId::Ping as u8);
        let payload = &[1, 2, 3];
        let view = PacketView::new(header, MsgId::Ping, payload);

        let owned = PacketOwned::from(view);
        assert_eq!(owned.msg_id, MsgId::Ping);
        assert_eq!(owned.payload.as_ref(), &[1, 2, 3]);
    }
}
