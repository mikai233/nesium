//! End-to-end integration tests for P2P signaling (Host-as-server) and relay fallback.

use std::net::SocketAddr;
use std::time::Duration;

use nesium_netd::net::quic_config;
use nesium_netd::net::tcp::run_tcp_listener_with_listener;
use nesium_netproto::constants::AUTO_PLAYER_INDEX;
use nesium_netproto::messages::Message;
use nesium_netproto::messages::session::{JoinAck, JoinRoom};
use nesium_netproto::{
    codec::{encode_message, try_decode_tcp_frames},
    messages::session::{
        FallbackToRelay, Hello, P2PCreateRoom, P2PFallbackNotice, P2PJoinAck, P2PJoinRoom,
        P2PRequestFallback, P2PRoomCreated, RequestFallbackRelay, TransportKind, Welcome,
    },
    msg_id::MsgId,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc,
    time::timeout,
};

fn install_crypto_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

struct RawClient {
    stream: TcpStream,
}

impl RawClient {
    async fn connect(addr: SocketAddr) -> anyhow::Result<Self> {
        Ok(Self {
            stream: TcpStream::connect(addr).await?,
        })
    }

    async fn hello(&mut self, name: &str) -> anyhow::Result<Welcome> {
        let hello = Hello {
            client_nonce: 123,
            transport: TransportKind::Tcp,
            proto_min: nesium_netproto::constants::VERSION,
            proto_max: nesium_netproto::constants::VERSION,
            name: name.to_string(),
        };
        let frame = encode_message(&hello)?;
        self.stream.write_all(&frame).await?;
        self.recv_one::<Welcome>(MsgId::Welcome).await
    }

    async fn send<T: Message>(&mut self, payload: &T) -> anyhow::Result<()> {
        let frame = encode_message(payload)?;
        self.stream.write_all(&frame).await?;
        Ok(())
    }

    async fn recv_one<T: serde::de::DeserializeOwned>(&mut self, want: MsgId) -> anyhow::Result<T> {
        let mut buf = vec![0u8; 4096];
        let n = timeout(Duration::from_secs(2), self.stream.read(&mut buf)).await??;
        buf.truncate(n);
        let (packets, _) = try_decode_tcp_frames(&buf)?;
        let pkt = packets
            .iter()
            .find(|p| p.msg_id() == want)
            .ok_or_else(|| anyhow::anyhow!("Expected {:?}, got {:?}", want, packets))?;
        Ok(postcard::from_bytes(pkt.payload)?)
    }

    async fn recv_any(&mut self) -> anyhow::Result<Vec<(MsgId, Vec<u8>)>> {
        let mut buf = vec![0u8; 4096];
        let n = timeout(Duration::from_secs(2), self.stream.read(&mut buf)).await??;
        buf.truncate(n);
        let (packets, _) = try_decode_tcp_frames(&buf)?;
        Ok(packets
            .into_iter()
            .map(|p| (p.msg_id(), p.payload.to_vec()))
            .collect())
    }
}

async fn spawn_server(app_name: &str) -> anyhow::Result<SocketAddr> {
    let addr: SocketAddr = "127.0.0.1:0".parse()?;
    let (event_tx, event_rx) = mpsc::channel(1024);

    // Clean up existing certs to avoid KeyMismatch from old runs
    let cert_dir = quic_config::default_quic_data_dir(app_name);
    if cert_dir.exists() {
        let _ = std::fs::remove_dir_all(&cert_dir);
    }

    // Bind once and keep it
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let server_addr = listener.local_addr()?;

    let tx_clone = event_tx.clone();
    let app_name_owned = app_name.to_string();
    tokio::spawn(async move {
        // Use run_tcp_listener_with_listener to use the existing listener
        if let Err(e) =
            run_tcp_listener_with_listener(listener, tx_clone, &app_name_owned, None).await
        {
            eprintln!("Server error: {}", e);
        }
    });

    tokio::spawn(async move {
        let _ = nesium_netd::run_server(event_rx, None).await;
    });

    // Validated: listener is active immediately, but give a tiny slack for spawn
    tokio::time::sleep(Duration::from_millis(50)).await;
    Ok(server_addr)
}

