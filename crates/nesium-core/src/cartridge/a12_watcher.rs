//! A12 watcher for MMC3-style IRQ clocking.
//!
//! MMC3-family mappers clock their IRQ counter on **rising edges** of PPU address line A12
//! (bit 12 of the PPU address bus). To avoid counting rapid toggles during pattern fetches,
//! the hardware effectively requires A12 to stay low for a short minimum time before a
//! rising edge is considered valid.
//!
//! This helper implements that behavior by:
//! - tracking how long A12 has been held low (`cycles_down`)
//! - detecting low->high transitions (rising edges)
//! - reporting a `Rise` only when the low time exceeded a configurable threshold
//! - handling `frame_cycle` wrap-around by treating it as a cycle counter that resets each frame

/// A12 transition reported by [`A12Watcher`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum A12StateChange {
    /// No relevant edge detected.
    None,
    /// A12 transitioned from low to high and satisfied the debounce requirement.
    Rise,
    /// A12 transitioned from high to low.
    Fall,
}

/// Watches PPU A12 (address bit 12, mask `0x1000`) and reports debounced transitions.
///
/// ## How the algorithm works
/// - `cycles_down == 0` means A12 is currently considered high (or we are not tracking low time).
/// - When A12 is observed low and `cycles_down == 0`, we start low-time tracking by setting
///   `cycles_down = 1` and return [`A12StateChange::Fall`].
/// - While A12 stays low, we accumulate elapsed time based on `frame_cycle` deltas.
/// - When A12 is observed high again, we return [`A12StateChange::Rise`] iff the accumulated
///   low time is **strictly greater than** `MIN_DELAY`. Then we reset `cycles_down` to 0.
///
/// ## Cycle units and wrap-around
/// `frame_cycle` is expected to be a cycle counter that is monotonic within a frame and wraps
/// back to 0 at the start of the next frame. `frame_len` is the number of cycles per frame in
/// the same units. Wrap-around is handled by adding `(frame_len - last_cycle) + frame_cycle`.
#[derive(Debug, Clone)]
pub struct A12Watcher {
    last_cycle: u32,
    cycles_down: u32,
    frame_len: u32,
}

impl Default for A12Watcher {
    fn default() -> Self {
        // Default to NTSC master cycles per frame.
        Self::new(89_342)
    }
}

impl A12Watcher {
    /// Create a new watcher.
    ///
    /// `frame_len` is the number of cycles in one frame for the `frame_cycle` unit you will
    /// pass to [`update_vram_address`]. For example:
    /// - if `frame_cycle` is measured in master cycles on NTSC, use 89342
    /// - if `frame_cycle` is measured in PPU cycles, use your PPU cycles-per-frame value
    #[inline]
    pub const fn new(frame_len: u32) -> Self {
        Self {
            last_cycle: 0,
            cycles_down: 0,
            frame_len,
        }
    }

    /// Reset internal edge/low-time tracking.
    #[inline]
    pub fn reset(&mut self) {
        self.last_cycle = 0;
        self.cycles_down = 0;
    }

    /// Observe a new PPU address and update edge/low-time tracking.
    ///
    /// Returns:
    /// - [`A12StateChange::Fall`] when A12 is first observed low (starts low-time tracking)
    /// - [`A12StateChange::Rise`] when A12 is observed high after being low long enough
    /// - [`A12StateChange::None`] otherwise
    ///
    /// `MIN_DELAY` is the minimum low-time (in `frame_cycle` units) required before a rising
    /// edge is considered valid. The comparison is **strictly greater than** (`> MIN_DELAY`).
    ///
    /// Notes:
    /// - `addr` should be the current PPU bus address; A12 is tested via `addr & 0x1000`.
    /// - `frame_cycle` must be monotonic within a frame and wrap back to 0 each new frame.
    #[inline]
    pub fn update_vram_address<const MIN_DELAY: u32>(
        &mut self,
        addr: u16,
        frame_cycle: u32,
    ) -> A12StateChange {
        let mut result = A12StateChange::None;

        // Accumulate time A12 has been low.
        if self.cycles_down > 0 {
            if self.last_cycle > frame_cycle {
                // Wrapped to a new frame.
                self.cycles_down = self
                    .cycles_down
                    .saturating_add(self.frame_len.saturating_sub(self.last_cycle))
                    .saturating_add(frame_cycle);
            } else {
                self.cycles_down = self
                    .cycles_down
                    .saturating_add(frame_cycle.saturating_sub(self.last_cycle));
            }
        }

        // PPU A12 is bit 12 of the PPU address bus (mask 0x1000).
        // A12=0 for $0000-$0FFF, A12=1 for $1000-$1FFF.
        let a12_high = (addr & 0x1000) != 0;

        if !a12_high {
            // A12 is low.
            if self.cycles_down == 0 {
                // First observation of low; start counting.
                self.cycles_down = 1;
                result = A12StateChange::Fall;
            }
        } else {
            // A12 is high.
            if self.cycles_down > MIN_DELAY {
                result = A12StateChange::Rise;
            }
            // Reset the low counter once A12 is high.
            self.cycles_down = 0;
        }

        self.last_cycle = frame_cycle;
        result
    }

    /// Convenience wrapper using the common MMC3 debounce threshold (`MIN_DELAY = 10`).
    #[inline]
    pub fn update(&mut self, addr: u16, frame_cycle: u32) -> A12StateChange {
        self.update_vram_address::<10>(addr, frame_cycle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_fall_and_rise_with_delay() {
        let mut w = A12Watcher::new(100);

        // Start high.
        assert_eq!(w.update_vram_address::<10>(0x1000, 0), A12StateChange::None);

        // Go low -> fall.
        assert_eq!(w.update_vram_address::<10>(0x0000, 1), A12StateChange::Fall);

        // Stay low for <= MIN_DELAY.
        assert_eq!(w.update_vram_address::<10>(0x0000, 5), A12StateChange::None);

        // Go high, but low time is 1(start) + (5-1)=5, not >10 => no rise.
        assert_eq!(w.update_vram_address::<10>(0x1000, 6), A12StateChange::None);

        // Low again.
        assert_eq!(
            w.update_vram_address::<10>(0x0000, 10),
            A12StateChange::Fall
        );
        // Low long enough.
        assert_eq!(
            w.update_vram_address::<10>(0x0000, 25),
            A12StateChange::None
        );
        // Rise accepted.
        assert_eq!(
            w.update_vram_address::<10>(0x1000, 26),
            A12StateChange::Rise
        );
    }

    #[test]
    fn accounts_for_frame_wrap() {
        let mut w = A12Watcher::new(100);

        // Go low near end of frame.
        assert_eq!(
            w.update_vram_address::<10>(0x0000, 90),
            A12StateChange::Fall
        );
        // Next frame cycle wraps.
        // cycles_down accumulates (100-90)+5 = 15 plus initial 1 => >10, so rise.
        assert_eq!(w.update_vram_address::<10>(0x1000, 5), A12StateChange::Rise);
    }
}
