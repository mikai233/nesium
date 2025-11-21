//! Minimal NES controller (joypad) model.
//!
//! Implements the standard 8-button pad readable through `$4016/$4017`.

/// Button ordering follows the NES shift register bit layout (A first).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Button {
    A = 0,
    B = 1,
    Select = 2,
    Start = 3,
    Up = 4,
    Down = 5,
    Left = 6,
    Right = 7,
}

/// Serially-readable controller state with latch/strobe behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Controller {
    strobe: bool,
    latched: u8,
    state: u8,
}

impl Controller {
    pub fn new() -> Self {
        Self {
            strobe: false,
            latched: 0,
            state: 0,
        }
    }

    /// Update a button's pressed state.
    pub fn set_button(&mut self, button: Button, pressed: bool) {
        let bit = 1u8 << (button as u8);
        if pressed {
            self.state |= bit;
        } else {
            self.state &= !bit;
        }
        if self.strobe {
            self.latched = self.state;
        }
    }

    /// Writes to `$4016` strobe bit (shared for both ports).
    pub fn write_strobe(&mut self, data: u8) {
        let strobe = (data & 0x01) != 0;
        self.strobe = strobe;
        if strobe {
            self.latched = self.state;
        }
    }

    /// Reads the next bit from the latched shift register.
    ///
    /// Bit 0 holds the current button; subsequent reads shift unless strobe is held high.
    pub fn read(&mut self) -> u8 {
        let bit = self.latched & 0x01;
        if !self.strobe {
            // After 8 reads hardware keeps returning 1s; simulate by shifting in ones.
            self.latched = (self.latched >> 1) | 0x80;
        }
        bit | 0x40 // Upper bits float high on hardware; keep them set.
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}
