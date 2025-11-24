//! CPU/PPU data bus open-bus model (Mesen2-style).
//!
//! The 2A03/2C02 data bus floats when no device is actively driving it. Reads
//! from write-only or unmapped addresses therefore return whatever value was
//! last on the bus, with charged bits slowly decaying back to 0 over time.
//! Mesen2 models this by tracking the last driven byte and letting bits leak to
//! zero after a few frames; we mirror that behaviour here with a per-bit decay
//! stamp and mask-aware updates (`SetOpenBus`/`ApplyOpenBus`).

/// Number of bus `step()` calls before a driven `1` bit decays to `0`.
///
/// For NTSC, a frame is ~29_780 CPU cycles. Using ~90k steps gives decay on
/// the order of ~3 frames (Mesen2 uses "3 frames" as a conservative upper
/// bound for individual bit decay).
const DECAY_TICKS: u64 = 90_000;

/// Tracks the last value driven on the data bus and per-bit decay.
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub(crate) struct OpenBus {
    value: u8,
    decay_deadline: [u64; 8],
    tick: u64,
}

impl OpenBus {
    pub(crate) fn new() -> Self {
        Self {
            value: 0,
            decay_deadline: [0; 8],
            tick: 0,
        }
    }

    /// Resets the open-bus state to its power-on value.
    pub(crate) fn reset(&mut self) {
        self.value = 0;
        self.decay_deadline = [0; 8];
        self.tick = 0;
    }

    /// Advances the internal clock by one CPU bus step and applies any pending
    /// bit decays. Call this once per CPU memory access to approximate real
    /// elapsed time on the bus.
    pub(crate) fn step(&mut self) {
        self.tick = self.tick.wrapping_add(1);
        self.apply_decay();
    }

    /// Returns the current bus value after applying decay without refreshing
    /// the decay timer (mirrors reads from floating addresses).
    pub(crate) fn sample(&mut self) -> u8 {
        self.apply_decay();
        self.value
    }

    /// Latches a freshly driven value onto the bus and refreshes per-bit decay
    /// timers (equivalent to Mesen2's `SetOpenBus(0xFF, value)`).
    pub(crate) fn latch(&mut self, value: u8) {
        self.set_masked(0xFF, value);
    }

    /// Mesen2-style `SetOpenBus(mask, value)`:
    /// - When `mask == 0xFF`, sets all 8 bits to `value` and refreshes stamps.
    /// - Otherwise, updates only bits selected by `mask`, shifting `value`/`mask`
    ///   as documented on NESdev and in Mesen2.
    pub(crate) fn set_masked(&mut self, mut mask: u8, mut value: u8) {
        if mask == 0xFF {
            self.value = value;
            let decay_at = self.tick.wrapping_add(DECAY_TICKS);
            for deadline in &mut self.decay_deadline {
                *deadline = decay_at;
            }
            return;
        }

        let mut open_bus = (self.value as u16) << 8;
        for (_bit, deadline) in self.decay_deadline.iter_mut().enumerate() {
            open_bus >>= 1;
            if mask & 0x01 != 0 {
                if value & 0x01 != 0 {
                    open_bus |= 0x80;
                } else {
                    open_bus &= 0xFF7F;
                }
                *deadline = self.tick.wrapping_add(DECAY_TICKS);
            } else if *deadline != 0 && self.tick.wrapping_sub(*deadline) > DECAY_TICKS {
                // When this bit hasn't been refreshed in a while, decay it to 0.
                open_bus &= 0xFF7F;
            }
            value >>= 1;
            mask >>= 1;
        }

        self.value = open_bus as u8;
    }

    /// Mesen2-style `ApplyOpenBus(mask, value)` helper:
    /// combines a freshly driven value with floating-bus bits.
    pub(crate) fn apply_masked(&mut self, mask: u8, value: u8) -> u8 {
        // Bits *not* in `mask` come from `value` and refresh decay.
        self.set_masked(!mask, value);
        value | (self.value & mask)
    }

    fn apply_decay(&mut self) {
        for (bit, deadline) in self.decay_deadline.iter_mut().enumerate() {
            if *deadline != 0 && self.tick >= *deadline {
                self.value &= !(1 << bit);
                *deadline = 0;
            }
        }
    }
}
