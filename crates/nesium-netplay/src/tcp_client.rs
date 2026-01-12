//! Async TCP client for netplay.
//!
//! This module provides a tokio-based TCP client that handles:
//! - Connection to server
//! - Message framing and encoding/decoding
//! - Async send/receive loops

use std::net::SocketAddr;
use std::sync::Arc;

use bytes::{Buf, BytesMut};
use nesium_netproto::{
    channel::{ChannelKind, channel_for_msg},
    codec_tcp::{encode_tcp_frame_auto, try_decode_tcp_frames},
    header::Header,
    messages::session::AttachChannel,
    messages::session::TransportKind,
    msg_id::MsgId,
    packet::PacketView,
};
use ring::digest;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::TcpStream,
    sync::{Mutex, mpsc},
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
    inner: Arc<TcpClientInner>,
}

enum TransportBackend {
    Tcp {
        addr: SocketAddr,
    },
    Quic {
        #[allow(dead_code)]
        endpoint: quinn::Endpoint,
        connection: quinn::Connection,
    },
}

struct TcpClientInner {
    backend: TransportBackend,
    event_tx: mpsc::Sender<TcpClientEvent>,
    control_cmd_tx: mpsc::Sender<TcpClientCommand>,
    input_cmd_tx: Mutex<Option<mpsc::Sender<TcpClientCommand>>>,
    bulk_cmd_tx: Mutex<Option<mpsc::Sender<TcpClientCommand>>>,
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
        let raw = bytes::Bytes::from(bytes);

        let preferred = channel_for_msg(msg_id);
        let cmd_tx = match preferred {
            ChannelKind::Control => self.inner.control_cmd_tx.clone(),
            ChannelKind::Input => self
                .inner
                .input_cmd_tx
                .lock()
                .await
                .as_ref()
                .cloned()
                .unwrap_or_else(|| self.inner.control_cmd_tx.clone()),
            ChannelKind::Bulk => self
                .inner
                .bulk_cmd_tx
                .lock()
                .await
                .as_ref()
                .cloned()
                .unwrap_or_else(|| self.inner.control_cmd_tx.clone()),
        };

        // Best-effort fallback: if a secondary channel is dead, retry on control.
        if cmd_tx
            .send(TcpClientCommand::SendRaw(raw.clone()))
            .await
            .is_err()
        {
            self.inner
                .control_cmd_tx
                .send(TcpClientCommand::SendRaw(raw))
                .await
                .map_err(|_| NetplayError::ChannelSend)?;
        }
        Ok(())
    }

    /// Attach a secondary logical channel by opening an additional TCP connection.
    ///
    /// This is used as a TCP fallback for transports that support true multiplexing (e.g. QUIC).
    pub async fn attach_channel(
        &self,
        session_token: u64,
        channel: ChannelKind,
    ) -> Result<(), NetplayError> {
        if channel == ChannelKind::Control {
            return Ok(());
        }

        let cmd_tx = match &self.inner.backend {
            TransportBackend::Tcp { addr } => {
                spawn_tcp_connection(*addr, self.inner.event_tx.clone(), false).await?
            }
            TransportBackend::Quic { connection, .. } => {
                spawn_quic_stream(connection.clone(), self.inner.event_tx.clone(), false).await?
            }
        };

        let msg = AttachChannel {
            session_token,
            channel,
        };
        let header = Header::new(MsgId::AttachChannel as u8);
        let bytes = encode_tcp_frame_auto(header, MsgId::AttachChannel, &msg)?;
        cmd_tx
            .send(TcpClientCommand::SendRaw(bytes::Bytes::from(bytes)))
            .await
            .map_err(|_| NetplayError::ChannelSend)?;

        match channel {
            ChannelKind::Input => *self.inner.input_cmd_tx.lock().await = Some(cmd_tx),
            ChannelKind::Bulk => *self.inner.bulk_cmd_tx.lock().await = Some(cmd_tx),
            ChannelKind::Control => {}
        }

        Ok(())
    }

    /// Request disconnect.
    pub async fn disconnect(&self) -> Result<(), NetplayError> {
        self.inner
            .control_cmd_tx
            .send(TcpClientCommand::Disconnect)
            .await
            .map_err(|_| NetplayError::ChannelSend)?;

        if let Some(tx) = self.inner.input_cmd_tx.lock().await.as_ref() {
            let _ = tx.send(TcpClientCommand::Disconnect).await;
        }
        if let Some(tx) = self.inner.bulk_cmd_tx.lock().await.as_ref() {
            let _ = tx.send(TcpClientCommand::Disconnect).await;
        }

        if let TransportBackend::Quic { connection, .. } = &self.inner.backend {
            connection.close(quinn::VarInt::from_u32(0), b"disconnect");
        }
        Ok(())
    }
}

