use std::path::PathBuf;

use flutter_rust_bridge::frb;

use crate::{ControlMessage, send_command};

#[frb]
pub fn start_nes_runtime() -> Result<(), String> {
    let _ = crate::start_thread_if_needed();
    Ok(())
}

#[frb]
pub fn load_rom(path: String) -> Result<(), String> {
    send_command(ControlMessage::LoadRom(PathBuf::from(path)))
}

#[frb]
pub fn reset_console() -> Result<(), String> {
    send_command(ControlMessage::Reset)
}
