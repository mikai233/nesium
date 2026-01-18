//! Server library - main loop logic extracted for testing.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64};

use nesium_netproto::{
    channel::ChannelKind,
    messages::session::{AttachChannel, P2PHostDisconnected, PlayerLeft},
    msg_id::MsgId,
};
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

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
static NEXT_SESSION_TOKEN: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConnRole {
    /// Newly accepted TCP connection with no session association yet.
    Unbound,
    /// Primary session/control connection (Hello/Welcome happens here).
    Control,
    /// Secondary channel connection attached to an existing session.
    Channel(ChannelKind),
}

/// Per-connection server-side context.
struct ConnCtx {
    outbound: OutboundTx,
    assigned_client_id: u32,
    name: String,
    role: ConnRole,
    /// Assigned on `Hello` and used to attach secondary channels.
    session_token: u64,
    /// Secondary channel outbounds (stored on the control connection).
    channels: HashMap<ChannelKind, OutboundTx>,
}

/// Run the server main loop.
///
/// This is the core server logic, extracted for testability.
pub async fn run_server(mut rx: mpsc::Receiver<InboundEvent>) -> anyhow::Result<()> {
    let mut conns: HashMap<ConnId, ConnCtx> = HashMap::new();
    let mut room_mgr = RoomManager::new();
    let mut token_to_control_conn: HashMap<u64, ConnId> = HashMap::new();

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
                        assigned_client_id: 0,
                        name: String::new(),
                        role: ConnRole::Unbound,
                        session_token: 0,
                        channels: HashMap::new(),
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
                    match ctx.role {
                        ConnRole::Control => {
                            if ctx.session_token != 0 {
                                token_to_control_conn.remove(&ctx.session_token);
                            }

                            if let Some(room_id) = room_mgr.get_client_room(ctx.assigned_client_id)
                            {
                                let (player_index, recipients) = {
                                    let Some(room) = room_mgr.get_room_mut(room_id) else {
                                        conns.remove(&conn_id);
                                        continue;
                                    };

                                    let player_index = if let Some(player) =
                                        room.remove_player(ctx.assigned_client_id)
                                    {
                                        info!(
                                            client_id = ctx.assigned_client_id,
                                            room_id,
                                            player_index = player.player_index,
                                            "Player left room"
                                        );
                                        Some(player.player_index)
                                    } else if room
                                        .remove_spectator(ctx.assigned_client_id)
                                        .is_some()
                                    {
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
                                        room.all_outbounds_msg(MsgId::PlayerLeft)
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

                                    for recipient in &recipients {
                                        let _ = send_msg_tcp(recipient, &msg).await;
                                    }
                                }

                                room_mgr.remove_client(ctx.assigned_client_id);
                            }

                            // Also remove from any P2P signaling watch lists (host/joiners).
                            if ctx.assigned_client_id != 0 {
                                // Check if this client is the P2P host of any room and broadcast disconnect
                                let host_rooms_to_notify =
                                    room_mgr.clear_p2p_host_for_client(ctx.assigned_client_id);
                                for (room_code, watchers) in host_rooms_to_notify {
                                    let notice = P2PHostDisconnected { room_code };
                                    for tx in &watchers {
                                        let _ = send_msg_tcp(tx, &notice).await;
                                    }
                                    info!(room_code, "Notified watchers of P2P host disconnect");
                                }

                                room_mgr.remove_p2p_watcher(ctx.assigned_client_id);
                            }
                        }
                        ConnRole::Channel(ch) => {
                            let client_id = ctx.assigned_client_id;
                            let token = ctx.session_token;

                            if client_id != 0
                                && let Some(room_id) = room_mgr.get_client_room(client_id)
                                && let Some(room) = room_mgr.get_room_mut(room_id)
                            {
                                room.clear_client_channel_outbound(client_id, ch);
                            }

                            if token != 0
                                && let Some(control_conn_id) = token_to_control_conn.get(&token)
                                && let Some(control_ctx) = conns.get_mut(control_conn_id)
                            {
                                control_ctx.channels.remove(&ch);
                            }
                        }
                        ConnRole::Unbound => {}
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
                if packet.msg_id() == MsgId::AttachChannel {
                    let Ok(msg) = postcard::from_bytes::<AttachChannel>(&packet.payload) else {
                        warn!(conn_id, %peer, "Bad AttachChannel message");
                        continue;
                    };

                    if msg.channel == ChannelKind::Control {
                        warn!(conn_id, %peer, "AttachChannel: invalid channel=Control");
                        continue;
                    }

                    let Some(control_conn_id) =
                        token_to_control_conn.get(&msg.session_token).copied()
                    else {
                        warn!(
                            conn_id,
                            %peer,
                            token = msg.session_token,
                            "AttachChannel: session token not found"
                        );
                        continue;
                    };

                    let Some(control_client_id) =
                        conns.get(&control_conn_id).map(|c| c.assigned_client_id)
                    else {
                        continue;
                    };

                    if control_client_id == 0 {
                        warn!(conn_id, %peer, "AttachChannel: control connection not handshaked yet");
                        continue;
                    }

                    let Some(outbound) = conns.get(&conn_id).map(|c| c.outbound.clone()) else {
                        continue;
                    };

                    if let Some(ctx) = conns.get_mut(&conn_id) {
                        ctx.assigned_client_id = control_client_id;
                        ctx.role = ConnRole::Channel(msg.channel);
                        ctx.session_token = msg.session_token;
                    }

                    if let Some(control_ctx) = conns.get_mut(&control_conn_id) {
                        // If the client re-attaches the same channel (e.g. reconnect), replace it.
                        control_ctx.channels.insert(msg.channel, outbound.clone());
                    }

                    if let Some(room_id) = room_mgr.get_client_room(control_client_id)
                        && let Some(room) = room_mgr.get_room_mut(room_id)
                    {
                        room.set_client_channel_outbound(control_client_id, msg.channel, outbound);
                    }

                    debug!(
                        conn_id,
                        %peer,
                        client_id = control_client_id,
                        channel = ?msg.channel,
                        "Attached channel connection"
                    );
                    continue;
                }

                let Some(ctx) = conns.get_mut(&conn_id) else {
                    continue;
                };

                dispatch_packet(ctx, conn_id, &peer, &packet, &mut room_mgr).await;

                if packet.msg_id() == MsgId::Hello
                    && ctx.role == ConnRole::Control
                    && ctx.session_token != 0
                {
                    token_to_control_conn.insert(ctx.session_token, conn_id);
                }
            }
        }
    }

    Ok(())
}
