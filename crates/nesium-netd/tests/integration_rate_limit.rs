//! Integration test for rate limiting behavior.

use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use nesium_netd::net::quic_config;
use nesium_netd::net::rate_limit::{IpRateLimiter, RateLimitConfig};
use nesium_netd::net::tcp::run_tcp_listener_with_listener;
use nesium_netproto::codec::try_decode_tcp_frames;
use nesium_netproto::messages::session::{ErrorCode, ErrorMsg};
use nesium_netproto::msg_id::MsgId;
use nesium_netproto::{
    codec::encode_message,
    messages::session::{Hello, TransportKind},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc,
};

fn install_crypto_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

/// Spawn test server with rate limiting enabled.
async fn spawn_rate_limited_server(
    app_name: &str,
    rate_config: RateLimitConfig,
) -> (SocketAddr, mpsc::Sender<()>) {
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
    let (event_tx, event_rx) = mpsc::channel(1024);

    let cert_dir = quic_config::default_quic_data_dir(app_name);
    if cert_dir.exists() {
        let _ = std::fs::remove_dir_all(&cert_dir);
    }

    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let server_addr = listener.local_addr().unwrap();

    // Create IP rate limiter
    let ip_limiter = if rate_config.conn_limit_enabled() {
        Some(Arc::new(IpRateLimiter::new(rate_config.clone())))
    } else {
        None
    };

    // Spawn TCP listener with IP rate limiting
    let tx_clone = event_tx.clone();
    let app_name_owned = app_name.to_string();
    tokio::spawn(async move {
        if let Err(e) =
            run_tcp_listener_with_listener(listener, tx_clone, &app_name_owned, ip_limiter).await
        {
            eprintln!("Listener error: {}", e);
        }
    });

    // Spawn server loop with message rate limiting
    let rate_config_clone = Some(rate_config);
    tokio::spawn(async move {
        tokio::select! {
            _ = nesium_netd::run_server(event_rx, rate_config_clone, None) => {},
            _ = shutdown_rx.recv() => {},
        }
    });

    // Give server time to start
    tokio::time::sleep(Duration::from_millis(50)).await;

    (server_addr, shutdown_tx)
}

#[tokio::test]
async fn test_message_rate_limit_closes_connection() -> anyhow::Result<()> {
    install_crypto_provider();

    // Configure aggressive rate limiting: 2 messages per second, burst of 4
    let rate_config = RateLimitConfig {
        conn_per_ip_per_sec: 0, // Disable IP limiting for this test
        msg_per_conn_per_sec: 2,
        burst_multiplier: 2,
    };

    let (addr, _shutdown) = spawn_rate_limited_server("test_rate_limit", rate_config).await;

    let mut stream = TcpStream::connect(addr).await?;

    // Send Hello message
    let hello = Hello {
        client_nonce: 12345,
        transport: TransportKind::Tcp,
        proto_min: nesium_netproto::constants::VERSION,
        proto_max: nesium_netproto::constants::VERSION,
        name: "TestClient".to_string(),
    };
    let frame = encode_message(&hello)?;
    stream.write_all(&frame).await?;

    // Read Welcome response
    let mut buf = vec![0u8; 4096];
    let n = stream.read(&mut buf).await?;
    assert!(n > 0, "Should receive Welcome message");

    // Now rapidly send messages to exceed rate limit
    // Burst allows 4 messages (2 * 2), so send 10 messages rapidly
    for i in 0..10 {
        let hello = Hello {
            client_nonce: 10000 + i,
            transport: TransportKind::Tcp,
            proto_min: nesium_netproto::constants::VERSION,
            proto_max: nesium_netproto::constants::VERSION,
            name: format!("Spam{}", i),
        };
        let frame = encode_message(&hello)?;

        // Try to send - this might fail if connection is already closed
        if stream.write_all(&frame).await.is_err() {
            // Connection closed (expected)
            return Ok(());
        }
    }

    // Try to read - should get EOF because connection was closed
    buf.clear();
    let n = stream.read(&mut buf).await?;
    assert_eq!(n, 0, "Connection should be closed (EOF) after rate limit");

    Ok(())
}

#[tokio::test]
async fn test_ip_rate_limit_rejects_connections() -> anyhow::Result<()> {
    install_crypto_provider();

    // Configure aggressive IP rate limiting: 2 connections per IP per second, burst of 2
    let rate_config = RateLimitConfig {
        conn_per_ip_per_sec: 2,
        msg_per_conn_per_sec: 0, // Disable message limiting for this test
        burst_multiplier: 1,
    };

    let (addr, _shutdown) = spawn_rate_limited_server("test_ip_rate_limit", rate_config).await;

    // First 2 connections should succeed (burst allows 2)
    let _conn1 = TcpStream::connect(addr).await?;
    let _conn2 = TcpStream::connect(addr).await?;

    // Third connection should be rejected immediately (IP rate limited)
    // It will connect at TCP level but be dropped before any data exchange
    let mut conn3 = TcpStream::connect(addr).await?;

    // Try to send Hello - connection should be closed or fail to read
    let hello = Hello {
        client_nonce: 99999,
        transport: TransportKind::Tcp,
        proto_min: nesium_netproto::constants::VERSION,
        proto_max: nesium_netproto::constants::VERSION,
        name: "RateLimited".to_string(),
    };
    let frame = encode_message(&hello)?;

    // Send might succeed (buffered), but read should fail or return EOF
    let _ = conn3.write_all(&frame).await;

    let mut buf = vec![0u8; 4096];
    let result = tokio::time::timeout(Duration::from_secs(1), conn3.read(&mut buf)).await;

    // Either timeout (no response), EOF (connection closed), or error message received
    match result {
        Ok(Ok(0)) => {
            // EOF - connection was closed before we could read the error, still acceptable
            Ok(())
        }
        Ok(Ok(n)) => {
            // Received data - verify it is a RateLimited error
            let (packets, _) = try_decode_tcp_frames(&buf[..n])?;
            let packet = packets
                .first()
                .ok_or_else(|| anyhow::anyhow!("No packet received"))?;

            assert_eq!(packet.msg_id(), MsgId::ErrorMsg);
            let err: ErrorMsg = postcard::from_bytes(&packet.payload)?;
            assert_eq!(err.code, ErrorCode::RateLimited);
            Ok(())
        }
        Err(_) | Ok(Err(_)) => {
            // Timeout or error - also acceptable as connection was dropped
            Ok(())
        }
    }
}
