//! Frame sequencer responsible for clocking envelopes, length counters, and
//! sweep units at quarter- and half-frame intervals.

/// Frame sequencer timing mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FrameCounterMode {
    #[default]
    FourStep,
    FiveStep,
}

/// Internal frame counter state.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct FrameCounter {
    mode: FrameCounterMode,
    irq_inhibit: bool,
    cycle: u64,
}

/// Indicates which frame units should be clocked after a frame counter tick.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct FrameTick {
    pub(super) quarter: bool,
    pub(super) half: bool,
    pub(super) frame_irq: bool,
}

/// Frame sequencer reset side effects applied after writing `$4017`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct FrameResetAction {
    pub(super) immediate_quarter: bool,
    pub(super) immediate_half: bool,
}

/// Frame sequencer timeline for 4-step mode: (CPU cycle, quarter, half, irq).
///
/// The values and period mirror Mesen2's NTSC `_stepCyclesNtsc[0]` table
/// (derived from NESdev), expressed directly in CPU cycles.
pub(super) const FRAME_STEP_4: &[(u16, bool, bool, bool)] = &[
    (7457, true, false, false),
    (14913, true, true, false),
    (22371, true, false, false),
    (29829, true, true, true),
];
pub(super) const FRAME_STEP_4_PERIOD: u16 = 29830;

/// Frame sequencer timeline for 5-step mode: (CPU cycle, quarter, half, irq).
///
/// The 5-step mode never generates frame IRQs; this table again mirrors the
/// NTSC timings from Mesen2's `_stepCyclesNtsc[1]`.
pub(super) const FRAME_STEP_5: &[(u16, bool, bool, bool)] = &[
    (7457, true, false, false),
    (14913, true, true, false),
    (22371, true, false, false),
    (29829, true, true, false),
    (37281, false, false, false),
];
pub(super) const FRAME_STEP_5_PERIOD: u16 = 37282;

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
    /// TODO: Emulate the hardware 3–4 CPU cycle delay before the new mode takes
    /// effect and the half-cycle alignment of the first tick.
    pub(super) fn configure(&mut self, value: u8) -> FrameResetAction {
        self.mode = if value & 0b1000_0000 == 0 {
            FrameCounterMode::FourStep
        } else {
            FrameCounterMode::FiveStep
        };
        self.irq_inhibit = value & 0b0100_0000 != 0;
        self.cycle = 0;
        let immediate = self.mode == FrameCounterMode::FiveStep;
        FrameResetAction {
            immediate_quarter: immediate,
            immediate_half: immediate,
        }
    }

    fn schedule(&self) -> &'static [(u16, bool, bool, bool)] {
        match self.mode {
            FrameCounterMode::FourStep => FRAME_STEP_4,
            FrameCounterMode::FiveStep => FRAME_STEP_5,
        }
    }

    fn period(&self) -> u16 {
        match self.mode {
            FrameCounterMode::FourStep => FRAME_STEP_4_PERIOD,
            FrameCounterMode::FiveStep => FRAME_STEP_5_PERIOD,
        }
    }

    /// Advances the frame counter by one CPU cycle and reports which frame
    /// units should be clocked on this tick.
    pub(super) fn clock(&mut self) -> FrameTick {
        self.cycle = self.cycle.wrapping_add(1);
        let mut tick = FrameTick::default();

        for &(step_tick, do_quarter, do_half, irq) in self.schedule() {
            if self.cycle == step_tick as u64 {
                tick.quarter |= do_quarter;
                tick.half |= do_half;
                tick.frame_irq |= irq;
            }
        }

        if self.cycle >= self.period() as u64 {
            self.cycle = 0;
        }

        tick.frame_irq &= !self.irq_inhibit;
        tick
    }
}
