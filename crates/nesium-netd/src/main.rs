use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

use nesium_netproto::header::Header;
use nesium_netproto::messages::session::{Hello, Welcome};
use nesium_netproto::msg_id::MsgId;
use tokio::sync::mpsc;

use crate::net::inbound::{ConnId, InboundEvent};
use crate::net::outbound::OutboundTx;

mod net;
mod observability;
mod proto_dispatch;
mod room;
mod session;

/// Monotonically increasing ids for demo purposes.
static NEXT_CLIENT_ID: AtomicU32 = AtomicU32::new(1);
static NEXT_SERVER_NONCE: AtomicU32 = AtomicU32::new(1);

/// Per-connection server-side context (minimal).
struct ConnCtx {
    outbound: OutboundTx,
    server_seq: u32,
    assigned_client_id: u32,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Network layer -> upper layer events.
    let (tx, mut rx) = mpsc::channel::<InboundEvent>(1024);

    // Start TCP listener.
    tokio::spawn(async move {
        let bind = "0.0.0.0:4000".parse().unwrap();
        let _ = net::tcp::run_tcp_listener(bind, tx).await;
    });

    // Connection table for this demo.
    let mut conns: HashMap<ConnId, ConnCtx> = HashMap::new();

    while let Some(ev) = rx.recv().await {
        match ev {
            InboundEvent::Connected {
                conn_id,
                peer,
                outbound,
                ..
            } => {
                // Create a new connection context.
                conns.insert(
                    conn_id,
                    ConnCtx {
                        outbound,
                        server_seq: 1,
                        assigned_client_id: 0,
                    },
                );
                println!("[net] connected: conn_id={} peer={}", conn_id, peer);
            }

            InboundEvent::Disconnected {
                conn_id,
                peer,
                reason,
                ..
            } => {
                conns.remove(&conn_id);
                println!(
                    "[net] disconnected: conn_id={} peer={} reason={}",
                    conn_id, peer, reason
                );
            }

            InboundEvent::Packet {
                conn_id,
                peer,
                packet,
                ..
            } => {
                // Only handle Hello for now.
                if packet.msg_id != MsgId::Hello {
                    // Ignore other messages in this minimal handshake step.
                    continue;
                }

                let Some(ctx) = conns.get_mut(&conn_id) else {
                    continue;
                };

                // Decode Hello payload.
                let hello: Hello = match postcard::from_bytes(&packet.payload) {
                    Ok(v) => v,
                    Err(e) => {
                        println!("[proto] bad Hello from {}: {}", peer, e);
                        continue;
                    }
                };

                // Assign a client_id once (idempotent-ish).
                if ctx.assigned_client_id == 0 {
                    ctx.assigned_client_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);
                }

                // Build Welcome payload.
                let welcome = Welcome {
                    server_nonce: NEXT_SERVER_NONCE.fetch_add(1, Ordering::Relaxed),
                    assigned_client_id: ctx.assigned_client_id,
                    room_id: 0,
                    tick_hz: 60,
                    input_delay_frames: 2,
                    max_payload: 4096,
                };

                // Build response header.
                let mut h = Header::new(MsgId::Welcome as u8);
                h.client_id = ctx.assigned_client_id;
                h.room_id = 0;
                h.seq = ctx.server_seq;
                ctx.server_seq = ctx.server_seq.wrapping_add(1);

                // Send Welcome back over TCP.
                // max_payload here is an application policy; keep it small for control-plane.
                if let Err(e) =
                    net::outbound::send_msg_tcp(&ctx.outbound, h, MsgId::Welcome, &welcome, 4096)
                        .await
                {
                    println!("[net] failed to send Welcome to {}: {}", peer, e);
                } else {
                    println!(
                        "[proto] Hello->Welcome ok: conn_id={} peer={} name={:?} client_id={}",
                        conn_id, peer, hello.name, ctx.assigned_client_id
                    );
                }
            }
        }
    }

    Ok(())
}
