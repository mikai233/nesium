use std::net::SocketAddr;
use std::time::Duration;

use nesium_netd::net::quic_config;
use nesium_netd::net::tcp::run_tcp_listener_with_listener;
use nesium_netplay::{
    NetplayCommand, NetplayConfig, NetplayEvent, NetplayInputProvider, SessionHandler, SyncMode,
    connect, create_input_provider,
};
use nesium_netproto::constants::SPECTATOR_PLAYER_INDEX;
use nesium_netproto::messages::session::TransportKind;
use tokio::sync::mpsc;
use tokio::sync::oneshot;
use tokio::time::{sleep, timeout};

async fn setup_server(app_name: &str) -> SocketAddr {
    let cert_dir = quic_config::default_quic_data_dir(app_name);
    if cert_dir.exists() {
        let _ = std::fs::remove_dir_all(&cert_dir);
    }

    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let (event_tx, event_rx) = mpsc::channel(1024);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let server_addr = listener.local_addr().unwrap();

    let tx_clone = event_tx.clone();
    let app_name_owned = app_name.to_string();
    tokio::spawn(async move {
        let _ = run_tcp_listener_with_listener(listener, tx_clone, &app_name_owned, None).await;
    });
    tokio::spawn(async move {
        let _ = nesium_netd::run_server(event_rx, None, None).await;
    });

    sleep(Duration::from_millis(100)).await;
    server_addr
}

struct ClientHarness {
    handle: nesium_netplay::TcpClientHandle,
    input: std::sync::Arc<nesium_netplay::SharedInputProvider>,
    cmd: mpsc::Sender<NetplayCommand>,
    game_rx: mpsc::Receiver<NetplayEvent>,
}

async fn spawn_client(
    server_addr: SocketAddr,
    name: &str,
    room_id: u32,
    desired_role: u8,
    has_rom: bool,
    sync_mode: SyncMode,
) -> ClientHarness {
    let (event_tx, event_rx) = mpsc::channel(256);
    let (game_tx, game_rx) = mpsc::channel(256);
    let handle = connect(server_addr, event_tx).await.unwrap();
    let input = create_input_provider();
    input.set_sync_mode(sync_mode);

    let (mut handler, cmd) = SessionHandler::new(
        handle.clone(),
        NetplayConfig {
            name: name.to_string(),
            transport: TransportKind::Tcp,
            spectator: desired_role == SPECTATOR_PLAYER_INDEX,
            room_id,
            desired_role,
            has_rom,
        },
        input.clone(),
        event_rx,
        game_tx,
    );
    tokio::spawn(async move {
        let _ = handler.run().await;
    });

    sleep(Duration::from_millis(100)).await;

    ClientHarness {
        handle,
        input,
        cmd,
        game_rx,
    }
}

async fn recv_event(rx: &mut mpsc::Receiver<NetplayEvent>, deadline: Duration) -> NetplayEvent {
    timeout(deadline, rx.recv())
        .await
        .expect("timed out waiting for event")
        .expect("event channel closed")
}

async fn drain_until_start_game(rx: &mut mpsc::Receiver<NetplayEvent>) {
    let deadline = Duration::from_secs(2);
    loop {
        match recv_event(rx, deadline).await {
            NetplayEvent::StartGame => return,
            _ => continue,
        }
    }
}

async fn drain_until_sync_state_and_start(rx: &mut mpsc::Receiver<NetplayEvent>) -> (u32, Vec<u8>) {
    let mut got_state: Option<(u32, Vec<u8>)> = None;
    let deadline = Duration::from_secs(2);
    loop {
        match recv_event(rx, deadline).await {
            NetplayEvent::SyncState(frame, data) => {
                got_state = Some((frame, data));
            }
            NetplayEvent::StartGame => {
                return got_state.expect("expected SyncState before StartGame");
            }
            _ => {}
        }
    }
}

async fn start_solo_room(host: &ClientHarness) -> u32 {
    host.cmd.send(NetplayCommand::CreateRoom).await.unwrap();
    sleep(Duration::from_millis(100)).await;
    let room_id = host.input.with_session(|s| s.room_id);
    assert_ne!(room_id, 0);
    room_id
}

