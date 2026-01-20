use std::net::SocketAddr;
use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use nesium_netproto::limits::TCP_RX_BUFFER_SIZE;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::{accept_async, tungstenite};
use tokio_util::codec::{BytesCodec, FramedWrite};
use tokio_util::sync::CancellationToken;
use tracing::warn;

use nesium_netproto::codec::encode_message;
use nesium_netproto::messages::session::{ErrorCode, ErrorMsg};

use crate::net::quic_config;
use crate::net::rate_limit::IpRateLimiter;
use crate::net::stream_adapter::WebSocketStream;

use super::framing::TcpFramer;
use super::inbound::{ConnId, InboundEvent, TransportKind, next_conn_id};
use super::outbound::spawn_writer;

pub use tokio_rustls::TlsAcceptor;

/// Start a TCP listener. All decoded packets and connection events are sent to `tx`.
pub async fn run_tcp_listener(
    bind: SocketAddr,
    tx: mpsc::Sender<InboundEvent>,
    app_name: &str,
) -> anyhow::Result<()> {
    let listener = TcpListener::bind(bind).await?;
    run_tcp_listener_with_listener(listener, tx, app_name, None).await
}

/// Run the accept loop on an existing listener.
pub async fn run_tcp_listener_with_listener(
    listener: TcpListener,
    tx: mpsc::Sender<InboundEvent>,
    app_name: &str,
    ip_rate_limiter: Option<Arc<IpRateLimiter>>,
) -> anyhow::Result<()> {
    let tls_acceptor = get_or_create_server_tls_acceptor(app_name)?;

    loop {
        let (stream, peer) = listener.accept().await?;

        // Check IP-based rate limit before processing connection
        if let Some(ref limiter) = ip_rate_limiter
            && !limiter.check(peer.ip())
        {
            warn!(%peer, "Connection rejected: IP rate limit exceeded");
            let _ = reject_with_rate_limit(stream).await;
            continue;
        }

        let conn_id = next_conn_id();

        let tx_clone = tx.clone();
        let tls_acceptor_clone = tls_acceptor.clone();
        tokio::spawn(async move {
            handle_tcp_connection(stream, peer, conn_id, tx_clone, tls_acceptor_clone).await;
        });
    }
}

pub fn get_or_create_server_tls_acceptor(
    app_name: &str,
) -> anyhow::Result<std::sync::Arc<tokio_rustls::TlsAcceptor>> {
    let dir = quic_config::default_quic_data_dir(app_name);
    let (cert_path, key_path) = quic_config::ensure_quic_cert_pair(&dir)?;

    let certs = rustls_pemfile::certs(&mut std::io::BufReader::new(std::fs::File::open(
        &cert_path,
    )?))
    .collect::<Result<Vec<_>, _>>()?;
    let key = rustls_pemfile::private_key(&mut std::io::BufReader::new(std::fs::File::open(
        &key_path,
    )?))?
    .ok_or_else(|| anyhow::anyhow!("No private key found in {}", key_path.display()))?;

    let server_config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| anyhow::anyhow!("Failed to build server config: {}", e))?;

    Ok(std::sync::Arc::new(tokio_rustls::TlsAcceptor::from(
        std::sync::Arc::new(server_config),
    )))
}

