//! Minimal NES controller (joypad) model.
//!
//! Implements the standard 8-button pad readable through `$4016/$4017`.

use crate::mem_block::MemBlock;

/// Button ordering follows the NES shift register bit layout (A first).
#[cfg_attr(
    feature = "savestate-serde",
    derive(serde::Serialize, serde::Deserialize)
)]
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
#[cfg_attr(
    feature = "savestate-serde",
    derive(serde::Serialize, serde::Deserialize)
)]
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

    pub fn set_state(&mut self, state: u8) {
        self.state = state;
        if self.strobe {
            self.latched = self.state;
        }
    }

    pub fn state(&self) -> u8 {
        self.state
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
        // Return only the serial data bit; bus-level open-bus bits are merged by CpuBus.
        bit
    }
}

impl Default for Controller {
    fn default() -> Self {
        Self::new()
    }
}

/// Two NES controller ports backed by a `MemBlock`, enabling boxed or stack
/// allocation depending on the active feature set.
pub type ControllerPorts = MemBlock<Controller, 2>;

/// Captures the serial stream some blargg test ROMs emit via `$4016` writes.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SerialLogger {
    state: SerialState,
    buffer: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
enum SerialState {
    #[default]
    Idle,
    Data {
        byte: u8,
        bit: u8,
    },
    Stop {
        byte: u8,
    },
}

impl SerialLogger {
    pub(crate) fn push_bit(&mut self, bit: bool) {
        use SerialState::*;
        self.state = match (self.state, bit) {
            // Waiting for start bit (0).
            (Idle, false) => Data { byte: 0, bit: 0 },
            (Idle, true) => Idle,

            // Collect 8 data bits, LSB first.
            (Data { mut byte, mut bit }, b) => {
                if b {
                    byte |= 1 << bit;
                }
                bit += 1;
                if bit >= 8 {
                    Stop { byte }
                } else {
                    Data { byte, bit }
                }
            }

            // Consume stop bit of value 1.
            (Stop { byte }, true) => {
                self.buffer.push(byte);
                Idle
            }

            // Framing error: reset state machine.
            (Stop { .. }, false) => Idle,
        };
    }

    pub(crate) fn drain(&mut self) -> Vec<u8> {
        let mut out = Vec::new();
        std::mem::swap(&mut self.buffer, &mut out);
        out
    }
}
