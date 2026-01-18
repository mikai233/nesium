//! Embedded Netplay Server API for Flutter.
//!
//! Allows starting/stopping a netplay server directly within the app.

use nesium_netd::net::inbound::{InboundEvent, next_conn_id};
use nesium_netd::net::tcp::{get_or_create_server_tls_acceptor, handle_tcp_connection};
use parking_lot::Mutex;
use std::net::SocketAddr;
use std::sync::{Arc, OnceLock};

use crate::frb_generated::StreamSink;
use flutter_rust_bridge::frb;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// Server status snapshot streamed to Flutter.
#[frb]
#[derive(Debug, Clone)]
pub struct ServerStatus {
    pub running: bool,
    pub port: u16,
    pub client_count: u32,
    pub bind_address: String,
    pub quic_enabled: bool,
    pub quic_bind_address: String,
    /// Leaf certificate SHA-256 fingerprint (base64url, no padding), for pinned QUIC client connections.
    pub quic_cert_sha256_fingerprint: String,
}

#[frb(ignore)]
pub struct EmbeddedServer {
    /// Sender to broadcast shutdown signal.
    pub shutdown_tx: Option<tokio::sync::watch::Sender<bool>>,
    /// Current bind address.
    pub bind_addr: Option<SocketAddr>,
    /// QUIC bind address (UDP).
    pub quic_bind_addr: Option<SocketAddr>,
    /// QUIC cert leaf SHA-256 fingerprint (AA:BB:..).
    pub quic_cert_sha256_fingerprint: Option<String>,
    /// Status stream sink.
    pub status_sink: Arc<Mutex<Option<StreamSink<ServerStatus>>>>,
    /// Number of connected clients.
    pub client_count: u32,
    /// Task handles to abort on shutdown.
    pub task_handles: Vec<JoinHandle<()>>,
}

impl EmbeddedServer {
    pub fn new() -> Self {
        Self {
            shutdown_tx: None,
            bind_addr: None,
            quic_bind_addr: None,
            quic_cert_sha256_fingerprint: None,
            status_sink: Arc::new(Mutex::new(None)),
            client_count: 0,
            task_handles: Vec::new(),
        }
    }

    pub fn is_running(&self) -> bool {
        self.shutdown_tx.is_some()
    }
}

static SERVER: OnceLock<Mutex<EmbeddedServer>> = OnceLock::new();

pub fn get_server() -> &'static Mutex<EmbeddedServer> {
    SERVER.get_or_init(|| Mutex::new(EmbeddedServer::new()))
}

