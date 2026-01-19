//! Server library - main loop logic extracted for testing.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, AtomicU64};
use std::time::{Duration, Instant};

use nesium_netproto::{
    channel::ChannelKind,
    messages::session::{AttachChannel, P2PHostDisconnected, PlayerLeft},
    msg_id::MsgId,
};
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::net::framing::PacketOwned;
use crate::net::inbound::{ConnId, InboundEvent};
use crate::net::outbound::{OutboundTx, send_msg_tcp};
use crate::net::rate_limit::{ConnRateLimiter, RateLimitConfig};
use crate::proto_dispatch::error::HandlerError;
use crate::proto_dispatch::handlers::{dispatch_packet, send_error_response};
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
    /// Per-connection message rate limiter (None if rate limiting is disabled).
    rate_limiter: Option<ConnRateLimiter>,
    /// Last time this connection received a message (for idle cleanup).
    last_activity: Instant,
    /// Real peer address of the connection.
    peer: SocketAddr,
}

/// Configuration for automatic room cleanup.
#[derive(Debug, Clone)]
pub struct RoomCleanupConfig {
    /// How often to check for inactive rooms.
    pub check_interval: Duration,
    /// Maximum allowed idle time before a room is cleaned up.
    pub max_idle_duration: Duration,
}

impl Default for RoomCleanupConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(10),
            max_idle_duration: Duration::from_secs(60),
        }
    }
}

/// Run the server main loop.
///
/// This is the core server logic, extracted for testability.
///
/// If `rate_config` is provided, per-connection message rate limiting will be enabled.
/// If `cleanup_config` is provided, rooms will be automatically cleaned up after inactivity.
pub async fn run_server(
    mut rx: mpsc::Receiver<InboundEvent>,
    rate_config: Option<RateLimitConfig>,
    cleanup_config: Option<RoomCleanupConfig>,
) -> anyhow::Result<()> {
    let mut conns: HashMap<ConnId, ConnCtx> = HashMap::new();
    let mut room_mgr = RoomManager::new();
    let mut token_to_control_conn: HashMap<u64, ConnId> = HashMap::new();

    // Setup cleanup timer if configured
    let cleanup_interval = cleanup_config
        .as_ref()
        .map(|c| c.check_interval)
        .unwrap_or(Duration::from_secs(3600)); // Fallback to 1 hour if disabled
    let max_idle = cleanup_config
        .as_ref()
        .map(|c| c.max_idle_duration)
        .unwrap_or(Duration::MAX);
    let cleanup_enabled = cleanup_config.is_some();

    let mut cleanup_timer = tokio::time::interval(cleanup_interval);

    info!("Server main loop started");

    loop {
        tokio::select! {
            ev = rx.recv() => {
                let Some(ev) = ev else {
                    break;
                };
                match ev {
                    InboundEvent::Connected {
                        conn_id,
                        peer,
                        outbound,
                        ..
                    } => {
                        handle_connected(&mut conns, conn_id, peer, outbound, rate_config.as_ref());
                    }

                    InboundEvent::Disconnected {
                        conn_id,
                        peer,
                        reason,
                        ..
                    } => {
                        handle_disconnected(
                            &mut conns,
                            &mut room_mgr,
                            &mut token_to_control_conn,
                            conn_id,
                            peer,
                            reason,
                        )
                        .await;
                    }

                    InboundEvent::Packet {
                        conn_id,
                        peer,
                        packet,
                        ..
                    } => {
                        handle_packet(
                            &mut conns,
                            &mut room_mgr,
                            &mut token_to_control_conn,
                            conn_id,
                            peer,
                            packet,
                            max_idle.as_secs() as u16,
                        )
                        .await;
                    }
                }
            }
            _ = cleanup_timer.tick(), if cleanup_enabled => {
                // Collect idle connections to disconnect
                let now = Instant::now();
                let idle_conns: Vec<ConnId> = conns
                    .iter()
                    .filter(|(_, ctx)| now.duration_since(ctx.last_activity) > max_idle)
                    .map(|(&id, _)| id)
                    .collect();

                for conn_id in idle_conns {
                    let peer = conns.get(&conn_id).map(|ctx| ctx.peer).unwrap_or(SocketAddr::from(([0, 0, 0, 0], 0)));
                    info!(conn_id, %peer, "Disconnecting idle connection");
                    handle_disconnected(
                        &mut conns,
                        &mut room_mgr,
                        &mut token_to_control_conn,
                        conn_id,
                        peer,
                        "idle timeout".to_string(),
                    )
                    .await;
                }
            }
        }
    }

    Ok(())
}

fn handle_connected(
    conns: &mut HashMap<ConnId, ConnCtx>,
    conn_id: ConnId,
    peer: SocketAddr,
    outbound: OutboundTx,
    rate_config: Option<&RateLimitConfig>,
) {
    conns.insert(
        conn_id,
        ConnCtx {
            outbound,
            assigned_client_id: 0,
            name: String::new(),
            role: ConnRole::Unbound,
            session_token: 0,
            channels: HashMap::new(),
            rate_limiter: rate_config.and_then(ConnRateLimiter::new),
            last_activity: Instant::now(),
            peer,
        },
    );
    debug!(conn_id, %peer, "Client connected");
}

