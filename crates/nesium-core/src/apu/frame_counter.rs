//! Frame sequencer responsible for clocking envelopes, length counters, and
//! sweep units at quarter- and half-frame intervals.

/// Frame sequencer timing mode.
#[cfg_attr(
    feature = "savestate-serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FrameCounterMode {
    #[default]
    FourStep,
    FiveStep,
}

/// Internal frame counter state.
#[cfg_attr(
    feature = "savestate-serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct FrameCounter {
    mode: FrameCounterMode,
    irq_inhibit: bool,
    /// Half-CPU-cycle counter (increments twice per CPU cycle).
    half_cycle: u64,
    /// Pending reconfiguration delay (3–4 CPU cycles after a `$4017` write).
    reset_delay_half: u8,
    /// Optional bootstrap offset applied when a reset reconfiguration completes.
    reset_headstart_half: u16,
    pending_mode: FrameCounterMode,
    pending_irq_inhibit: bool,
}

/// Indicates which frame units should be clocked after a frame counter tick.
#[cfg_attr(
    feature = "savestate-serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct FrameTick {
    pub(super) quarter: bool,
    pub(super) half: bool,
    pub(super) frame_irq: bool,
}

/// Frame sequencer reset side effects applied after writing `$4017`.
#[cfg_attr(
    feature = "savestate-serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct FrameResetAction {
    pub(super) immediate_quarter: bool,
    pub(super) immediate_half: bool,
}

/// Hardware latency for the implicit `$4017` write performed on power-on and
/// warm resets. Measurements (and Mesen2's model) apply the write effects 3
/// CPU cycles after reset.
const RESET_WRITE_DELAY_CYCLES: u8 = 3;
/// Frame counter head-start applied on reset so the implicit `$4017` write
/// effectively predates the first executed instruction by roughly 9–12 CPU
/// cycles. Expressed in half-cycles.
const RESET_HEADSTART_HALF_CYCLES: u16 = 0;

/// Frame sequencer timeline for 4-step mode: (CPU cycle, quarter, half, irq).
///
/// The values and period mirror Mesen2's NTSC `_stepCyclesNtsc[0]` table
/// (derived from NESdev), expressed directly in CPU cycles.
///
/// We store timings in half-CPU-cycles to model the documented 0.5-cycle
/// alignment of frame ticks (e.g., 7457.5 CPU cycles becomes 14915 half-cycles).
pub(super) const FRAME_STEP_4: &[(u32, bool, bool, bool)] = &[
    (14915, true, false, false), // 7457.5
    (29827, true, true, false),  // 14913.5
    (44743, true, false, false), // 22371.5
    (59659, true, true, true),   // 29829.5
];
pub(super) const FRAME_STEP_4_PERIOD: u32 = 59660;

/// Frame sequencer timeline for 5-step mode: (CPU cycle, quarter, half, irq).
///
/// The 5-step mode never generates frame IRQs; this table again mirrors the
/// NTSC timings from Mesen2's `_stepCyclesNtsc[1]`.
pub(super) const FRAME_STEP_5: &[(u32, bool, bool, bool)] = &[
    (14915, true, false, false),  // 7457.5
    (29827, true, true, false),   // 14913.5
    (44743, true, false, false),  // 22371.5
    (59659, false, false, false), // 29829.5 (no clock)
    (74563, true, true, false),   // 37281.5
];
pub(super) const FRAME_STEP_5_PERIOD: u32 = 74564;

impl FrameCounter {
    pub(super) fn mode(&self) -> FrameCounterMode {
        self.mode
    }

    /// Reconfigures the frame sequencer according to the written value.
    ///
    /// Writing to `$4017` resets the sequencer phase; in 5-step mode the APU
    /// immediately clocks both quarter- and half-frame units (matching
    /// hardware behaviour where the first tick is effectively pulled forward).
    ///
    pub(super) fn configure(&mut self, value: u8, current_cpu_cycle: u64) -> FrameResetAction {
        self.pending_mode = if value & 0b1000_0000 == 0 {
            FrameCounterMode::FourStep
        } else {
            FrameCounterMode::FiveStep
        };
        self.reset_headstart_half = 0;
        self.pending_irq_inhibit = value & 0b0100_0000 != 0;
        // Hardware applies the new mode 3–4 CPU cycles after the write. The
        // latency depends on the CPU cycle parity at the time of the write:
        //  - write on an odd CPU cycle => 3-cycle delay
        //  - write on an even CPU cycle => 4-cycle delay
        // The parity check mirrors Nesdev/Mesen2 behaviour where the frame
        // sequencer is clocked every other CPU cycle, so writes mid-cycle
        // align differently.
        let is_odd = (current_cpu_cycle & 1) == 1;
        let delay_cycles: u8 = if is_odd { 3 } else { 4 };
        self.reset_delay_half = delay_cycles.saturating_mul(2);
        let immediate = self.pending_mode == FrameCounterMode::FiveStep;
        FrameResetAction {
            immediate_quarter: immediate,
            immediate_half: immediate,
        }
    }

