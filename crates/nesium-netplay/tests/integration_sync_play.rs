use nesium_netplay::{
    NetplayCommand, NetplayConfig, NetplayEvent, NetplayInputProvider, SessionHandler, SyncMode,
    connect, create_input_provider,
};
use nesium_netproto::messages::session::TransportKind;
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};

async fn setup_server(app_name: &str) -> SocketAddr {
    let cert_dir = nesium_netd::net::quic_config::default_quic_data_dir(app_name);
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
        let _ = nesium_netd::net::tcp::run_tcp_listener_with_listener(
            listener,
            tx_clone,
            &app_name_owned,
        )
        .await;
    });
    tokio::spawn(async move {
        let _ = nesium_netd::run_server(event_rx).await;
    });
    sleep(Duration::from_millis(100)).await;
    server_addr
}

#[tokio::test]
async fn test_lockstep_gameplay_sync() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let server_addr = setup_server("test_lockstep_sync").await;

    // Connect Player 1 (Host)
    let (c1_event_tx, c1_event_rx) = mpsc::channel(100);
    let (c1_game_tx, mut c1_game_rx) = mpsc::channel(100);
    let c1_handle = connect(server_addr, c1_event_tx).await.unwrap();
    let c1_input = create_input_provider();
    let (mut c1_handler, c1_cmd) = SessionHandler::new(
        c1_handle,
        NetplayConfig {
            name: "P1".to_string(),
            transport: TransportKind::Tcp,
            spectator: false,
            room_code: 0,
        },
        c1_input.clone(),
        c1_event_rx,
        c1_game_tx,
    );
    tokio::spawn(async move { c1_handler.run().await });
    c1_cmd.send(NetplayCommand::CreateRoom).await.unwrap();
    sleep(Duration::from_millis(100)).await;
    let room_id = c1_input.with_session(|s| s.room_id);

    // Connect Player 2
    let (c2_event_tx, c2_event_rx) = mpsc::channel(100);
    let (c2_game_tx, mut c2_game_rx) = mpsc::channel(100);
    let c2_handle = connect(server_addr, c2_event_tx).await.unwrap();
    let c2_input = create_input_provider();
    let (mut c2_handler, c2_cmd) = SessionHandler::new(
        c2_handle,
        NetplayConfig {
            name: "P2".to_string(),
            transport: TransportKind::Tcp,
            spectator: false,
            room_code: room_id,
        },
        c2_input.clone(),
        c2_event_rx,
        c2_game_tx,
    );
    tokio::spawn(async move { c2_handler.run().await });
    sleep(Duration::from_millis(100)).await;

    // Load ROM and start game in Lockstep (default)
    c1_cmd.send(NetplayCommand::SendRom(vec![1])).await.unwrap();
    let _ = c2_game_rx.recv().await; // Receive LoadRom
    c1_cmd.send(NetplayCommand::RomLoaded).await.unwrap();
    c2_cmd.send(NetplayCommand::RomLoaded).await.unwrap();

    // Both should receive StartGame
    assert!(matches!(
        c1_game_rx.recv().await,
        Some(NetplayEvent::StartGame)
    ));
    assert!(matches!(
        c2_game_rx.recv().await,
        Some(NetplayEvent::StartGame)
    ));

    // Verify sync mode is Lockstep
    assert_eq!(c1_input.sync_mode(), SyncMode::Lockstep);

    // Simulate gameplay: Player 1 sends input for frame 0
    c1_cmd
        .send(NetplayCommand::SendInput(0, 0x01))
        .await
        .unwrap();
    // P1 shouldn't be able to advance yet because it's missing P2's input
    assert!(!c1_input.is_frame_ready(0));

    // Player 2 sends input for frame 0
    c2_cmd
        .send(NetplayCommand::SendInput(0, 0x02))
        .await
        .unwrap();

    // Give some time for network broadcast
    sleep(Duration::from_millis(200)).await;

    // Now both should be able to advance frame 0
    assert!(c1_input.is_frame_ready(0));
    assert!(c2_input.is_frame_ready(0));

    let inputs1 = c1_input.poll_inputs(0).unwrap();
    let inputs2 = c2_input.poll_inputs(0).unwrap();
    assert_eq!(inputs1[0], 0x01);
    assert_eq!(inputs1[1], 0x02);
    assert_eq!(inputs2[0], 0x01);
    assert_eq!(inputs2[1], 0x02);
}

