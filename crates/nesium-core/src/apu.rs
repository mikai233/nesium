//! Audio Processing Unit (APU) scaffolding.
//!
//! The NES APU exposes a set of memory mapped registers between `0x4000` and
//! `0x4017`. The CPU configures the five sound channels through those registers
//! and polls the status register (`0x4015`) to detect frame IRQs or DMC activity.
//! This module provides the foundations of that interface: register storage,
//! frame counter configuration, and helpers that the bus can call into. The
//! actual audio synthesis logic will be layered on top of these primitives.

use core::fmt;

use crate::{
    memory::apu::{self as apu_mem},
    ram::apu::RegisterRam,
};

/// Frame sequencer timing mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FrameCounterMode {
    #[default]
    FourStep,
    FiveStep,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct FrameCounter {
    mode: FrameCounterMode,
    irq_inhibit: bool,
}

impl FrameCounter {
    fn configure(&mut self, value: u8) {
        self.mode = if value & 0b1000_0000 == 0 {
            FrameCounterMode::FourStep
        } else {
            FrameCounterMode::FiveStep
        };
        self.irq_inhibit = value & 0b0100_0000 != 0;
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct StatusRegister {
    pulse1_enable: bool,
    pulse2_enable: bool,
    triangle_enable: bool,
    noise_enable: bool,
    dmc_enable: bool,
    frame_interrupt: bool,
    dmc_interrupt: bool,
}

impl StatusRegister {
    fn write(&mut self, value: u8) {
        self.pulse1_enable = value & 0b0000_0001 != 0;
        self.pulse2_enable = value & 0b0000_0010 != 0;
        self.triangle_enable = value & 0b0000_0100 != 0;
        self.noise_enable = value & 0b0000_1000 != 0;
        self.dmc_enable = value & 0b0001_0000 != 0;
    }

    fn read(&mut self) -> u8 {
        let mut value = 0u8;
        value |= self.pulse1_enable as u8;
        value |= (self.pulse2_enable as u8) << 1;
        value |= (self.triangle_enable as u8) << 2;
        value |= (self.noise_enable as u8) << 3;
        value |= (self.dmc_enable as u8) << 4;
        value |= (self.frame_interrupt as u8) << 6;
        value |= (self.dmc_interrupt as u8) << 7;
        self.frame_interrupt = false;
        value
    }
}

/// Lightweight NES APU representation.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Apu {
    registers: RegisterRam,
    frame_counter: FrameCounter,
    status: StatusRegister,
    cycles: u64,
}

impl fmt::Debug for Apu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Apu")
            .field("frame_counter", &self.frame_counter)
            .field("status", &self.status)
            .field("cycles", &self.cycles)
            .finish()
    }
}

impl Apu {
    pub fn new() -> Self {
        Self {
            registers: RegisterRam::new(),
            frame_counter: FrameCounter::default(),
            status: StatusRegister::default(),
            cycles: 0,
        }
    }

    pub fn reset(&mut self) {
        self.registers.fill(0);
        self.frame_counter = FrameCounter::default();
        self.status = StatusRegister::default();
        self.cycles = 0;
    }

    pub fn cpu_write(&mut self, addr: u16, value: u8) {
        match addr {
            apu_mem::REGISTER_BASE..=apu_mem::CHANNEL_REGISTER_END => {
                let idx = (addr - apu_mem::REGISTER_BASE) as usize;
                self.registers[idx] = value;
            }
            apu_mem::STATUS => self.status.write(value),
            apu_mem::FRAME_COUNTER => self.frame_counter.configure(value),
            _ => {}
        }
    }

    pub fn cpu_read(&mut self, addr: u16) -> u8 {
        match addr {
            apu_mem::STATUS => self.status.read(),
            _ => 0,
        }
    }

    pub fn clock(&mut self) {
        self.cycles = self.cycles.wrapping_add(1);
    }

    pub fn sample(&self) -> f32 {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_channel_registers() {
        let mut apu = Apu::new();
        apu.cpu_write(apu_mem::REGISTER_BASE, 0xAA);
        apu.cpu_write(apu_mem::REGISTER_BASE + 4, 0x55);
        apu.cpu_write(apu_mem::REGISTER_BASE + 8, 0x0F);

        assert_eq!(apu.registers[0], 0xAA);
        assert_eq!(apu.registers[4], 0x55);
        assert_eq!(apu.registers[8], 0x0F);
    }

    #[test]
    fn updates_status_flags() {
        let mut apu = Apu::new();
        apu.cpu_write(apu_mem::STATUS, 0b0001_1101);

        assert!(apu.status.pulse1_enable);
        assert!(apu.status.triangle_enable);
        assert!(apu.status.noise_enable);
        assert!(!apu.status.pulse2_enable);
    }

    #[test]
    fn frame_counter_configuration() {
        let mut apu = Apu::new();
        apu.cpu_write(apu_mem::FRAME_COUNTER, 0b1000_0000);
        assert_eq!(apu.frame_counter.mode, FrameCounterMode::FiveStep);

        apu.cpu_write(apu_mem::FRAME_COUNTER, 0);
        assert_eq!(apu.frame_counter.mode, FrameCounterMode::FourStep);
    }

    #[test]
    fn reading_status_clears_frame_interrupt() {
        let mut apu = Apu::new();
        apu.status.frame_interrupt = true;
        let first = apu.cpu_read(apu_mem::STATUS);
        assert_eq!(first & 0b0100_0000, 0b0100_0000);
        let second = apu.cpu_read(apu_mem::STATUS);
        assert_eq!(second & 0b0100_0000, 0);
    }
}
