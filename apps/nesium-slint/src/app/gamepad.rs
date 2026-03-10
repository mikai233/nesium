use gilrs::{Button as GilrsButton, Event, GamepadId, Gilrs};

/// Thin wrapper around `gilrs` that keeps gamepad state fresh and provides
/// helpers for querying connected pads.
pub struct GamepadManager {
    gilrs: Gilrs,
}

impl GamepadManager {
    pub fn new() -> Option<Self> {
        Gilrs::new().ok().map(|gilrs| Self { gilrs })
    }

    /// Pump the event queue so button state stays up to date.
    pub fn poll(&mut self) {
        while let Some(Event { .. }) = self.gilrs.next_event() {
            // We don't need individual events at the moment; polling keeps
            // the internal state (is_pressed, axes, etc.) current.
        }
    }

    /// Returns a snapshot of connected gamepads and their display names.
    pub fn gamepads(&self) -> Vec<(GamepadId, String)> {
        self.gilrs
            .gamepads()
            .map(|(id, gp)| (id, gp.name().to_owned()))
            .collect()
    }

    /// Returns `true` if the given gamepad button is currently pressed.
    pub fn is_pressed(&self, id: GamepadId, button: GilrsButton) -> bool {
        self.gilrs
            .connected_gamepad(id)
            .map(|gp| gp.is_pressed(button))
            .unwrap_or(false)
    }
}