#[tokio::test]
async fn test_rollback_gameplay_sync() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let server_addr = setup_server("test_rollback_sync").await;

    // Connect Player 1 (Host)
    let (c1_event_tx, c1_event_rx) = mpsc::channel(100);
    let (c1_game_tx, mut c1_game_rx) = mpsc::channel(100);
    let c1_handle = connect(server_addr, c1_event_tx).await.unwrap();
    let c1_input = create_input_provider();
    let (mut c1_handler, c1_cmd) = SessionHandler::new(
        c1_handle,
        NetplayConfig {
            name: "P1".to_string(),
            transport: TransportKind::Tcp,
            spectator: false,
            room_code: 0,
        },
        c1_input.clone(),
        c1_event_rx,
        c1_game_tx,
    );
    tokio::spawn(async move { c1_handler.run().await });

    // Host decides the room sync mode at creation time.
    c1_input.set_sync_mode(SyncMode::Rollback);
    c1_cmd.send(NetplayCommand::CreateRoom).await.unwrap();
    sleep(Duration::from_millis(100)).await;
    let room_id = c1_input.with_session(|s| s.room_id);

    // Connect Player 2
    let (c2_event_tx, c2_event_rx) = mpsc::channel(100);
    let (c2_game_tx, mut c2_game_rx) = mpsc::channel(100);
    let c2_handle = connect(server_addr, c2_event_tx).await.unwrap();
    let c2_input = create_input_provider();
    let (mut c2_handler, c2_cmd) = SessionHandler::new(
        c2_handle,
        NetplayConfig {
            name: "P2".to_string(),
            transport: TransportKind::Tcp,
            spectator: false,
            room_code: room_id,
        },
        c2_input.clone(),
        c2_event_rx,
        c2_game_tx,
    );
    tokio::spawn(async move { c2_handler.run().await });
    sleep(Duration::from_millis(100)).await;

    // Start game
    c1_cmd.send(NetplayCommand::SendRom(vec![1])).await.unwrap();
    let _ = c2_game_rx.recv().await;
    c1_cmd.send(NetplayCommand::RomLoaded).await.unwrap();
    c2_cmd.send(NetplayCommand::RomLoaded).await.unwrap();
    let _ = c1_game_rx.recv().await;
    let _ = c2_game_rx.recv().await;

    // Verify both sides switched to the room's sync mode (from JoinAck)
    assert_eq!(c1_input.sync_mode(), SyncMode::Rollback);
    assert_eq!(c2_input.sync_mode(), SyncMode::Rollback);

    // Gameplay: Rollback should ALWAYS be able to advance
    assert!(c1_input.is_frame_ready(0));
    assert!(c1_input.is_frame_ready(100));

    // P1 sends input for frame 0 and advances
    c1_input.send_input_to_server(0, 0x01);
    sleep(Duration::from_millis(50)).await;

    let inputs = c1_input.poll_inputs(0).unwrap();
    assert_eq!(inputs[0], 0x01);
    assert_eq!(inputs[1], 0); // P2 predicted as 0

    // Advance P1 to frame 1 to ensure rollback can trigger for frame 0
    let _ = c1_input.poll_inputs(1).unwrap();

    // P2 sends input for frame 0 LATE (differing from prediction)
    c2_input.send_input_to_server(0, 0xAA);

    sleep(Duration::from_millis(100)).await;

    // P1 should have detected misprediction and requested rollback
    let rollback = c1_input.pending_rollback();
    assert!(rollback.is_some());
    assert_eq!(rollback.unwrap().target_frame, 0);
}