async fn spawn_tcp_connection(
    addr: SocketAddr,
    event_tx: mpsc::Sender<TcpClientEvent>,
    emit_lifecycle: bool,
) -> Result<mpsc::Sender<TcpClientCommand>, NetplayError> {
    let stream = TcpStream::connect(addr).await.map_err(|e| {
        NetplayError::ConnectionFailed(format!("Failed to connect to {}: {}", addr, e))
    })?;

    let _ = stream.set_nodelay(true);
    let (read_half, write_half) = stream.into_split();

    let (cmd_tx, cmd_rx) = mpsc::channel::<TcpClientCommand>(256);

    let event_tx_writer = event_tx.clone();
    tokio::spawn(async move {
        writer_loop(write_half, cmd_rx, event_tx_writer, emit_lifecycle).await;
    });

    tokio::spawn(async move {
        reader_loop(read_half, event_tx, emit_lifecycle).await;
    });

    Ok(cmd_tx)
}

async fn spawn_quic_stream(
    connection: quinn::Connection,
    event_tx: mpsc::Sender<TcpClientEvent>,
    emit_lifecycle: bool,
) -> Result<mpsc::Sender<TcpClientCommand>, NetplayError> {
    let (send, recv) = connection.open_bi().await.map_err(|e| {
        NetplayError::ConnectionFailed(format!("Failed to open QUIC stream: {}", e))
    })?;

    let (cmd_tx, cmd_rx) = mpsc::channel::<TcpClientCommand>(256);

    let event_tx_writer = event_tx.clone();
    tokio::spawn(async move {
        writer_loop(send, cmd_rx, event_tx_writer, emit_lifecycle).await;
    });

    tokio::spawn(async move {
        reader_loop(recv, event_tx, emit_lifecycle).await;
    });

    Ok(cmd_tx)
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

    let control_cmd_tx = spawn_tcp_connection(addr, event_tx.clone(), true).await?;

    Ok(TcpClientHandle {
        inner: Arc::new(TcpClientInner {
            backend: TransportBackend::Tcp { addr },
            event_tx,
            control_cmd_tx,
            input_cmd_tx: Mutex::new(None),
            bulk_cmd_tx: Mutex::new(None),
        }),
    })
}

pub async fn connect_quic(
    addr: SocketAddr,
    server_name: &str,
    event_tx: mpsc::Sender<TcpClientEvent>,
) -> Result<TcpClientHandle, NetplayError> {
    connect_quic_inner(addr, server_name, event_tx, None).await
}

pub async fn connect_quic_pinned(
    addr: SocketAddr,
    server_name: &str,
    pinned_sha256_fingerprint: &str,
    event_tx: mpsc::Sender<TcpClientEvent>,
) -> Result<TcpClientHandle, NetplayError> {
    let expected = parse_sha256_fingerprint(pinned_sha256_fingerprint)?;
    connect_quic_inner(addr, server_name, event_tx, Some(expected)).await
}

