//! End-to-end integration tests for ROM synchronization.

use std::net::SocketAddr;
use std::time::Duration;

use nesium_netd::net::{quic_config, tcp::run_tcp_listener_with_listener};
use nesium_netproto::{
    codec::{encode_message, try_decode_tcp_frames},
    constants::AUTO_PLAYER_INDEX,
    messages::session::{
        Hello, JoinAck, JoinRoom, LoadRom, RomLoaded, StartGame, TransportKind, Welcome,
    },
    msg_id::MsgId,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc,
    time::timeout,
};

/// Mock test client (copied from integration_smoke.rs and extended).
struct TestClient {
    stream: TcpStream,
    client_id: u32,
    room_id: u32,
}

impl TestClient {
    async fn connect(addr: SocketAddr) -> anyhow::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self {
            stream,
            client_id: 0,
            room_id: 0,
        })
    }

    async fn send_hello(&mut self, name: &str) -> anyhow::Result<()> {
        let hello = Hello {
            client_nonce: 12345,
            transport: TransportKind::Tcp,
            proto_min: nesium_netproto::constants::VERSION,
            proto_max: nesium_netproto::constants::VERSION,
            name: name.to_string(),
        };

        let frame = encode_message(&hello)?;
        self.stream.write_all(&frame).await?;
        Ok(())
    }

    async fn recv_welcome(&mut self) -> anyhow::Result<Welcome> {
        let mut buf = vec![0u8; 4096];
        let n = timeout(Duration::from_secs(2), self.stream.read(&mut buf)).await??;
        buf.truncate(n);

        let (packets, _) = try_decode_tcp_frames(&buf)?;
        assert_eq!(packets.len(), 1, "Expected 1 Welcome packet");
        let packet = &packets[0];
        assert_eq!(packet.msg_id(), MsgId::Welcome);

        let welcome: Welcome = postcard::from_bytes(packet.payload)?;
        self.client_id = welcome.assigned_client_id;
        Ok(welcome)
    }

    async fn send_join_room(&mut self, room_code: u32) -> anyhow::Result<()> {
        let join = JoinRoom {
            room_code,
            preferred_sync_mode: None,
            desired_role: AUTO_PLAYER_INDEX,
            has_rom: false,
        };

        let frame = encode_message(&join)?;
        self.stream.write_all(&frame).await?;
        Ok(())
    }

    async fn recv_join_ack(&mut self) -> anyhow::Result<JoinAck> {
        let mut buf = vec![0u8; 4096];
        let n = timeout(Duration::from_secs(2), self.stream.read(&mut buf)).await??;
        buf.truncate(n);

        let (packets, _) = try_decode_tcp_frames(&buf)?;
        // Find the JoinAck packet (may be bundled with PlayerJoined notifications)
        let packet = packets
            .iter()
            .find(|p| p.msg_id() == MsgId::JoinAck)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Expected JoinAck packet, got {:?}",
                    packets.iter().map(|p| p.msg_id()).collect::<Vec<_>>()
                )
            })?;

        let ack: JoinAck = postcard::from_bytes(packet.payload)?;
        if ack.ok {
            self.room_id = ack.room_id;
        }
        Ok(ack)
    }

    async fn send_load_rom(&mut self, data: Vec<u8>) -> anyhow::Result<()> {
        let msg = LoadRom { data };

        let frame = encode_message(&msg)?;
        self.stream.write_all(&frame).await?;
        Ok(())
    }

    async fn recv_load_rom(&mut self) -> anyhow::Result<Vec<u8>> {
        let mut buf = vec![0u8; 65536]; // Larger buffer for ROM
        let n = timeout(Duration::from_secs(2), self.stream.read(&mut buf)).await??;
        buf.truncate(n);

        let (packets, _) = try_decode_tcp_frames(&buf)?;
        assert_eq!(packets.len(), 1, "Expected 1 LoadRom packet");
        let packet = &packets[0];
        assert_eq!(packet.msg_id(), MsgId::LoadRom);

        let msg: LoadRom = postcard::from_bytes(packet.payload)?;
        Ok(msg.data)
    }

    async fn send_rom_loaded(&mut self) -> anyhow::Result<()> {
        let msg = RomLoaded {};

        let frame = encode_message(&msg)?;
        self.stream.write_all(&frame).await?;
        Ok(())
    }

    async fn recv_start_game(&mut self) -> anyhow::Result<()> {
        let start = std::time::Instant::now();
        loop {
            if start.elapsed() > Duration::from_secs(5) {
                anyhow::bail!("Timeout waiting for StartGame");
            }

            let mut buf = vec![0u8; 4096];
            // Use a short read timeout to allow checking elapsed time
            let res = timeout(Duration::from_millis(500), self.stream.read(&mut buf)).await;
            let n = match res {
                Ok(Ok(n)) => n,
                Ok(Err(e)) => return Err(e.into()),
                Err(_) => continue, // Timeout, check total time
            };

            if n == 0 {
                anyhow::bail!("Connection closed");
            }
            buf.truncate(n);

            if let Ok((packets, _)) = try_decode_tcp_frames(&buf) {
                if let Some(packet) = packets.iter().find(|p| p.msg_id() == MsgId::StartGame) {
                    let _: StartGame = postcard::from_bytes(packet.payload)?;
                    return Ok(());
                }
                // If not StartGame, ignore and continue reading (e.g. PlayerJoined)
            }
        }
    }
}

