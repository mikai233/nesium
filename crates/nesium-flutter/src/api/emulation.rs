use flutter_rust_bridge::frb;

#[frb]
pub fn set_integer_fps_mode(enabled: bool) -> Result<(), String> {
    crate::runtime_handle()
        .set_integer_fps_target(if enabled { Some(60) } else { None })
        .map_err(|e| e.to_string())?;
    Ok(())
}
