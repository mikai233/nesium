use std::net::SocketAddr;
use std::time::Duration;

use nesium_netd::{RoomCleanupConfig, net::tcp::run_tcp_listener_with_listener};
use nesium_netproto::{
    channel::ChannelKind,
    codec::encode_message,
    messages::session::{AttachChannel, Hello, TransportKind, Welcome},
    messages::sync::Ping,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::mpsc,
    time::timeout,
};

async fn spawn_test_server(max_idle: Duration) -> (SocketAddr, mpsc::Sender<()>) {
    let _ = rustls::crypto::ring::default_provider().install_default();
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
    let (event_tx, event_rx) = mpsc::channel(1024);

    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    let server_addr = listener.local_addr().unwrap();

    let tx_clone = event_tx.clone();
    tokio::spawn(async move {
        let _ = run_tcp_listener_with_listener(listener, tx_clone, "test_idle", None).await;
    });

    let cleanup_config = RoomCleanupConfig {
        check_interval: Duration::from_millis(100),
        max_idle_duration: max_idle,
    };

    tokio::spawn(async move {
        tokio::select! {
             _ = nesium_netd::run_server(event_rx, None, Some(cleanup_config)) => {},
             _ = shutdown_rx.recv() => {},
        }
    });

    tokio::time::sleep(Duration::from_millis(50)).await;
    (server_addr, shutdown_tx)
}

#[tokio::test]
async fn test_secondary_channel_activity_propagation() -> anyhow::Result<()> {
    println!("Starting test_secondary_channel_activity_propagation...");
    let max_idle = Duration::from_millis(500);
    let (addr, _shutdown) = spawn_test_server(max_idle).await;

    println!("Connecting control channel...");
    let mut control = TcpStream::connect(addr).await?;
    let hello = Hello {
        client_nonce: 1,
        transport: TransportKind::Tcp,
        proto_min: 1,
        proto_max: 1,
        name: "Control".to_string(),
    };
    control.write_all(&encode_message(&hello)?).await?;

    let mut buf = vec![0u8; 1024];
    let n = timeout(Duration::from_secs(1), control.read(&mut buf)).await??;
    if n < 4 {
        anyhow::bail!("Too few bytes");
    }
    let (frames, _) = nesium_netproto::codec::try_decode_tcp_frames(&buf[..n])?;
    let welcome: Welcome = postcard::from_bytes(&frames[0].payload)?;
    let token = welcome.session_token;
    println!("Control established, token: {:X}", token);

    println!("Connecting input channel...");
    let mut input = TcpStream::connect(addr).await?;
    let attach = AttachChannel {
        session_token: token,
        channel: ChannelKind::Input,
    };
    input.write_all(&encode_message(&attach)?).await?;
    println!("Input attached");

    for i in 0..10 {
        let ping = Ping { t_ms: 0 };
        control.write_all(&encode_message(&ping)?).await?;
        tokio::time::sleep(Duration::from_millis(150)).await;

        let mut peek_buf = [0u8; 1];
        match timeout(Duration::from_millis(10), input.peek(&mut peek_buf)).await {
            Ok(Ok(0)) => anyhow::bail!("Input EOF at iteration {}", i),
            Ok(Err(e)) => anyhow::bail!("Input error at iteration {}: {}", i, e),
            _ => { /* Still open */ }
        }
    }
    println!("Activity propagation successful (Input stayed alive)");

    // Drain any pending pongs so they don't interfere with EOF check
    let mut drain_buf = [0u8; 1024];
    while let Ok(Ok(n)) = timeout(Duration::from_millis(10), control.read(&mut drain_buf)).await {
        if n == 0 {
            break;
        }
    }
    while let Ok(Ok(n)) = timeout(Duration::from_millis(10), input.read(&mut drain_buf)).await {
        if n == 0 {
            break;
        }
    }

    println!("Stopping activity, waiting for timeout...");
    tokio::time::sleep(max_idle * 2).await;

    println!("Checking for EOF...");
    let mut buf = [0u8; 1];
    let res_control = timeout(Duration::from_secs(1), control.read(&mut buf)).await??;
    let res_input = timeout(Duration::from_secs(1), input.read(&mut buf)).await??;

    assert_eq!(res_control, 0, "Control should be EOF");
    assert_eq!(res_input, 0, "Input should be EOF");
    println!("Both channels disconnected cleanily by server");

    Ok(())
}

#[tokio::test]
async fn test_input_channel_activity_keeps_control_alive() -> anyhow::Result<()> {
    println!("Starting test_input_channel_activity_keeps_control_alive...");
    let max_idle = Duration::from_millis(500);
    let (addr, _shutdown) = spawn_test_server(max_idle).await;

    println!("Connecting control channel...");
    let mut control = TcpStream::connect(addr).await?;
    let hello = Hello {
        client_nonce: 2,
        transport: TransportKind::Tcp,
        proto_min: 1,
        proto_max: 1,
        name: "Control2".to_string(),
    };
    control.write_all(&encode_message(&hello)?).await?;

    let mut buf = vec![0u8; 1024];
    let n = timeout(Duration::from_secs(1), control.read(&mut buf)).await??;
    if n < 4 {
        anyhow::bail!("Too few bytes for Welcome: {}", n);
    }
    let (frames, _) = nesium_netproto::codec::try_decode_tcp_frames(&buf[..n])?;
    let welcome: Welcome = postcard::from_bytes(&frames[0].payload)?;
    let token = welcome.session_token;
    println!("Control established, token: {:X}", token);

    println!("Connecting input channel...");
    let mut input = TcpStream::connect(addr).await?;
    let attach = AttachChannel {
        session_token: token,
        channel: ChannelKind::Input,
    };
    input.write_all(&encode_message(&attach)?).await?;
    eprintln!("Input attached with token {:X}", token);

    for i in 0..10 {
        let ping = Ping { t_ms: 0 };
        input.write_all(&encode_message(&ping)?).await?;
        tokio::time::sleep(Duration::from_millis(150)).await;

        let mut peek_buf = [0u8; 1];
        match timeout(Duration::from_millis(10), control.peek(&mut peek_buf)).await {
            Ok(Ok(0)) => anyhow::bail!("Control EOF at iteration {}", i),
            Ok(Err(e)) => anyhow::bail!("Control error at iteration {}: {}", i, e),
            _ => { /* Still open */ }
        }
    }
    eprintln!("Activity propagation successful (Control stayed alive due to Input)");

    Ok(())
}
