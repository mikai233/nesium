use flutter_rust_bridge::frb;

use crate::PadButton;

#[frb]
pub fn set_button(pad: u8, button: PadButton, pressed: bool) -> Result<(), String> {
    crate::runtime_handle().set_button(pad as usize, button.into(), pressed);
    Ok(())
}
