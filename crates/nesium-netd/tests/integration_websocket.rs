use std::net::SocketAddr;
use std::sync::Arc;

use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use nesium_netd::net::tcp::run_tcp_listener_with_listener;
use nesium_netd::net::{inbound::InboundEvent, quic_config};
use nesium_netproto::{codec::encode_message, messages::sync::Ping, msg_id::MsgId};
use tokio::sync::mpsc;
use tokio_rustls::rustls;
use tokio_tungstenite::tungstenite::Message;

// Actually better implementation of spawn_test_server that runs the loop:
async fn run_mock_server(app_name: &str) -> (SocketAddr, mpsc::Receiver<InboundEvent>) {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (tx, rx) = mpsc::channel(100);

    let app_name = app_name.to_string();
    tokio::spawn(async move {
        if let Err(e) = run_tcp_listener_with_listener(listener, tx, &app_name).await {
            eprintln!("Server error: {}", e);
        }
    });

    (addr, rx)
}

fn install_crypto_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

#[tokio::test]
async fn test_ws_connection() {
    let _ = tracing_subscriber::fmt::try_init();
    install_crypto_provider();
    let (addr, mut rx) = run_mock_server("test_ws_connection").await;
    let url = format!("ws://{}/", addr);

    let (mut ws_stream, _) = tokio_tungstenite::connect_async(&url)
        .await
        .expect("Connect failed");

    // 1. Send data from Client to Server
    let ping = Ping { t_ms: 42 };
    let frame_bytes = encode_message(&ping).unwrap();

    ws_stream
        .send(Message::Binary(Bytes::from(frame_bytes)))
        .await
        .unwrap();

    // 2. Server should receive InboundEvent::Connected then Packet
    let connected = rx.recv().await.expect("Expected connected event");
    let outbound_tx = if let InboundEvent::Connected { outbound, .. } = connected {
        outbound
    } else {
        panic!("Expected Connected event, got {:?}", connected);
    };

    let event = rx.recv().await.expect("Expected packet");
    match event {
        InboundEvent::Packet { packet, .. } => {
            assert_eq!(packet.msg_id(), MsgId::Ping);
            let received_ping: Ping = postcard::from_bytes(&packet.payload).unwrap();
            assert_eq!(received_ping.t_ms, 42);

            // 3. Send response from Server to Client
            let response = Bytes::from_static(b"response");
            outbound_tx.send(response.clone()).await.unwrap();
        }
        _ => panic!("Expected Packet event, got {:?}", event),
    }

    // 4. Client receives response
    let msg = ws_stream
        .next()
        .await
        .expect("Stream closed")
        .expect("Error");
    if let Message::Binary(data) = msg {
        assert_eq!(data, Bytes::from_static(b"response"));
    } else {
        panic!("Expected binary message, got {:?}", msg);
    }
}

#[tokio::test]
async fn test_wss_connection() {
    install_crypto_provider();

    let app_name = "test_wss_connection";
    // The server handles cert generation internally now.
    let (addr, mut rx) = run_mock_server(app_name).await;
    let url = format!("wss://{}/", addr);

    // Wait a brief moment to ensure certs are written (though ensure_quic_cert_pair handles it)
    // We need to trust the generated cert.
    let dir = quic_config::default_quic_data_dir(app_name);
    let (cert_path, _) = quic_config::ensure_quic_cert_pair(&dir).unwrap();

    let cert_pem = std::fs::read(cert_path).unwrap();
    let mut reader = std::io::BufReader::new(&cert_pem[..]);
    let cert_der = rustls_pemfile::certs(&mut reader).next().unwrap().unwrap();

    // Custom client config to trust self-signed
    let mut root_store = rustls::RootCertStore::empty();
    root_store.add(cert_der).unwrap();

    let client_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    let connector = tokio_tungstenite::Connector::Rustls(Arc::new(client_config));
    let (mut ws_stream, _) =
        tokio_tungstenite::connect_async_tls_with_config(&url, None, false, Some(connector))
            .await
            .expect("Connect failed");

    // 1. Send
    let ping = Ping { t_ms: 99 };
    let frame_bytes = encode_message(&ping).unwrap();

    ws_stream
        .send(Message::Binary(Bytes::from(frame_bytes)))
        .await
        .unwrap();

    // 2. Receive
    let connected = rx.recv().await.unwrap(); // Connected
    let outbound_tx = if let InboundEvent::Connected { outbound, .. } = connected {
        outbound
    } else {
        panic!("Expected Connected event");
    };

    let event = rx.recv().await.unwrap();
    if let InboundEvent::Packet { packet, .. } = event {
        assert_eq!(packet.msg_id(), MsgId::Ping);
        // We know payload is correct if ID matches and verification above passed
        outbound_tx
            .send(Bytes::from_static(b"secure_reply"))
            .await
            .unwrap();
    } else {
        panic!("Expected packet, got {:?}", event);
    }

    // 3. Client receive
    let msg = ws_stream.next().await.unwrap().unwrap();
    if let Message::Binary(data) = msg {
        assert_eq!(data, Bytes::from_static(b"secure_reply"));
    } else {
        panic!("Expected binary, got {:?}", msg);
    }
}
