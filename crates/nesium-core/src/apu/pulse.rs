//! Pulse channel state, including sweep and envelope units.

use super::{envelope::Envelope, length_counter::LengthCounter, tables::PULSE_DUTY_TABLE};

#[cfg_attr(
    feature = "savestate-serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) enum PulseChannel {
    Pulse1,
    Pulse2,
}

#[cfg_attr(
    feature = "savestate-serde",
    derive(serde::Serialize, serde::Deserialize)
)]
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
        // Hardware sweep divider period is P+1.
        self.period = ((value >> 4) & 0b0000_0111).saturating_add(1);
        self.negate = value & 0b0000_1000 != 0;
        self.shift = value & 0b0000_0111;
        self.reload = true;
    }

    pub(super) fn reload(&mut self) {
        self.reload = true;
    }

    fn muted(&self, timer_period: u16) -> bool {
        timer_period < 8 || (!self.negate && self.target_period(timer_period) > 0x07FF)
    }

    fn target_period(&self, timer_period: u16) -> u32 {
        let base = timer_period as u32;
        let delta = (timer_period >> self.shift) as u32;
        if self.negate {
            match self.channel {
                PulseChannel::Pulse1 => base.wrapping_sub(delta).wrapping_sub(1),
                PulseChannel::Pulse2 => base.wrapping_sub(delta),
            }
        } else {
            base + delta
        }
    }

    pub(super) fn clock(&mut self, timer_period: &mut u16) {
        let should_mutate = self.enabled && self.shift != 0 && !self.muted(*timer_period);

        // Mesen2 behavior: divider is decremented first (with 8-bit wrap), then
        // checked for zero.
        self.divider = self.divider.wrapping_sub(1);
        if self.divider == 0 {
            if should_mutate {
                *timer_period = self.target_period(*timer_period) as u16;
            }
            self.divider = self.period;
        }

        if self.reload {
            self.reload = false;
            self.divider = self.period;
        }
    }
}

#[cfg_attr(
    feature = "savestate-serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Pulse {
    duty: u8,
    duty_pos: u8,
    timer: u16,
    timer_reload: u16,
    timer_period: u16,
    pub(super) envelope: Envelope,
    pub(super) length: LengthCounter,
    pub(super) sweep: Sweep,
    enabled: bool,
    current_output: u8,
}

impl Pulse {
    pub(super) fn new(channel: PulseChannel) -> Self {
        Self {
            duty: 0,
            duty_pos: 0,
            timer: 0,
            timer_reload: 0,
            timer_period: 0,
            envelope: Envelope::default(),
            length: LengthCounter::default(),
            sweep: Sweep {
                channel,
                ..Sweep::default()
            },
            enabled: false,
            current_output: 0,
        }
    }

    pub(super) fn write_control(&mut self, value: u8) {
        self.duty = (value >> 6) & 0b0000_0011;
        self.envelope.configure(value);
        self.length.set_halt_pending(self.envelope.halt_length());
        self.refresh_output();
    }

    pub(super) fn write_sweep(&mut self, value: u8) {
        self.sweep.write(value);
        self.refresh_output();
    }

    pub(super) fn write_timer_low(&mut self, value: u8) {
        self.timer_period = (self.timer_period & 0x0700) | value as u16;
        self.timer_reload = (self.timer_period << 1) | 1;
        self.refresh_output();
    }

    pub(super) fn write_timer_high(&mut self, value: u8) {
        self.timer_period = (self.timer_period & 0x00FF) | (((value & 0b0000_0111) as u16) << 8);
        self.timer_reload = (self.timer_period << 1) | 1;
        self.duty_pos = 0;
        self.envelope.restart();
        self.length.load(value >> 3, self.enabled);
        // Do not reset the timer on high-byte writes (Mesen2 / hardware behavior).
        self.refresh_output();
    }

    pub(super) fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.length.clear();
        }
    }

    pub(super) fn step_timer(&mut self) {
        if self.timer == 0 {
            // Match Mesen2's ApuTimer behavior: before the first timer write,
            // reload is 0 (ticks every CPU cycle). After low/high writes,
            // reload is 2*period+1.
            self.timer = self.timer_reload;
            // Mesen clocks the sequencer "backward" through the 8-step table.
            self.duty_pos = self.duty_pos.wrapping_sub(1) & 0b111;
            self.refresh_output();
        } else {
            self.timer = self.timer.saturating_sub(1);
        }
    }

    pub(super) fn clock_envelope(&mut self) {
        self.envelope.clock();
    }

    pub(super) fn clock_length(&mut self) {
        self.length.clock();
    }

    pub(super) fn apply_length_halt(&mut self) {
        self.length.apply_pending_halt();
    }

    pub(super) fn clock_sweep(&mut self) {
        let before = self.timer_period;
        self.sweep.clock(&mut self.timer_period);
        if self.timer_period != before {
            self.timer_reload = (self.timer_period << 1) | 1;
        }
    }

    pub(super) fn output(&self) -> u8 {
        self.current_output
    }

    #[inline]
    fn compute_output(&self) -> u8 {
        if !self.enabled || !self.length.active() || self.sweep.muted(self.timer_period) {
            return 0;
        }

        if PULSE_DUTY_TABLE[self.duty as usize][self.duty_pos as usize] == 0 {
            0
        } else {
            self.envelope.output()
        }
    }

    #[inline]
    fn refresh_output(&mut self) {
        self.current_output = self.compute_output();
    }

    pub(super) fn length_active(&self) -> bool {
        self.length.active()
    }
}
