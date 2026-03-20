//! Triangle channel state and linear counter.

use super::{length_counter::LengthCounter, tables::TRIANGLE_SEQUENCE};

#[cfg_attr(
    feature = "savestate-serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(super) struct Triangle {
    control_flag: bool,
    linear_reload_value: u8,
    linear_counter: u8,
    linear_reload: bool,
    length: LengthCounter,
    timer: u16,
    timer_period: u16,
    sequence_pos: u8,
    last_output: u8,
    enabled: bool,
}

impl Triangle {
    pub(super) fn write_control(&mut self, value: u8) {
        self.control_flag = value & 0b1000_0000 != 0;
        self.linear_reload_value = value & 0b0111_1111;
        self.length.set_halt_pending(self.control_flag);
    }

    pub(super) fn write_timer_low(&mut self, value: u8) {
        self.timer_period = (self.timer_period & 0xFF00) | value as u16;
    }

    pub(super) fn write_timer_high(&mut self, value: u8) {
        self.timer_period = (self.timer_period & 0x00FF) | (((value & 0b0000_0111) as u16) << 8);
        self.length.load(value >> 3, self.enabled);
        self.linear_reload = true;
        // Writing $400B does not reset timer or waveform sequence position.
    }

    pub(super) fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length.clear();
        }
    }

    pub(super) fn clock_linear_counter(&mut self) {
        if self.linear_reload {
            self.linear_counter = self.linear_reload_value;
        } else if self.linear_counter > 0 {
            self.linear_counter -= 1;
        }

        if !self.control_flag {
            self.linear_reload = false;
        }
    }

    pub(super) fn step_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_period;
            if self.length.active() && self.linear_counter > 0 {
                self.sequence_pos = (self.sequence_pos + 1) & 0b1_1111;
                self.last_output = TRIANGLE_SEQUENCE[self.sequence_pos as usize];
            }
        } else {
            self.timer = self.timer.saturating_sub(1);
        }
    }

    pub(super) fn clock_length(&mut self) {
        self.length.clock();
    }

    pub(super) fn apply_length_halt(&mut self) {
        self.length.apply_pending_halt();
    }

    pub(super) fn output(&self) -> u8 {
        // Triangle DAC keeps its last output value when channel gating blocks
        // sequence advancement.
        self.last_output
    }

    pub(super) fn length_active(&self) -> bool {
        self.length.active()
    }
}