async fn broadcast_rom_and_start(host: &mut ClientHarness) {
    host.cmd
        .send(NetplayCommand::SendRom(vec![1, 2, 3]))
        .await
        .unwrap();
    host.cmd.send(NetplayCommand::RomLoaded).await.unwrap();
    drain_until_start_game(&mut host.game_rx).await;
}

async fn send_state_and_inputs(host: &ClientHarness, snapshot_frame: u32, inputs_to: u32) {
    host.cmd
        .send(NetplayCommand::ProvideState(
            snapshot_frame,
            vec![0xAA, 0xBB],
        ))
        .await
        .unwrap();
    for f in snapshot_frame..=inputs_to {
        host.cmd
            .send(NetplayCommand::SendInput(f, 0x01))
            .await
            .unwrap();
    }
    sleep(Duration::from_millis(100)).await;
}

#[tokio::test]
async fn lockstep_midgame_join_manual_role_does_not_stall_first_frame() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let server_addr = setup_server("test_lockstep_midgame_join_manual_role").await;

    let mut host = spawn_client(server_addr, "P1", 0, 0, false, SyncMode::Lockstep).await;
    let room_id = start_solo_room(&host).await;
    broadcast_rom_and_start(&mut host).await;

    send_state_and_inputs(&host, 0, 20).await;

    let mut joiner = spawn_client(server_addr, "P2", room_id, 1, true, SyncMode::Lockstep).await;

    let (_frame, _state) = drain_until_sync_state_and_start(&mut joiner.game_rx).await;

    timeout(Duration::from_secs(2), async {
        loop {
            if joiner.input.is_frame_ready(0) {
                break;
            }
            sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("joiner stuck at first frame");
}

#[tokio::test]
async fn rollback_midgame_join_manual_role_does_not_stall_first_frame() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let server_addr = setup_server("test_rollback_midgame_join_manual_role").await;

    let mut host = spawn_client(server_addr, "P1", 0, 0, false, SyncMode::Rollback).await;
    let room_id = start_solo_room(&host).await;
    broadcast_rom_and_start(&mut host).await;

    send_state_and_inputs(&host, 0, 20).await;

    let mut joiner = spawn_client(server_addr, "P2", room_id, 1, true, SyncMode::Rollback).await;

    let (_frame, _state) = drain_until_sync_state_and_start(&mut joiner.game_rx).await;

    timeout(Duration::from_secs(2), async {
        loop {
            if joiner.input.is_frame_ready(0) {
                break;
            }
            sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("joiner stuck at first frame");
}

#[tokio::test]
async fn lockstep_rejoin_manual_role_does_not_stall_first_frame() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let server_addr = setup_server("test_lockstep_rejoin_manual_role").await;

    let mut host = spawn_client(server_addr, "P1", 0, 0, false, SyncMode::Lockstep).await;
    let room_id = start_solo_room(&host).await;

    let mut p2 = spawn_client(server_addr, "P2", room_id, 1, false, SyncMode::Lockstep).await;

    host.cmd
        .send(NetplayCommand::SendRom(vec![1]))
        .await
        .unwrap();
    let _ = recv_event(&mut p2.game_rx, Duration::from_secs(2)).await; // LoadRom
    host.cmd.send(NetplayCommand::RomLoaded).await.unwrap();
    p2.cmd.send(NetplayCommand::RomLoaded).await.unwrap();

    drain_until_start_game(&mut host.game_rx).await;
    drain_until_start_game(&mut p2.game_rx).await;

    send_state_and_inputs(&host, 5, 30).await;

    p2.handle.disconnect().await.unwrap();
    sleep(Duration::from_millis(200)).await;

    // Host should keep progressing (not waiting on P2 anymore).
    timeout(Duration::from_secs(2), async {
        loop {
            host.cmd
                .send(NetplayCommand::SendInput(31, 0x01))
                .await
                .unwrap();
            sleep(Duration::from_millis(50)).await;
            if host.input.is_frame_ready(31) {
                break;
            }
        }
    })
    .await
    .expect("host stalled after P2 disconnect");

    let mut p2_re = spawn_client(server_addr, "P2r", room_id, 1, true, SyncMode::Lockstep).await;
    let (_frame, _state) = drain_until_sync_state_and_start(&mut p2_re.game_rx).await;

    timeout(Duration::from_secs(2), async {
        loop {
            if p2_re.input.is_frame_ready(0) {
                break;
            }
            sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("rejoiner stuck at first frame");
}

#[tokio::test]
async fn rollback_rejoin_manual_role_does_not_stall_first_frame() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let server_addr = setup_server("test_rollback_rejoin_manual_role").await;

    let mut host = spawn_client(server_addr, "P1", 0, 0, false, SyncMode::Rollback).await;
    let room_id = start_solo_room(&host).await;

    let mut p2 = spawn_client(server_addr, "P2", room_id, 1, false, SyncMode::Rollback).await;

    host.cmd
        .send(NetplayCommand::SendRom(vec![1]))
        .await
        .unwrap();
    let _ = recv_event(&mut p2.game_rx, Duration::from_secs(2)).await; // LoadRom
    host.cmd.send(NetplayCommand::RomLoaded).await.unwrap();
    p2.cmd.send(NetplayCommand::RomLoaded).await.unwrap();

    drain_until_start_game(&mut host.game_rx).await;
    drain_until_start_game(&mut p2.game_rx).await;

    send_state_and_inputs(&host, 5, 30).await;

    p2.handle.disconnect().await.unwrap();
    sleep(Duration::from_millis(200)).await;

    // Host should keep progressing (not waiting on P2 anymore).
    timeout(Duration::from_secs(2), async {
        loop {
            host.cmd
                .send(NetplayCommand::SendInput(31, 0x01))
                .await
                .unwrap();
            sleep(Duration::from_millis(50)).await;
            if host.input.is_frame_ready(31) {
                break;
            }
        }
    })
    .await
    .expect("host stalled after P2 disconnect");

    let mut p2_re = spawn_client(server_addr, "P2r", room_id, 1, true, SyncMode::Rollback).await;
    let (_frame, _state) = drain_until_sync_state_and_start(&mut p2_re.game_rx).await;

    timeout(Duration::from_secs(2), async {
        loop {
            if p2_re.input.is_frame_ready(0) {
                break;
            }
            sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("rejoiner stuck at first frame");
}

#[tokio::test]
async fn query_room_reports_occupied_mask_before_join() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let server_addr = setup_server("test_query_room_occupied_mask").await;

    let host = spawn_client(server_addr, "P1", 0, 0, false, SyncMode::Lockstep).await;
    let room_id = start_solo_room(&host).await;

    // Query client: do not join any room.
    let querier = spawn_client(
        server_addr,
        "Q",
        0,
        SPECTATOR_PLAYER_INDEX,
        false,
        SyncMode::Lockstep,
    )
    .await;

    let (tx, rx) = oneshot::channel();
    querier
        .cmd
        .send(NetplayCommand::QueryRoom { room_id, resp: tx })
        .await
        .unwrap();

    let info = timeout(Duration::from_secs(2), rx)
        .await
        .expect("timed out waiting for RoomInfo")
        .expect("oneshot canceled")
        .expect("room query failed");

    assert!(info.ok);
    assert_eq!(info.room_id, room_id);
    assert_eq!(info.occupied_mask & 0b0001, 0b0001);
    assert_eq!(info.occupied_mask & 0b0010, 0);

    // Keep host alive until the query completes.
    host.cmd
        .send(NetplayCommand::SendRom(vec![1]))
        .await
        .unwrap();
}

/// Test reconnection when server has NO cached state.
///
/// This reproduces the bug where P2 gets stuck after loading ROM because:
/// 1. P2 disconnects immediately after game start (no periodic state sync yet)
/// 2. P2 reconnects - server has no cached_state
/// 3. Server sends RequestState to host
/// 4. Host should respond with ProvideState
/// 5. Server sends SyncState + BeginCatchUp to P2
///
/// If step 4 or 5 fails, P2 will be stuck forever.
///
/// This test simulates the runtime behavior by polling `take_state_sync_request()`
/// and sending state when requested.
#[tokio::test]
async fn lockstep_rejoin_without_cached_state_does_not_hang() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let server_addr = setup_server("test_lockstep_rejoin_no_cached_state").await;

    // 1. Create room with P1
    let mut host = spawn_client(server_addr, "P1", 0, 0, false, SyncMode::Lockstep).await;
    let room_id = start_solo_room(&host).await;

    // 2. P2 joins
    let mut p2 = spawn_client(server_addr, "P2", room_id, 1, false, SyncMode::Lockstep).await;

    // 3. Load ROM and start game
    host.cmd
        .send(NetplayCommand::SendRom(vec![1, 2, 3]))
        .await
        .unwrap();
    let _ = recv_event(&mut p2.game_rx, Duration::from_secs(2)).await; // LoadRom
    host.cmd.send(NetplayCommand::RomLoaded).await.unwrap();
    p2.cmd.send(NetplayCommand::RomLoaded).await.unwrap();

    drain_until_start_game(&mut host.game_rx).await;
    drain_until_start_game(&mut p2.game_rx).await;

    // 4. Play a few frames together (but NO state sync yet - that happens every 60 frames)
    for f in 0..5 {
        host.cmd
            .send(NetplayCommand::SendInput(f, 0x01))
            .await
            .unwrap();
        p2.cmd
            .send(NetplayCommand::SendInput(f, 0x02))
            .await
            .unwrap();
    }
    sleep(Duration::from_millis(100)).await;

    // 5. P2 disconnects IMMEDIATELY - before any periodic state sync
    p2.handle.disconnect().await.unwrap();
    sleep(Duration::from_millis(200)).await;

    // 6. Verify host can continue solo
    for f in 5..10 {
        host.cmd
            .send(NetplayCommand::SendInput(f, 0x01))
            .await
            .unwrap();
    }
    sleep(Duration::from_millis(100)).await;
    assert!(
        host.input.is_frame_ready(5),
        "Host should be able to advance after P2 disconnect"
    );

    // 7. P2 reconnects - server has NO cached_state!
    let mut p2_re = spawn_client(server_addr, "P2r", room_id, 1, true, SyncMode::Lockstep).await;

    // 8. SIMULATE RUNTIME: Poll for state sync request and respond
    // In real runtime, `maybe_send_periodic_netplay_state` does this.
    // We spawn a task to simulate this behavior.
    let host_input_clone = host.input.clone();
    let host_cmd_clone = host.cmd.clone();
    let state_sync_task = tokio::spawn(async move {
        // Poll for up to 2 seconds
        for _ in 0..40 {
            if host_input_clone.take_state_sync_request() {
                eprintln!("[Test] Host detected state sync request, sending ProvideState");
                // Send state to server (simulating runtime behavior)
                let _ = host_cmd_clone
                    .send(NetplayCommand::ProvideState(
                        5,
                        vec![0xDE, 0xAD, 0xBE, 0xEF],
                    ))
                    .await;
                return true;
            }
            sleep(Duration::from_millis(50)).await;
        }
        false
    });

    // 9. P2 should receive SyncState and StartGame (via BeginCatchUp)
    let result = timeout(Duration::from_secs(5), async {
        let mut got_sync_state = false;
        let mut got_start = false;
        loop {
            match recv_event(&mut p2_re.game_rx, Duration::from_secs(4)).await {
                NetplayEvent::SyncState(frame, _data) => {
                    eprintln!("[Test] P2 received SyncState at frame {}", frame);
                    got_sync_state = true;
                }
                NetplayEvent::StartGame => {
                    eprintln!("[Test] P2 received StartGame");
                    got_start = true;
                }
                other => {
                    eprintln!("[Test] P2 received {:?}", other);
                }
            }
            if got_sync_state && got_start {
                break;
            }
        }
    })
    .await;

    // Check if state sync was requested
    let state_was_requested = state_sync_task.await.unwrap_or(false);
    eprintln!("[Test] State sync was requested: {}", state_was_requested);

    assert!(
        result.is_ok(),
        "P2 should receive SyncState + StartGame after reconnection (bug: stuck waiting for state)"
    );

    // 10. Verify P2 can advance
    timeout(Duration::from_secs(2), async {
        loop {
            if p2_re.input.is_frame_ready(0) {
                break;
            }
            sleep(Duration::from_millis(20)).await;
        }
    })
    .await
    .expect("P2 should be able to advance from frame 0 after reconnection");
}
