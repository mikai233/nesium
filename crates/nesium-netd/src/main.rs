use clap::Parser;
use tokio::sync::mpsc;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

use nesium_netd::run_server;

/// NES Netplay Server
#[derive(Parser, Debug)]
#[command(name = "nesium-netd")]
#[command(about = "NES emulator netplay relay server", long_about = None)]
struct Args {
    /// TCP bind address
    #[arg(short, long, default_value = "0.0.0.0:5233")]
    bind: String,

    /// Log level (trace, debug, info, warn, error)
    #[arg(short, long, default_value = "info")]
    log_level: Level,

    /// Maximum payload size in bytes
    #[arg(long, default_value = "4096")]
    max_payload: usize,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(args.log_level)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .compact()
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    // Network layer -> upper layer events.
    let (tx, rx) = mpsc::channel(1024);

    // Start TCP listener.
    let bind_addr = args.bind.parse()?;
    tokio::spawn(async move {
        let _ = nesium_netd::net::tcp::run_tcp_listener(bind_addr, tx).await;
    });

    info!("Netplay server started on {}", args.bind);
    info!("Log level: {}", args.log_level);

    // Run server loop
    run_server(rx).await
}
