//! Delta Modulation Channel (DMC) state machine.

use super::{
    StatusFlags,
    tables::{DMC_RATE_TABLE, DMC_SAMPLE_ADDR_STRIDE, DMC_SAMPLE_BASE, DMC_SAMPLE_LEN_STRIDE},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Dmc {
    irq_enable: bool,
    loop_flag: bool,
    enabled: bool,
    rate_index: u8,
    output_level: u8,
    sample_address: u16,
    sample_length: u16,
    current_address: u16,
    bytes_remaining: u16,
    sample_buffer: Option<u8>,
    shift_register: u8,
    bits_remaining: u8,
    silence: bool,
    /// Current down-counter used to time DMC bit output in CPU cycles.
    timer: u16,
    /// Reload value for the DMC timer. This mirrors Mesen2's use of
    /// `ApuTimer` with a period of `DMC_RATE_TABLE[rate] - 1`, so each
    /// bit output occurs every `DMC_RATE_TABLE[rate]` CPU cycles.
    timer_period: u16,
}

impl Default for Dmc {
    fn default() -> Self {
        Self {
            irq_enable: false,
            loop_flag: false,
            enabled: false,
            rate_index: 0,
            output_level: 0,
            sample_address: DMC_SAMPLE_BASE,
            sample_length: 1,
            current_address: DMC_SAMPLE_BASE,
            bytes_remaining: 0,
            sample_buffer: None,
            shift_register: 0,
            // Hardware powers up with the bit counter at 8; Mesen2 mirrors
            // this and we follow suit so the first reloaded sample is
            // consumed over a full 8-bit period.
            bits_remaining: 8,
            silence: true,
            // Default to the first (slowest) NTSC rate. The effective DMC
            // bit period in CPU cycles is `timer_period + 1`, matching the
            // lookup table entries.
            timer: DMC_RATE_TABLE[0] - 1,
            timer_period: DMC_RATE_TABLE[0] - 1,
        }
    }
}

impl Dmc {
    pub(super) fn write_control(&mut self, value: u8, status: &mut StatusFlags) {
        self.irq_enable = value & 0b1000_0000 != 0;
        if !self.irq_enable {
            status.dmc_interrupt = false;
        }
        self.loop_flag = value & 0b0100_0000 != 0;
        self.rate_index = value & 0b0000_1111;

        // Update the timer period to match the selected rate. Mesen2 programs
        // its `ApuTimer` with `DMC_RATE_TABLE[index] - 1`, so that each DMC
        // output tick occurs every `lookup[index]` CPU cycles. We mirror that
        // convention here so bit timing aligns with Mesen2's DMC behaviour.
        let idx = self.rate_index as usize;
        self.timer_period = DMC_RATE_TABLE[idx].saturating_sub(1);
    }

    pub(super) fn write_direct_load(&mut self, value: u8) {
        // Direct 7-bit DAC load on `$4011`. Large instantaneous jumps in the
        // output level can produce very audible pops, especially when games
        // use `$4011` as a crude DAC for kick drums or other percussive
        // sounds.
        //
        // Mesen2 exposes an optional "ReduceDmcPopping" setting that halves
        // large jumps in `_outputLevel` when writing `$4011`. Here we apply a
        // similar smoothing unconditionally to better match Mesen2 with that
        // option enabled and to tame worst-case pops in common games.
        let new_level = value & 0b0111_1111;
        let previous = self.output_level;
        self.output_level = new_level;

        let diff = (self.output_level as i16 - previous as i16).abs();
        if diff > 50 {
            let delta = self.output_level as i16 - previous as i16;
            let smoothed = self.output_level as i16 - delta / 2;
            self.output_level = smoothed.clamp(0, 127) as u8;
        }
    }

    pub(super) fn write_sample_address(&mut self, value: u8) {
        self.sample_address = DMC_SAMPLE_BASE.wrapping_add((value as u16) * DMC_SAMPLE_ADDR_STRIDE);
    }

    pub(super) fn write_sample_length(&mut self, value: u8) {
        self.sample_length = (value as u16) * DMC_SAMPLE_LEN_STRIDE + 1;
    }

    pub(super) fn set_enabled(&mut self, enabled: bool, status: &mut StatusFlags) {
        self.enabled = enabled;
        if !enabled {
            self.bytes_remaining = 0;
        } else if self.bytes_remaining == 0 {
            self.restart_sample();
        }
        if enabled {
            status.dmc_interrupt = false;
        }
    }

    pub(super) fn active(&self) -> bool {
        self.bytes_remaining > 0
    }

    pub(super) fn clock<F>(&mut self, mut reader: F, status: &mut StatusFlags)
    where
        F: FnMut(u16) -> u8,
    {
        if self.enabled && self.tick_timer() {
            self.shift_output();
        }

        if self.enabled {
            self.fetch_sample(&mut reader, status);
        }
    }

    pub(super) fn output(&self) -> u8 {
        self.output_level
    }

    fn restart_sample(&mut self) {
        self.current_address = self.sample_address;
        self.bytes_remaining = self.sample_length;
    }

    fn period(&self) -> u16 {
        // Effective DMC bit period in CPU cycles. The internal down-counter
        // counts from `timer_period` down to zero, so each bit tick spans
        // `timer_period + 1` CPU cycles, matching the values in
        // `DMC_RATE_TABLE`.
        self.timer_period + 1
    }

    /// Advances the internal DMC timer by one CPU cycle and reports whether a
    /// bit output tick should occur on this cycle.
    fn tick_timer(&mut self) -> bool {
        if self.timer == 0 {
            // Reload from the programmed period and signal a bit tick.
            self.timer = self.timer_period;
            true
        } else {
            self.timer = self.timer.saturating_sub(1);
            false
        }
    }

    fn next_address(addr: u16) -> u16 {
        if addr == 0xFFFF {
            0x8000
        } else {
            addr.wrapping_add(1)
        }
    }

    fn shift_output(&mut self) {
        if self.bits_remaining == 0 {
            if let Some(sample) = self.sample_buffer.take() {
                self.shift_register = sample;
                self.bits_remaining = 8;
                self.silence = false;
            } else {
                self.silence = true;
            }
        }

        if !self.silence {
            if self.shift_register & 1 != 0 {
                if self.output_level <= 125 {
                    self.output_level += 2;
                }
            } else if self.output_level >= 2 {
                self.output_level -= 2;
            }
        }

        if self.bits_remaining > 0 {
            self.shift_register >>= 1;
            self.bits_remaining -= 1;
        }
    }

    fn fetch_sample<F>(&mut self, reader: &mut F, status: &mut StatusFlags)
    where
        F: FnMut(u16) -> u8,
    {
        // TODO: Wire this path to the CPU bus and model the DMA-like stalls the
        // DMC triggers while fetching bytes.
        if self.sample_buffer.is_some() || self.bytes_remaining == 0 {
            return;
        }

        let byte = reader(self.current_address);
        self.sample_buffer = Some(byte);
        self.current_address = Self::next_address(self.current_address);
        self.bytes_remaining = self.bytes_remaining.saturating_sub(1);

        if self.bytes_remaining == 0 {
            if self.loop_flag {
                self.restart_sample();
            } else if self.irq_enable {
                status.dmc_interrupt = true;
            }
        }
    }
}
