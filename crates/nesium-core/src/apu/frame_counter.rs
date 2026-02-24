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

#[cfg_attr(
    feature = "savestate-serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum FrameType {
    None,
    Quarter,
    Half,
}

const FRAME_TYPES: [FrameType; 6] = [
    FrameType::Quarter,
    FrameType::Half,
    FrameType::Quarter,
    FrameType::None,
    FrameType::Half,
    FrameType::None,
];

/// Mesen2 NTSC 4-step timeline (CPU cycles).
pub(super) const FRAME_STEP_4: [u32; 6] = [7457, 14913, 22371, 29828, 29829, 29830];
/// Mesen2 NTSC 5-step timeline (CPU cycles).
pub(super) const FRAME_STEP_5: [u32; 6] = [7457, 14913, 22371, 29829, 37281, 37282];

pub(super) const FRAME_STEP_4_PERIOD: u32 = FRAME_STEP_4[5];
pub(super) const FRAME_STEP_5_PERIOD: u32 = FRAME_STEP_5[5];

/// Internal frame counter state.
#[cfg_attr(
    feature = "savestate-serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct FrameCounter {
    mode: FrameCounterMode,
    irq_inhibit: bool,
    /// Current sequence cycle position (CPU-cycle domain).
    sequence_cycle: u32,
    /// Current step index in the 6-step timeline.
    current_step: u8,
    /// Suppresses the next frame-tick edge after a frame tick is produced.
    block_frame_counter_tick: u8,
    /// Delayed write to `$4017` (3-4 CPU cycles depending on parity).
    pending_write: Option<PendingWrite>,
}

#[cfg_attr(
    feature = "savestate-serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct PendingWrite {
    value: u8,
    delay_cycles: u8,
}

impl Default for FrameCounter {
    fn default() -> Self {
        Self {
            mode: FrameCounterMode::FourStep,
            irq_inhibit: false,
            sequence_cycle: 0,
            current_step: 0,
            block_frame_counter_tick: 0,
            pending_write: None,
        }
    }
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
    pub(super) frame_irq_clear: bool,
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

const RESET_WRITE_DELAY_CYCLES: u8 = 3;

impl FrameCounter {
    pub(super) fn mode(&self) -> FrameCounterMode {
        self.mode
    }

    fn step_cycles(&self) -> &'static [u32; 6] {
        match self.mode {
            FrameCounterMode::FourStep => &FRAME_STEP_4,
            FrameCounterMode::FiveStep => &FRAME_STEP_5,
        }
    }

    /// Reconfigures the frame sequencer according to `$4017`.
    ///
    /// Mesen2 applies the write after 3-4 CPU cycles:
    /// - CPU odd cycle write -> 4-cycle delay
    /// - CPU even cycle write -> 3-cycle delay
    pub(super) fn configure(&mut self, value: u8, current_cpu_cycle: u64) -> FrameResetAction {
        let odd_cycle = (current_cpu_cycle & 1) == 1;
        let delay_cycles = if odd_cycle { 4 } else { 3 };
        self.pending_write = Some(PendingWrite {
            value,
            delay_cycles,
        });
        // IRQ inhibit changes immediately on write (mode changes later).
        self.irq_inhibit = value & 0b0100_0000 != 0;
        FrameResetAction {
            immediate_quarter: false,
            immediate_half: false,
        }
    }

    /// Schedule the implicit `$4017` write performed during reset.
    pub(super) fn configure_after_reset(&mut self, value: u8) -> FrameResetAction {
        self.sequence_cycle = 0;
        self.current_step = 0;
        self.block_frame_counter_tick = 0;
        self.pending_write = Some(PendingWrite {
            value,
            delay_cycles: RESET_WRITE_DELAY_CYCLES,
        });
        // Mesen reset path clears inhibit and applies pending mode later.
        self.irq_inhibit = false;
        FrameResetAction {
            immediate_quarter: false,
            immediate_half: false,
        }
    }

    /// Immediate frame-counter configuration (used by tests/tools).
    pub(super) fn configure_immediate(&mut self, value: u8) -> FrameResetAction {
        self.mode = if value & 0x80 == 0 {
            FrameCounterMode::FourStep
        } else {
            FrameCounterMode::FiveStep
        };
        self.irq_inhibit = value & 0x40 != 0;
        self.sequence_cycle = 0;
        self.current_step = 0;
        self.block_frame_counter_tick = 0;
        self.pending_write = None;
        FrameResetAction {
            immediate_quarter: false,
            immediate_half: false,
        }
    }

    fn apply_frame_type_tick(&mut self, tick: &mut FrameTick, frame_type: FrameType) {
        if self.block_frame_counter_tick != 0 {
            return;
        }
        match frame_type {
            FrameType::None => {}
            FrameType::Quarter => {
                tick.quarter = true;
                self.block_frame_counter_tick = 2;
            }
            FrameType::Half => {
                tick.quarter = true;
                tick.half = true;
                self.block_frame_counter_tick = 2;
            }
        }
    }

    fn apply_pending_write(&mut self, tick: &mut FrameTick) {
        if let Some(mut pending) = self.pending_write {
            if pending.delay_cycles > 0 {
                pending.delay_cycles -= 1;
            }
            if pending.delay_cycles == 0 {
                self.mode = if pending.value & 0x80 == 0 {
                    FrameCounterMode::FourStep
                } else {
                    FrameCounterMode::FiveStep
                };
                self.sequence_cycle = 0;
                self.current_step = 0;
                self.pending_write = None;

                // 5-step write clocks both quarter+half immediately, unless
                // a frame tick is currently blocked.
                if self.mode == FrameCounterMode::FiveStep && self.block_frame_counter_tick == 0 {
                    tick.quarter = true;
                    tick.half = true;
                    self.block_frame_counter_tick = 2;
                }
            } else {
                self.pending_write = Some(pending);
            }
        }
    }

    /// Advances the frame counter by one CPU cycle and reports frame events.
    pub(super) fn step(&mut self) -> FrameTick {
        let mut tick = FrameTick::default();

        self.sequence_cycle = self.sequence_cycle.wrapping_add(1);
        let step_cycles = self.step_cycles();

        if self.sequence_cycle == step_cycles[self.current_step as usize] {
            // In 4-step mode, frame IRQ is asserted on the last 3 cycles.
            if self.mode == FrameCounterMode::FourStep && self.current_step >= 3 {
                if !self.irq_inhibit {
                    tick.frame_irq = true;
                } else if self.current_step == 5 {
                    // Keep the explicit clear edge to mirror Mesen behavior
                    // when inhibit is set while the 3-cycle IRQ window runs.
                    tick.frame_irq_clear = true;
                }
            }

            self.apply_frame_type_tick(&mut tick, FRAME_TYPES[self.current_step as usize]);

            self.current_step += 1;
            if self.current_step == 6 {
                self.current_step = 0;
                self.sequence_cycle = 0;
            }
        }

        self.apply_pending_write(&mut tick);

        if self.block_frame_counter_tick > 0 {
            self.block_frame_counter_tick -= 1;
        }

        tick
    }
}