async fn handle_disconnected(
    conns: &mut HashMap<ConnId, ConnCtx>,
    room_mgr: &mut RoomManager,
    token_to_control_conn: &mut HashMap<u64, ConnId>,
    conn_id: ConnId,
    peer: SocketAddr,
    reason: String,
) {
    if let Some(ctx) = conns.get(&conn_id) {
        match ctx.role {
            ConnRole::Control => {
                if ctx.session_token != 0 {
                    token_to_control_conn.remove(&ctx.session_token);
                }

                if let Some(room) = room_mgr.client_room_mut(ctx.assigned_client_id) {
                    let room_id = room.id;
                    let mut recipients = Vec::new();

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

                    if player_index.is_some() {
                        recipients = room.all_outbounds_msg(MsgId::PlayerLeft);
                    }

                    if room.is_empty() {
                        room_mgr.remove_room(room_id);
                        info!(room_id, "Removed empty room");
                    }

                    if let Some(p_idx) = player_index {
                        let msg = PlayerLeft {
                            client_id: ctx.assigned_client_id,
                            player_index: p_idx,
                        };

                        for recipient in &recipients {
                            let _ = send_msg_tcp(recipient, &msg).await;
                        }
                    }

                    room_mgr.unregister_client(ctx.assigned_client_id);
                }

                // Also remove from any P2P signaling watch lists (host/joiners).
                if ctx.assigned_client_id != 0 {
                    // Check if this client is the P2P host of any room and broadcast disconnect
                    let host_rooms_to_notify =
                        room_mgr.clear_p2p_host_for_client(ctx.assigned_client_id);
                    for (room_id, watchers) in host_rooms_to_notify {
                        let notice = P2PHostDisconnected { room_id };
                        for tx in &watchers {
                            let _ = send_msg_tcp(tx, &notice).await;
                        }
                        info!(room_id, "Notified watchers of P2P host disconnect");
                    }

                    room_mgr.remove_p2p_watcher(ctx.assigned_client_id);
                }
            }
            ConnRole::Channel(ch) => {
                let client_id = ctx.assigned_client_id;
                let token = ctx.session_token;

                if client_id != 0 {
                    if let Some(room) = room_mgr.client_room_mut(client_id) {
                        room.clear_client_channel_outbound(client_id, ch);
                    }
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

async fn handle_packet(
    conns: &mut HashMap<ConnId, ConnCtx>,
    room_mgr: &mut RoomManager,
    token_to_control_conn: &mut HashMap<u64, ConnId>,
    conn_id: ConnId,
    peer: SocketAddr,
    packet: PacketOwned,
    room_idle_timeout_secs: u16,
) {
    if packet.msg_id() == MsgId::AttachChannel {
        let res: Result<(), &'static str> = (|| {
            let msg = postcard::from_bytes::<AttachChannel>(&packet.payload)
                .map_err(|_| "Bad AttachChannel message")?;

            if msg.channel == ChannelKind::Control {
                return Err("AttachChannel: invalid channel=Control");
            }

            let control_conn_id = token_to_control_conn
                .get(&msg.session_token)
                .copied()
                .ok_or("AttachChannel: session token not found")?;

            if control_conn_id == conn_id {
                return Err("AttachChannel: session token points to self");
            }

            let (control_client_id, control_name) = {
                let control_ctx = conns
                    .get(&control_conn_id)
                    .ok_or("AttachChannel: control connection gone")?;

                if control_ctx.assigned_client_id == 0 {
                    return Err("AttachChannel: control connection not handshaked yet");
                }
                (control_ctx.assigned_client_id, control_ctx.name.clone())
            };

            let channel_outbound = {
                let ctx = conns
                    .get(&conn_id)
                    .ok_or("AttachChannel: current connection gone")?;
                ctx.outbound.clone()
            };

            // 1. Record in control connection so later JoinRoom can find it
            if let Some(control_ctx) = conns.get_mut(&control_conn_id) {
                control_ctx
                    .channels
                    .insert(msg.channel, channel_outbound.clone());
            }

            // 2. Update current (channel) connection's state
            if let Some(ctx) = conns.get_mut(&conn_id) {
                ctx.assigned_client_id = control_client_id;
                ctx.name = control_name;
                ctx.role = ConnRole::Channel(msg.channel);
                ctx.session_token = msg.session_token;
            }

            // 3. Update room if already in one
            if let Some(room) = room_mgr.client_room_mut(control_client_id) {
                room.set_client_channel_outbound(control_client_id, msg.channel, channel_outbound);
            }

            Ok(())
        })();

        if let Err(reason) = res {
            warn!(conn_id, %peer, %reason, "AttachChannel failed (will disconnect)");
            conns.remove(&conn_id);
        }
        return;
    }

    let Some(ctx) = conns.get_mut(&conn_id) else {
        return;
    };

    // Check per-connection message rate limit
    if let Some(ref limiter) = ctx.rate_limiter {
        if !limiter.check() {
            warn!(conn_id, %peer, "Connection closed: message rate limit exceeded");
            send_error_response(ctx, HandlerError::rate_limited()).await;
            conns.remove(&conn_id);
            return;
        }
    }

    // Update connection activity timestamp
    ctx.last_activity = Instant::now();

    if !dispatch_packet(
        ctx,
        conn_id,
        &peer,
        &packet,
        room_mgr,
        room_idle_timeout_secs,
    )
    .await
    {
        // Remove from conns - this drops the outbound sender and triggers disconnect
        conns.remove(&conn_id);
        return;
    }

    // Refresh context and update lookup table if needed
    if packet.msg_id() == MsgId::Hello {
        if let Some(ctx) = conns.get(&conn_id) {
            if ctx.role == ConnRole::Control && ctx.session_token != 0 {
                token_to_control_conn.insert(ctx.session_token, conn_id);
            }
        }
    }
}