#[tokio::test]
async fn test_rollback_overlapping_rollbacks() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let server_addr = setup_server("test_overlapping_rb").await;

    // Connect P1 and P2
    let (c1_event_tx, c1_event_rx) = mpsc::channel(100);
    let (c1_game_tx, _) = mpsc::channel(100);
    let c1_handle = connect(server_addr, c1_event_tx).await.unwrap();
    let c1_input = create_input_provider();
    let (mut c1_handler, c1_cmd) = SessionHandler::new(
        c1_handle,
        NetplayConfig {
            name: "P1".to_string(),
            transport: TransportKind::Tcp,
            spectator: false,
            room_code: 0,
        },
        c1_input.clone(),
        c1_event_rx,
        c1_game_tx,
    );
    tokio::spawn(async move { c1_handler.run().await });

    // Host decides the room sync mode at creation time.
    c1_input.set_sync_mode(SyncMode::Rollback);
    c1_cmd.send(NetplayCommand::CreateRoom).await.unwrap();
    sleep(Duration::from_millis(100)).await;
    let room_id = c1_input.with_session(|s| s.room_id);

    let (c2_event_tx, c2_event_rx) = mpsc::channel(100);
    let (c2_game_tx, _) = mpsc::channel(100);
    let c2_handle = connect(server_addr, c2_event_tx).await.unwrap();
    let c2_input = create_input_provider();
    let (mut c2_handler, _) = SessionHandler::new(
        c2_handle,
        NetplayConfig {
            name: "P2".to_string(),
            transport: TransportKind::Tcp,
            spectator: false,
            room_code: room_id,
        },
        c2_input.clone(),
        c2_event_rx,
        c2_game_tx,
    );
    tokio::spawn(async move { c2_handler.run().await });
    sleep(Duration::from_millis(100)).await;
    assert_eq!(c1_input.sync_mode(), SyncMode::Rollback);
    assert_eq!(c2_input.sync_mode(), SyncMode::Rollback);

    // Initial sync
    c1_input.send_input_to_server(0, 0);
    c2_input.send_input_to_server(0, 0);
    sleep(Duration::from_millis(100)).await;

    // P1 advances to frame 10
    for f in 1..=10 {
        c1_input.send_input_to_server(f, 0x01);
        let _ = c1_input.poll_inputs(f);
    }

    // P2 sends late input for frame 8 (triggers rollback 8)
    c2_input.send_input_to_server(8, 0xAA);
    sleep(Duration::from_millis(50)).await;
    let rb = c1_input.pending_rollback().unwrap();
    assert_eq!(rb.target_frame, 8);

    // P2 sends even later input for frame 5 (should override to rollback 5)
    c2_input.send_input_to_server(5, 0xBB);
    sleep(Duration::from_millis(100)).await;
    let rb = c1_input.pending_rollback().unwrap();
    assert_eq!(rb.target_frame, 5);
}

