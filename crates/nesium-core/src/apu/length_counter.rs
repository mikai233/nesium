//! Shared length counter used by pulse, triangle, and noise channels.

use super::tables::LENGTH_TABLE;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct LengthCounter {
    value: u8,
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
            self.value = LENGTH_TABLE[index as usize];
        }
    }

    pub(super) fn clock(&mut self, halt: bool) {
        if self.value > 0 && !halt {
            self.value -= 1;
        }
    }
}
