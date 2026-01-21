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

    // Initialize tracing to see logs in console
    let _ = tracing_subscriber::fmt::try_init();
}
