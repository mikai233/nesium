//! Delta Modulation Channel (DMC) state machine.

use super::{
    StatusFlags,
    tables::{DMC_RATE_TABLE, DMC_SAMPLE_ADDR_STRIDE, DMC_SAMPLE_BASE, DMC_SAMPLE_LEN_STRIDE},
};
use crate::bus::{DmcDmaEvent, PendingDma};

#[cfg_attr(
    feature = "savestate-serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Dmc {
    irq_enable: bool,
    loop_flag: bool,
    enabled: bool,
    /// Deferred disable window used after `$4015` bit-4 is cleared.
    /// Mesen models this as a 2/3 CPU-cycle delay based on CPU parity.
    disable_delay: u8,
    /// Deferred DMA-start window used after enabling DMC via `$4015`.
    /// Also follows the 2/3 CPU-cycle odd/even rule.
    transfer_start_delay: u8,
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
    /// Address of the most recent DMA fetch (for bus-side DMA emulation).
    last_fetch_addr: u16,
    /// Pending DMA fetch address to be performed during stall cycles.
    pending_fetch: Option<u16>,
}

impl Default for Dmc {
    fn default() -> Self {
        Self {
            irq_enable: false,
            loop_flag: false,
            enabled: false,
            disable_delay: 0,
            transfer_start_delay: 0,
            rate_index: 0,
            output_level: 0,
            sample_address: DMC_SAMPLE_BASE,
            sample_length: 1,
            current_address: 0,
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
            last_fetch_addr: 0,
            pending_fetch: None,
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
        // Hardware: direct 7-bit DAC load, no smoothing.
        self.output_level = value & 0b0111_1111;
    }

    pub(super) fn write_sample_address(&mut self, value: u8) {
        self.sample_address = DMC_SAMPLE_BASE.wrapping_add((value as u16) * DMC_SAMPLE_ADDR_STRIDE);
    }

    pub(super) fn write_sample_length(&mut self, value: u8) {
        self.sample_length = (value as u16) * DMC_SAMPLE_LEN_STRIDE + 1;
    }

    pub(super) fn set_enabled(&mut self, enabled: bool, cpu_cycle: u64) {
        self.enabled = enabled;

        if !enabled {
            if self.disable_delay == 0 {
                self.disable_delay = Self::delay_for_cpu_cycle(cpu_cycle);
            }
        } else if self.bytes_remaining == 0 {
            self.restart_sample();
            self.transfer_start_delay = Self::delay_for_cpu_cycle(cpu_cycle);
        }
    }

    pub(super) fn active(&self) -> bool {
        self.bytes_remaining > 0
    }

    pub(super) fn step(&mut self, pending_dma: &mut PendingDma) {
        self.process_delays(pending_dma);

        if self.tick_timer() {
            self.shift_output();
        }

        if self.transfer_start_delay == 0 {
            self.fetch_sample(pending_dma);
        }
    }

    pub(super) fn output(&self) -> u8 {
        self.output_level
    }

    fn restart_sample(&mut self) {
        self.current_address = self.sample_address;
        self.bytes_remaining = self.sample_length;
    }

    #[inline]
    fn delay_for_cpu_cycle(cpu_cycle: u64) -> u8 {
        // Match Mesen parity directly from the CPU cycle counter used at the
        // `$4015` write site.
        if (cpu_cycle & 0x01) == 0 { 2 } else { 3 }
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

    fn process_delays(&mut self, pending_dma: &mut PendingDma) {
        if self.disable_delay > 0 {
            self.disable_delay -= 1;
            if self.disable_delay == 0 {
                self.bytes_remaining = 0;
                self.transfer_start_delay = 0;
                self.pending_fetch = None;
                pending_dma.dmc = Some(DmcDmaEvent::Abort);
            }
        }

        if self.transfer_start_delay > 0 {
            self.transfer_start_delay -= 1;
        }
    }

    fn shift_output(&mut self) {
        if !self.silence {
            if self.shift_register & 1 != 0 {
                if self.output_level <= 125 {
                    self.output_level += 2;
                }
            } else if self.output_level >= 2 {
                self.output_level -= 2;
            }

            self.shift_register >>= 1;
        }

        self.bits_remaining = self.bits_remaining.saturating_sub(1);
        if self.bits_remaining == 0 {
            self.bits_remaining = 8;
            if let Some(sample) = self.sample_buffer.take() {
                self.shift_register = sample;
                self.silence = false;
            } else {
                self.silence = true;
            }
        }
    }

    fn fetch_sample(&mut self, pending_dma: &mut PendingDma) {
        if self.sample_buffer.is_some() || self.bytes_remaining == 0 || self.pending_fetch.is_some()
        {
            return;
        }

        self.last_fetch_addr = self.current_address;
        self.pending_fetch = Some(self.current_address);
        // Each sample fetch steals 4 CPU cycles on hardware; queue the DMA
        // request immediately so the bus can model the stolen cycles.
        pending_dma.dmc = Some(DmcDmaEvent::Request {
            addr: self.pending_fetch.expect("pending fetch address"),
        });
    }

    pub(super) fn last_fetch_addr(&self) -> u16 {
        self.last_fetch_addr
    }

    pub(super) fn finish_dma_fetch(&mut self, byte: u8, status: &mut StatusFlags) {
        if self.pending_fetch.take().is_some() {
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
}