fn install_crypto_provider() {
    let _ = tokio_rustls::rustls::crypto::ring::default_provider().install_default();
}

/// Spawn test server on a given address.
async fn spawn_test_server(app_name: &str) -> (SocketAddr, mpsc::Sender<()>) {
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
    let (event_tx, event_rx) = mpsc::channel(1024);

    let cert_dir = quic_config::default_quic_data_dir(app_name);
    if cert_dir.exists() {
        let _ = std::fs::remove_dir_all(&cert_dir);
    }

    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let server_addr = listener.local_addr().unwrap();

    // Spawn TCP listener
    let tx_clone = event_tx.clone();
    let app_name_owned = app_name.to_string();
    tokio::spawn(async move {
        if let Err(e) = run_tcp_listener_with_listener(listener, tx_clone, &app_name_owned).await {
            eprintln!("Listener error: {}", e);
        }
    });

    // Spawn server loop
    tokio::spawn(async move {
        tokio::select! {
            _ = nesium_netd::run_server(event_rx) => {},
            _ = shutdown_rx.recv() => {},
        }
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(50)).await;

    (server_addr, shutdown_tx)
}

#[tokio::test]
async fn test_rom_sync_flow() -> anyhow::Result<()> {
    install_crypto_provider();
    let (addr, _shutdown) = spawn_test_server("test_rom_sync").await;

    // 1. Client 1 (Host) Connects and Creates Room
    let mut c1 = TestClient::connect(addr).await?;
    c1.send_hello("Host").await?;
    c1.recv_welcome().await?;
    c1.send_join_room(0).await?;
    let ack1 = c1.recv_join_ack().await?;
    assert_eq!(ack1.player_index, 0);
    let room_code = c1.room_id;

    // 2. Client 2 (Joiner) Connects
    let mut c2 = TestClient::connect(addr).await?;
    c2.send_hello("Joiner").await?;
    c2.recv_welcome().await?;
    c2.send_join_room(room_code).await?;
    let ack2 = c2.recv_join_ack().await?;
    assert_eq!(ack2.player_index, 1);

    // 3. Host Sends LoadRom
    let rom_data = vec![0xDE, 0xAD, 0xBE, 0xEF];
    c1.send_load_rom(rom_data.clone()).await?;
    c1.send_rom_loaded().await?;

    // 4. Joiner Receives LoadRom
    let received_rom = c2.recv_load_rom().await?;
    assert_eq!(received_rom, rom_data);

    // 5. Joiner Sends RomLoaded
    c2.send_rom_loaded().await?;

    // 6. Both Receive StartGame
    // Order depends on broadcast loop, but both should get it.
    // We check sequentially with timeout flexibility in recv.
    c1.recv_start_game().await?;
    c2.recv_start_game().await?;

    Ok(())
}
