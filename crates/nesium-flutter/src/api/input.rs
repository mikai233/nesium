use flutter_rust_bridge::frb;

use crate::{ControlMessage, PadButton, send_command};

#[frb]
pub fn set_button(pad: u8, button: PadButton, pressed: bool) -> Result<(), String> {
    send_command(ControlMessage::SetButton {
        pad,
        button,
        pressed,
    })
}
