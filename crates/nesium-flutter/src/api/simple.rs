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
    #[cfg(target_os = "android")]
    {
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(log::LevelFilter::Info)
                .with_tag("nesium"),
        );
    }

    #[cfg(not(target_os = "android"))]
    {
        use tracing_subscriber::EnvFilter;
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

        if let Err(e) = tracing_subscriber::fmt().with_env_filter(filter).try_init() {
            eprintln!("[nesium-flutter] Failed to initialize tracing: {:?}", e);
        }
    }

    // Move this AFTER tracing initialization, as it might set up its own logger
    flutter_rust_bridge::setup_default_user_utils();
}
