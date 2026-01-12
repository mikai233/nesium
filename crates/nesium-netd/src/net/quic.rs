use std::net::SocketAddr;

use bytes::Bytes;
use nesium_netproto::limits::TCP_RX_BUFFER_SIZE;
use quinn::{Endpoint, ServerConfig};
use tokio::io::AsyncReadExt;
use tokio::sync::mpsc;
use tracing::{debug, warn};

use super::framing::TcpFramer;
use super::inbound::{ConnId, InboundEvent, TransportKind, next_conn_id};

/// Start a QUIC listener. All decoded packets and connection events are sent to `tx`.
///
/// Wire format on each bidirectional stream is identical to TCP framing:
/// `[u32 len prefix][Header][Payload]`.
pub async fn run_quic_listener(
    bind: SocketAddr,
    server_config: ServerConfig,
    tx: mpsc::Sender<InboundEvent>,
) -> anyhow::Result<()> {
    let endpoint = Endpoint::server(server_config, bind)?;

    loop {
        let Some(connecting) = endpoint.accept().await else {
            break;
        };

        let tx_clone = tx.clone();
        tokio::spawn(async move {
            let Ok(conn) = connecting.await else {
                return;
            };

            let peer = conn.remote_address();
            debug!(%peer, "QUIC connection established");

            loop {
                let stream = conn.accept_bi().await;
                let (send, recv) = match stream {
                    Ok(v) => v,
                    Err(_) => break,
                };

                let conn_id = next_conn_id();
                let tx_stream = tx_clone.clone();

                tokio::spawn(async move {
                    handle_quic_stream(conn_id, peer, recv, send, tx_stream).await;
                });
            }
        });
    }

    Ok(())
}

async fn handle_quic_stream(
    conn_id: ConnId,
    peer: SocketAddr,
    mut recv: quinn::RecvStream,
    mut send: quinn::SendStream,
    tx: mpsc::Sender<InboundEvent>,
) {
    let (out_tx, mut out_rx) = mpsc::channel::<Bytes>(1024);

    tx.send(InboundEvent::Connected {
        conn_id,
        peer,
        transport: TransportKind::Quic,
        outbound: out_tx.clone(),
    })
    .await
    .ok();

    let writer = tokio::spawn(async move {
        while let Some(frame) = out_rx.recv().await {
            if send.write_all(&frame).await.is_err() {
                break;
            }
        }
        let _ = send.finish();
    });

    let mut framer = TcpFramer::new(8 * 1024);
    let mut disconnect_reason = "eof".to_string();

    loop {
        if framer.buf_mut().len() > TCP_RX_BUFFER_SIZE {
            disconnect_reason = format!("rx buffer exceeded limit ({} bytes)", TCP_RX_BUFFER_SIZE);
            break;
        }

        framer.buf_mut().reserve(4096);
        match recv.read_buf(framer.buf_mut()).await {
            Ok(0) => {
                disconnect_reason = "eof".to_string();
                break;
            }
            Ok(_) => {}
            Err(e) => {
                disconnect_reason = format!("read error: {}", e);
                break;
            }
        }

        match framer.drain_packets() {
            Ok(packets) => {
                let mut closed = false;
                for packet in packets {
                    if tx
                        .send(InboundEvent::Packet {
                            conn_id,
                            peer,
                            transport: TransportKind::Quic,
                            packet,
                        })
                        .await
                        .is_err()
                    {
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
                disconnect_reason = format!("protocol error: {}", e);
                break;
            }
        }
    }

    let _ = tx
        .send(InboundEvent::Disconnected {
            conn_id,
            peer,
            transport: TransportKind::Quic,
            reason: disconnect_reason.clone(),
        })
        .await;

    drop(out_tx);
    if writer.await.is_err() {
        warn!(conn_id, %peer, "QUIC writer task join error");
    }
}
