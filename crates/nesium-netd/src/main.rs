use clap::Parser;
use tokio::sync::mpsc;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

use nesium_netd::net::quic_config;
use nesium_netd::run_server;
use std::net::SocketAddr;
use std::path::PathBuf;

/// NES Netplay Server
#[derive(Parser, Debug)]
#[command(name = "nesium-netd")]
#[command(about = "NES emulator netplay relay server", long_about = None)]
struct Args {
    /// TCP bind address
    #[arg(short, long, default_value = "0.0.0.0:5233")]
    bind: String,

    /// Enable QUIC listener (UDP) alongside TCP.
    ///
    /// Default is enabled for convenient LAN testing. Disable with `--enable-quic=false`.
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
    enable_quic: bool,

    /// QUIC bind address (defaults to `--bind` when omitted)
    #[arg(long)]
    quic_bind: Option<String>,

    /// QUIC TLS certificate (PEM)
    #[arg(long)]
    quic_cert: Option<PathBuf>,

    /// QUIC TLS private key (PEM)
    #[arg(long)]
    quic_key: Option<PathBuf>,

    /// Directory used to store auto-generated QUIC cert/key (when `--quic-bind` is set and
    /// `--quic-cert/--quic-key` are not provided).
    #[arg(long)]
    quic_data_dir: Option<PathBuf>,

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
    let bind_addr: SocketAddr = args.bind.parse()?;
    let tx_tcp = tx.clone();
    tokio::spawn(async move {
        let _ = nesium_netd::net::tcp::run_tcp_listener(bind_addr, tx_tcp).await;
    });

    // Start QUIC listener (optional).
    if args.enable_quic {
        let quic_bind = args.quic_bind.clone().unwrap_or_else(|| args.bind.clone());
        let quic_addr: SocketAddr = quic_bind.parse()?;

        let (cert, key, auto_generated) = match (&args.quic_cert, &args.quic_key) {
            (Some(cert), Some(key)) => (cert.clone(), key.clone(), false),
            (None, None) => {
                let dir = args
                    .quic_data_dir
                    .clone()
                    .unwrap_or_else(|| quic_config::default_quic_data_dir("nesium-netd"));
                let (cert, key) = quic_config::ensure_quic_cert_pair(&dir)?;
                (cert, key, true)
            }
            _ => {
                anyhow::bail!(
                    "Must provide both --quic-cert and --quic-key, or neither (auto-generate)"
                );
            }
        };

        let server_config = quic_config::build_quic_server_config(&cert, &key)?;

        let tx_quic = tx.clone();
        tokio::spawn(async move {
            let _ =
                nesium_netd::net::quic::run_quic_listener(quic_addr, server_config, tx_quic).await;
        });

        info!("QUIC enabled on {}", quic_bind);
        if auto_generated {
            info!(
                "QUIC cert/key auto-generated at {} and {}",
                cert.display(),
                key.display()
            );
        }
        if let Ok(fp) = quic_config::sha256_fingerprint_base64url_from_pem(&cert) {
            info!("QUIC cert SHA-256 fingerprint (base64url): {}", fp);
            info!("Use this with pinned client connect (netplay_connect_*_pinned).");
        }
    }

    info!("Netplay server started on {}", args.bind);
    info!("Log level: {}", args.log_level);

    // Run server loop
    run_server(rx).await
}