async fn connect_quic_inner(
    addr: SocketAddr,
    server_name: &str,
    event_tx: mpsc::Sender<TcpClientEvent>,
    pinned_sha256: Option<[u8; 32]>,
) -> Result<TcpClientHandle, NetplayError> {
    info!("Connecting to netplay server (QUIC) at {}", addr);

    let tls = if let Some(expected) = pinned_sha256 {
        // Ensure a crypto provider is available for signature verification.
        // This is normally installed lazily by `ClientConfig::builder()`, but we depend on it here.
        let _ = rustls::crypto::ring::default_provider().install_default();

        let supported_algs = rustls::crypto::CryptoProvider::get_default()
            .map(|p| p.signature_verification_algorithms)
            .ok_or_else(|| {
                NetplayError::ConnectionFailed(
                    "No rustls CryptoProvider available for pinned QUIC".to_string(),
                )
            })?;

        let verifier = Arc::new(PinnedSha256CertVerifier {
            expected,
            supported_algs,
        });

        let mut cfg = rustls::ClientConfig::builder()
            .with_root_certificates(rustls::RootCertStore::empty())
            .with_no_client_auth();
        cfg.dangerous().set_certificate_verifier(verifier);
        cfg
    } else {
        let mut roots = rustls::RootCertStore::empty();
        let native = rustls_native_certs::load_native_certs();
        if !native.errors.is_empty() {
            warn!(
                errors = native.errors.len(),
                "Failed to load some native root certs"
            );
        }
        for cert in native.certs {
            let _ = roots.add(cert);
        }

        rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth()
    };

    let crypto = quinn::crypto::rustls::QuicClientConfig::try_from(tls).map_err(|e| {
        NetplayError::ConnectionFailed(format!("Failed to build QUIC crypto config: {}", e))
    })?;
    let mut client_config = quinn::ClientConfig::new(Arc::new(crypto));
    let mut transport = quinn::TransportConfig::default();
    transport.keep_alive_interval(Some(std::time::Duration::from_secs(5)));
    // Increase idle timeout to 30s as a fallback
    transport.max_idle_timeout(Some(quinn::VarInt::from_u32(30_000).into()));
    client_config.transport_config(Arc::new(transport));

    let mut endpoint = quinn::Endpoint::client("0.0.0.0:0".parse().unwrap()).map_err(|e| {
        NetplayError::ConnectionFailed(format!("Failed to create QUIC endpoint: {}", e))
    })?;

    endpoint.set_default_client_config(client_config);

    let connecting = endpoint.connect(addr, server_name).map_err(|e| {
        NetplayError::ConnectionFailed(format!("Failed to start QUIC connect: {}", e))
    })?;

    let connection = connecting
        .await
        .map_err(|e| NetplayError::ConnectionFailed(format!("QUIC connect failed: {}", e)))?;

    let (send, recv) = connection.open_bi().await.map_err(|e| {
        NetplayError::ConnectionFailed(format!("Failed to open QUIC control stream: {}", e))
    })?;

    let (control_cmd_tx, control_cmd_rx) = mpsc::channel::<TcpClientCommand>(256);

    let event_tx_inner = event_tx.clone();
    let event_tx_writer = event_tx.clone();
    tokio::spawn(async move {
        writer_loop(send, control_cmd_rx, event_tx_writer, true).await;
    });
    tokio::spawn(async move {
        reader_loop(recv, event_tx, true).await;
    });

    Ok(TcpClientHandle {
        inner: Arc::new(TcpClientInner {
            backend: TransportBackend::Quic {
                endpoint,
                connection,
            },
            event_tx: event_tx_inner,
            control_cmd_tx,
            input_cmd_tx: Mutex::new(None),
            bulk_cmd_tx: Mutex::new(None),
        }),
    })
}

#[derive(Debug)]
struct PinnedSha256CertVerifier {
    expected: [u8; 32],
    supported_algs: rustls::crypto::WebPkiSupportedAlgorithms,
}

