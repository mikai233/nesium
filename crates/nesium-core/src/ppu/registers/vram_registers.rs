use super::VramAddr;

/// Internal VRAM register block matching the NESDev `v/t/x/w` terminology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) struct VramRegisters {
    /// Current VRAM address (`v`).
    pub(crate) v: VramAddr,
    /// Temporary VRAM address (`t`).
    pub(crate) t: VramAddr,
    /// Fine X scroll component (`x`, 0..7).
    pub(crate) x: u8,
    /// Write toggle (`w`): false => first write, true => second write.
    pub(crate) w: bool,
}

impl VramRegisters {
    /// Writes to `$2005` (PPUSCROLL), updating coarse X/Y and fine X/Y.
    pub(crate) fn write_scroll(&mut self, value: u8) {
        if !self.w {
            self.t.set_coarse_x(value >> 3);
            self.x = value & 0b111;
        } else {
            self.t.set_coarse_y(value >> 3);
            self.t.set_fine_y(value & 0b111);
        }
        self.w = !self.w;
    }

    /// Writes to `$2006` (PPUADDR), updating `t`. On the second write,
    /// returns the completed address so the caller can commit `v` after
    /// emulating hardware delay.
    pub(crate) fn write_addr(&mut self, value: u8) -> Option<VramAddr> {
        let second_write = self.w;
        if !second_write {
            let hi = u16::from(value & 0b0011_1111) << 8;
            let lo = self.t.raw() & 0x00FF;
            self.t.set_raw(hi | lo);
        } else {
            let hi = self.t.raw() & 0x7F00;
            self.t.set_raw(hi | u16::from(value));
        }
        self.w = !self.w;
        second_write.then_some(self.t)
    }

    /// Resets the write toggle so the next `$2005/$2006` write is treated
    /// as the first half of the pair.
    pub(crate) fn reset_latch(&mut self) {
        self.w = false;
    }
}
