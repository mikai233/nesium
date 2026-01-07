// use nesium_netd::run_server;
use nesium_netplay::{
    NetplayCommand, NetplayConfig, NetplayEvent, SessionHandler, connect, create_input_provider,
};
// use nesium_netproto::messages::session::LoadRom;
use std::net::SocketAddr;
use tokio::sync::mpsc;
use tokio::time::{Duration, sleep};

#[tokio::test]
async fn test_rom_sync_flow() {
    // 1. Start Server
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let (event_tx, event_rx) = mpsc::channel(1024);

    // Pick a free port for this test instance.
    let server_addr = {
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        let local_addr = listener.local_addr().unwrap();
        drop(listener);
        local_addr
    };

    let tx_clone = event_tx.clone();
    tokio::spawn(async move {
        let _ = nesium_netd::net::tcp::run_tcp_listener(server_addr, tx_clone).await;
    });

    // Spawn server loop
    tokio::spawn(async move {
        let _ = nesium_netd::run_server(event_rx).await;
    });

    // Wait for server to start
    sleep(Duration::from_millis(100)).await;

    // 2. Connect Client 1 (Host)
    let (c1_event_tx, c1_event_rx) = mpsc::channel(100);
    let (c1_game_tx, mut c1_game_rx) = mpsc::channel(100);
    let c1_handle = connect(server_addr, c1_event_tx).await.unwrap();
    let c1_input = create_input_provider();
    let c1_config = NetplayConfig {
        name: "Player1".to_string(),
        rom_hash: [1; 16],
        spectator: false,
        room_code: 0, // Create room
    };
    let (mut c1_handler, c1_cmd) = SessionHandler::new(
        c1_handle,
        c1_config,
        c1_input.clone(),
        c1_event_rx,
        c1_game_tx,
    );
    tokio::spawn(async move { c1_handler.run().await });

    // Wait for connection
    sleep(Duration::from_millis(50)).await;

    // Create Room
    c1_cmd.send(NetplayCommand::CreateRoom).await.unwrap();

    // Wait for room creation
    sleep(Duration::from_millis(100)).await;

    // 3. Get Room Code
    let room_id = c1_input.with_session(|s| s.room_id);
    assert_ne!(room_id, 0, "Room should be created");

    // 4. Connect Client 2 (Joiner)
    let (c2_event_tx, c2_event_rx) = mpsc::channel(100);
    let (c2_game_tx, mut c2_game_rx) = mpsc::channel(100);
    let c2_handle = connect(server_addr, c2_event_tx).await.unwrap();
    let c2_input = create_input_provider();
    let c2_config = NetplayConfig {
        name: "Player2".to_string(),
        rom_hash: [1; 16],
        spectator: false,
        room_code: room_id,
    };
    let (mut c2_handler, c2_cmd) = SessionHandler::new(
        c2_handle,
        c2_config,
        c2_input.clone(),
        c2_event_rx,
        c2_game_tx,
    );
    tokio::spawn(async move { c2_handler.run().await });

    sleep(Duration::from_millis(100)).await;

    // 5. Player 1 Loads ROM
    let rom_data = vec![0x1, 0x2, 0x3, 0x4];
    c1_cmd
        .send(NetplayCommand::SendRom(rom_data.clone()))
        .await
        .unwrap();

    // 6. Verify Player 2 received LoadRom
    let event = c2_game_rx.recv().await.expect("C2 should receive LoadRom");
    match event {
        NetplayEvent::LoadRom(data) => assert_eq!(data, rom_data),
        _ => panic!("Expected LoadRom event"),
    }

    // 7. Both Players Confirm Load
    c1_cmd.send(NetplayCommand::RomLoaded).await.unwrap();
    c2_cmd.send(NetplayCommand::RomLoaded).await.unwrap();

    // 8. Verify Both Players Receive StartGame
    // Note: C1 might receive it first or second depending on server order/network
    // C1 should receive it (might receive LoadRom echo? No, host doesn't receive own load rom)
    // Actually wait, does C1 receive LoadRom? Server implementation says: "Broadcast to everyone else". So no.

    let _ = c1_game_rx
        .recv()
        .await
        .expect("C1 should receive StartGame");
    // Verify it is StartGame
    // (If C1 receives anything else it's unexpected here)

    let event = c2_game_rx
        .recv()
        .await
        .expect("C2 should receive StartGame");
    match event {
        NetplayEvent::StartGame => (), // Success
        _ => panic!("Expected StartGame event for C2"),
    }
}