impl rustls::client::danger::ServerCertVerifier for PinnedSha256CertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &rustls::pki_types::CertificateDer<'_>,
        _intermediates: &[rustls::pki_types::CertificateDer<'_>],
        _server_name: &rustls::pki_types::ServerName<'_>,
        _ocsp_response: &[u8],
        _now: rustls::pki_types::UnixTime,
    ) -> Result<rustls::client::danger::ServerCertVerified, rustls::Error> {
        let actual = digest::digest(&digest::SHA256, end_entity.as_ref());
        if actual.as_ref() != self.expected {
            return Err(rustls::Error::InvalidCertificate(
                rustls::CertificateError::UnknownIssuer,
            ));
        }
        Ok(rustls::client::danger::ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls12_signature(message, cert, dss, &self.supported_algs)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &rustls::pki_types::CertificateDer<'_>,
        dss: &rustls::DigitallySignedStruct,
    ) -> Result<rustls::client::danger::HandshakeSignatureValid, rustls::Error> {
        rustls::crypto::verify_tls13_signature(message, cert, dss, &self.supported_algs)
    }

    fn supported_verify_schemes(&self) -> Vec<rustls::SignatureScheme> {
        self.supported_algs.supported_schemes()
    }
}

fn parse_sha256_fingerprint(s: &str) -> Result<[u8; 32], NetplayError> {
    use base64::Engine as _;
    let cleaned: String = s
        .chars()
        // Allow:
        // - hex with colons: "AA:BB:.."
        // - base64url without padding: "..." (may contain '-' / '_')
        .filter(|c| *c != ':' && !c.is_whitespace())
        .collect();

    // 1) Hex (with or without colons)
    if cleaned.len() == 64 && cleaned.chars().all(|c| c.is_ascii_hexdigit()) {
        let bytes =
            hex::decode(cleaned).map_err(|e| NetplayError::ConnectionFailed(format!("{e}")))?;
        let mut out = [0u8; 32];
        out.copy_from_slice(&bytes);
        return Ok(out);
    }

    // 2) base64url/base64 (no padding or with padding)
    let decoded = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(cleaned.as_bytes())
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(cleaned.as_bytes()))
        .or_else(|_| base64::engine::general_purpose::STANDARD.decode(cleaned.as_bytes()))
        .map_err(|e| {
            NetplayError::ConnectionFailed(format!("Invalid fingerprint encoding: {e}"))
        })?;

    if decoded.len() != 32 {
        return Err(NetplayError::ConnectionFailed(format!(
            "Expected SHA-256 fingerprint to be 32 bytes, got {}",
            decoded.len()
        )));
    }

    let mut out = [0u8; 32];
    out.copy_from_slice(&decoded);
    Ok(out)
}

#[cfg(test)]
mod fingerprint_tests {
    use super::parse_sha256_fingerprint;
    use base64::Engine as _;

    #[test]
    fn parse_accepts_hex_and_base64url() {
        let bytes = [0xABu8; 32];
        let hex = bytes
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<String>();
        let hex_colon = bytes
            .iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(":");
        let b64url = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);

        assert_eq!(parse_sha256_fingerprint(&hex).unwrap(), bytes);
        assert_eq!(parse_sha256_fingerprint(&hex_colon).unwrap(), bytes);
        assert_eq!(parse_sha256_fingerprint(&b64url).unwrap(), bytes);
    }

    #[test]
    fn parse_accepts_base64url_with_urlsafe_chars() {
        let bytes = [0xFFu8; 32];
        let b64url = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);
        assert!(b64url.contains('_') || b64url.contains('-'));
        assert_eq!(parse_sha256_fingerprint(&b64url).unwrap(), bytes);
    }
}

