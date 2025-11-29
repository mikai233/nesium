//! Noise channel state and linear feedback shift register (LFSR).

use super::{envelope::Envelope, length_counter::LengthCounter, tables::NOISE_PERIOD_TABLE};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Noise {
    pub(super) envelope: Envelope,
    pub(super) length: LengthCounter,
    mode: bool,
    timer_period: u16,
    timer: u16,
    shift_register: u16,
    enabled: bool,
}

impl Default for Noise {
    fn default() -> Self {
        Self {
            envelope: Envelope::default(),
            length: LengthCounter::default(),
            mode: false,
            timer_period: NOISE_PERIOD_TABLE[0],
            timer: 0,
            shift_register: 1,
            enabled: false,
        }
    }
}

impl Noise {
    pub(super) fn write_control(&mut self, value: u8) {
        self.envelope.configure(value);
    }

    pub(super) fn write_mode_and_period(&mut self, value: u8) {
        self.mode = value & 0b1000_0000 != 0;
        let idx = (value & 0b0000_1111) as usize;
        self.timer_period = NOISE_PERIOD_TABLE[idx];
    }

    pub(super) fn write_length(&mut self, value: u8) {
        self.length.load(value >> 3, self.enabled);
        self.envelope.restart();
        self.timer = self.timer_period;
    }

    pub(super) fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length.clear();
        }
    }

    pub(super) fn clock_timer(&mut self) {
        if self.timer == 0 {
            self.timer = self.timer_period;
            self.step_lfsr();
        } else {
            self.timer = self.timer.saturating_sub(1);
        }
    }

    fn step_lfsr(&mut self) {
        let tap = if self.mode { 6 } else { 1 };
        let bit = (self.shift_register ^ (self.shift_register >> tap)) & 1;
        self.shift_register >>= 1;
        self.shift_register |= bit << 14;
    }

    pub(super) fn clock_envelope(&mut self) {
        self.envelope.clock();
    }

    pub(super) fn clock_length(&mut self) {
        self.length.clock(self.envelope.halt_length());
    }

    pub(super) fn output(&self) -> u8 {
        if !self.enabled || !self.length.active() || (self.shift_register & 1) != 0 {
            0
        } else {
            self.envelope.output()
        }
    }

    pub(super) fn length_active(&self) -> bool {
        self.length.active()
    }
}