/// Start the embedded netplay server on the specified port.
///
/// Pass `port = 0` to let the OS pick an available port.
#[frb]
pub async fn netserver_start(port: u16) -> Result<u16, String> {
    let server_mutex = get_server();
    {
        let server = server_mutex.lock();
        if server.is_running() {
            return Err("Server is already running".to_string());
        }
    }

    // Create the bind address
    let bind_addr: SocketAddr = format!("0.0.0.0:{}", port)
        .parse()
        .map_err(|e| format!("Invalid address: {}", e))?;

    // Bind the listener first to get the actual port (Async, no lock held)
    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .map_err(|e| format!("Failed to bind: {}", e))?;

    let actual_addr = listener
        .local_addr()
        .map_err(|e| format!("Failed to get local address: {}", e))?;
    let actual_port = actual_addr.port();

    // Start QUIC listener on the same port number (UDP uses a separate socket).
    let quic_bind_addr: SocketAddr = format!("0.0.0.0:{}", actual_port)
        .parse()
        .map_err(|e| format!("Invalid QUIC bind address: {}", e))?;

    let (quic_server_config, quic_fingerprint) = {
        let dir = nesium_netd::net::quic_config::default_quic_data_dir("nesium_flutter");
        let (cert_path, key_path) = nesium_netd::net::quic_config::ensure_quic_cert_pair(&dir)
            .map_err(|e| format!("Failed to ensure QUIC cert/key: {}", e))?;
        let fp = nesium_netd::net::quic_config::sha256_fingerprint_base64url_from_pem(&cert_path)
            .map_err(|e| format!("Failed to compute QUIC cert fingerprint: {}", e))?;
        let cfg = nesium_netd::net::quic_config::build_quic_server_config(&cert_path, &key_path)
            .map_err(|e| format!("Failed to build QUIC server config: {}", e))?;
        (cfg, fp)
    };

    // Create shutdown broadcast channel
    let (shutdown_tx, mut shutdown_rx_listener) = tokio::sync::watch::channel(false);

    // Create channels for server communication
    let (event_tx, mut event_rx) = mpsc::channel(1024);
    let (server_tx, server_rx) = mpsc::channel(1024);

    let mut server = server_mutex.lock();
    server.client_count = 0;
    server.task_handles.clear();
    server.quic_bind_addr = None;
    server.quic_cert_sha256_fingerprint = None;
    let status_sink = server.status_sink.clone();

    // Spawn the TCP listener task
    let shutdown_tx_clone = shutdown_tx.clone();
    let event_tx_listener = event_tx.clone();
    let listener_handle = tokio::spawn(async move {
        loop {
            let tls_acceptor = match get_or_create_server_tls_acceptor("nesium_flutter") {
                Ok(a) => a,
                Err(e) => {
                    tracing::error!("Failed to create TLS acceptor: {}", e);
                    tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
                    continue;
                }
            };

            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, peer)) => {
                            let conn_id = next_conn_id();
                            let tx_clone = event_tx_listener.clone();
                            let mut shutdown_rx_conn = shutdown_tx_clone.subscribe();
                            let tls_acceptor = tls_acceptor.clone();
                            tokio::spawn(async move {
                                tokio::select! {
                                    _ = handle_tcp_connection(
                                        stream, peer, conn_id, tx_clone, tls_acceptor
                                    ) => {},
                                    _ = shutdown_rx_conn.changed() => {
                                        tracing::debug!(conn_id, "Closing connection task due to server shutdown");
                                    }
                                }
                            });
                        }
                        Err(e) => {
                            tracing::warn!("Accept error: {}", e);
                            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        }
                    }
                }
                _ = shutdown_rx_listener.changed() => {
                    tracing::info!("Listener shutting down");
                    break;
                }
            }
        }
    });
    server.task_handles.push(listener_handle);

    // Spawn the QUIC listener task (aborted on shutdown).
    let tx_quic = event_tx.clone();
    let quic_handle = tokio::spawn(async move {
        let _ =
            nesium_netd::net::quic::run_quic_listener(quic_bind_addr, quic_server_config, tx_quic)
                .await;
    });
    server.task_handles.push(quic_handle);

    // Spawn monitoring task to track client count
    let status_sink_clone = status_sink.clone();
    let bind_address = actual_addr.to_string();
    let quic_bind_address = quic_bind_addr.to_string();
    let quic_fingerprint_clone = quic_fingerprint.clone();
    let mut shutdown_rx_monitor = shutdown_tx.subscribe();
    let monitor_handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                ev_opt = event_rx.recv() => {
                    let Some(ev) = ev_opt else { break; };
                    match ev {
                        InboundEvent::Connected { .. } => {
                            let mut s = get_server().lock();
                            s.client_count += 1;
                            let count = s.client_count;
                            notify_server_status(
                                &status_sink_clone,
                                true,
                                actual_port,
                                count,
                                bind_address.clone(),
                                true,
                                quic_bind_address.clone(),
                                quic_fingerprint_clone.clone(),
                            );
                        }
                        InboundEvent::Disconnected { .. } => {
                            let mut s = get_server().lock();
                            s.client_count = s.client_count.saturating_sub(1);
                            let count = s.client_count;
                            notify_server_status(
                                &status_sink_clone,
                                true,
                                actual_port,
                                count,
                                bind_address.clone(),
                                true,
                                quic_bind_address.clone(),
                                quic_fingerprint_clone.clone(),
                            );
                        }
                        _ => {}
                    }
                    if let Err(_) = server_tx.send(ev).await {
                        break;
                    }
                }
                _ = shutdown_rx_monitor.changed() => {
                    tracing::info!("Monitor shutting down");
                    break;
                }
            }
        }
    });
    server.task_handles.push(monitor_handle);

    // Spawn the server main loop
    let server_handle = tokio::spawn(async move {
        let _ = nesium_netd::run_server(server_rx, None).await;
    });
    server.task_handles.push(server_handle);

    server.shutdown_tx = Some(shutdown_tx);
    server.bind_addr = Some(actual_addr);
    server.quic_bind_addr = Some(quic_bind_addr);
    server.quic_cert_sha256_fingerprint = Some(quic_fingerprint.clone());

    // Notify status update
    notify_server_status(
        &status_sink,
        true,
        actual_port,
        0,
        actual_addr.to_string(),
        true,
        quic_bind_addr.to_string(),
        quic_fingerprint.clone(),
    );

    tracing::info!("Embedded server started on {}", actual_addr);
    tracing::info!("Embedded QUIC enabled on {}", quic_bind_addr);
    tracing::info!(
        "Embedded QUIC cert SHA-256 fingerprint: {}",
        quic_fingerprint
    );

    Ok(actual_port)
}

