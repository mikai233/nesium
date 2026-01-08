//! Server library - main loop logic extracted for testing.

use std::collections::HashMap;
use std::sync::atomic::AtomicU32;

use nesium_netproto::{header::Header, messages::session::PlayerLeft, msg_id::MsgId};
use tokio::sync::mpsc;
use tracing::{debug, info};

use crate::net::inbound::{ConnId, InboundEvent};
use crate::net::outbound::{OutboundTx, send_msg_tcp};
use crate::proto_dispatch::handlers::dispatch_packet;
use crate::room::state::RoomManager;

// Export modules publicly for testing
pub mod net;
pub mod observability;
pub mod proto_dispatch;
pub mod room;
pub mod session;

/// Monotonically increasing IDs.
static NEXT_CLIENT_ID: AtomicU32 = AtomicU32::new(1);
static NEXT_SERVER_NONCE: AtomicU32 = AtomicU32::new(1);

/// Per-connection server-side context.
struct ConnCtx {
    outbound: OutboundTx,
    server_seq: u32,
    assigned_client_id: u32,
    rom_hash: [u8; 16],
    name: String,
}

/// Run the server main loop.
///
/// This is the core server logic, extracted for testability.
pub async fn run_server(mut rx: mpsc::Receiver<InboundEvent>) -> anyhow::Result<()> {
    let mut conns: HashMap<ConnId, ConnCtx> = HashMap::new();
    let mut room_mgr = RoomManager::new();

    info!("Server main loop started");

    while let Some(ev) = rx.recv().await {
        match ev {
            InboundEvent::Connected {
                conn_id,
                peer,
                outbound,
                ..
            } => {
                conns.insert(
                    conn_id,
                    ConnCtx {
                        outbound,
                        server_seq: 1,
                        assigned_client_id: 0,
                        rom_hash: [0; 16],
                        name: String::new(),
                    },
                );
                debug!(conn_id, %peer, "Client connected");
            }

            InboundEvent::Disconnected {
                conn_id,
                peer,
                reason,
                ..
            } => {
                if let Some(ctx) = conns.get(&conn_id) {
                    if let Some(room_id) = room_mgr.get_client_room(ctx.assigned_client_id) {
                        let (player_index, recipients) = {
                            let Some(room) = room_mgr.get_room_mut(room_id) else {
                                conns.remove(&conn_id);
                                continue;
                            };

                            let player_index =
                                if let Some(player) = room.remove_player(ctx.assigned_client_id) {
                                    info!(
                                        client_id = ctx.assigned_client_id,
                                        room_id,
                                        player_index = player.player_index,
                                        "Player left room"
                                    );
                                    Some(player.player_index)
                                } else if room.remove_spectator(ctx.assigned_client_id).is_some() {
                                    info!(
                                        client_id = ctx.assigned_client_id,
                                        room_id,
                                        role = "spectator",
                                        "Client left room"
                                    );
                                    None // Spectators don't have a player_index
                                } else {
                                    None
                                };

                            let recipients = if player_index.is_some() {
                                room.all_outbounds()
                            } else {
                                Vec::new()
                            };

                            if room.is_empty() {
                                room_mgr.remove_room(room_id);
                                info!(room_id, "Removed empty room");
                            }

                            (player_index, recipients)
                        };

                        if let Some(p_idx) = player_index {
                            let msg = PlayerLeft {
                                client_id: ctx.assigned_client_id,
                                player_index: p_idx,
                            };
                            let mut h = Header::new(MsgId::PlayerLeft as u8);
                            h.client_id = ctx.assigned_client_id;
                            h.room_id = room_id;
                            h.seq = 0;

                            for recipient in &recipients {
                                let _ = send_msg_tcp(recipient, h, MsgId::PlayerLeft, &msg).await;
                            }
                        }

                        room_mgr.remove_client(ctx.assigned_client_id);
                    }
                }

                conns.remove(&conn_id);
                info!(conn_id, %peer, %reason, "Client disconnected");
            }

            InboundEvent::Packet {
                conn_id,
                peer,
                packet,
                ..
            } => {
                let Some(ctx) = conns.get_mut(&conn_id) else {
                    continue;
                };

                dispatch_packet(ctx, conn_id, &peer, &packet, &mut room_mgr).await;
            }
        }
    }

    Ok(())
}
