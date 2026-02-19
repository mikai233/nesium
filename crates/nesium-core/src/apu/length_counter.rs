//! Shared length counter used by pulse, triangle, and noise channels.

use super::tables::LENGTH_TABLE;

#[cfg_attr(
    feature = "savestate-serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct LengthCounter {
    value: u8,
    halt: bool,
    pending_halt: bool,
    pending_reload: u8,
    reload_prev_value: u8,
}

impl LengthCounter {
    pub(super) fn clear(&mut self) {
        self.value = 0;
    }

    pub(super) fn active(&self) -> bool {
        self.value > 0
    }

    pub(super) fn load(&mut self, index: u8, enabled: bool) {
        if enabled {
            self.pending_reload = LENGTH_TABLE[index as usize];
            self.reload_prev_value = self.value;
        }
    }

    pub(super) fn set_halt_pending(&mut self, halt: bool) {
        self.pending_halt = halt;
    }

    pub(super) fn apply_pending_halt(&mut self) {
        if self.pending_reload != 0 {
            // A length reload written on the same cycle as a length clock only
            // takes effect when the counter did not change during that clock.
            if self.value == self.reload_prev_value {
                self.value = self.pending_reload;
            }
            self.pending_reload = 0;
        }
        self.halt = self.pending_halt;
    }

    pub(super) fn clock(&mut self) {
        if self.value > 0 && !self.halt {
            self.value -= 1;
        }
    }
}
