//! Embedded Netplay Server API for Flutter.
//!
//! Allows starting/stopping a netplay server directly within the app.

use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

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
}

#[frb(ignore)]
struct EmbeddedServer {
    /// Sender to broadcast shutdown signal.
    shutdown_tx: Option<tokio::sync::watch::Sender<bool>>,
    /// Current bind address.
    bind_addr: Option<SocketAddr>,
    /// Status stream sink.
    status_sink: Arc<Mutex<Option<StreamSink<ServerStatus>>>>,
    /// Number of connected clients.
    client_count: u32,
    /// Task handles to abort on shutdown.
    task_handles: Vec<JoinHandle<()>>,
}

impl EmbeddedServer {
    fn new() -> Self {
        Self {
            shutdown_tx: None,
            bind_addr: None,
            status_sink: Arc::new(Mutex::new(None)),
            client_count: 0,
            task_handles: Vec::new(),
        }
    }

    fn is_running(&self) -> bool {
        self.shutdown_tx.is_some()
    }
}

static SERVER: OnceLock<Mutex<EmbeddedServer>> = OnceLock::new();
static NEXT_CONN_ID: AtomicU64 = AtomicU64::new(1);

fn get_server() -> &'static Mutex<EmbeddedServer> {
    SERVER.get_or_init(|| Mutex::new(EmbeddedServer::new()))
}

/// Start the embedded netplay server on the specified port.
///
/// Pass `port = 0` to let the OS pick an available port.
#[frb]
pub async fn netserver_start(port: u16) -> Result<u16, String> {
    let server_mutex = get_server();
    {
        let server = server_mutex.lock().map_err(|e| e.to_string())?;
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

    // Create shutdown broadcast channel
    let (shutdown_tx, mut shutdown_rx_listener) = tokio::sync::watch::channel(false);

    // Create channels for server communication
    let (event_tx, mut event_rx) = mpsc::channel(1024);
    let (server_tx, server_rx) = mpsc::channel(1024);

    let mut server = server_mutex.lock().map_err(|e| e.to_string())?;
    server.client_count = 0;
    server.task_handles.clear();
    let status_sink = server.status_sink.clone();

    // Spawn the TCP listener task
    let shutdown_tx_clone = shutdown_tx.clone();
    let listener_handle = tokio::spawn(async move {
        loop {
            tokio::select! {
                result = listener.accept() => {
                    match result {
                        Ok((stream, peer)) => {
                            let conn_id = NEXT_CONN_ID.fetch_add(1, Ordering::Relaxed);
                            let tx_clone = event_tx.clone();
                            let mut shutdown_rx_conn = shutdown_tx_clone.subscribe();
                            tokio::spawn(async move {
                                tokio::select! {
                                    _ = nesium_netd::net::tcp::handle_tcp_connection(
                                        stream, peer, conn_id, tx_clone
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

    // Spawn monitoring task to track client count
    let status_sink_clone = status_sink.clone();
    let bind_address = actual_addr.to_string();
    let mut shutdown_rx_monitor = shutdown_tx.subscribe();
    let monitor_handle = tokio::spawn(async move {
        use nesium_netd::net::inbound::InboundEvent;

        loop {
            tokio::select! {
                ev_opt = event_rx.recv() => {
                    let Some(ev) = ev_opt else { break; };
                    match ev {
                        InboundEvent::Connected { .. } => {
                            let mut s = get_server().lock().unwrap();
                            s.client_count += 1;
                            let count = s.client_count;
                            notify_server_status(&status_sink_clone, true, actual_port, count, bind_address.clone());
                        }
                        InboundEvent::Disconnected { .. } => {
                            let mut s = get_server().lock().unwrap();
                            s.client_count = s.client_count.saturating_sub(1);
                            let count = s.client_count;
                            notify_server_status(&status_sink_clone, true, actual_port, count, bind_address.clone());
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
        let _ = nesium_netd::run_server(server_rx).await;
    });
    server.task_handles.push(server_handle);

    server.shutdown_tx = Some(shutdown_tx);
    server.bind_addr = Some(actual_addr);

    // Notify status update
    notify_server_status(&status_sink, true, actual_port, 0, actual_addr.to_string());

    tracing::info!("Embedded server started on {}", actual_addr);

    Ok(actual_port)
}

/// Stop the embedded netplay server.
#[frb]
pub async fn netserver_stop() -> Result<(), String> {
    let server_mutex = get_server();
    let (tx, task_handles, status_sink) = {
        let mut server = server_mutex.lock().map_err(|e| e.to_string())?;
        let tx = server.shutdown_tx.take();
        let handles = std::mem::take(&mut server.task_handles);
        if tx.is_some() {
            server.bind_addr = None;
            server.client_count = 0;
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

        // Notify status update
        notify_server_status(&status_sink, false, 0, 0, String::new());

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
    server_mutex.lock().map(|s| s.is_running()).unwrap_or(false)
}

/// Get the current server port (0 if not running).
#[frb]
pub fn netserver_get_port() -> u16 {
    let server_mutex = get_server();
    server_mutex
        .lock()
        .ok()
        .and_then(|s| s.bind_addr.map(|a| a.port()))
        .unwrap_or(0)
}

/// Subscribe to server status updates.
#[frb]
pub fn netserver_status_stream(sink: StreamSink<ServerStatus>) -> Result<(), String> {
    let server_mutex = get_server();
    let server = server_mutex.lock().map_err(|e| e.to_string())?;

    // Send initial status
    let running = server.is_running();
    let addr = server.bind_addr;
    let count = server.client_count;
    let status = ServerStatus {
        running,
        port: addr.map(|a| a.port()).unwrap_or(0),
        client_count: count,
        bind_address: addr.map(|a| a.to_string()).unwrap_or_default(),
    };
    let _ = sink.add(status);

    // Store sink for future updates
    if let Ok(mut sink_guard) = server.status_sink.lock() {
        *sink_guard = Some(sink);
    }

    Ok(())
}

fn notify_server_status(
    sink_lock: &Arc<Mutex<Option<StreamSink<ServerStatus>>>>,
    running: bool,
    port: u16,
    client_count: u32,
    bind_address: String,
) {
    if let Ok(guard) = sink_lock.lock() {
        if let Some(ref sink) = *guard {
            let _ = sink.add(ServerStatus {
                running,
                port,
                client_count,
                bind_address,
            });
        }
    }
}
