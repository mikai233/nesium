//! Netplay control API for Flutter.

use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use nesium_netplay::{
    NetplayCommand, NetplayConfig, SPECTATOR_PLAYER_INDEX, SessionHandler, SessionState,
    SharedInputProvider,
};
use std::sync::{Arc, Mutex, OnceLock};
use tokio::sync::mpsc;

/// Netplay connection state.
#[frb]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetplayState {
    Disconnected,
    Connecting,
    Connected,
    InRoom,
}

/// Netplay status snapshot streamed to Flutter.
#[frb]
#[derive(Debug, Clone)]
pub struct NetplayStatus {
    pub state: NetplayState,
    pub client_id: u32,
    pub room_id: u32,
    /// Player index: 0, 1, or `SPECTATOR_PLAYER_INDEX` for spectator
    pub player_index: u8,
    pub error: Option<String>,
}

#[frb]
#[derive(Debug, Clone)]
pub enum NetplayGameEvent {
    LoadRom { data: Vec<u8> },
    StartGame,
    PauseSync { paused: bool },
    ResetSync { kind: u8 },
    SyncState { frame: u32, data: Vec<u8> },
    PlayerLeft { player_index: u8 },
}

#[frb(ignore)]
pub struct NetplayManager {
    pub input_provider: Arc<SharedInputProvider>,
    pub session_task: Mutex<Option<tokio::task::JoinHandle<Result<(), String>>>>,
    pub command_tx: Mutex<Option<mpsc::Sender<NetplayCommand>>>,
    pub status_sink: Arc<Mutex<Option<StreamSink<NetplayStatus>>>>,
    pub game_event_sink: Arc<Mutex<Option<StreamSink<NetplayGameEvent>>>>,
    pub polling_task: Mutex<Option<tokio::task::JoinHandle<()>>>,
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
        }
    })
}

/// Connect to netplay server and perform handshake.
#[frb]
pub async fn netplay_connect(server_addr: String, player_name: String) -> Result<(), String> {
    let mgr = get_manager();

    // Stop any existing session
    let _ = netplay_disconnect().await;

    let addr = server_addr
        .parse()
        .map_err(|e| format!("Invalid address: {}", e))?;

    // Create event channel for TCP client -> handler
    let (event_tx, event_rx) = mpsc::channel(256);
    // Create channel for handler -> Flutter (game events)
    let (game_event_tx, mut game_event_rx) = mpsc::channel(32);

    // Create TCP client
    let client = nesium_netplay::connect(addr, event_tx)
        .await
        .map_err(|e| format!("Failed to connect: {}", e))?;

    let config = NetplayConfig {
        name: player_name,
        rom_hash: [0; 16], // TODO: Get ROM hash from runtime
        spectator: false,
        room_code: 0,
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
pub async fn netplay_join_room(room_code: u32) -> Result<(), String> {
    let mgr = get_manager();
    let tx = lock_unpoison(&mgr.command_tx).clone();
    if let Some(tx) = tx {
        tx.send(NetplayCommand::JoinRoom(room_code))
            .await
            .map_err(|e| format!("Failed to send command: {}", e))?;
        Ok(())
    } else {
        Err("Not connected".to_string())
    }
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

/// Disconnect from netplay server.
#[frb]
pub async fn netplay_disconnect() -> Result<(), String> {
    let mgr = get_manager();

    if let Some(task) = lock_unpoison(&mgr.session_task).take() {
        task.abort();
    }
    *lock_unpoison(&mgr.command_tx) = None;

    mgr.input_provider.set_active(false);
    mgr.input_provider.with_session(|s| s.reset());
    let _ = crate::runtime_handle().disable_netplay();

    notify_status(&mgr.status_sink, &mgr.input_provider, None);

    Ok(())
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
        let (state, client_id, room_id, player_index) = input_provider.with_session(|s| {
            let state = match s.state {
                SessionState::Disconnected => NetplayState::Disconnected,
                SessionState::Connecting | SessionState::Handshake => NetplayState::Connecting,
                SessionState::WaitingForRoom => NetplayState::Connected,
                SessionState::Playing { .. }
                | SessionState::Spectating { .. }
                | SessionState::Syncing { .. } => NetplayState::InRoom,
            };
            (
                state,
                s.client_id,
                s.room_id,
                s.local_player_index.unwrap_or(SPECTATOR_PLAYER_INDEX),
            )
        });

        let status = NetplayStatus {
            state,
            client_id,
            room_id,
            player_index,
            error,
        };

        let _ = sink.add(status);
    }
}

/// Lock a `std::sync::Mutex` without panicking on poisoning.
///
/// `Mutex::lock()` can fail if another thread panicked while holding the lock. In the Flutter FFI
/// layer we prefer recovering the inner value over crashing the entire process.
fn lock_unpoison<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
