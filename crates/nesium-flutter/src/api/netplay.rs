//! Netplay control API for Flutter.

use crate::api::net_utils::try_upnp_mapping;
use crate::api::net_utils::{get_local_addrs, get_public_ip};
use crate::api::server::get_server;
use crate::api::server::{netserver_get_port, netserver_is_running, netserver_start};
use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use nesium_netplay::{
    NetplayCommand, NetplayConfig, SPECTATOR_PLAYER_INDEX, SessionHandler, SessionState,
    SharedInputProvider,
};
use nesium_netproto::codec::{encode_message, try_decode_tcp_frames};
use nesium_netproto::constants::AUTO_PLAYER_INDEX;

use nesium_netproto::messages::session::P2PFallbackNotice;
use nesium_netproto::messages::session::{
    ErrorMsg, P2PCreateRoom, P2PJoinAck, P2PJoinRoom, P2PRoomCreated, Welcome,
};
use nesium_netproto::messages::session::{Hello, TransportKind};
use nesium_netproto::msg_id::MsgId;
use parking_lot::Mutex;
use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::lookup_host;
use tokio::sync::mpsc;
use tokio::sync::oneshot;

async fn resolve_addr(addr_str: &str) -> Result<SocketAddr, String> {
    if let Ok(addr) = addr_str.parse::<SocketAddr>() {
        return Ok(addr);
    }
    let mut addrs = lookup_host(addr_str)
        .await
        .map_err(|e| format!("Failed to resolve address '{}': {}", addr_str, e))?;
    addrs
        .next()
        .ok_or_else(|| format!("No addresses found for '{}'", addr_str))
}

/// Netplay connection state.
#[frb]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetplayState {
    Disconnected,
    Connecting,
    Connected,
    InRoom,
}

/// Actual transport used by the current netplay session.
#[frb]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetplayTransport {
    Unknown,
    Tcp,
    Quic,
}

/// Netplay synchronization mode.
#[frb]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SyncMode {
    /// Wait for all players' confirmed inputs before advancing each frame.
    /// Best for low-latency networks (LAN, same region).
    #[default]
    Lockstep,
    /// Predict remote inputs and rollback/resimulate on misprediction.
    /// Best for high-latency networks (cross-region, internet).
    Rollback,
}

/// Netplay status snapshot streamed to Flutter.
#[frb]
#[derive(Debug, Clone)]
pub struct NetplayStatus {
    pub state: NetplayState,
    pub transport: NetplayTransport,
    /// True if QUIC connection failed and we fell back to TCP (only for Auto connect modes).
    pub tcp_fallback_from_quic: bool,
    pub client_id: u32,
    pub room_id: u32,
    /// Player index: 0, 1, or `SPECTATOR_PLAYER_INDEX` for spectator
    pub player_index: u8,
    pub players: Vec<NetplayPlayer>,
    /// Current synchronization mode for the room
    pub sync_mode: SyncMode,
    pub error: Option<String>,
}

#[frb]
#[derive(Debug, Clone)]
pub struct NetplayPlayer {
    pub client_id: u32,
    pub name: String,
    pub player_index: u8,
}

/// Room snapshot for pre-join UI (which slots are occupied, etc.).
#[frb]
#[derive(Debug, Clone)]
pub struct NetplayRoomInfo {
    pub ok: bool,
    pub room_id: u32,
    pub started: bool,
    pub sync_mode: SyncMode,
    /// Bitmask: bit N set if slot N is occupied (0..3).
    pub occupied_mask: u8,
}

#[frb]
#[derive(Debug, Clone)]
pub enum NetplayGameEvent {
    LoadRom {
        data: Vec<u8>,
    },
    StartGame,
    PauseSync {
        paused: bool,
    },
    ResetSync {
        kind: u8,
    },
    SyncState {
        frame: u32,
        data: Vec<u8>,
    },
    PlayerLeft {
        player_index: u8,
    },
    /// Server error (e.g., room not found, permission denied)
    Error {
        error_code: u16,
    },
    /// Server instructed this client to reconnect to relay mode.
    FallbackToRelay {
        relay_addr: String,
        relay_room_code: u32,
        reason: String,
    },
}

#[frb]
#[derive(Debug, Clone)]
pub struct P2PJoinInfo {
    pub ok: bool,
    pub room_code: u32,
    pub host_addrs: Vec<String>,
    pub host_room_code: u32,
    pub host_quic_cert_sha256_fingerprint: Option<String>,
    pub host_quic_server_name: Option<String>,
    pub fallback_required: bool,
    pub fallback_reason: Option<String>,
}

#[frb]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum P2PConnectMode {
    Direct,
    Relay,
}

/// Set the netplay synchronization mode.
///
/// Host only, should be called before game starts.
#[frb]
pub fn netplay_set_sync_mode(mode: SyncMode) -> Result<(), String> {
    let mgr = get_manager();
    let internal_mode = match mode {
        SyncMode::Lockstep => nesium_netplay::SyncMode::Lockstep,
        SyncMode::Rollback => nesium_netplay::SyncMode::Rollback,
    };
    mgr.input_provider.set_sync_mode(internal_mode);
    Ok(())
}

/// Get the current netplay synchronization mode.
#[frb]
pub fn netplay_get_sync_mode() -> SyncMode {
    let mgr = get_manager();
    match mgr.input_provider.sync_mode() {
        nesium_netplay::SyncMode::Lockstep => SyncMode::Lockstep,
        nesium_netplay::SyncMode::Rollback => SyncMode::Rollback,
    }
}

async fn start_netplay_session_with_client(
    client: nesium_netplay::TcpClientHandle,
    event_rx: mpsc::Receiver<nesium_netplay::TcpClientEvent>,
    transport: TransportKind,
    tcp_fallback_from_quic: bool,
    player_name: String,
    room_code: u32,
    desired_role: u8,
    has_rom: bool,
) -> Result<(), String> {
    let mgr = get_manager();

    let _ = netplay_disconnect_inner(false).await;

    let (game_event_tx, mut game_event_rx) = mpsc::channel(32);

    // `client` is already connected; we just need to wire it into the handler.
    mgr.input_provider.with_session(|s| {
        s.tcp_fallback_from_quic = tcp_fallback_from_quic;
    });

    let config = NetplayConfig {
        name: player_name,
        transport,
        spectator: false,
        room_code,
        desired_role,
        has_rom,
    };

    let (mut handler, cmd_tx) = SessionHandler::new(
        client,
        config,
        mgr.input_provider.clone(),
        event_rx,
        game_event_tx,
    );

    *lock_unpoison(&mgr.command_tx) = Some(cmd_tx);

    let status_sink = mgr.status_sink.clone();
    let game_event_sink = mgr.game_event_sink.clone();
    let input_provider = mgr.input_provider.clone();

    tokio::spawn(async move {
        while let Some(event) = game_event_rx.recv().await {
            if let Some(sink) = lock_unpoison(&game_event_sink).as_ref() {
                let frb_event = match event {
                    nesium_netplay::NetplayEvent::LoadRom(data) => {
                        NetplayGameEvent::LoadRom { data }
                    }
                    nesium_netplay::NetplayEvent::StartGame => NetplayGameEvent::StartGame,
                    nesium_netplay::NetplayEvent::PauseSync { paused } => {
                        NetplayGameEvent::PauseSync { paused }
                    }
                    nesium_netplay::NetplayEvent::ResetSync(kind) => {
                        NetplayGameEvent::ResetSync { kind }
                    }
                    nesium_netplay::NetplayEvent::SyncState(frame, data) => {
                        NetplayGameEvent::SyncState { frame, data }
                    }
                    nesium_netplay::NetplayEvent::PlayerLeft { player_index } => {
                        NetplayGameEvent::PlayerLeft { player_index }
                    }
                    nesium_netplay::NetplayEvent::Error { code } => NetplayGameEvent::Error {
                        error_code: code as u16,
                    },
                    nesium_netplay::NetplayEvent::FallbackToRelay {
                        relay_addr,
                        relay_room_code,
                        reason,
                    } => NetplayGameEvent::FallbackToRelay {
                        relay_addr: relay_addr.to_string(),
                        relay_room_code,
                        reason,
                    },
                };
                let _ = sink.add(frb_event);
            }
        }
    });

    let task = tokio::spawn(async move {
        notify_status(&status_sink, &input_provider, None);

        if let Err(e) = handler.run().await {
            let err_msg = e.to_string();
            notify_status(&status_sink, &input_provider, Some(err_msg.clone()));
            return Err(err_msg);
        }

        notify_status(&status_sink, &input_provider, None);
        Ok(())
    });

    *lock_unpoison(&mgr.session_task) = Some(task);

    crate::runtime_handle()
        .enable_netplay(mgr.input_provider.clone())
        .map_err(|e| e.to_string())?;

    Ok(())
}