/// Handle a single TCP connection. Public to allow embedding server in other crates.
pub async fn handle_tcp_connection(
    stream: TcpStream,
    peer: SocketAddr,
    conn_id: ConnId,
    tx: mpsc::Sender<InboundEvent>,
    tls_acceptor: std::sync::Arc<tokio_rustls::TlsAcceptor>,
) {
    let _ = stream.set_nodelay(true);

    // Protocol Sniffing via Peeking
    let mut peek_buf = [0u8; 4];
    let protocol = match stream.peek(&mut peek_buf).await {
        Ok(n) if n >= 2 => {
            if &peek_buf[0..2] == b"NS" {
                ParsedProtocol::Native
            } else if n >= 4 && &peek_buf[0..4] == b"GET " {
                ParsedProtocol::WebSocket
            } else if n >= 2 && peek_buf[0] == 0x16 && peek_buf[1] == 0x03 {
                ParsedProtocol::Tls
            } else {
                ParsedProtocol::Native // Default fallback
            }
        }
        Ok(_) => ParsedProtocol::Native, // Too short, fallback
        Err(e) => {
            warn!("Failed to peek stream: {}", e);
            return;
        }
    };

    match protocol {
        ParsedProtocol::WebSocket => {
            match accept_async(stream).await {
                Ok(ws_stream) => {
                    let (write, read) = ws_stream.split();
                    // Adapt the WebSocket frame sink to a Bytes sink
                    let sink =
                        write
                            .sink_map_err(std::io::Error::other)
                            .with(|msg: bytes::Bytes| {
                                std::future::ready(Ok(tungstenite::Message::Binary(msg)))
                            });

                    let adapted_read = WebSocketStream::new(read);
                    handle_connection_inner(adapted_read, sink, peer, conn_id, tx).await;
                }
                Err(e) => {
                    warn!("WebSocket handshake failed: {}", e);
                }
            }
        }
        ParsedProtocol::Tls => {
            match tls_acceptor.accept(stream).await {
                Ok(tls_stream) => {
                    // Inside TLS, we assume usage of WebSocket (WSS).
                    match accept_async(tls_stream).await {
                        Ok(ws_stream) => {
                            let (write, read) = ws_stream.split();
                            let sink = write.sink_map_err(std::io::Error::other).with(
                                |msg: bytes::Bytes| {
                                    std::future::ready(Ok(tungstenite::Message::Binary(msg)))
                                },
                            );

                            let adapted_read = WebSocketStream::new(read);
                            handle_connection_inner(adapted_read, sink, peer, conn_id, tx).await;
                        }
                        Err(e) => {
                            warn!("WSS handshake failed: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("TLS handshake failed: {}", e);
                }
            }
        }
        ParsedProtocol::Native => {
            let (read, write) = stream.into_split();
            // Adapt the AsyncWrite to a Bytes sink using FramedWrite
            // bytes::BytesCodec handles writing Bytes trait to AsyncWrite
            let sink = FramedWrite::new(write, BytesCodec::new());
            handle_connection_inner(read, sink, peer, conn_id, tx).await;
        }
    }
}

enum ParsedProtocol {
    Native,
    WebSocket,
    Tls,
}

async fn reject_with_rate_limit(mut stream: TcpStream) {
    // Protocol Sniffing via Peeking (similar to handle_tcp_connection)
    let mut peek_buf = [0u8; 4];
    let protocol = match stream.peek(&mut peek_buf).await {
        Ok(n) if n >= 2 => {
            if &peek_buf[0..2] == b"NS" {
                ParsedProtocol::Native
            } else if n >= 4 && &peek_buf[0..4] == b"GET " {
                ParsedProtocol::WebSocket
            } else if n >= 2 && peek_buf[0] == 0x16 && peek_buf[1] == 0x03 {
                ParsedProtocol::Tls
            } else {
                ParsedProtocol::Native // Default fallback
            }
        }
        _ => ParsedProtocol::Native,
    };

    match protocol {
        ParsedProtocol::WebSocket => {
            let _ = stream
                .write_all(b"HTTP/1.1 429 Too Many Requests\r\nConnection: close\r\n\r\n")
                .await;
        }
        ParsedProtocol::Native => {
            let msg = ErrorMsg {
                code: ErrorCode::RateLimited,
            };
            if let Ok(frame) = encode_message(&msg) {
                let _ = stream.write_all(&frame).await;
            }
        }
        ParsedProtocol::Tls => {
            // Can't easily send TLS error without handshake, just drop
        }
    }
    let _ = stream.flush().await;
    let _ = stream.shutdown().await;
}

async fn handle_connection_inner<R, S>(
    mut read: R,
    write: S,
    peer: SocketAddr,
    conn_id: ConnId,
    tx: mpsc::Sender<InboundEvent>,
) where
    R: tokio::io::AsyncRead + Unpin,
    S: futures_util::Sink<bytes::Bytes, Error = std::io::Error> + Unpin + Send + 'static,
{
    // Outbound queue (framed bytes).
    let (out_tx, out_rx) = mpsc::channel::<bytes::Bytes>(1024);
    let writer = spawn_writer(write, out_rx);

    let cancel_token = CancellationToken::new();

    // Notify upper layer that a connection is established.
    tx.send(InboundEvent::Connected {
        conn_id,
        peer,
        transport: TransportKind::Tcp,
        outbound: out_tx.clone(),
        cancel_token: cancel_token.clone(),
    })
    .await
    .ok();

    // Framer keeps bytes across reads.
    let mut framer = TcpFramer::new(8 * 1024);

    let mut disconnect_reason = "eof".to_string();

    loop {
        // Hard cap to avoid unbounded buffering (derived from limits module).
        if framer.buf_mut().len() > TCP_RX_BUFFER_SIZE {
            disconnect_reason = format!("rx buffer exceeded limit ({} bytes)", TCP_RX_BUFFER_SIZE);
            break;
        }

        framer.buf_mut().reserve(4096);
        let read_res = tokio::select! {
            res = read.read_buf(framer.buf_mut()) => res,
            _ = cancel_token.cancelled() => {
                disconnect_reason = "cancelled by server".to_string();
                break;
            }
        };

        match read_res {
            Ok(n) => {
                if n == 0 {
                    disconnect_reason = "eof".to_string();
                    break;
                }
            }
            Err(e) => {
                disconnect_reason = format!("read error: {}", e);
                break;
            }
        }

        match framer.drain_packets() {
            Ok(packets) => {
                let mut closed = false;
                for packet in packets {
                    // Forward decoded packets to upper layer.
                    if tx
                        .send(InboundEvent::Packet {
                            conn_id,
                            peer,
                            transport: TransportKind::Tcp,
                            packet,
                        })
                        .await
                        .is_err()
                    {
                        // Upper layer is gone -> stop connection task.
                        disconnect_reason = "inbound channel closed".to_string();
                        closed = true;
                        break;
                    }
                }
                if closed {
                    break;
                }
            }
            Err(e) => {
                // Protocol error -> close connection.
                disconnect_reason = format!("protocol error: {}", e);
                break;
            }
        }
    }

    // Notify disconnect (best-effort).
    let _ = tx
        .send(InboundEvent::Disconnected {
            conn_id,
            peer,
            transport: TransportKind::Tcp,
            reason: disconnect_reason.clone(),
        })
        .await;

    // Close outbound channel so writer can exit.
    drop(out_tx);

    // Await writer task; ignore errors here (connection is closing anyway).
    let _ = writer.await;
}
