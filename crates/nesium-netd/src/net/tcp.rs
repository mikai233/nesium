use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};

use tokio::io::AsyncReadExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

use super::framing::TcpFramer;
use super::inbound::{ConnId, InboundEvent, TransportKind};
use super::outbound::spawn_tcp_writer;

static NEXT_CONN_ID: AtomicU64 = AtomicU64::new(1);

/// Start a TCP listener. All decoded packets and connection events are sent to `tx`.
pub async fn run_tcp_listener(
    bind: SocketAddr,
    tx: mpsc::Sender<InboundEvent>,
) -> anyhow::Result<()> {
    let listener = TcpListener::bind(bind).await?;

    loop {
        let (stream, peer) = listener.accept().await?;
        let conn_id = NEXT_CONN_ID.fetch_add(1, Ordering::Relaxed);

        let tx_clone = tx.clone();
        tokio::spawn(async move {
            handle_tcp_connection(stream, peer, conn_id, tx_clone).await;
        });
    }
}

/// Handle a single TCP connection. Public to allow embedding server in other crates.
pub async fn handle_tcp_connection(
    stream: TcpStream,
    peer: SocketAddr,
    conn_id: ConnId,
    tx: mpsc::Sender<InboundEvent>,
) {
    let _ = stream.set_nodelay(true);

    // Split the stream so read/write can progress independently.
    let (mut read, write) = stream.into_split();

    // Outbound queue (framed bytes).
    let (out_tx, out_rx) = mpsc::channel::<bytes::Bytes>(1024);
    let writer = spawn_tcp_writer(write, out_rx);

    // Notify upper layer that a connection is established.
    tx.send(InboundEvent::Connected {
        conn_id,
        peer,
        transport: TransportKind::Tcp,
        outbound: out_tx.clone(),
    })
    .await
    .ok();

    // Framer keeps bytes across reads.
    let mut framer = TcpFramer::new(8 * 1024);

    // Hard cap to avoid unbounded buffering.
    const MAX_RX_BUFFER: usize = 256 * 1024;

    let mut disconnect_reason = "eof".to_string();

    loop {
        if framer.buf_mut().len() > MAX_RX_BUFFER {
            disconnect_reason = format!("rx buffer exceeded limit ({} bytes)", MAX_RX_BUFFER);
            break;
        }

        framer.buf_mut().reserve(4096);
        match read.read_buf(framer.buf_mut()).await {
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