#[tokio::test]
async fn test_rollback_input_holes() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let server_addr = setup_server("test_input_holes").await;

    // Connect P1 and P2
    let (c1_event_tx, c1_event_rx) = mpsc::channel(100);
    let (c1_game_tx, _) = mpsc::channel(100);
    let c1_handle = connect(server_addr, c1_event_tx).await.unwrap();
    let c1_input = create_input_provider();
    let (mut c1_handler, c1_cmd) = SessionHandler::new(
        c1_handle,
        NetplayConfig {
            name: "P1".to_string(),
            transport: TransportKind::Tcp,
            spectator: false,
            room_code: 0,
        },
        c1_input.clone(),
        c1_event_rx,
        c1_game_tx,
    );
    tokio::spawn(async move { c1_handler.run().await });

    // Host decides the room sync mode at creation time.
    c1_input.set_sync_mode(SyncMode::Rollback);
    c1_cmd.send(NetplayCommand::CreateRoom).await.unwrap();
    sleep(Duration::from_millis(100)).await;
    let room_id = c1_input.with_session(|s| s.room_id);

    let (c2_event_tx, c2_event_rx) = mpsc::channel(100);
    let (c2_game_tx, _) = mpsc::channel(100);
    let c2_handle = connect(server_addr, c2_event_tx).await.unwrap();
    let c2_input = create_input_provider();
    let (mut c2_handler, _) = SessionHandler::new(
        c2_handle,
        NetplayConfig {
            name: "P2".to_string(),
            transport: TransportKind::Tcp,
            spectator: false,
            room_code: room_id,
        },
        c2_input.clone(),
        c2_event_rx,
        c2_game_tx,
    );
    tokio::spawn(async move { c2_handler.run().await });
    sleep(Duration::from_millis(100)).await;
    assert_eq!(c1_input.sync_mode(), SyncMode::Rollback);
    assert_eq!(c2_input.sync_mode(), SyncMode::Rollback);

    // P1 sends 0, 1, 2, 3, 4
    for f in 0..=4 {
        c1_input.send_input_to_server(f, 0x01);
    }

    // P2 sends 0, 1, then MISSES 2, sends 3, 4
    c2_input.send_input_to_server(0, 0x02);
    c2_input.send_input_to_server(1, 0x02);
    // Hole at frame 2
    c2_input.send_input_to_server(3, 0x02);
    c2_input.send_input_to_server(4, 0x02);

    sleep(Duration::from_millis(200)).await;

    // P1's confirmed frame should be 1 (because 2 is missing from P2)
    let confirmed = c1_input.last_confirmed_frame();
    assert_eq!(confirmed, 1);

    // Now P2 sends the missing frame 2
    c2_input.send_input_to_server(2, 0x02);
    sleep(Duration::from_millis(200)).await;

    // Now confirmed frame should jump to 4
    let confirmed = c1_input.last_confirmed_frame();
    assert_eq!(confirmed, 4);
}

#[tokio::test]
async fn test_lockstep_dynamic_delay() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let server_addr = setup_server("test_dynamic_delay").await;

    // Connect P1
    let (c1_event_tx, c1_event_rx) = mpsc::channel(100);
    let (c1_game_tx, _) = mpsc::channel(100);
    let c1_handle = connect(server_addr, c1_event_tx).await.unwrap();
    let c1_input = create_input_provider();
    let (mut c1_handler, c1_cmd) = SessionHandler::new(
        c1_handle,
        NetplayConfig {
            name: "P1".to_string(),
            transport: TransportKind::Tcp,
            spectator: false,
            room_code: 0,
        },
        c1_input.clone(),
        c1_event_rx,
        c1_game_tx,
    );
    tokio::spawn(async move { c1_handler.run().await });
    c1_cmd.send(NetplayCommand::CreateRoom).await.unwrap();
    sleep(Duration::from_millis(100)).await;

    // Initial delay is 2
    assert_eq!(c1_input.input_delay(), 2);

    // Change delay to 5
    c1_input.set_input_delay(5);
    assert_eq!(c1_input.input_delay(), 5);

    // Verify it changed in the strategy too
    c1_input.with_sync(|s| {
        // We can't easily check the private field, but we can verify it doesn't panic
        s.set_input_delay(5);
    });
}