pub async fn connect_auto(
    addr: SocketAddr,
    server_name: Option<&str>,
    event_tx: mpsc::Sender<TcpClientEvent>,
) -> Result<(TcpClientHandle, TransportKind), NetplayError> {
    if let Some(server_name) = server_name {
        if !server_name.trim().is_empty() {
            match tokio::time::timeout(
                std::time::Duration::from_millis(1500),
                connect_quic(addr, server_name, event_tx.clone()),
            )
            .await
            {
                Ok(Ok(handle)) => return Ok((handle, TransportKind::Quic)),
                Ok(Err(e)) => warn!(error = %e, "QUIC connect failed; falling back to TCP"),
                Err(_) => warn!("QUIC connect timed out; falling back to TCP"),
            }
        }
    }

    let handle = connect(addr, event_tx).await?;
    Ok((handle, TransportKind::Tcp))
}

pub async fn connect_auto_pinned(
    addr: SocketAddr,
    server_name: &str,
    pinned_sha256_fingerprint: &str,
    event_tx: mpsc::Sender<TcpClientEvent>,
) -> Result<(TcpClientHandle, TransportKind), NetplayError> {
    match tokio::time::timeout(
        std::time::Duration::from_millis(1500),
        connect_quic_pinned(
            addr,
            server_name,
            pinned_sha256_fingerprint,
            event_tx.clone(),
        ),
    )
    .await
    {
        Ok(Ok(handle)) => Ok((handle, TransportKind::Quic)),
        Ok(Err(e)) => {
            warn!(error = %e, "Pinned QUIC connect failed; falling back to TCP");
            let handle = connect(addr, event_tx).await?;
            Ok((handle, TransportKind::Tcp))
        }
        Err(_) => {
            warn!("Pinned QUIC connect timed out; falling back to TCP");
            let handle = connect(addr, event_tx).await?;
            Ok((handle, TransportKind::Tcp))
        }
    }
}

/// Writer task: receives commands and writes to socket.
async fn writer_loop(
    mut write: impl AsyncWrite + Unpin,
    mut cmd_rx: mpsc::Receiver<TcpClientCommand>,
    event_tx: mpsc::Sender<TcpClientEvent>,
    emit_lifecycle: bool,
) {
    loop {
        match cmd_rx.recv().await {
            Some(TcpClientCommand::SendRaw(bytes)) => {
                trace!("Sending {} bytes to server", bytes.len());
                if let Err(e) = write.write_all(&bytes).await {
                    if emit_lifecycle {
                        error!("Write error: {}", e);
                        let _ = event_tx.send(TcpClientEvent::Error(e.to_string())).await;
                    } else {
                        warn!("Write error (secondary channel): {}", e);
                    }
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
    mut read: impl AsyncRead + Unpin,
    event_tx: mpsc::Sender<TcpClientEvent>,
    emit_lifecycle: bool,
) {
    let mut buf = BytesMut::with_capacity(8 * 1024);
    let mut connected_sent = false;

    loop {
        // Send connected event on first successful read attempt
        if emit_lifecycle && !connected_sent {
            let _ = event_tx.send(TcpClientEvent::Connected).await;
            connected_sent = true;
        }

        buf.reserve(4096);
        match read.read_buf(&mut buf).await {
            Ok(0) => {
                // EOF
                info!("Server closed connection");
                if emit_lifecycle {
                    let _ = event_tx
                        .send(TcpClientEvent::Disconnected {
                            reason: "server closed connection".to_string(),
                        })
                        .await;
                }
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
                        if emit_lifecycle {
                            let _ = event_tx
                                .send(TcpClientEvent::Error(format!("Protocol error: {}", e)))
                                .await;
                            let _ = event_tx
                                .send(TcpClientEvent::Disconnected {
                                    reason: format!("protocol error: {}", e),
                                })
                                .await;
                        }
                        break;
                    }
                }
            }
            Err(e) => {
                error!("Read error: {}", e);
                if emit_lifecycle {
                    let _ = event_tx
                        .send(TcpClientEvent::Disconnected {
                            reason: e.to_string(),
                        })
                        .await;
                }
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
