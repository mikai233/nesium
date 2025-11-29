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
    timer: u16,
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
            bits_remaining: 0,
            silence: true,
            timer: DMC_RATE_TABLE[0],
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
    }

    pub(super) fn write_direct_load(&mut self, value: u8) {
        self.output_level = value & 0b0111_1111;
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
        if self.timer == 0 {
            self.timer = self.period();
            self.shift_output();
        } else {
            self.timer = self.timer.saturating_sub(1);
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
        DMC_RATE_TABLE[self.rate_index as usize]
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