#[tokio::test]
async fn test_lockstep_mid_session_dynamic_delay() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let server_addr = setup_server("test_mid_session_delay").await;

    // Connect P1 and P2
    let (c1_event_tx, c1_event_rx) = mpsc::channel(100);
    let (c1_game_tx, mut c1_game_rx) = mpsc::channel(100);
    let c1_handle = connect(server_addr, c1_event_tx).await.unwrap();
    let c1_input = create_input_provider();
    let (mut c1_handler, c1_cmd) = SessionHandler::new(
        c1_handle,
        NetplayConfig {
            name: "P1".to_string(),
            transport: TransportKind::Tcp,
            spectator: false,
            room_code: 0,
        },
        c1_input.clone(),
        c1_event_rx,
        c1_game_tx,
    );
    tokio::spawn(async move { c1_handler.run().await });
    c1_cmd.send(NetplayCommand::CreateRoom).await.unwrap();
    sleep(Duration::from_millis(100)).await;
    let room_id = c1_input.with_session(|s| s.room_id);

    let (c2_event_tx, c2_event_rx) = mpsc::channel(100);
    let (c2_game_tx, mut c2_game_rx) = mpsc::channel(100);
    let c2_handle = connect(server_addr, c2_event_tx).await.unwrap();
    let c2_input = create_input_provider();
    let (mut c2_handler, c2_cmd) = SessionHandler::new(
        c2_handle,
        NetplayConfig {
            name: "P2".to_string(),
            transport: TransportKind::Tcp,
            spectator: false,
            room_code: room_id,
        },
        c2_input.clone(),
        c2_event_rx,
        c2_game_tx,
    );
    tokio::spawn(async move { c2_handler.run().await });
    sleep(Duration::from_millis(100)).await;

    // Start game
    c1_cmd.send(NetplayCommand::SendRom(vec![1])).await.unwrap();
    let _ = c2_game_rx.recv().await;
    c1_cmd.send(NetplayCommand::RomLoaded).await.unwrap();
    c2_cmd.send(NetplayCommand::RomLoaded).await.unwrap();
    let _ = c1_game_rx.recv().await;
    let _ = c2_game_rx.recv().await;

    // 1. Initial gameplay with delay 2
    c1_input.set_input_delay(2);
    c2_input.set_input_delay(2);

    // Frame 0 setup
    c1_input.send_input_to_server(0, 0x01);
    c2_input.send_input_to_server(0, 0x02);
    sleep(Duration::from_millis(100)).await;
    assert!(c1_input.is_frame_ready(0));

    // 2. Mid-session Change delay to 10
    c1_input.set_input_delay(10);
    c2_input.set_input_delay(10);

    // Frame 1 should still work
    c1_input.send_input_to_server(1, 0x03);
    c2_input.send_input_to_server(1, 0x04);
    sleep(Duration::from_millis(100)).await;

    assert!(c1_input.is_frame_ready(1));
    let inputs = c1_input.poll_inputs(1).unwrap();
    assert_eq!(inputs[0], 0x03);
    assert_eq!(inputs[1], 0x04);

    // 3. Verify no out-of-bounds or lag-induced deadlock
    for f in 2..10 {
        c1_input.send_input_to_server(f, 1);
        c2_input.send_input_to_server(f, 2);
    }
    sleep(Duration::from_millis(200)).await;
    assert!(c1_input.is_frame_ready(9));
}

