use flutter_rust_bridge::frb;

#[frb]
pub fn set_integer_fps_mode(enabled: bool) -> Result<(), String> {
    crate::runtime_handle()
        .set_integer_fps_target(if enabled { Some(60) } else { None })
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[frb]
pub fn save_state(path: String) -> Result<(), String> {
    crate::runtime_handle()
        .save_state(path)
        .map_err(|e| e.to_string())
}

#[frb]
pub fn load_state(path: String) -> Result<(), String> {
    crate::runtime_handle()
        .load_state(path)
        .map_err(|e| e.to_string())
}

#[frb]
pub async fn save_state_to_memory() -> Result<Vec<u8>, String> {
    crate::runtime_handle()
        .save_state_to_memory()
        .map_err(|e| e.to_string())
}

#[frb]
pub async fn load_state_from_memory(data: Vec<u8>) -> Result<(), String> {
    crate::runtime_handle()
        .load_state_from_memory(data)
        .map_err(|e| e.to_string())
}

#[frb]
pub fn set_rewind_config(enabled: bool, capacity: u64) -> Result<(), String> {
    crate::runtime_handle().set_rewind_config(enabled, capacity);
    Ok(())
}

#[frb]
pub fn set_rewinding(rewinding: bool) -> Result<(), String> {
    crate::runtime_handle()
        .set_rewinding(rewinding)
        .map_err(|e| e.to_string())
}

#[frb]
pub fn set_fast_forwarding(fast_forwarding: bool) -> Result<(), String> {
    crate::runtime_handle()
        .set_fast_forwarding(fast_forwarding)
        .map_err(|e| e.to_string())
}

#[frb]
pub fn set_fast_forward_speed(speed_percent: u16) -> Result<(), String> {
    crate::runtime_handle()
        .set_fast_forward_speed(speed_percent)
        .map_err(|e| e.to_string())
}

#[frb]
pub fn set_rewind_speed(speed_percent: u16) -> Result<(), String> {
    crate::runtime_handle()
        .set_rewind_speed(speed_percent)
        .map_err(|e| e.to_string())
}

#[frb]
pub fn load_tas_movie(data: String) -> Result<(), String> {
    let movie = nesium_support::tas::fm2::parse_str(&data).map_err(|e| e.to_string())?;
    crate::runtime_handle()
        .load_movie(movie)
        .map_err(|e| e.to_string())
}