#[tokio::test]
async fn p2p_signaling_room_create_and_join() -> anyhow::Result<()> {
    install_crypto_provider();
    let server_addr = spawn_server("test_p2p_sig_1").await?;

    // Host creates signaling room.
    let mut host = RawClient::connect(server_addr).await?;
    let _welcome = host.hello("host").await?;
    host.send(&P2PCreateRoom {
        host_addrs: vec![
            "10.0.0.2:9999".parse().unwrap(),
            "127.0.0.1:9999".parse().unwrap(),
        ],
        host_room_id: 4242,
        host_quic_cert_sha256_fingerprint: Some("deadbeef".to_string()),
        host_quic_server_name: Some("nesium".to_string()),
    })
    .await?;
    let created: P2PRoomCreated = host.recv_one(MsgId::P2PRoomCreated).await?;
    assert_ne!(created.room_id, 0);

    // Joiner fetches host info.
    let mut joiner = RawClient::connect(server_addr).await?;
    let _welcome = joiner.hello("joiner").await?;
    joiner
        .send(&P2PJoinRoom {
            room_id: created.room_id,
        })
        .await?;
    let ack: P2PJoinAck = joiner.recv_one(MsgId::P2PJoinAck).await?;
    assert!(ack.ok);
    assert_eq!(ack.room_id, created.room_id);
    assert_eq!(ack.host_room_id, 4242);
    assert_eq!(ack.host_addrs.len(), 2);
    assert_eq!(
        ack.host_quic_cert_sha256_fingerprint.as_deref(),
        Some("deadbeef")
    );
    assert_eq!(ack.host_quic_server_name.as_deref(), Some("nesium"));
    assert!(!ack.fallback_required);

    Ok(())
}

#[tokio::test]
async fn p2p_fallback_notice_is_broadcast_to_watchers() -> anyhow::Result<()> {
    install_crypto_provider();
    let server_addr = spawn_server("test_p2p_sig_2").await?;

    // Host creates signaling room.
    let mut host = RawClient::connect(server_addr).await?;
    let _welcome = host.hello("host").await?;
    host.send(&P2PCreateRoom {
        host_addrs: vec!["127.0.0.1:9999".parse().unwrap()],
        host_room_id: 1,
        host_quic_cert_sha256_fingerprint: None,
        host_quic_server_name: None,
    })
    .await?;
    let created: P2PRoomCreated = host.recv_one(MsgId::P2PRoomCreated).await?;

    // Joiner joins signaling room.
    let mut joiner = RawClient::connect(server_addr).await?;
    let _welcome = joiner.hello("joiner").await?;
    joiner
        .send(&P2PJoinRoom {
            room_id: created.room_id,
        })
        .await?;
    let _ack: P2PJoinAck = joiner.recv_one(MsgId::P2PJoinAck).await?;

    // Joiner requests fallback.
    joiner
        .send(&P2PRequestFallback {
            room_id: created.room_id,
            reason: "direct connect failed".to_string(),
        })
        .await?;

    // Both should receive a notice (order not guaranteed).
    let host_msgs = host.recv_any().await?;
    let joiner_msgs = joiner.recv_any().await?;

    let decode_notice = |msgs: &[(MsgId, Vec<u8>)]| -> Option<P2PFallbackNotice> {
        msgs.iter()
            .find(|(id, _)| *id == MsgId::P2PFallbackNotice)
            .and_then(|(_, b)| postcard::from_bytes::<P2PFallbackNotice>(b).ok())
    };

    let n1 = decode_notice(&host_msgs).expect("host should receive P2PFallbackNotice");
    let n2 = decode_notice(&joiner_msgs).expect("joiner should receive P2PFallbackNotice");
    assert_eq!(n1.room_id, created.room_id);
    assert_eq!(n2.room_id, created.room_id);
    assert!(n1.reason.contains("direct"));
    assert!(n2.reason.contains("direct"));

    Ok(())
}

#[tokio::test]
async fn host_can_broadcast_fallback_to_direct_clients() -> anyhow::Result<()> {
    install_crypto_provider();
    let server_addr = spawn_server("test_p2p_sig_3").await?;

    // Host joins a normal netplay room (acts as authoritative server for direct clients).
    let mut host = RawClient::connect(server_addr).await?;
    let _welcome = host.hello("host").await?;
    host.send(&JoinRoom {
        room_id: 0,
        preferred_sync_mode: None,
        desired_role: AUTO_PLAYER_INDEX,
        has_rom: false,
    })
    .await?;
    let ack: JoinAck = host.recv_one(MsgId::JoinAck).await?;
    let room_id = ack.room_id;

    // Joiner joins the same room.
    let mut joiner = RawClient::connect(server_addr).await?;
    let _welcome = joiner.hello("joiner").await?;
    joiner
        .send(&JoinRoom {
            room_id,
            preferred_sync_mode: None,
            desired_role: AUTO_PLAYER_INDEX,
            has_rom: false,
        })
        .await?;
    let _ack2: JoinAck = joiner.recv_one(MsgId::JoinAck).await?;

    // Host requests fallback broadcast.
    host.send(&RequestFallbackRelay {
        relay_addr: server_addr,
        relay_room_id: room_id,
        reason: "switching to relay".to_string(),
    })
    .await?;

    // Joiner receives instruction.
    let msg: FallbackToRelay = joiner.recv_one(MsgId::FallbackToRelay).await?;
    assert_eq!(msg.relay_addr, server_addr);
    assert_eq!(msg.relay_room_id, room_id);
    assert!(msg.reason.contains("relay"));

    Ok(())
}
