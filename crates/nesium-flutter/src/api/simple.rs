use nesium_netd::net::quic_config::set_data_dir_override;

#[flutter_rust_bridge::frb(sync)] // Synchronous mode for simplicity of the demo
pub fn greet(name: String) -> String {
    format!("Hello, {name}!")
}

#[flutter_rust_bridge::frb(sync)]
pub fn init_app_paths(data_dir: String) {
    set_data_dir_override(std::path::PathBuf::from(data_dir));
}

#[flutter_rust_bridge::frb(init)]
pub fn init_app() {
    // Default utilities - feel free to customize
    flutter_rust_bridge::setup_default_user_utils();

    // Initialize tracing (netd/netplay, etc.).
    // VSCode Flutter "Debug Console" typically shows stdout, but may not show stderr,
    // so prefer stdout for logs here.
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

    let _ = tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stdout)
        .with_ansi(false)
        .try_init();
}
