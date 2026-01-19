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
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::net::framing::PacketOwned;
use crate::net::inbound::{ConnId, InboundEvent};
use crate::net::outbound::{OutboundTx, send_msg};
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
    assigned_client_id: Option<u32>,
    name: String,
    role: ConnRole,
    /// Assigned on `Hello` and used to attach secondary channels.
    session_token: Option<u64>,
    /// Secondary channel outbounds (stored on the control connection).
    channels: HashMap<ChannelKind, OutboundTx>,
    /// Per-connection message rate limiter (None if rate limiting is disabled).
    rate_limiter: Option<ConnRateLimiter>,
    /// Last time this connection received a message (for idle cleanup).
    last_activity: Instant,
    /// Real peer address of the connection.
    peer: SocketAddr,
    /// Cancellation token to shut down the connection handler.
    cancel_token: CancellationToken,
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
                        cancel_token,
                        ..
                    } => {
                        handle_connected(
                            &mut conns,
                            conn_id,
                            peer,
                            outbound,
                            cancel_token,
                            rate_config.as_ref(),
                        );
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
                    .filter(|(_, ctx)| {
                        // If this is a secondary channel, take the max of its activity and its parent's activity
                        let last_activity = ctx.session_token
                            .filter(|_| matches!(ctx.role, ConnRole::Channel(_)))
                            .and_then(|t| token_to_control_conn.get(&t))
                            .and_then(|id| conns.get(id))
                            .map(|control_ctx| ctx.last_activity.max(control_ctx.last_activity))
                            .unwrap_or(ctx.last_activity);

                        now.duration_since(last_activity) > max_idle
                    })
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
    cancel_token: CancellationToken,
    rate_config: Option<&RateLimitConfig>,
) {
    conns.insert(
        conn_id,
        ConnCtx {
            outbound,
            assigned_client_id: None,
            name: String::new(),
            role: ConnRole::Unbound,
            session_token: None,
            channels: HashMap::new(),
            rate_limiter: rate_config.and_then(ConnRateLimiter::new),
            last_activity: Instant::now(),
            peer,
            cancel_token,
        },
    );
    info!(conn_id, %peer, "Connection established");
}

async fn handle_disconnected(
    conns: &mut HashMap<ConnId, ConnCtx>,
    room_mgr: &mut RoomManager,
    token_to_control_conn: &mut HashMap<u64, ConnId>,
    conn_id: ConnId,
    peer: SocketAddr,
    reason: String,
) {
    let (role, session_token, assigned_client_id, cancel_token) = match conns.get(&conn_id) {
        Some(ctx) => (
            ctx.role,
            ctx.session_token,
            ctx.assigned_client_id,
            ctx.cancel_token.clone(),
        ),
        None => return,
    };

    match role {
        ConnRole::Control => {
            if let Some(token) = session_token {
                token_to_control_conn.remove(&token);
            }

            if let Some(client_id) = assigned_client_id {
                if let Some(room) = room_mgr.client_room_mut(client_id) {
                    let room_id = room.id;
                    let mut recipients = Vec::new();

                    let player_index = match room.remove_player(client_id) {
                        Some(player) => {
                            info!(
                                client_id,
                                room_id,
                                player_index = player.player_index,
                                "Player left room"
                            );
                            Some(player.player_index)
                        }
                        None => {
                            if room.remove_spectator(client_id).is_some() {
                                info!(client_id, room_id, role = "spectator", "Client left room");
                            }
                            None
                        }
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
                            client_id,
                            player_index: p_idx,
                        };

                        for recipient in &recipients {
                            let _ = send_msg(recipient, &msg).await;
                        }
                    }

                    room_mgr.unregister_client(client_id);
                }

                // Check if this client is the P2P host of any room and broadcast disconnect
                let host_rooms_to_notify = room_mgr.clear_p2p_host_for_client(client_id);
                for (room_id, watchers) in host_rooms_to_notify {
                    let notice = P2PHostDisconnected { room_id };
                    for tx in &watchers {
                        let _ = send_msg(tx, &notice).await;
                    }
                    info!(room_id, "Notified watchers of P2P host disconnect");
                }

                room_mgr.remove_p2p_watcher(client_id);
            }
        }
        ConnRole::Channel(ch) => {
            if let Some(client_id) = assigned_client_id {
                if let Some(room) = room_mgr.client_room_mut(client_id) {
                    room.clear_client_channel_outbound(client_id, ch);
                }
            }

            if let Some(token) = session_token {
                if let Some(control_conn_id) = token_to_control_conn.get(&token) {
                    if let Some(control_ctx) = conns.get_mut(control_conn_id) {
                        control_ctx.channels.remove(&ch);
                    }
                }
            }
        }
        ConnRole::Unbound => {}
    }

    // Trigger cancellation to stop the reader/writer tasks
    cancel_token.cancel();

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
    let now = Instant::now();
    let (role, session_token) = match conns.get_mut(&conn_id) {
        Some(ctx) => {
            ctx.last_activity = now;
            (ctx.role, ctx.session_token)
        }
        None => return,
    };

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

                if control_ctx.assigned_client_id.is_none() {
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
                ctx.session_token = Some(msg.session_token);
            }

            // 3. Update room if already in one
            if let Some(client_id) = control_client_id {
                if let Some(room) = room_mgr.client_room_mut(client_id) {
                    room.set_client_channel_outbound(client_id, msg.channel, channel_outbound);
                }
            }

            Ok(())
        })();

        if let Err(reason) = res {
            warn!(conn_id, %peer, %reason, "AttachChannel failed (will disconnect)");
            conns.remove(&conn_id);
        } else {
            // Update activity for successful AttachChannel too
            if let Some(ctx) = conns.get_mut(&conn_id) {
                ctx.last_activity = Instant::now();
            }
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

    // Propagate activity to parent control connection if this is a secondary channel
    if matches!(role, ConnRole::Channel(_)) {
        if let Some(control_ctx) = session_token
            .and_then(|t| token_to_control_conn.get(&t))
            .filter(|&&id| id != conn_id)
            .and_then(|&id| conns.get_mut(&id))
        {
            control_ctx.last_activity = now;
        }
    }

    let Some(ctx) = conns.get_mut(&conn_id) else {
        return;
    };

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
        warn!(conn_id, "Dispatch failed for connection");
        // Remove from conns - this drops the outbound sender and triggers disconnect
        conns.remove(&conn_id);
        return;
    }

    // Refresh context and update lookup table if needed
    if packet.msg_id() == MsgId::Hello {
        if let Some(token) = conns
            .get(&conn_id)
            .filter(|c| c.role == ConnRole::Control)
            .and_then(|c| c.session_token)
        {
            token_to_control_conn.insert(token, conn_id);
        }
    }
}