    /// Schedule a frame counter reset as if `$4017` were written during a
    /// console reset. Hardware re-applies the last frame counter value a few
    /// CPU cycles before the CPU resumes execution; we mirror the 3-cycle
    /// delay used by Mesen2 for this implicit write.
    pub(super) fn configure_after_reset(&mut self, value: u8) -> FrameResetAction {
        self.pending_mode = if value & 0b1000_0000 == 0 {
            FrameCounterMode::FourStep
        } else {
            FrameCounterMode::FiveStep
        };
        self.reset_headstart_half = if self.pending_mode == FrameCounterMode::FiveStep {
            FRAME_STEP_5[1].0 as u16
        } else {
            RESET_HEADSTART_HALF_CYCLES
        };
        self.pending_irq_inhibit = value & 0b0100_0000 != 0;
        self.reset_delay_half = RESET_WRITE_DELAY_CYCLES.saturating_mul(2);
        // Unlike a CPU write to $4017, the implicit reset write does not clock
        // the frame units immediately even when requesting 5-step mode.
        let immediate = false;
        FrameResetAction {
            immediate_quarter: immediate,
            immediate_half: immediate,
        }
    }

    /// Reconfigure the frame counter immediately (used for power-on/warm reset
    /// where the hardware state is effectively re-applied without the 3–4
    /// cycle latency of a CPU write).
    pub(super) fn configure_immediate(&mut self, value: u8) -> FrameResetAction {
        self.mode = if value & 0b1000_0000 == 0 {
            FrameCounterMode::FourStep
        } else {
            FrameCounterMode::FiveStep
        };
        self.irq_inhibit = value & 0b0100_0000 != 0;
        self.half_cycle = 0;
        self.reset_delay_half = 0;
        self.reset_headstart_half = 0;
        let immediate = self.mode == FrameCounterMode::FiveStep;
        FrameResetAction {
            immediate_quarter: immediate,
            immediate_half: immediate,
        }
    }

    fn schedule(&self) -> &'static [(u32, bool, bool, bool)] {
        match self.mode {
            FrameCounterMode::FourStep => FRAME_STEP_4,
            FrameCounterMode::FiveStep => FRAME_STEP_5,
        }
    }

    fn period(&self) -> u32 {
        match self.mode {
            FrameCounterMode::FourStep => FRAME_STEP_4_PERIOD,
            FrameCounterMode::FiveStep => FRAME_STEP_5_PERIOD,
        }
    }

    /// Advances the frame counter by one CPU cycle and reports which frame
    /// units should be clocked on this tick.
    pub(super) fn step(&mut self) -> FrameTick {
        // Apply any pending reconfiguration after the latency window.
        if self.reset_delay_half > 0 {
            self.reset_delay_half = self.reset_delay_half.saturating_sub(2);
            if self.reset_delay_half == 0 {
                self.mode = self.pending_mode;
                self.irq_inhibit = self.pending_irq_inhibit;
                self.half_cycle = self.reset_headstart_half as u64;
                self.reset_headstart_half = 0;
            }
            return FrameTick::default();
        }

        let mut tick = FrameTick::default();

        // Each CPU cycle advances two half-cycles; check on both edges to catch
        // events scheduled on the half-step boundaries.
        for _ in 0..2 {
            self.half_cycle = self.half_cycle.wrapping_add(1);
            for &(step_tick, do_quarter, do_half, irq) in self.schedule() {
                if self.half_cycle == step_tick as u64 {
                    tick.quarter |= do_quarter;
                    tick.half |= do_half;
                    tick.frame_irq |= irq;
                }
            }

            if self.half_cycle >= self.period() as u64 {
                self.half_cycle = 0;
            }
        }

        tick.frame_irq &= !self.irq_inhibit;
        tick
    }
}
