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
    init_logging();

    flutter_rust_bridge::setup_default_user_utils();
}

#[cfg(target_os = "android")]
fn init_logging() {
    android_logger::init_once(
        android_logger::Config::default()
            .with_max_level(log::LevelFilter::Info)
            .with_tag("nesium"),
    );
}

#[cfg(any(target_os = "ios", target_os = "macos"))]
fn init_logging() {
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::prelude::*;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(tracing_oslog::OsLogger::new(
            "io.github.mikai233.nesium",
            "main",
        ))
        .with(tracing_subscriber::fmt::layer());

    if let Err(e) = registry.try_init() {
        eprintln!("Failed to initialize tracing: {:?}", e);
    }
}

#[cfg(not(any(target_os = "android", target_os = "ios", target_os = "macos")))]
fn init_logging() {
    use tracing_subscriber::EnvFilter;
    use tracing_subscriber::prelude::*;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let registry = tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer());

    if let Err(e) = registry.try_init() {
        eprintln!("Failed to initialize tracing: {:?}", e);
    }
}