/// Stop the embedded netplay server.
#[frb]
pub async fn netserver_stop() -> Result<(), String> {
    let server_mutex = get_server();
    let (tx, task_handles, status_sink) = {
        let mut server = server_mutex.lock();
        let tx = server.shutdown_tx.take();
        let handles = std::mem::take(&mut server.task_handles);
        if tx.is_some() {
            server.bind_addr = None;
            server.client_count = 0;
            server.quic_bind_addr = None;
            server.quic_cert_sha256_fingerprint = None;
        }
        (tx, handles, server.status_sink.clone())
    };

    if let Some(tx) = tx {
        // Broadcast shutdown signal
        let _ = tx.send(true);

        // Abort all handles
        for handle in task_handles {
            handle.abort();
        }

        // Stop any active P2P host watcher (best effort).
        if let Some(task) = crate::api::netplay::get_manager()
            .p2p_watch_task
            .lock()
            .take()
        {
            task.abort();
        }

        // Notify status update
        notify_server_status(
            &status_sink,
            false,
            0,
            0,
            String::new(),
            false,
            String::new(),
            String::new(),
        );

        tracing::info!("Embedded server stopped and all tasks aborted");
        Ok(())
    } else {
        Err("Server is not running".to_string())
    }
}

/// Check if the embedded server is currently running.
#[frb]
pub fn netserver_is_running() -> bool {
    let server_mutex = get_server();
    server_mutex.lock().is_running()
}

/// Get the current server port (0 if not running).
#[frb]
pub fn netserver_get_port() -> u16 {
    let server_mutex = get_server();
    server_mutex.lock().bind_addr.map(|a| a.port()).unwrap_or(0)
}

/// Subscribe to server status updates.
#[frb]
pub fn netserver_status_stream(sink: StreamSink<ServerStatus>) -> Result<(), String> {
    let server_mutex = get_server();
    let server = server_mutex.lock();

    // Send initial status
    let running = server.is_running();
    let addr = server.bind_addr;
    let count = server.client_count;
    let quic_enabled = server.quic_bind_addr.is_some();
    let status = ServerStatus {
        running,
        port: addr.map(|a| a.port()).unwrap_or(0),
        client_count: count,
        bind_address: addr.map(|a| a.to_string()).unwrap_or_default(),
        quic_enabled,
        quic_bind_address: server
            .quic_bind_addr
            .map(|a| a.to_string())
            .unwrap_or_default(),
        quic_cert_sha256_fingerprint: server
            .quic_cert_sha256_fingerprint
            .clone()
            .unwrap_or_default(),
    };
    let _ = sink.add(status);

    // Store sink for future updates
    let mut sink_guard = server.status_sink.lock();
    *sink_guard = Some(sink);

    Ok(())
}

fn notify_server_status(
    sink_lock: &Arc<Mutex<Option<StreamSink<ServerStatus>>>>,
    running: bool,
    port: u16,
    client_count: u32,
    bind_address: String,
    quic_enabled: bool,
    quic_bind_address: String,
    quic_cert_sha256_fingerprint: String,
) {
    let guard = sink_lock.lock();
    if let Some(ref sink) = *guard {
        let _ = sink.add(ServerStatus {
            running,
            port,
            client_count,
            bind_address,
            quic_enabled,
            quic_bind_address,
            quic_cert_sha256_fingerprint,
        });
    }
}
