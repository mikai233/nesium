//! CPU data bus open-bus model (Mesen2-style).
//!
//! The 2A03 data bus floats when no device is actively driving it. Reads from
//! write-only/unmapped addresses therefore return whatever value was last on
//! the bus, with charged bits slowly decaying back to 0 over time. Mesen2
//! models this by tracking the last driven byte and letting bits leak to zero
//! after roughly one second of CPU time; we mirror that behaviour here.

/// Number of CPU bus steps before a driven `1` bit decays to `0`.
///
/// NTSC CPU frequency is ~1.79 MHz, so this is roughly one second of elapsed
/// CPU time as used by Mesen2's open-bus model.
const DECAY_TICKS: u64 = 1_789_000;

/// Tracks the last value driven on the CPU data bus and per-bit decay.
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
    /// timers.
    pub(crate) fn latch(&mut self, value: u8) {
        self.value = value;
        let decay_at = self.tick.wrapping_add(DECAY_TICKS);
        for (bit, deadline) in self.decay_deadline.iter_mut().enumerate() {
            *deadline = if value & (1 << bit) != 0 { decay_at } else { 0 };
        }
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