#[frb(ignore)]
pub struct NetplayManager {
    pub input_provider: Arc<SharedInputProvider>,
    pub session_task: Mutex<Option<tokio::task::JoinHandle<Result<(), String>>>>,
    pub command_tx: Mutex<Option<mpsc::Sender<NetplayCommand>>>,
    pub status_sink: Arc<Mutex<Option<StreamSink<NetplayStatus>>>>,
    pub game_event_sink: Arc<Mutex<Option<StreamSink<NetplayGameEvent>>>>,
    pub polling_task: Mutex<Option<tokio::task::JoinHandle<()>>>,
    pub p2p_watch_task: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

static MANAGER: OnceLock<NetplayManager> = OnceLock::new();

#[frb(ignore)]
pub fn get_manager() -> &'static NetplayManager {
    MANAGER.get_or_init(|| {
        let input_provider = nesium_netplay::create_input_provider();
        NetplayManager {
            input_provider,
            session_task: Mutex::new(None),
            command_tx: Mutex::new(None),
            status_sink: Arc::new(Mutex::new(None)),
            game_event_sink: Arc::new(Mutex::new(None)),
            polling_task: Mutex::new(None),
            p2p_watch_task: Mutex::new(None),
        }
    })
}

/// Connect to netplay server and perform handshake.
#[frb]
pub async fn netplay_connect(server_addr: String, player_name: String) -> Result<(), String> {
    let mgr = get_manager();

    // Stop any existing session
    let _ = netplay_disconnect().await;

    let addr = resolve_addr(&server_addr).await?;

    // Create event channel for TCP client -> handler
    let (event_tx, event_rx) = mpsc::channel(256);
    // Create channel for handler -> Flutter (game events)
    let (game_event_tx, mut game_event_rx) = mpsc::channel(32);

    // Create TCP client
    let client = nesium_netplay::connect(addr, event_tx)
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    mgr.input_provider.with_session(|s| {
        s.tcp_fallback_from_quic = false;
    });

    let config = NetplayConfig {
        name: player_name,
        transport: nesium_netproto::messages::session::TransportKind::Tcp,
        spectator: false,
        room_code: 0,
        desired_role: AUTO_PLAYER_INDEX,
        has_rom: false,
    };

    let (mut handler, cmd_tx) = SessionHandler::new(
        client,
        config,
        mgr.input_provider.clone(),
        event_rx,
        game_event_tx,
    );

    *lock_unpoison(&mgr.command_tx) = Some(cmd_tx);

    let status_sink = mgr.status_sink.clone();
    let game_event_sink = mgr.game_event_sink.clone();
    let input_provider = mgr.input_provider.clone();

    // Spawn event forwarding task
    tokio::spawn(async move {
        while let Some(event) = game_event_rx.recv().await {
            if let Some(sink) = lock_unpoison(&game_event_sink).as_ref() {
                let frb_event = match event {
                    nesium_netplay::NetplayEvent::LoadRom(data) => {
                        NetplayGameEvent::LoadRom { data }
                    }
                    nesium_netplay::NetplayEvent::StartGame => NetplayGameEvent::StartGame,
                    nesium_netplay::NetplayEvent::PauseSync { paused } => {
                        NetplayGameEvent::PauseSync { paused }
                    }
                    nesium_netplay::NetplayEvent::ResetSync(kind) => {
                        NetplayGameEvent::ResetSync { kind }
                    }
                    nesium_netplay::NetplayEvent::SyncState(frame, data) => {
                        NetplayGameEvent::SyncState { frame, data }
                    }
                    nesium_netplay::NetplayEvent::PlayerLeft { player_index } => {
                        NetplayGameEvent::PlayerLeft { player_index }
                    }
                    nesium_netplay::NetplayEvent::Error { code } => NetplayGameEvent::Error {
                        error_code: code as u16,
                    },
                    nesium_netplay::NetplayEvent::FallbackToRelay {
                        relay_addr,
                        relay_room_code,
                        reason,
                    } => NetplayGameEvent::FallbackToRelay {
                        relay_addr: relay_addr.to_string(),
                        relay_room_code,
                        reason,
                    },
                };
                let _ = sink.add(frb_event);
            }
        }
    });

    let task = tokio::spawn(async move {
        // Notify connecting
        notify_status(&status_sink, &input_provider, None);

        if let Err(e) = handler.run().await {
            let err_msg = e.to_string();
            notify_status(&status_sink, &input_provider, Some(err_msg.clone()));
            return Err(err_msg);
        }

        notify_status(&status_sink, &input_provider, None);
        Ok(())
    });

    *lock_unpoison(&mgr.session_task) = Some(task);

    // Enable in runtime
    crate::runtime_handle()
        .enable_netplay(mgr.input_provider.clone())
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Connect to netplay server and perform handshake (QUIC preferred, TCP fallback).
#[frb]
pub async fn netplay_connect_auto(
    server_addr: String,
    server_name: String,
    player_name: String,
) -> Result<(), String> {
    let mgr = get_manager();

    let _ = netplay_disconnect().await;

    let addr = resolve_addr(&server_addr).await?;

    let (event_tx, event_rx) = mpsc::channel(256);
    let (game_event_tx, mut game_event_rx) = mpsc::channel(32);

    let (client, chosen_transport) =
        nesium_netplay::connect_auto(addr, Some(server_name.as_str()), event_tx)
            .await
            .map_err(|e| format!("Failed to connect (auto): {}", e))?;

    mgr.input_provider.with_session(|s| {
        s.tcp_fallback_from_quic =
            chosen_transport == nesium_netproto::messages::session::TransportKind::Tcp;
    });

    let config = NetplayConfig {
        name: player_name,
        transport: chosen_transport,
        spectator: false,
        room_code: 0,
        desired_role: AUTO_PLAYER_INDEX,
        has_rom: false,
    };

    let (mut handler, cmd_tx) = SessionHandler::new(
        client,
        config,
        mgr.input_provider.clone(),
        event_rx,
        game_event_tx,
    );

    *lock_unpoison(&mgr.command_tx) = Some(cmd_tx);

    let status_sink = mgr.status_sink.clone();
    let game_event_sink = mgr.game_event_sink.clone();
    let input_provider = mgr.input_provider.clone();

    tokio::spawn(async move {
        while let Some(event) = game_event_rx.recv().await {
            if let Some(sink) = lock_unpoison(&game_event_sink).as_ref() {
                let frb_event = match event {
                    nesium_netplay::NetplayEvent::LoadRom(data) => {
                        NetplayGameEvent::LoadRom { data }
                    }
                    nesium_netplay::NetplayEvent::StartGame => NetplayGameEvent::StartGame,
                    nesium_netplay::NetplayEvent::PauseSync { paused } => {
                        NetplayGameEvent::PauseSync { paused }
                    }
                    nesium_netplay::NetplayEvent::ResetSync(kind) => {
                        NetplayGameEvent::ResetSync { kind }
                    }
                    nesium_netplay::NetplayEvent::SyncState(frame, data) => {
                        NetplayGameEvent::SyncState { frame, data }
                    }
                    nesium_netplay::NetplayEvent::PlayerLeft { player_index } => {
                        NetplayGameEvent::PlayerLeft { player_index }
                    }
                    nesium_netplay::NetplayEvent::Error { code } => NetplayGameEvent::Error {
                        error_code: code as u16,
                    },
                    nesium_netplay::NetplayEvent::FallbackToRelay {
                        relay_addr,
                        relay_room_code,
                        reason,
                    } => NetplayGameEvent::FallbackToRelay {
                        relay_addr: relay_addr.to_string(),
                        relay_room_code,
                        reason,
                    },
                };
                let _ = sink.add(frb_event);
            }
        }
    });

    let task = tokio::spawn(async move {
        notify_status(&status_sink, &input_provider, None);

        if let Err(e) = handler.run().await {
            let err_msg = e.to_string();
            notify_status(&status_sink, &input_provider, Some(err_msg.clone()));
            return Err(err_msg);
        }

        notify_status(&status_sink, &input_provider, None);
        Ok(())
    });

    *lock_unpoison(&mgr.session_task) = Some(task);

    crate::runtime_handle()
        .enable_netplay(mgr.input_provider.clone())
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Connect to netplay server and perform handshake (QUIC pinned SHA-256 fingerprint preferred, TCP fallback).
#[frb]
pub async fn netplay_connect_auto_pinned(
    server_addr: String,
    server_name: String,
    pinned_sha256_fingerprint: String,
    player_name: String,
) -> Result<(), String> {
    let mgr = get_manager();

    let _ = netplay_disconnect().await;

    let addr = resolve_addr(&server_addr).await?;

    let (event_tx, event_rx) = mpsc::channel(256);
    let (game_event_tx, mut game_event_rx) = mpsc::channel(32);

    let (client, chosen_transport) = nesium_netplay::connect_auto_pinned(
        addr,
        &server_name,
        &pinned_sha256_fingerprint,
        event_tx,
    )
    .await
    .map_err(|e| format!("Failed to connect (auto pinned): {}", e))?;

    mgr.input_provider.with_session(|s| {
        s.tcp_fallback_from_quic =
            chosen_transport == nesium_netproto::messages::session::TransportKind::Tcp;
    });

    let config = NetplayConfig {
        name: player_name,
        transport: chosen_transport,
        spectator: false,
        room_code: 0,
        desired_role: AUTO_PLAYER_INDEX,
        has_rom: false,
    };

    let (mut handler, cmd_tx) = SessionHandler::new(
        client,
        config,
        mgr.input_provider.clone(),
        event_rx,
        game_event_tx,
    );

    *lock_unpoison(&mgr.command_tx) = Some(cmd_tx);

    let status_sink = mgr.status_sink.clone();
    let game_event_sink = mgr.game_event_sink.clone();
    let input_provider = mgr.input_provider.clone();

    tokio::spawn(async move {
        while let Some(event) = game_event_rx.recv().await {
            if let Some(sink) = lock_unpoison(&game_event_sink).as_ref() {
                let frb_event = match event {
                    nesium_netplay::NetplayEvent::LoadRom(data) => {
                        NetplayGameEvent::LoadRom { data }
                    }
                    nesium_netplay::NetplayEvent::StartGame => NetplayGameEvent::StartGame,
                    nesium_netplay::NetplayEvent::PauseSync { paused } => {
                        NetplayGameEvent::PauseSync { paused }
                    }
                    nesium_netplay::NetplayEvent::ResetSync(kind) => {
                        NetplayGameEvent::ResetSync { kind }
                    }
                    nesium_netplay::NetplayEvent::SyncState(frame, data) => {
                        NetplayGameEvent::SyncState { frame, data }
                    }
                    nesium_netplay::NetplayEvent::PlayerLeft { player_index } => {
                        NetplayGameEvent::PlayerLeft { player_index }
                    }
                    nesium_netplay::NetplayEvent::Error { code } => NetplayGameEvent::Error {
                        error_code: code as u16,
                    },
                    nesium_netplay::NetplayEvent::FallbackToRelay {
                        relay_addr,
                        relay_room_code,
                        reason,
                    } => NetplayGameEvent::FallbackToRelay {
                        relay_addr: relay_addr.to_string(),
                        relay_room_code,
                        reason,
                    },
                };
                let _ = sink.add(frb_event);
            }
        }
    });

    let task = tokio::spawn(async move {
        notify_status(&status_sink, &input_provider, None);

        if let Err(e) = handler.run().await {
            let err_msg = e.to_string();
            notify_status(&status_sink, &input_provider, Some(err_msg.clone()));
            return Err(err_msg);
        }

        notify_status(&status_sink, &input_provider, None);
        Ok(())
    });

    *lock_unpoison(&mgr.session_task) = Some(task);

    crate::runtime_handle()
        .enable_netplay(mgr.input_provider.clone())
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Connect to netplay server over QUIC and perform handshake.
#[frb]
pub async fn netplay_connect_quic(
    server_addr: String,
    server_name: String,
    player_name: String,
) -> Result<(), String> {
    let mgr = get_manager();

    let _ = netplay_disconnect().await;

    let addr = resolve_addr(&server_addr).await?;

    let (event_tx, event_rx) = mpsc::channel(256);
    let (game_event_tx, mut game_event_rx) = mpsc::channel(32);

    let client = nesium_netplay::connect_quic(addr, &server_name, event_tx)
        .await
        .map_err(|e| format!("Failed to connect (quic): {}", e))?;

    mgr.input_provider.with_session(|s| {
        s.tcp_fallback_from_quic = false;
    });

    let config = NetplayConfig {
        name: player_name,
        transport: nesium_netproto::messages::session::TransportKind::Quic,
        spectator: false,
        room_code: 0,
        desired_role: AUTO_PLAYER_INDEX,
        has_rom: false,
    };

    let (mut handler, cmd_tx) = SessionHandler::new(
        client,
        config,
        mgr.input_provider.clone(),
        event_rx,
        game_event_tx,
    );

    *lock_unpoison(&mgr.command_tx) = Some(cmd_tx);

    let status_sink = mgr.status_sink.clone();
    let game_event_sink = mgr.game_event_sink.clone();
    let input_provider = mgr.input_provider.clone();

    tokio::spawn(async move {
        while let Some(event) = game_event_rx.recv().await {
            if let Some(sink) = lock_unpoison(&game_event_sink).as_ref() {
                let frb_event = match event {
                    nesium_netplay::NetplayEvent::LoadRom(data) => {
                        NetplayGameEvent::LoadRom { data }
                    }
                    nesium_netplay::NetplayEvent::StartGame => NetplayGameEvent::StartGame,
                    nesium_netplay::NetplayEvent::PauseSync { paused } => {
                        NetplayGameEvent::PauseSync { paused }
                    }
                    nesium_netplay::NetplayEvent::ResetSync(kind) => {
                        NetplayGameEvent::ResetSync { kind }
                    }
                    nesium_netplay::NetplayEvent::SyncState(frame, data) => {
                        NetplayGameEvent::SyncState { frame, data }
                    }
                    nesium_netplay::NetplayEvent::PlayerLeft { player_index } => {
                        NetplayGameEvent::PlayerLeft { player_index }
                    }
                    nesium_netplay::NetplayEvent::Error { code } => NetplayGameEvent::Error {
                        error_code: code as u16,
                    },
                    nesium_netplay::NetplayEvent::FallbackToRelay {
                        relay_addr,
                        relay_room_code,
                        reason,
                    } => NetplayGameEvent::FallbackToRelay {
                        relay_addr: relay_addr.to_string(),
                        relay_room_code,
                        reason,
                    },
                };
                let _ = sink.add(frb_event);
            }
        }
    });

    let task = tokio::spawn(async move {
        notify_status(&status_sink, &input_provider, None);

        if let Err(e) = handler.run().await {
            let err_msg = e.to_string();
            notify_status(&status_sink, &input_provider, Some(err_msg.clone()));
            return Err(err_msg);
        }

        notify_status(&status_sink, &input_provider, None);
        Ok(())
    });

    *lock_unpoison(&mgr.session_task) = Some(task);

    crate::runtime_handle()
        .enable_netplay(mgr.input_provider.clone())
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Connect to netplay server over QUIC (pinned SHA-256 fingerprint) and perform handshake.
#[frb]
pub async fn netplay_connect_quic_pinned(
    server_addr: String,
    server_name: String,
    pinned_sha256_fingerprint: String,
    player_name: String,
) -> Result<(), String> {
    let mgr = get_manager();

    let _ = netplay_disconnect().await;

    let addr = resolve_addr(&server_addr).await?;

    let (event_tx, event_rx) = mpsc::channel(256);
    let (game_event_tx, mut game_event_rx) = mpsc::channel(32);

    let client = nesium_netplay::connect_quic_pinned(
        addr,
        &server_name,
        &pinned_sha256_fingerprint,
        event_tx,
    )
    .await
    .map_err(|e| format!("Failed to connect (quic pinned): {}", e))?;

    mgr.input_provider.with_session(|s| {
        s.tcp_fallback_from_quic = false;
    });

    let config = NetplayConfig {
        name: player_name,
        transport: nesium_netproto::messages::session::TransportKind::Quic,
        spectator: false,
        room_code: 0,
        desired_role: AUTO_PLAYER_INDEX,
        has_rom: false,
    };

    let (mut handler, cmd_tx) = SessionHandler::new(
        client,
        config,
        mgr.input_provider.clone(),
        event_rx,
        game_event_tx,
    );

    *lock_unpoison(&mgr.command_tx) = Some(cmd_tx);

    let status_sink = mgr.status_sink.clone();
    let game_event_sink = mgr.game_event_sink.clone();
    let input_provider = mgr.input_provider.clone();

    tokio::spawn(async move {
        while let Some(event) = game_event_rx.recv().await {
            if let Some(sink) = lock_unpoison(&game_event_sink).as_ref() {
                let frb_event = match event {
                    nesium_netplay::NetplayEvent::LoadRom(data) => {
                        NetplayGameEvent::LoadRom { data }
                    }
                    nesium_netplay::NetplayEvent::StartGame => NetplayGameEvent::StartGame,
                    nesium_netplay::NetplayEvent::PauseSync { paused } => {
                        NetplayGameEvent::PauseSync { paused }
                    }
                    nesium_netplay::NetplayEvent::ResetSync(kind) => {
                        NetplayGameEvent::ResetSync { kind }
                    }
                    nesium_netplay::NetplayEvent::SyncState(frame, data) => {
                        NetplayGameEvent::SyncState { frame, data }
                    }
                    nesium_netplay::NetplayEvent::PlayerLeft { player_index } => {
                        NetplayGameEvent::PlayerLeft { player_index }
                    }
                    nesium_netplay::NetplayEvent::Error { code } => NetplayGameEvent::Error {
                        error_code: code as u16,
                    },
                    nesium_netplay::NetplayEvent::FallbackToRelay {
                        relay_addr,
                        relay_room_code,
                        reason,
                    } => NetplayGameEvent::FallbackToRelay {
                        relay_addr: relay_addr.to_string(),
                        relay_room_code,
                        reason,
                    },
                };
                let _ = sink.add(frb_event);
            }
        }
    });

    let task = tokio::spawn(async move {
        notify_status(&status_sink, &input_provider, None);

        if let Err(e) = handler.run().await {
            let err_msg = e.to_string();
            notify_status(&status_sink, &input_provider, Some(err_msg.clone()));
            return Err(err_msg);
        }

        notify_status(&status_sink, &input_provider, None);
        Ok(())
    });

    *lock_unpoison(&mgr.session_task) = Some(task);

    crate::runtime_handle()
        .enable_netplay(mgr.input_provider.clone())
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Create a new netplay room.
#[frb]
pub async fn netplay_create_room() -> Result<(), String> {
    let mgr = get_manager();
    let tx = lock_unpoison(&mgr.command_tx).clone();
    if let Some(tx) = tx {
        tx.send(NetplayCommand::CreateRoom)
            .await
            .map_err(|e| format!("Failed to send command: {}", e))?;
        Ok(())
    } else {
        Err("Not connected".to_string())
    }
}

/// Join an existing netplay room by code.
#[frb]
pub async fn netplay_join_room(
    room_code: u32,
    desired_role: u8,
    has_rom: bool,
) -> Result<(), String> {
    let valid_role = desired_role < 4
        || desired_role == AUTO_PLAYER_INDEX
        || desired_role == SPECTATOR_PLAYER_INDEX;
    if !valid_role {
        return Err(format!("Invalid desired_role: {}", desired_role));
    }

    let mgr = get_manager();
    let tx = lock_unpoison(&mgr.command_tx).clone();
    if let Some(tx) = tx {
        tx.send(NetplayCommand::JoinRoom {
            room_code,
            desired_role,
            has_rom,
        })
        .await
        .map_err(|e| format!("Failed to send command: {}", e))?;
        Ok(())
    } else {
        Err("Not connected".to_string())
    }
}

/// Query room occupancy/state by join code (before joining).
#[frb]
pub async fn netplay_query_room(room_code: u32) -> Result<NetplayRoomInfo, String> {
    let mgr = get_manager();
    let tx = lock_unpoison(&mgr.command_tx).clone();
    let Some(tx) = tx else {
        return Err("Not connected".to_string());
    };

    let (resp_tx, resp_rx) = oneshot::channel();
    tx.send(NetplayCommand::QueryRoom {
        room_code,
        resp: resp_tx,
    })
    .await
    .map_err(|e| format!("Failed to send command: {}", e))?;

    let info = tokio::time::timeout(Duration::from_secs(2), resp_rx)
        .await
        .map_err(|_| "Timed out waiting for room info".to_string())?
        .map_err(|_| "Room info response canceled".to_string())?
        .map_err(|e| e)?;

    Ok(NetplayRoomInfo {
        ok: info.ok,
        room_id: info.room_id,
        started: info.started,
        sync_mode: match info.sync_mode {
            nesium_netproto::messages::session::SyncMode::Lockstep => SyncMode::Lockstep,
            nesium_netproto::messages::session::SyncMode::Rollback => SyncMode::Rollback,
        },
        occupied_mask: info.occupied_mask,
    })
}

/// Switch player role (1P, 2P, Spectator).
#[frb]
pub async fn netplay_switch_role(role: u8) -> Result<(), String> {
    let mgr = get_manager();
    let tx = lock_unpoison(&mgr.command_tx).clone();
    if let Some(tx) = tx {
        tx.send(NetplayCommand::SwitchRole(role))
            .await
            .map_err(|e| format!("Failed to send command: {}", e))?;
        Ok(())
    } else {
        Err("Not connected".to_string())
    }
}

/// Host-only: ask the current server to instruct all connected clients to reconnect to relay mode.
#[frb]
pub async fn netplay_request_fallback_relay(
    relay_addr: String,
    relay_room_code: u32,
    reason: String,
) -> Result<(), String> {
    let mgr = get_manager();
    let relay_addr_parsed = resolve_addr(&relay_addr).await?;

    let tx = lock_unpoison(&mgr.command_tx).clone();
    if let Some(tx) = tx {
        tx.send(NetplayCommand::RequestFallbackRelay {
            relay_addr: relay_addr_parsed,
            relay_room_code,
            reason,
        })
        .await
        .map_err(|e| format!("Failed to send command: {}", e))?;
        Ok(())
    } else {
        Err("Not connected".to_string())
    }
}

/// Disconnect from netplay server.
#[frb]
pub async fn netplay_disconnect() -> Result<(), String> {
    // Do not stop P2P host signaling watcher here.
    // Hosting is controlled by the embedded server lifecycle (see `netserver_stop`), and
    // disconnecting from a netplay session should not implicitly de-register an active P2P host.
    netplay_disconnect_inner(false).await
}

#[frb]
pub async fn netplay_is_connected() -> bool {
    let mgr = get_manager();
    if lock_unpoison(&mgr.command_tx).is_none() {
        return false;
    }
    mgr.input_provider
        .with_session(|s| !matches!(s.state, SessionState::Disconnected))
}

/// Subscribe to netplay status updates.
#[frb]
pub async fn netplay_status_stream(sink: StreamSink<NetplayStatus>) -> Result<(), String> {
    let mgr = get_manager();
    *lock_unpoison(&mgr.status_sink) = Some(sink);

    // Cancel old polling task
    if let Some(task) = lock_unpoison(&mgr.polling_task).take() {
        task.abort();
    }

    // Spawn new polling task
    let status_sink = mgr.status_sink.clone();
    let input_provider = mgr.input_provider.clone();

    let task = tokio::spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            notify_status(&status_sink, &input_provider, None);
        }
    });

    *lock_unpoison(&mgr.polling_task) = Some(task);

    // Send initial status
    notify_status(&mgr.status_sink, &mgr.input_provider, None);

    Ok(())
}

/// Subscribe to Game events (LoadRom, StartGame).
#[frb]
pub async fn netplay_game_event_stream(sink: StreamSink<NetplayGameEvent>) -> Result<(), String> {
    let mgr = get_manager();
    *lock_unpoison(&mgr.game_event_sink) = Some(sink);
    Ok(())
}

async fn netplay_disconnect_inner(stop_p2p_watch: bool) -> Result<(), String> {
    let mgr = get_manager();

    if let Some(task) = lock_unpoison(&mgr.session_task).take() {
        task.abort();
    }
    if stop_p2p_watch {
        if let Some(task) = lock_unpoison(&mgr.p2p_watch_task).take() {
            task.abort();
        }
    }

    *lock_unpoison(&mgr.command_tx) = None;

    mgr.input_provider.set_active(false);
    mgr.input_provider.with_session(|s| s.reset());
    let _ = crate::runtime_handle().disable_netplay();

    notify_status(&mgr.status_sink, &mgr.input_provider, None);
    Ok(())
}

async fn signaling_connect_and_handshake(
    signaling_addr: SocketAddr,
    name: &str,
) -> Result<tokio::net::TcpStream, String> {
    let mut stream = tokio::net::TcpStream::connect(signaling_addr)
        .await
        .map_err(|e| format!("Failed to connect signaling server: {e}"))?;

    let hello = Hello {
        client_nonce: 0,
        transport: TransportKind::Tcp,
        proto_min: nesium_netproto::constants::VERSION,
        proto_max: nesium_netproto::constants::VERSION,
        name: name.to_string(),
    };
    let frame = encode_message(&hello).map_err(|e| format!("Failed to encode Hello: {e}"))?;
    stream
        .write_all(&frame)
        .await
        .map_err(|e| format!("Failed to send Hello: {e}"))?;

    let mut buf = Vec::<u8>::with_capacity(4096);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    loop {
        if tokio::time::Instant::now() > deadline {
            return Err("Timed out waiting for Welcome".to_string());
        }

        let mut tmp = [0u8; 4096];
        let n = tokio::time::timeout(Duration::from_millis(500), stream.read(&mut tmp))
            .await
            .map_err(|_| "Timed out waiting for Welcome".to_string())?
            .map_err(|e| format!("Failed to read Welcome: {e}"))?;
        if n == 0 {
            return Err("Signaling server closed connection".to_string());
        }

        buf.extend_from_slice(&tmp[..n]);
        let (packets, consumed) =
            try_decode_tcp_frames(&buf).map_err(|e| format!("Protocol decode error: {e}"))?;

        if let Some(pkt) = packets.iter().find(|p| p.msg_id() == MsgId::Welcome) {
            let _: Welcome =
                postcard::from_bytes(pkt.payload).map_err(|e| format!("Bad Welcome: {e}"))?;
            break;
        }

        buf.drain(..consumed);
    }

    Ok(stream)
}

async fn signaling_request<
    TReq: nesium_netproto::messages::Message,
    TResp: serde::de::DeserializeOwned,
>(
    signaling_addr: SocketAddr,
    name: &str,
    req: &TReq,
    want: MsgId,
) -> Result<TResp, String> {
    let mut stream = signaling_connect_and_handshake(signaling_addr, name).await?;

    let frame = encode_message(req).map_err(|e| format!("Encode failed: {e}"))?;
    stream
        .write_all(&frame)
        .await
        .map_err(|e| format!("Send failed: {e}"))?;

    let mut buf = Vec::<u8>::with_capacity(4096);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
    loop {
        if tokio::time::Instant::now() > deadline {
            return Err(format!("Timed out waiting for {:?}", want));
        }

        let mut tmp = [0u8; 4096];
        let n = tokio::time::timeout(Duration::from_millis(500), stream.read(&mut tmp))
            .await
            .map_err(|_| format!("Timed out waiting for {:?}", want))?
            .map_err(|e| format!("Read failed: {e}"))?;
        if n == 0 {
            return Err("Signaling server closed connection".to_string());
        }

        buf.extend_from_slice(&tmp[..n]);
        let (packets, consumed) =
            try_decode_tcp_frames(&buf).map_err(|e| format!("Protocol decode error: {e}"))?;

        if let Some(pkt) = packets.iter().find(|p| p.msg_id() == MsgId::ErrorMsg) {
            let msg: ErrorMsg =
                postcard::from_bytes(pkt.payload).map_err(|e| format!("Bad error msg: {e}"))?;
            return Err(format!("Signaling server error: {:?}", msg.code));
        }

        if let Some(pkt) = packets.iter().find(|p| p.msg_id() == want) {
            let resp: TResp =
                postcard::from_bytes(pkt.payload).map_err(|e| format!("Bad response: {e}"))?;
            return Ok(resp);
        }

        buf.drain(..consumed);
    }
}

/// Create a P2P signaling room on `nesium-netd` and publish direct-connect info for the host.
///
/// Returns the room code that joiners should use on the signaling server (and for relay fallback).
#[frb]
pub async fn netplay_p2p_create_room(
    signaling_addr: String,
    host_addrs: Vec<String>,
    host_room_code: u32,
    host_quic_cert_sha256_fingerprint: Option<String>,
    host_quic_server_name: Option<String>,
    name: String,
) -> Result<u32, String> {
    let signaling_addr = resolve_addr(&signaling_addr).await?;

    let host_addrs: Vec<SocketAddr> = host_addrs
        .into_iter()
        .map(|s| {
            s.parse()
                .map_err(|e| format!("Invalid host addr '{s}': {e}"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let req = P2PCreateRoom {
        host_addrs,
        host_room_code,
        host_quic_cert_sha256_fingerprint,
        host_quic_server_name,
    };

    let resp: P2PRoomCreated =
        signaling_request(signaling_addr, &name, &req, MsgId::P2PRoomCreated).await?;
    Ok(resp.room_code)
}

/// Start the full P2P host workflow:
/// 1. Start embedded server (OS-assigned port).
/// 2. Discover local IPs and Public IP.
/// 3. Create P2P room on signaling server.
/// 4. Watch for fallback notices.
///
/// Returns the P2P room code.
#[frb]
pub async fn netplay_p2p_host_start(
    signaling_addr: String,
    relay_addr: String,
    player_name: String,
) -> Result<u32, String> {
    // Preflight: ensure signaling server is reachable before starting the embedded server.
    // This avoids leaving a running embedded server when P2P signaling is misconfigured.
    let signaling_addr_parsed = resolve_addr(&signaling_addr).await?;
    let _ = signaling_connect_and_handshake(signaling_addr_parsed, &player_name).await?;

    // 1. Ensure server is running
    let started_server = !netserver_is_running();
    let port = if started_server {
        netserver_start(0).await?
    } else {
        netserver_get_port()
    };

    // 2. Try UPnP mapping (best effort)
    try_upnp_mapping(port, "nesium");

    // 3. Discover addresses
    let mut host_addrs = get_local_addrs();
    if let Some(pub_ip) = get_public_ip() {
        if !host_addrs.contains(&pub_ip) {
            host_addrs.push(pub_ip);
        }
    }

    let host_addrs_str: Vec<String> = host_addrs
        .into_iter()
        .map(|ip| format!("{}:{}", ip, port))
        .collect();

    // 3. Get QUIC fingerprint (if available)
    let fingerprint = get_server().lock().quic_cert_sha256_fingerprint.clone();

    // 4. Register and watch fallback
    let res = netplay_p2p_host_create_and_watch_fallback(
        signaling_addr,
        relay_addr,
        host_addrs_str,
        0, // host_room_code = 0 means default room
        fingerprint,
        Some("localhost".to_string()),
        player_name,
    )
    .await;

    if res.is_err() && started_server {
        let _ = crate::api::server::netserver_stop().await;
    }

    res
}

/// Host flow:
/// - Creates a P2P signaling room on `signaling_addr` (netd).
/// - Spawns a background watcher for `P2PFallbackNotice`.
/// - On fallback notice: tells direct clients to reconnect to `relay_addr`, stops the embedded
///   server (best-effort), and connects this device to relay mode as a client.
///
/// UI requirement:
/// - `relay_addr` is user-provided (manual input), e.g. `example.com:15000` or `1.2.3.4:15000`.
#[frb]
pub async fn netplay_p2p_host_create_and_watch_fallback(
    signaling_addr: String,
    relay_addr: String,
    host_addrs: Vec<String>,
    host_room_code: u32,
    host_quic_cert_sha256_fingerprint: Option<String>,
    host_quic_server_name: Option<String>,
    player_name: String,
) -> Result<u32, String> {
    let signaling_addr_parsed = resolve_addr(&signaling_addr).await?;
    let relay_addr_parsed = resolve_addr(&relay_addr).await?;

    let host_addrs: Vec<SocketAddr> = host_addrs
        .into_iter()
        .map(|s| {
            s.parse()
                .map_err(|e| format!("Invalid host addr '{s}': {e}"))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut stream = signaling_connect_and_handshake(signaling_addr_parsed, &player_name).await?;
    let create = P2PCreateRoom {
        host_addrs,
        host_room_code,
        host_quic_cert_sha256_fingerprint,
        host_quic_server_name,
    };

    let frame = encode_message(&create).map_err(|e| format!("Encode P2PCreateRoom failed: {e}"))?;
    stream
        .write_all(&frame)
        .await
        .map_err(|e| format!("Send P2PCreateRoom failed: {e}"))?;

    // Read P2PRoomCreated on the same connection.
    let mut buf = Vec::<u8>::with_capacity(4096);
    let created: P2PRoomCreated = {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        loop {
            if tokio::time::Instant::now() > deadline {
                return Err("Timed out waiting for P2PRoomCreated".to_string());
            }
            let mut tmp = [0u8; 4096];
            let n = tokio::time::timeout(Duration::from_millis(500), stream.read(&mut tmp))
                .await
                .map_err(|_| "Timed out waiting for P2PRoomCreated".to_string())?
                .map_err(|e| format!("Read failed: {e}"))?;
            if n == 0 {
                return Err("Signaling server closed connection".to_string());
            }
            buf.extend_from_slice(&tmp[..n]);
            let (packets, consumed) =
                try_decode_tcp_frames(&buf).map_err(|e| format!("Decode failed: {e}"))?;
            if let Some(pkt) = packets.iter().find(|p| p.msg_id() == MsgId::ErrorMsg) {
                let msg: ErrorMsg =
                    postcard::from_bytes(pkt.payload).map_err(|e| format!("Bad error msg: {e}"))?;
                return Err(format!("Signaling server error: {:?}", msg.code));
            }
            if let Some(pkt) = packets.iter().find(|p| p.msg_id() == MsgId::P2PRoomCreated) {
                let v: P2PRoomCreated =
                    postcard::from_bytes(pkt.payload).map_err(|e| format!("Bad response: {e}"))?;
                buf.drain(..consumed);
                break v;
            }
            buf.drain(..consumed);
        }
    };

    let room_code = created.room_code;
    if room_code == 0 {
        return Err("Signaling server returned room_code=0".to_string());
    }

    // Replace any existing watcher.
    let mgr = get_manager();
    if let Some(task) = lock_unpoison(&mgr.p2p_watch_task).take() {
        task.abort();
    }

    let player_name_clone = player_name.clone();

    let watch = tokio::spawn(async move {
        let mut buf = buf;
        let mut stream = stream;

        loop {
            let mut fallback_triggered = false;
            let mut tmp = [0u8; 4096];
            let n = match stream.read(&mut tmp).await {
                Ok(n) => n,
                Err(e) => {
                    tracing::warn!(error = %e, "P2P signaling watch read error");
                    break;
                }
            };
            if n == 0 {
                tracing::warn!("P2P signaling watch EOF");
                break;
            }

            buf.extend_from_slice(&tmp[..n]);
            let (packets, consumed) = match try_decode_tcp_frames(&buf) {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!(error = %e, "P2P signaling watch decode error");
                    break;
                }
            };

            for pkt in packets {
                if pkt.msg_id() != MsgId::P2PFallbackNotice {
                    continue;
                }

                let notice: P2PFallbackNotice = match postcard::from_bytes(pkt.payload) {
                    Ok(v) => v,
                    Err(e) => {
                        tracing::warn!(error = %e, "Bad P2PFallbackNotice");
                        continue;
                    }
                };

                tracing::warn!(
                    room_code = notice.room_code,
                    reason = %notice.reason,
                    requested_by_client_id = notice.requested_by_client_id,
                    "P2P fallback notice received"
                );

                let _ = netplay_request_fallback_relay(
                    relay_addr.clone(),
                    room_code,
                    notice.reason.clone(),
                )
                .await;

                // 2) Stop embedded server (best-effort) to avoid confusing users with an active local host server.
                let _ = crate::api::server::netserver_stop().await;

                // 3) Switch this device to relay mode and join the same room_code on relay server.
                let (event_tx, event_rx) = mpsc::channel(256);
                match nesium_netplay::connect(relay_addr_parsed, event_tx).await {
                    Ok(client) => {
                        let _ = start_netplay_session_with_client(
                            client,
                            event_rx,
                            nesium_netproto::messages::session::TransportKind::Tcp,
                            false,
                            player_name_clone.clone(),
                            room_code,
                            0,
                            true,
                        )
                        .await;
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "Failed to connect relay after fallback");
                    }
                }

                fallback_triggered = true;
                break;
            }

            buf.drain(..consumed);
            if fallback_triggered {
                break;
            }
        }
    });

    *lock_unpoison(&mgr.p2p_watch_task) = Some(watch);
    Ok(room_code)
}

/// Fetch host direct-connect info for a given P2P signaling room.
#[frb]
pub async fn netplay_p2p_join_room(
    signaling_addr: String,
    room_code: u32,
    name: String,
) -> Result<P2PJoinInfo, String> {
    let signaling_addr = resolve_addr(&signaling_addr).await?;

    let req = nesium_netproto::messages::session::P2PJoinRoom { room_code };
    let ack: nesium_netproto::messages::session::P2PJoinAck = signaling_request(
        signaling_addr,
        &name,
        &req,
        nesium_netproto::msg_id::MsgId::P2PJoinAck,
    )
    .await?;

    Ok(P2PJoinInfo {
        ok: ack.ok,
        room_code: ack.room_code,
        host_addrs: ack.host_addrs.iter().map(|a| a.to_string()).collect(),
        host_room_code: ack.host_room_code,
        host_quic_cert_sha256_fingerprint: ack.host_quic_cert_sha256_fingerprint,
        host_quic_server_name: ack.host_quic_server_name,
        fallback_required: ack.fallback_required,
        fallback_reason: ack.fallback_reason,
    })
}

/// Join a P2P room via signaling, then try direct-connect to the host server.
///
/// If direct-connect fails, requests relay fallback on the signaling server and connects to the
/// user-provided relay server address (netd authoritative mode).
///
/// Notes:
/// - `relay_addr` must point to a running `nesium-netd` instance.
/// - `room_code` is the P2P signaling room code, and is reused as the relay room code in fallback mode.
#[frb]
pub async fn netplay_p2p_connect_join_auto(
    signaling_addr: String,
    relay_addr: String,
    room_code: u32,
    player_name: String,
    desired_role: u8,
    has_rom: bool,
) -> Result<P2PConnectMode, String> {
    let valid_role = desired_role < 4
        || desired_role == AUTO_PLAYER_INDEX
        || desired_role == SPECTATOR_PLAYER_INDEX;
    if !valid_role {
        return Err(format!("Invalid desired_role: {}", desired_role));
    }

    let signaling_addr_parsed = resolve_addr(&signaling_addr).await?;
    let relay_addr_parsed = resolve_addr(&relay_addr).await?;

    let ack: P2PJoinAck = signaling_request(
        signaling_addr_parsed,
        &player_name,
        &P2PJoinRoom { room_code },
        MsgId::P2PJoinAck,
    )
    .await?;

    if !ack.ok {
        return Err("P2P join rejected by signaling server".to_string());
    }

    if ack.fallback_required {
        // Skip direct connect and use relay immediately.
        let (event_tx, event_rx) = mpsc::channel(256);
        let client = nesium_netplay::connect(relay_addr_parsed, event_tx)
            .await
            .map_err(|e| format!("Failed to connect relay: {e}"))?;
        start_netplay_session_with_client(
            client,
            event_rx,
            TransportKind::Tcp,
            false,
            player_name,
            room_code,
            desired_role,
            has_rom,
        )
        .await?;
        return Ok(P2PConnectMode::Relay);
    }

    // Try direct connect to host addresses in order.
    let mut last_err: Option<String> = None;
    for host_addr in &ack.host_addrs {
        let (event_tx, event_rx) = mpsc::channel(256);

        let attempt = async {
            if let (Some(fp), Some(server_name)) = (
                ack.host_quic_cert_sha256_fingerprint.as_deref(),
                ack.host_quic_server_name.as_deref(),
            ) {
                nesium_netplay::connect_auto_pinned(*host_addr, server_name, fp, event_tx)
                    .await
                    .map(|(c, t)| (c, t))
            } else if let Some(server_name) = ack.host_quic_server_name.as_deref() {
                nesium_netplay::connect_auto(*host_addr, Some(server_name), event_tx)
                    .await
                    .map(|(c, t)| (c, t))
            } else {
                nesium_netplay::connect(*host_addr, event_tx)
                    .await
                    .map(|c| (c, TransportKind::Tcp))
            }
        };

        match attempt.await {
            Ok((client, transport)) => {
                let tcp_fallback_from_quic = match transport {
                    TransportKind::Tcp => ack.host_quic_server_name.is_some(),
                    TransportKind::Quic => false,
                };
                start_netplay_session_with_client(
                    client,
                    event_rx,
                    transport,
                    tcp_fallback_from_quic,
                    player_name,
                    ack.host_room_code,
                    desired_role,
                    has_rom,
                )
                .await?;
                return Ok(P2PConnectMode::Direct);
            }
            Err(e) => {
                last_err = Some(format!("{e}"));
            }
        }
    }

    // Direct connect failed -> request fallback, then connect to relay server.
    let reason = last_err.unwrap_or_else(|| "direct connect failed".to_string());
    let _ = netplay_p2p_request_fallback(
        signaling_addr.clone(),
        room_code,
        reason.clone(),
        player_name.clone(),
    )
    .await;

    let (event_tx, event_rx) = mpsc::channel(256);
    let client = nesium_netplay::connect(relay_addr_parsed, event_tx)
        .await
        .map_err(|e| format!("Failed to connect relay: {e}"))?;
    start_netplay_session_with_client(
        client,
        event_rx,
        TransportKind::Tcp,
        false,
        player_name,
        room_code,
        desired_role,
        has_rom,
    )
    .await?;

    Ok(P2PConnectMode::Relay)
}

/// Request switching a P2P signaling room into relay fallback mode (netd authoritative C/S).
#[frb]
pub async fn netplay_p2p_request_fallback(
    signaling_addr: String,
    room_code: u32,
    reason: String,
    name: String,
) -> Result<(), String> {
    let signaling_addr: SocketAddr = signaling_addr
        .parse()
        .map_err(|e| format!("Invalid signaling address: {e}"))?;

    let mut stream = signaling_connect_and_handshake(signaling_addr, &name).await?;
    let req = nesium_netproto::messages::session::P2PRequestFallback { room_code, reason };
    let frame = encode_message(&req).map_err(|e| format!("Encode failed: {e}"))?;
    stream
        .write_all(&frame)
        .await
        .map_err(|e| format!("Send failed: {e}"))?;
    Ok(())
}

/// Send ROM to other players.
#[frb]
pub async fn netplay_send_rom(data: Vec<u8>) -> Result<(), String> {
    let mgr = get_manager();
    let (in_room, local_player_index) = mgr.input_provider.with_session(|s| {
        let in_room = matches!(
            s.state,
            SessionState::Playing { .. }
                | SessionState::Spectating { .. }
                | SessionState::Syncing { .. }
        );
        (in_room, s.local_player_index)
    });

    if !in_room {
        return Err("Not in a room".to_string());
    }

    // Spectators cannot broadcast ROM (but can receive it).
    if local_player_index.is_none() || local_player_index == Some(SPECTATOR_PLAYER_INDEX) {
        return Err("Spectator cannot load/broadcast the ROM".to_string());
    }

    let tx = lock_unpoison(&mgr.command_tx).clone();
    if let Some(tx) = tx {
        tx.send(NetplayCommand::SendRom(data))
            .await
            .map_err(|e| format!("Failed to send command: {}", e))?;
        Ok(())
    } else {
        Err("Not connected".to_string())
    }
}

/// Confirm ROM loaded to server.
#[frb]
pub async fn netplay_send_rom_loaded() -> Result<(), String> {
    let mgr = get_manager();
    let tx = lock_unpoison(&mgr.command_tx).clone();
    if let Some(tx) = tx {
        tx.send(NetplayCommand::RomLoaded)
            .await
            .map_err(|e| format!("Failed to send command: {}", e))?;
        Ok(())
    } else {
        Err("Not connected".to_string())
    }
}

/// Send pause state to other players.
#[frb]
pub async fn netplay_send_pause(paused: bool) -> Result<(), String> {
    let mgr = get_manager();
    let tx = lock_unpoison(&mgr.command_tx).clone();
    if let Some(tx) = tx {
        tx.send(NetplayCommand::SendPause(paused))
            .await
            .map_err(|e| format!("Failed to send command: {}", e))?;
        Ok(())
    } else {
        Err("Not connected".to_string())
    }
}

/// Send reset to other players.
#[frb]
pub async fn netplay_send_reset(kind: u8) -> Result<(), String> {
    let mgr = get_manager();
    let tx = lock_unpoison(&mgr.command_tx).clone();
    if let Some(tx) = tx {
        tx.send(NetplayCommand::SendReset(kind))
            .await
            .map_err(|e| format!("Failed to send command: {}", e))?;
        Ok(())
    } else {
        Err("Not connected".to_string())
    }
}

/// Request current game state from server (for reconnection).
#[frb]
pub async fn netplay_request_state() -> Result<(), String> {
    let mgr = get_manager();
    let tx = lock_unpoison(&mgr.command_tx).clone();
    if let Some(tx) = tx {
        tx.send(NetplayCommand::RequestState)
            .await
            .map_err(|e| format!("Failed to send command: {}", e))?;
        Ok(())
    } else {
        Err("Not connected".to_string())
    }
}

/// Provide current game state to server for caching.
#[frb]
pub async fn netplay_provide_state(frame: u32, data: Vec<u8>) -> Result<(), String> {
    let mgr = get_manager();
    let tx = lock_unpoison(&mgr.command_tx).clone();
    if let Some(tx) = tx {
        tx.send(NetplayCommand::ProvideState(frame, data))
            .await
            .map_err(|e| format!("Failed to send command: {}", e))?;
        Ok(())
    } else {
        Err("Not connected".to_string())
    }
}

fn notify_status(
    sink_lock: &Arc<Mutex<Option<StreamSink<NetplayStatus>>>>,
    input_provider: &Arc<SharedInputProvider>,
    error: Option<String>,
) {
    if let Some(sink) = lock_unpoison(sink_lock).as_ref() {
        let sync_mode = match input_provider.sync_mode() {
            nesium_netplay::SyncMode::Lockstep => SyncMode::Lockstep,
            nesium_netplay::SyncMode::Rollback => SyncMode::Rollback,
        };

        let (state, transport, tcp_fallback_from_quic, client_id, room_id, player_index, players) =
            input_provider.with_session(|s| {
                let state = match s.state {
                    SessionState::Disconnected => NetplayState::Disconnected,
                    SessionState::Connecting | SessionState::Handshake => NetplayState::Connecting,
                    SessionState::WaitingForRoom => NetplayState::Connected,
                    SessionState::Playing { .. }
                    | SessionState::Spectating { .. }
                    | SessionState::Syncing { .. } => NetplayState::InRoom,
                };

                let transport = match s.transport {
                    Some(nesium_netproto::messages::session::TransportKind::Tcp) => {
                        NetplayTransport::Tcp
                    }
                    Some(nesium_netproto::messages::session::TransportKind::Quic) => {
                        NetplayTransport::Quic
                    }
                    None => NetplayTransport::Unknown,
                };

                let mut players: Vec<NetplayPlayer> = s
                    .players
                    .values()
                    .map(|p| NetplayPlayer {
                        client_id: p.client_id,
                        name: p.name.clone(),
                        player_index: p.player_index,
                    })
                    .collect();

                // Include self if in a room
                if matches!(state, NetplayState::InRoom) {
                    players.push(NetplayPlayer {
                        client_id: s.client_id,
                        name: s.local_name.clone(),
                        player_index: s.local_player_index.unwrap_or(SPECTATOR_PLAYER_INDEX),
                    });
                }

                // Sort players by player_index
                players.sort_by_key(|p| p.player_index);

                (
                    state,
                    transport,
                    s.tcp_fallback_from_quic,
                    s.client_id,
                    s.room_id,
                    s.local_player_index.unwrap_or(SPECTATOR_PLAYER_INDEX),
                    players,
                )
            });

        let status = NetplayStatus {
            state,
            transport,
            tcp_fallback_from_quic,
            client_id,
            room_id,
            player_index,
            players,
            sync_mode,
            error,
        };

        let _ = sink.add(status);
    }
}

/// Lock a `parking_lot::Mutex` (compatibility wrapper).
fn lock_unpoison<T>(mutex: &Mutex<T>) -> parking_lot::MutexGuard<'_, T> {
    mutex.lock()
}
