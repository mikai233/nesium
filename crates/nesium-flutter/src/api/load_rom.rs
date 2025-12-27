use std::path::PathBuf;

use flutter_rust_bridge::frb;

use nesium_core::reset_kind::ResetKind;

use crate::ensure_runtime;

#[frb]
pub fn start_nes_runtime() -> Result<(), String> {
    ensure_runtime();
    Ok(())
}

#[frb]
pub fn load_rom(path: String) -> Result<(), String> {
    crate::runtime_handle()
        .load_rom(PathBuf::from(path))
        .map_err(|e| e.to_string())
}

#[frb]
pub fn reset_console() -> Result<(), String> {
    crate::runtime_handle()
        .reset(ResetKind::Soft)
        .map_err(|e| e.to_string())
}

#[frb]
pub fn power_reset_console() -> Result<(), String> {
    crate::runtime_handle()
        .reset(ResetKind::PowerOn)
        .map_err(|e| e.to_string())
}

#[frb]
pub fn eject_console() -> Result<(), String> {
    crate::runtime_handle().eject().map_err(|e| e.to_string())
}
