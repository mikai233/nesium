use super::savestate::PpuOpenBusState;
/// PPU-local open-bus latch with per-bit decay.
///
/// Mirrors Mesen2's `_openBus` / `_openBusDecayStamp` behaviour, using the
/// PPU frame counter as the decay time base. The interface is modeled after
/// NesPpu::SetOpenBus / NesPpu::ApplyOpenBus.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct PpuOpenBus {
    value: u8,
    decay_stamp: [u32; 8],
}

impl PpuOpenBus {
    pub(crate) fn new() -> Self {
        Self {
            value: 0,
            decay_stamp: [0; 8],
        }
    }

    pub(crate) fn reset(&mut self) {
        self.value = 0;
        self.decay_stamp = [0; 8];
    }

    /// Equivalent to Mesen2's `SetOpenBus(mask, value)`.
    pub(crate) fn set(&mut self, mut mask: u8, mut value: u8, frame: u32) {
        // Fast path: full overwrite of the bus.
        if mask == 0xFF {
            self.value = value;
            for stamp in &mut self.decay_stamp {
                *stamp = frame;
            }
            return;
        }

        // Same rolling 16-bit trick as Mesen: shift the current value into
        // the high byte and rotate it down as we walk the bits.
        let mut open_bus: u16 = (self.value as u16) << 8;

        for bit in 0..8 {
            // Shift one bit down so the new bit enters at bit 7.
            open_bus >>= 1;

            if (mask & 0x01) != 0 {
                // This bit is actively driven by `value`.
                if (value & 0x01) != 0 {
                    open_bus |= 0x80;
                } else {
                    open_bus &= 0xFF7F;
                }
                self.decay_stamp[bit] = frame;
            } else {
                // This bit is coming from the existing open-bus state; if
                // it hasn't been refreshed for more than ~3 frames, it
                // decays to 0.
                if frame.wrapping_sub(self.decay_stamp[bit]) > 3 {
                    open_bus &= 0xFF7F;
                }
            }

            value >>= 1;
            mask >>= 1;
        }

        self.value = open_bus as u8;
    }

    /// Equivalent to Mesen2's `ApplyOpenBus(mask, value)`.
    ///
    /// This updates the decay stamps (via `set(!mask, value)`) and then
    /// returns a value whose `mask`ed bits are taken from the open-bus
    /// latch and the rest from `value`.
    pub(crate) fn apply(&mut self, mask: u8, value: u8, frame: u32) -> u8 {
        self.set(!mask, value, frame);
        value | (self.value & mask)
    }

    /// Returns the latched open-bus value without affecting decay.
    pub(crate) fn sample(&self) -> u8 {
        self.value
    }

    pub(crate) fn save_state(&self) -> PpuOpenBusState {
        PpuOpenBusState {
            value: self.value,
            decay_stamp: self.decay_stamp,
        }
    }

    pub(crate) fn load_state(&mut self, state: PpuOpenBusState) {
        self.value = state.value;
        self.decay_stamp = state.decay_stamp;
    }
}