#[tokio::test]
async fn test_spectator_late_join() {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let server_addr = setup_server("test_spectator_late_join").await;

    // 1. Connect P1 and P2
    let (c1_event_tx, c1_event_rx) = mpsc::channel(100);
    let (c1_game_tx, mut c1_game_rx) = mpsc::channel(100);
    let c1_handle = connect(server_addr, c1_event_tx).await.unwrap();
    let c1_input = create_input_provider();
    let (mut c1_handler, c1_cmd) = SessionHandler::new(
        c1_handle,
        NetplayConfig {
            name: "P1".to_string(),
            transport: TransportKind::Tcp,
            spectator: false,
            room_code: 0,
        },
        c1_input.clone(),
        c1_event_rx,
        c1_game_tx,
    );
    tokio::spawn(async move { c1_handler.run().await });
    c1_cmd.send(NetplayCommand::CreateRoom).await.unwrap();
    sleep(Duration::from_millis(100)).await;
    let room_id = c1_input.with_session(|s| s.room_id);

    let (c2_event_tx, c2_event_rx) = mpsc::channel(100);
    let (c2_game_tx, mut c2_game_rx) = mpsc::channel(100);
    let c2_handle = connect(server_addr, c2_event_tx).await.unwrap();
    let c2_input = create_input_provider();
    let (mut c2_handler, c2_cmd) = SessionHandler::new(
        c2_handle,
        NetplayConfig {
            name: "P2".to_string(),
            transport: TransportKind::Tcp,
            spectator: false,
            room_code: room_id,
        },
        c2_input.clone(),
        c2_event_rx,
        c2_game_tx,
    );
    tokio::spawn(async move { c2_handler.run().await });
    sleep(Duration::from_millis(100)).await;

    // 2. Start game
    c1_cmd.send(NetplayCommand::SendRom(vec![1])).await.unwrap();
    let _ = c2_game_rx.recv().await;
    c1_cmd.send(NetplayCommand::RomLoaded).await.unwrap();
    c2_cmd.send(NetplayCommand::RomLoaded).await.unwrap();
    let _ = c1_game_rx.recv().await;
    let _ = c2_game_rx.recv().await;

    // 3. Play frames 0-10
    for f in 0..=10 {
        c1_input.send_input_to_server(f, 0x01);
        c2_input.send_input_to_server(f, 0x02);
    }
    sleep(Duration::from_millis(200)).await;

    // 4. P1 provides state for frame 5
    c1_input.send_state(5, &[0xAA, 0xBB]);
    sleep(Duration::from_millis(100)).await;

    // 5. Connect P3 as spectator
    let (c3_event_tx, c3_event_rx) = mpsc::channel(100);
    let (c3_game_tx, mut c3_game_rx) = mpsc::channel(100);
    let c3_handle = connect(server_addr, c3_event_tx).await.unwrap();
    let c3_input = create_input_provider();
    let (mut c3_handler, c3_cmd) = SessionHandler::new(
        c3_handle,
        NetplayConfig {
            name: "P3".to_string(),
            transport: TransportKind::Tcp,
            spectator: true,
            room_code: room_id,
        },
        c3_input.clone(),
        c3_event_rx,
        c3_game_tx,
    );
    tokio::spawn(async move { c3_handler.run().await });
    sleep(Duration::from_millis(100)).await;

    // P3 receives ROM
    let rom_event = c3_game_rx.recv().await.unwrap();
    assert!(matches!(rom_event, NetplayEvent::LoadRom(_)));
    c3_cmd.send(NetplayCommand::RomLoaded).await.unwrap();

    // 6. P3 should receive SyncState and StartGame (used for BeginCatchUp)
    let state_event = c3_game_rx.recv().await.unwrap();
    if let NetplayEvent::SyncState(frame, data) = state_event {
        assert_eq!(frame, 5);
        assert_eq!(data, vec![0xAA, 0xBB]);
    } else {
        panic!("Expected SyncState event, got {:?}", state_event);
    }

    let start_event = c3_game_rx.recv().await.unwrap();
    assert!(matches!(start_event, NetplayEvent::StartGame));

    // 7. Verify P3 has inputs for session frames 5-10
    // Emulator frame F corresponds to session frame F + 5.
    // So session frames 5..=10 correspond to emulator frames 0..=5.
    sleep(Duration::from_millis(300)).await;
    for f in 0..=5 {
        let session_frame = f + 5;
        assert!(
            c3_input.is_frame_ready(f),
            "Emulator frame {} (session {}) should be ready for spectator",
            f,
            session_frame
        );
        let inputs = c3_input
            .poll_inputs(f)
            .expect("Failed to poll emulator frame");
        assert_eq!(
            inputs[0], 0x01,
            "P1 input mismatch at session frame {}",
            session_frame
        );
        assert_eq!(
            inputs[1], 0x02,
            "P2 input mismatch at session frame {}",
            session_frame
        );
    }
}
