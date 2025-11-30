//! Audio Processing Unit (APU).
//!
//! The NES APU exposes five programmable sound generators (2x pulse, triangle,
//! noise, DMC) behind a small set of CPU-visible registers. This module keeps
//! the channel logic, frame sequencer, and mixer in well-scoped submodules so
//! each hardware block is easy to follow and cross-reference against Nesdev.
//!
//! Remaining accuracy work:
//! - TODO: Model the 3–4 CPU cycle delay after `$4017` writes and the half-cycle
//!   alignment of frame sequencer ticks.
//! - TODO: Wire DMC sample fetches through the CPU bus and model the DMA-like
//!   CPU stalls they trigger.
//! - TODO: Add PAL/Dendy timing tables (frame sequencer, noise/DMC rates) and a
//!   region selector so PAL test ROMs can pass.

mod dmc;
mod envelope;
mod expansion;
mod frame_counter;
mod length_counter;
mod noise;
mod pulse;
mod tables;
mod triangle;

use core::fmt;

use crate::{
    memory::apu::{self as apu_mem},
    ram::apu::RegisterRam,
};

pub use expansion::ExpansionAudio;
pub use frame_counter::FrameCounterMode;

use dmc::Dmc;
use frame_counter::{FrameCounter, FrameResetAction};
use noise::Noise;
use pulse::Pulse;
use triangle::Triangle;

/// Light-weight interrupt flags latched by the APU.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct StatusFlags {
    frame_interrupt: bool,
    dmc_interrupt: bool,
}

