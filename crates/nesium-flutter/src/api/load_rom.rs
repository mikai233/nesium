use std::path::PathBuf;

use flutter_rust_bridge::frb;

use nesium_core::reset_kind::ResetKind;

#[frb]
pub fn start_nes_runtime() -> Result<(), String> {
    let _ = crate::runtime_handle();
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

#[frb]
pub fn get_rom_hash() -> Option<Vec<u8>> {
    crate::runtime_handle().rom_hash().map(|h| h.to_vec())
}