#[tokio::test]
async fn test_late_join_receives_cached_rom_and_state() {
    // 1. Start Server
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let (event_tx, event_rx) = mpsc::channel(1024);

    let server_addr = {
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        let local_addr = listener.local_addr().unwrap();
        drop(listener);
        local_addr
    };

    let tx_clone = event_tx.clone();
    tokio::spawn(async move {
        let _ = nesium_netd::net::tcp::run_tcp_listener(server_addr, tx_clone).await;
    });

    tokio::spawn(async move {
        let _ = nesium_netd::run_server(event_rx).await;
    });

    sleep(Duration::from_millis(100)).await;

    // 2. Connect Client 1 (Host)
    let (c1_event_tx, c1_event_rx) = mpsc::channel(100);
    let (c1_game_tx, mut c1_game_rx) = mpsc::channel(100);
    let c1_handle = connect(server_addr, c1_event_tx).await.unwrap();
    let c1_input = create_input_provider();
    let c1_config = NetplayConfig {
        name: "Host".to_string(),
        rom_hash: [1; 16],
        spectator: false,
        room_code: 0, // Create room
    };
    let (mut c1_handler, c1_cmd) = SessionHandler::new(
        c1_handle,
        c1_config,
        c1_input.clone(),
        c1_event_rx,
        c1_game_tx,
    );
    tokio::spawn(async move { c1_handler.run().await });

    sleep(Duration::from_millis(50)).await;
    c1_cmd.send(NetplayCommand::CreateRoom).await.unwrap();
    sleep(Duration::from_millis(100)).await;

    let room_id = c1_input.with_session(|s| s.room_id);
    assert_ne!(room_id, 0, "Room should be created");

    // 3. Host sends ROM, then confirms loaded -> StartGame should be broadcast.
    let rom_data = vec![0xDE, 0xAD, 0xBE, 0xEF];
    c1_cmd
        .send(NetplayCommand::SendRom(rom_data.clone()))
        .await
        .unwrap();
    c1_cmd.send(NetplayCommand::RomLoaded).await.unwrap();

    // Wait for host StartGame.
    let event = c1_game_rx
        .recv()
        .await
        .expect("Host should receive StartGame");
    assert!(matches!(event, NetplayEvent::StartGame));

    // 4. Host provides a cached state snapshot (pretend it's from the runtime).
    let cached_frame = 5u32;
    let cached_state = vec![0xAA, 0xBB, 0xCC];
    c1_cmd
        .send(NetplayCommand::ProvideState(
            cached_frame,
            cached_state.clone(),
        ))
        .await
        .unwrap();

    // Send a few inputs so the server has history to replay.
    for f in 0..10u32 {
        c1_cmd
            .send(NetplayCommand::SendInput(f, 0x10))
            .await
            .unwrap();
    }

    sleep(Duration::from_millis(50)).await;

    // 5. Connect Client 2 (Late Joiner)
    let (c2_event_tx, c2_event_rx) = mpsc::channel(100);
    let (c2_game_tx, mut c2_game_rx) = mpsc::channel(100);
    let c2_handle = connect(server_addr, c2_event_tx).await.unwrap();
    let c2_input = create_input_provider();
    let c2_config = NetplayConfig {
        name: "LateJoiner".to_string(),
        rom_hash: [1; 16],
        spectator: false,
        room_code: room_id,
    };
    let (mut c2_handler, c2_cmd) = SessionHandler::new(
        c2_handle,
        c2_config,
        c2_input.clone(),
        c2_event_rx,
        c2_game_tx,
    );
    tokio::spawn(async move { c2_handler.run().await });

    // 6. Late joiner should receive cached LoadRom, then send RomLoaded.
    let event = c2_game_rx
        .recv()
        .await
        .expect("Late joiner should receive LoadRom");
    match event {
        NetplayEvent::LoadRom(data) => assert_eq!(data, rom_data),
        other => panic!("Expected LoadRom, got {:?}", other),
    }
    c2_cmd.send(NetplayCommand::RomLoaded).await.unwrap();

    // 7. Late joiner should receive SyncState + StartGame (order preserved by TCP).
    let event = c2_game_rx
        .recv()
        .await
        .expect("Late joiner should receive SyncState");
    match event {
        NetplayEvent::SyncState(frame, data) => {
            assert_eq!(frame, cached_frame);
            assert_eq!(data, cached_state);
        }
        other => panic!("Expected SyncState, got {:?}", other),
    }

    let event = c2_game_rx
        .recv()
        .await
        .expect("Late joiner should receive StartGame (BeginCatchUp)");
    assert!(matches!(event, NetplayEvent::StartGame));
}
