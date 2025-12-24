use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[command(name = "nesium-netd")]
pub struct Config {
    /// UDP relay bind address, e.g. 0.0.0.0:3456
    #[arg(long, default_value = "0.0.0.0:3456")]
    pub udp_bind: String,

    /// Drop peers that haven't sent anything for N seconds
    #[arg(long, default_value_t = 20)]
    pub peer_ttl_secs: u64,

    /// Cleanup interval in seconds
    #[arg(long, default_value_t = 5)]
    pub cleanup_interval_secs: u64,

    /// (optional) HTTP signaling bind address, e.g. 0.0.0.0:8080
    #[cfg(feature = "http")]
    #[arg(long, default_value = "0.0.0.0:8080")]
    pub http_bind: String,
}