/// Fully modelled NES APU with envelope, sweep, length/linear counters and the
/// frame sequencer.
pub struct Apu {
    registers: RegisterRam,
    frame_counter: FrameCounter,
    status: StatusFlags,
    cycles: u64,
    pulse: [Pulse; 2],
    triangle: Triangle,
    noise: Noise,
    dmc: Dmc,
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
            status: StatusFlags::default(),
            cycles: 0,
            pulse: [
                Pulse::new(pulse::PulseChannel::Pulse1),
                Pulse::new(pulse::PulseChannel::Pulse2),
            ],
            triangle: Triangle::default(),
            noise: Noise::default(),
            dmc: Dmc::default(),
        }
    }

    pub fn reset(&mut self) {
        self.registers.fill(0);
        self.frame_counter = FrameCounter::default();
        self.status = StatusFlags::default();
        self.cycles = 0;
        self.pulse = [
            Pulse::new(pulse::PulseChannel::Pulse1),
            Pulse::new(pulse::PulseChannel::Pulse2),
        ];
        self.triangle = Triangle::default();
        self.noise = Noise::default();
        self.dmc = Dmc::default();
    }

    pub fn cpu_write(&mut self, addr: u16, value: u8) {
        if let Some(reg) = apu_mem::Register::from_cpu_addr(addr) {
            if let Some(idx) = reg.channel_ram_index() {
                self.registers[idx] = value;
            }

            match reg {
                apu_mem::Register::Pulse1Control => self.pulse[0].write_control(value),
                apu_mem::Register::Pulse1Sweep => self.pulse[0].write_sweep(value),
                apu_mem::Register::Pulse1TimerLow => self.pulse[0].write_timer_low(value),
                apu_mem::Register::Pulse1TimerHigh => self.pulse[0].write_timer_high(value),
                apu_mem::Register::Pulse2Control => self.pulse[1].write_control(value),
                apu_mem::Register::Pulse2Sweep => self.pulse[1].write_sweep(value),
                apu_mem::Register::Pulse2TimerLow => self.pulse[1].write_timer_low(value),
                apu_mem::Register::Pulse2TimerHigh => self.pulse[1].write_timer_high(value),
                apu_mem::Register::TriangleControl => self.triangle.write_control(value),
                apu_mem::Register::TriangleTimerLow => self.triangle.write_timer_low(value),
                apu_mem::Register::TriangleTimerHigh => self.triangle.write_timer_high(value),
                apu_mem::Register::NoiseControl => self.noise.write_control(value),
                apu_mem::Register::NoiseModeAndPeriod => self.noise.write_mode_and_period(value),
                apu_mem::Register::NoiseLength => self.noise.write_length(value),
                apu_mem::Register::DmcControl => self.dmc.write_control(value, &mut self.status),
                apu_mem::Register::DmcDirectLoad => self.dmc.write_direct_load(value),
                apu_mem::Register::DmcSampleAddress => self.dmc.write_sample_address(value),
                apu_mem::Register::DmcSampleLength => self.dmc.write_sample_length(value),
                apu_mem::Register::Status => self.write_status(value),
                apu_mem::Register::FrameCounter => {
                    let reset = self.frame_counter.configure(value);
                    self.status.frame_interrupt = false;
                    self.apply_frame_reset(reset);
                }
            }
        }
    }

    pub fn cpu_read(&mut self, addr: u16) -> u8 {
        match apu_mem::Register::from_cpu_addr(addr) {
            Some(apu_mem::Register::Status) => self.read_status(),
            _ => 0,
        }
    }

    fn apply_frame_reset(&mut self, reset: FrameResetAction) {
        if reset.immediate_quarter {
            self.clock_quarter_frame();
        }
        if reset.immediate_half {
            self.clock_half_frame();
        }
    }

    fn write_status(&mut self, value: u8) {
        self.pulse[0].set_enabled(value & 0b0000_0001 != 0);
        self.pulse[1].set_enabled(value & 0b0000_0010 != 0);
        self.triangle.set_enabled(value & 0b0000_0100 != 0);
        self.noise.set_enabled(value & 0b0000_1000 != 0);
        self.dmc
            .set_enabled(value & 0b0001_0000 != 0, &mut self.status);
        self.status.dmc_interrupt = false;
    }

    fn read_status(&mut self) -> u8 {
        let mut value = 0u8;

        value |= u8::from(self.pulse[0].length_active());
        value |= u8::from(self.pulse[1].length_active()) << 1;
        value |= u8::from(self.triangle.length_active()) << 2;
        value |= u8::from(self.noise.length_active()) << 3;
        value |= u8::from(self.dmc.active()) << 4;
        value |= u8::from(self.status.frame_interrupt) << 6;
        value |= u8::from(self.status.dmc_interrupt) << 7;

        // Reading $4015 clears both interrupt sources.
        self.status.frame_interrupt = false;
        self.status.dmc_interrupt = false;

        value
    }

    /// Returns `true` when either the frame sequencer or DMC have latched an IRQ.
    pub fn irq_pending(&self) -> bool {
        self.status.frame_interrupt || self.status.dmc_interrupt
    }

    /// Clears any pending IRQ sources to mimic the CPU ack cycle.
    ///
    /// The frame interrupt remains latched until `$4015` is read; DMC IRQs can
    /// be cleared by either reading `$4015` or by disabling DMC, so we mirror
    /// the latter here.
    pub fn clear_irq(&mut self) {
        self.status.dmc_interrupt = false;
    }

    fn clock_quarter_frame(&mut self) {
        for pulse in &mut self.pulse {
            pulse.clock_envelope();
        }
        self.noise.clock_envelope();
        self.triangle.clock_linear_counter();
    }

    fn clock_half_frame(&mut self) {
        for pulse in &mut self.pulse {
            pulse.clock_length();
            pulse.clock_sweep();
        }
        self.triangle.clock_length();
        self.noise.clock_length();
    }

    /// Per-CPU-cycle APU tick using a provided CPU memory reader for DMC fetches.
    ///
    /// The default [`clock`](Self::clock) uses a zeroed reader so sound output
    /// remains deterministic even when the caller does not wire up CPU reads.
    pub fn clock_with_reader<F>(&mut self, mut reader: F)
    where
        F: FnMut(u16) -> u8,
    {
        self.cycles = self.cycles.wrapping_add(1);

        let tick = self.frame_counter.clock();

        if tick.quarter {
            self.clock_quarter_frame();
        }
        if tick.half {
            self.clock_half_frame();
        }
        if tick.frame_irq {
            self.status.frame_interrupt = true;
        }

        for pulse in &mut self.pulse {
            pulse.clock_timer();
        }
        self.triangle.clock_timer();
        self.noise.clock_timer();
        self.dmc.clock(&mut reader, &mut self.status);

    }

    /// Per-CPU-cycle APU tick. DMC memory fetches return zero bytes unless the
    /// caller uses [`clock_with_reader`](Self::clock_with_reader).
    pub fn clock(&mut self) {
        self.clock_with_reader(|_| 0);
    }

    /// Mixed audio sample using the NES non-linear mixer approximation.
    pub fn sample(&self) -> f32 {
        let p1 = self.pulse[0].output() as f32;
        let p2 = self.pulse[1].output() as f32;
        let t = self.triangle.output() as f32;
        let n = self.noise.output() as f32;
        let d = self.dmc.output() as f32;

        let pulse_out = if p1 == 0.0 && p2 == 0.0 {
            0.0
        } else {
            95.88 / ((8128.0 / (p1 + p2)) + 100.0)
        };

        let tnd_out = if t == 0.0 && n == 0.0 && d == 0.0 {
            0.0
        } else {
            159.79 / ((1.0 / (t / 8227.0 + n / 12241.0 + d / 22638.0)) + 100.0)
        };

        pulse_out + tnd_out
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
    fn status_enables_channels_and_length_counters() {
        let mut apu = Apu::new();
        apu.cpu_write(apu_mem::STATUS, 0b0000_0001);
        apu.cpu_write(0x4003, 0b1111_1000); // load a long length value

        // Length counter latched because pulse1 is enabled.
        assert!(apu.pulse[0].length_active());

        // Disable and ensure the length counter clears.
        apu.cpu_write(apu_mem::STATUS, 0);
        assert!(!apu.pulse[0].length_active());
    }

    #[test]
    fn frame_counter_configuration() {
        let mut apu = Apu::new();
        apu.cpu_write(apu_mem::FRAME_COUNTER, 0b1000_0000);
        assert_eq!(apu.frame_counter.mode(), FrameCounterMode::FiveStep);

        apu.cpu_write(apu_mem::FRAME_COUNTER, 0);
        assert_eq!(apu.frame_counter.mode(), FrameCounterMode::FourStep);
    }

    #[test]
    fn frame_irq_flag_set_and_cleared() {
        let mut apu = Apu::new();
        apu.cpu_write(apu_mem::FRAME_COUNTER, 0); // 4-step, IRQs enabled

        for _ in 0..=frame_counter::FRAME_STEP_4_PERIOD as u64 {
            apu.clock();
        }
        assert!(apu.status.frame_interrupt);

        let first = apu.cpu_read(apu_mem::STATUS);
        assert_eq!(first & 0b0100_0000, 0b0100_0000);

        let second = apu.cpu_read(apu_mem::STATUS);
        assert_eq!(second & 0b0100_0000, 0);
    }

    #[test]
    fn dmc_status_bit_and_irq_clear() {
        let mut apu = Apu::new();
        apu.cpu_write(0x4013, 0x01); // sample length = 17 bytes
        apu.cpu_write(apu_mem::STATUS, 0b0001_0000); // enable DMC

        // Active bit should report bytes remaining.
        let status = apu.cpu_read(apu_mem::STATUS);
        assert_eq!(status & 0b0001_0000, 0b0001_0000);

        // Force an IRQ and ensure reads clear it.
        apu.status.dmc_interrupt = true;
        let first = apu.cpu_read(apu_mem::STATUS);
        assert_eq!(first & 0b1000_0000, 0b1000_0000);
        let second = apu.cpu_read(apu_mem::STATUS);
        assert_eq!(second & 0b1000_0000, 0);
    }
}
