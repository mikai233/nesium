//! Pulse channel state, including sweep and envelope units.

use super::{envelope::Envelope, length_counter::LengthCounter, tables::PULSE_DUTY_TABLE};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum PulseChannel {
    Pulse1,
    Pulse2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Sweep {
    enabled: bool,
    negate: bool,
    shift: u8,
    period: u8,
    divider: u8,
    reload: bool,
    channel: PulseChannel,
}

impl Default for Sweep {
    fn default() -> Self {
        Self {
            enabled: false,
            negate: false,
            shift: 0,
            period: 0,
            divider: 0,
            reload: false,
            channel: PulseChannel::Pulse1,
        }
    }
}

impl Sweep {
    pub(super) fn write(&mut self, value: u8) {
        self.enabled = value & 0b1000_0000 != 0;
        self.period = (value >> 4) & 0b0000_0111;
        self.negate = value & 0b0000_1000 != 0;
        self.shift = value & 0b0000_0111;
        self.reload = true;
    }

    pub(super) fn reload(&mut self) {
        self.reload = true;
    }

    fn muted(&self, timer_period: u16) -> bool {
        timer_period < 8 || self.target_period(timer_period) > 0x07FF
    }

    fn target_period(&self, timer_period: u16) -> u16 {
        let delta = timer_period >> self.shift;
        if self.negate {
            match self.channel {
                PulseChannel::Pulse1 => timer_period.wrapping_sub(delta).wrapping_sub(1),
                PulseChannel::Pulse2 => timer_period.wrapping_sub(delta),
            }
        } else {
            timer_period.wrapping_add(delta)
        }
    }

    pub(super) fn clock(&mut self, timer_period: &mut u16) {
        let should_mutate = self.enabled && self.shift != 0 && !self.muted(*timer_period);

        if self.divider == 0 {
            if should_mutate {
                *timer_period = self.target_period(*timer_period);
            }
            self.divider = self.period;
        } else {
            self.divider = self.divider.saturating_sub(1);
        }

        if self.reload {
            self.reload = false;
            self.divider = self.period;
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Pulse {
    duty: u8,
    duty_pos: u8,
    timer: u16,
    timer_period: u16,
    phase_toggle: bool,
    pub(super) envelope: Envelope,
    pub(super) length: LengthCounter,
    pub(super) sweep: Sweep,
    enabled: bool,
}

impl Pulse {
    pub(super) fn new(channel: PulseChannel) -> Self {
        Self {
            duty: 0,
            duty_pos: 0,
            timer: 0,
            timer_period: 0,
            phase_toggle: false,
            envelope: Envelope::default(),
            length: LengthCounter::default(),
            sweep: Sweep {
                channel,
                ..Sweep::default()
            },
            enabled: false,
        }
    }

    pub(super) fn write_control(&mut self, value: u8) {
        self.duty = (value >> 6) & 0b0000_0011;
        self.envelope.configure(value);
    }

    pub(super) fn write_sweep(&mut self, value: u8) {
        self.sweep.write(value);
    }

    pub(super) fn write_timer_low(&mut self, value: u8) {
        self.timer_period = (self.timer_period & 0xFF00) | value as u16;
        self.phase_toggle = false;
    }

    pub(super) fn write_timer_high(&mut self, value: u8) {
        self.timer_period = (self.timer_period & 0x00FF) | (((value & 0b0000_0111) as u16) << 8);
        self.duty_pos = 0;
        self.phase_toggle = false;
        self.envelope.restart();
        self.length.load(value >> 3, self.enabled);
        self.timer = self.timer_period;
        self.sweep.reload();
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
            // Pulse timer output advances the sequencer every other timer reload.
            self.phase_toggle = !self.phase_toggle;
            if self.phase_toggle {
                self.duty_pos = (self.duty_pos + 1) & 0b111;
            }
        } else {
            self.timer = self.timer.saturating_sub(1);
        }
    }

    pub(super) fn clock_envelope(&mut self) {
        self.envelope.clock();
    }

    pub(super) fn clock_length(&mut self) {
        self.length.clock(self.envelope.halt_length());
    }

    pub(super) fn clock_sweep(&mut self) {
        self.sweep.clock(&mut self.timer_period);
    }

    pub(super) fn output(&self) -> u8 {
        if !self.enabled || !self.length.active() || self.sweep.muted(self.timer_period) {
            return 0;
        }

        if PULSE_DUTY_TABLE[self.duty as usize][self.duty_pos as usize] == 0 {
            0
        } else {
            self.envelope.output()
        }
    }

    pub(super) fn length_active(&self) -> bool {
        self.length.active()
    }
}
