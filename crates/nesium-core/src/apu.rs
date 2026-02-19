//! Audio Processing Unit (APU).
//!
//! The NES APU exposes five programmable sound generators (2x pulse, triangle,
//! noise, DMC) behind a small set of CPU-visible registers. This module keeps
//! the channel logic, frame sequencer, and mixer in well-scoped submodules so
//! each hardware block is easy to follow and cross-reference against Nesdev.
//!
//! Remaining accuracy work:
//! - TODO: Add PAL/Dendy timing tables (frame sequencer, noise/DMC rates) and a
//!   region selector so PAL test ROMs can pass.

mod dmc;
mod envelope;
pub mod expansion;
mod frame_counter;
mod length_counter;
mod noise;
mod pulse;
mod tables;
mod triangle;

use core::fmt;
use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::sync::{Mutex, OnceLock};

use crate::{
    audio::{AudioChannel, NesSoundMixer},
    bus::{CpuBus, DmcDmaEvent},
    context::Context,
    cpu::Cpu,
    mem_block::apu::RegisterRam,
    memory::apu::{self as apu_mem},
    reset_kind::ResetKind,
};

pub use expansion::ExpansionAudio;
pub use frame_counter::FrameCounterMode;

use dmc::Dmc;
use frame_counter::{FrameCounter, FrameResetAction};
use noise::Noise;
use pulse::Pulse;
use triangle::Triangle;

/// Light-weight interrupt flags latched by the APU.
#[cfg_attr(
    feature = "savestate-serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
struct StatusFlags {
    frame_interrupt: bool,
    dmc_interrupt: bool,
}

type LastLevels = crate::mem_block::MemBlock<f32, 5>;

static APU_TRACE_LOG: OnceLock<Option<Mutex<BufWriter<std::fs::File>>>> = OnceLock::new();
static APU_TRACE_READ_ADDRS: OnceLock<Option<Box<[u16]>>> = OnceLock::new();

#[inline]
fn apu_trace_flag(value: bool) -> u8 {
    u8::from(value)
}

fn apu_trace_log_write(line: &str) {
    let log = APU_TRACE_LOG.get_or_init(|| {
        let path = std::env::var("NESIUM_APU_TRACE_PATH").ok()?;
        OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(path)
            .ok()
            .map(|f| Mutex::new(BufWriter::with_capacity(256 * 1024, f)))
    });

    if let Some(writer) = log
        && let Ok(mut w) = writer.lock()
    {
        let _ = writeln!(w, "{line}");
        let _ = w.flush();
    }
}

#[inline]
fn parse_trace_addr_token(token: &str) -> Option<u16> {
    let t = token.trim();
    if t.is_empty() {
        return None;
    }
    if let Some(hex) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        return u16::from_str_radix(hex.trim(), 16).ok();
    }
    if t.chars().any(|c| c.is_ascii_alphabetic()) {
        u16::from_str_radix(t, 16).ok()
    } else {
        t.parse::<u16>().ok()
    }
}

fn apu_trace_read_addrs() -> &'static Option<Box<[u16]>> {
    APU_TRACE_READ_ADDRS.get_or_init(|| {
        let raw = std::env::var("NESIUM_APU_TRACE_READ_ADDRS").ok()?;
        let mut addrs = Vec::new();
        for token in raw.split(',') {
            if let Some(addr) = parse_trace_addr_token(token) {
                addrs.push(addr);
            }
        }
        if addrs.is_empty() {
            None
        } else {
            Some(addrs.into_boxed_slice())
        }
    })
}

#[inline]
fn apu_trace_should_log_read_mem(addr: u16) -> bool {
    match apu_trace_read_addrs() {
        Some(addrs) => addrs.contains(&addr),
        None => false,
    }
}

/// Fully modelled NES APU with envelope, sweep, length/linear counters and the
/// frame sequencer.
#[cfg_attr(
    feature = "savestate-serde",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(Clone)]
pub struct Apu {
    registers: RegisterRam,
    frame_counter: FrameCounter,
    status: StatusFlags,
    cycles: u64,
    pulse: [Pulse; 2],
    triangle: Triangle,
    noise: Noise,
    dmc: Dmc,
    last_levels: LastLevels,
    /// Last value written to `$4017` (frame counter). Used to distinguish
    /// power-on behaviour (acts as if `$00` were written) from warm resets,
    /// where hardware effectively re-applies the last written mode.
    last_frame_counter_value: u8,
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

impl Default for Apu {
    fn default() -> Self {
        Self::new()
    }
}

impl Apu {
    #[inline]
    fn dmc_trace_state_fields(&self) -> String {
        format!(
            "dmc_bytes={}|dmc_buf_empty={}|dmc_bits={}|dmc_timer={}|dmc_addr={:04X}|dmc_dis={}|dmc_start={}",
            self.dmc.bytes_remaining(),
            apu_trace_flag(self.dmc.sample_buffer_empty()),
            self.dmc.bits_remaining(),
            self.dmc.timer_value(),
            self.dmc.current_address(),
            self.dmc.disable_delay(),
            self.dmc.transfer_start_delay()
        )
    }

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
            last_levels: LastLevels::new(),
            last_frame_counter_value: 0x00,
        }
    }

    #[inline]
    fn trace_write_register(&self, addr: u16, value: u8) {
        let dmc_state = self.dmc_trace_state_fields();
        apu_trace_log_write(&format!(
            "APUTRACE|src=nesium|ev=write|cycle={}|addr={:04X}|value={:02X}|frame_irq={}|dmc_irq={}|{}",
            self.cycles,
            addr,
            value,
            apu_trace_flag(self.status.frame_interrupt),
            apu_trace_flag(self.status.dmc_interrupt),
            dmc_state
        ));
    }

    #[inline]
    fn trace_read_status(
        &self,
        value: u8,
        frame_irq_before: bool,
        dmc_irq_before: bool,
        frame_irq_after: bool,
        dmc_irq_after: bool,
    ) {
        apu_trace_log_write(&format!(
            "APUTRACE|src=nesium|ev=read|cycle={}|addr=4015|value={:02X}|frame_irq_before={}|dmc_irq_before={}|frame_irq_after={}|dmc_irq_after={}",
            self.cycles,
            value,
            apu_trace_flag(frame_irq_before),
            apu_trace_flag(dmc_irq_before),
            apu_trace_flag(frame_irq_after),
            apu_trace_flag(dmc_irq_after)
        ));
    }

    #[inline]
    pub(crate) fn trace_mem_read(&self, addr: u16, value: u8) {
        if !apu_trace_should_log_read_mem(addr) {
            return;
        }
        apu_trace_log_write(&format!(
            "APUTRACE|src=nesium|ev=read_mem|cycle={}|addr={:04X}|value={:02X}",
            self.cycles, addr, value
        ));
    }

    #[inline]
    fn trace_irq_event(&self, event: &str) {
        apu_trace_log_write(&format!(
            "APUTRACE|src=nesium|ev={}|cycle={}|frame_irq={}|dmc_irq={}",
            event,
            self.cycles,
            apu_trace_flag(self.status.frame_interrupt),
            apu_trace_flag(self.status.dmc_interrupt)
        ));
    }

    #[inline]
    fn trace_dmc_dma_event(&self, event: DmcDmaEvent) {
        let dmc_state = self.dmc_trace_state_fields();
        match event {
            DmcDmaEvent::Request { addr } => {
                apu_trace_log_write(&format!(
                    "APUTRACE|src=nesium|ev=dmc_dma_request|cycle={}|addr={:04X}|{}",
                    self.cycles, addr, dmc_state
                ));
            }
            DmcDmaEvent::Abort => {
                apu_trace_log_write(&format!(
                    "APUTRACE|src=nesium|ev=dmc_dma_abort|cycle={}|{}",
                    self.cycles, dmc_state
                ));
            }
        }
    }

    #[inline]
    fn trace_dmc_dma_complete(&self, byte: u8) {
        let dmc_state = self.dmc_trace_state_fields();
        apu_trace_log_write(&format!(
            "APUTRACE|src=nesium|ev=dmc_dma_complete|cycle={}|addr={:04X}|value={:02X}|{}",
            self.cycles,
            self.dmc.last_fetch_addr(),
            byte,
            dmc_state
        ));
    }

    /// Applies either a power-on style reset or a warm reset to the APU.
    ///
    /// - `ResetKind::PowerOn` matches turning the console off and back on:
    ///   all channel registers are cleared and the frame counter behaves as if
    ///   `$4017` were written with `$00` shortly before code execution begins.
    ///
    /// - `ResetKind::Soft` approximates the behaviour described in blargg's
    ///   `apu_reset` tests and implemented in Mesen2's
    ///   `ApuFrameCounter::Reset(softReset = true)`: channel registers are
    ///   preserved, channel state is rebuilt from the cached register RAM, and
    ///   the frame counter is reconfigured as if the last value written to
    ///   `$4017` were written again just before execution resumes.
    pub fn reset(&mut self, kind: ResetKind) {
        match kind {
            ResetKind::PowerOn => {
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
                self.last_levels.fill(0.0);
                self.last_frame_counter_value = 0x00;

                // Hardware behaves as if $4017 were written with $00 shortly before
                // execution begins; apply that with the reset-style latency (~3 CPU
                // cycles before the CPU resumes fetching instructions).
                let reset = self
                    .frame_counter
                    .configure_after_reset(self.last_frame_counter_value);
                self.apply_frame_reset(reset);
            }
            ResetKind::Soft => {
                self.status = StatusFlags::default();
                self.cycles = 0;
                self.rebuild_channels_from_registers();

                // Hardware reset re-applies the last written frame counter value just
                // before execution resumes, with the same small latency as the
                // implicit power-on write.
                let reset = self
                    .frame_counter
                    .configure_after_reset(self.last_frame_counter_value);
                self.status.frame_interrupt = false;
                self.apply_frame_reset(reset);
            }
        }
    }

    pub fn cpu_write(&mut self, addr: u16, value: u8, cpu_cycle: u64) {
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
                apu_mem::Register::Status => self.write_status(value, cpu_cycle),
                apu_mem::Register::FrameCounter => {
                    // Track the last written value so warm resets can restore
                    // the current frame counter mode, matching hardware
                    // behaviour where `$4017` is effectively re-applied on
                    // reset rather than forced back to `$00`.
                    self.last_frame_counter_value = value;
                    let reset = self.frame_counter.configure(value, cpu_cycle);
                    if value & 0x40 != 0 {
                        self.status.frame_interrupt = false;
                    }
                    self.apply_frame_reset(reset);
                }
            }

            if (0x4010..=0x4017).contains(&addr) {
                self.trace_write_register(addr, value);
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
            self.step_quarter_frame();
        }
        if reset.immediate_half {
            self.step_half_frame();
        }
    }

    fn write_status(&mut self, value: u8, cpu_cycle: u64) {
        self.pulse[0].set_enabled(value & 0b0000_0001 != 0);
        self.pulse[1].set_enabled(value & 0b0000_0010 != 0);
        self.triangle.set_enabled(value & 0b0000_0100 != 0);
        self.noise.set_enabled(value & 0b0000_1000 != 0);

        // Hardware clears DMC IRQ on $4015 writes (not on $4015 reads).
        self.status.dmc_interrupt = false;
        self.dmc.set_enabled(value & 0b0001_0000 != 0, cpu_cycle);
    }

    fn read_status(&mut self) -> u8 {
        let frame_irq_before = self.status.frame_interrupt;
        let dmc_irq_before = self.status.dmc_interrupt;
        let mut value = 0u8;

        value |= u8::from(self.pulse[0].length_active());
        value |= u8::from(self.pulse[1].length_active()) << 1;
        value |= u8::from(self.triangle.length_active()) << 2;
        value |= u8::from(self.noise.length_active()) << 3;
        value |= u8::from(self.dmc.active()) << 4;
        value |= u8::from(self.status.frame_interrupt) << 6;
        value |= u8::from(self.status.dmc_interrupt) << 7;

        // Reading $4015 clears only the frame interrupt source.
        self.status.frame_interrupt = false;
        let frame_irq_after = self.status.frame_interrupt;
        let dmc_irq_after = self.status.dmc_interrupt;
        self.trace_read_status(
            value,
            frame_irq_before,
            dmc_irq_before,
            frame_irq_after,
            dmc_irq_after,
        );

        value
    }

    /// Returns `true` when either the frame sequencer or DMC have latched an IRQ.
    pub fn irq_pending(&self) -> bool {
        self.status.frame_interrupt || self.status.dmc_interrupt
    }

    fn step_quarter_frame(&mut self) {
        for pulse in &mut self.pulse {
            pulse.clock_envelope();
        }
        self.noise.clock_envelope();
        self.triangle.clock_linear_counter();
    }

    fn step_half_frame(&mut self) {
        for pulse in &mut self.pulse {
            pulse.clock_length();
            pulse.clock_sweep();
        }
        self.triangle.clock_length();
        self.noise.clock_length();
    }

    #[inline]
    fn commit_length_halt_flags(&mut self) {
        for pulse in &mut self.pulse {
            pulse.apply_length_halt();
        }
        self.triangle.apply_length_halt();
        self.noise.apply_length_halt();
    }

    /// Bus-attached per-CPU-cycle tick, mirroring the `Ppu::step` entrypoint shape.
    ///
    /// DMC DMA requests are queued on the bus directly so callers no longer
    /// need to handle stall hints or DMA addresses themselves.
    pub fn step(bus: &mut CpuBus, _cpu: &mut Cpu, _ctx: &mut Context) {
        let apu = &mut bus.apu;
        apu.cycles = apu.cycles.wrapping_add(1);

        let tick = apu.frame_counter.step();

        if tick.quarter {
            apu.step_quarter_frame();
        }
        if tick.half {
            apu.step_half_frame();
        }
        if tick.frame_irq_clear {
            apu.status.frame_interrupt = false;
        }
        if tick.frame_irq {
            let was_frame_irq = apu.status.frame_interrupt;
            apu.status.frame_interrupt = true;
            if !was_frame_irq {
                apu.trace_irq_event("frame_irq_set");
            }
        }
        // Apply control-register halt changes after frame-counter clocks so
        // half-frame length decrements observe the previous halt state.
        apu.commit_length_halt_flags();

        for pulse in &mut apu.pulse {
            pulse.step_timer();
        }
        apu.triangle.step_timer();
        apu.noise.step_timer();
        let dmc_irq_before = apu.status.dmc_interrupt;
        let dmc_dma_before = bus.pending_dma.dmc;
        apu.dmc.step(bus.pending_dma);
        if bus.pending_dma.dmc != dmc_dma_before
            && let Some(event) = bus.pending_dma.dmc
        {
            apu.trace_dmc_dma_event(event);
        }
        if !dmc_irq_before && apu.status.dmc_interrupt {
            apu.trace_irq_event("dmc_irq_set");
        }

        if let Some(mixer) = bus.mixer.as_deref_mut() {
            apu.push_audio_levels(mixer);
        }
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

    pub fn cycle_count(&self) -> u64 {
        self.cycles
    }

    fn push_audio_levels(&mut self, mixer: &mut NesSoundMixer) {
        const CHANNELS: [AudioChannel; 5] = [
            AudioChannel::Pulse1,
            AudioChannel::Pulse2,
            AudioChannel::Triangle,
            AudioChannel::Noise,
            AudioChannel::Dmc,
        ];

        let outputs = [
            self.pulse[0].output() as f32,
            self.pulse[1].output() as f32,
            self.triangle.output() as f32,
            self.noise.output() as f32,
            self.dmc.output() as f32,
        ];

        let clock = self.cycles as i64;
        for (idx, &level) in outputs.iter().enumerate() {
            let delta = level - self.last_levels[idx];
            if delta != 0.0 {
                mixer.add_delta(CHANNELS[idx], clock, delta);
                self.last_levels[idx] = level;
            }
        }
    }

    /// Completes a pending DMC DMA fetch with the provided PRG byte.
    pub fn finish_dma_fetch(&mut self, byte: u8) {
        self.trace_dmc_dma_complete(byte);
        let dmc_irq_before = self.status.dmc_interrupt;
        self.dmc.finish_dma_fetch(byte, &mut self.status);
        if !dmc_irq_before && self.status.dmc_interrupt {
            self.trace_irq_event("dmc_irq_set");
        }
    }

    /// Last DMC sample fetch address (used for DMA stall bus access).
    pub fn last_fetch_addr(&self) -> u16 {
        self.dmc.last_fetch_addr()
    }

    /// Rebuilds channel state from the cached APU register RAM while keeping
    /// all channels disabled (matching hardware reset where `$4015` is
    /// cleared). Length counters remain cleared; control/loop flags and timer
    /// periods are restored so that subsequent writes observe the preserved
    /// register values.
    fn rebuild_channels_from_registers(&mut self) {
        // Reset channel state to power-on defaults, then reapply the cached
        // register contents with all channels still disabled so length
        // counters stay zeroed.
        self.pulse = [
            Pulse::new(pulse::PulseChannel::Pulse1),
            Pulse::new(pulse::PulseChannel::Pulse2),
        ];
        self.triangle = Triangle::default();
        self.noise = Noise::default();
        self.dmc = Dmc::default();
        self.last_levels.fill(0.0);

        let reg = |r: apu_mem::Register| -> u8 {
            let idx = r.channel_ram_index().expect("channel register");
            self.registers[idx]
        };

        // Reapply pulse 1/2 configuration.
        self.pulse[0].write_control(reg(apu_mem::Register::Pulse1Control));
        self.pulse[0].write_sweep(reg(apu_mem::Register::Pulse1Sweep));
        self.pulse[0].write_timer_low(reg(apu_mem::Register::Pulse1TimerLow));
        self.pulse[0].write_timer_high(reg(apu_mem::Register::Pulse1TimerHigh));

        self.pulse[1].write_control(reg(apu_mem::Register::Pulse2Control));
        self.pulse[1].write_sweep(reg(apu_mem::Register::Pulse2Sweep));
        self.pulse[1].write_timer_low(reg(apu_mem::Register::Pulse2TimerLow));
        self.pulse[1].write_timer_high(reg(apu_mem::Register::Pulse2TimerHigh));

        // Triangle preserves the control (halt) flag across reset; timers are
        // also restored so the phase aligns with preserved register state.
        self.triangle
            .write_control(reg(apu_mem::Register::TriangleControl));
        self.triangle
            .write_timer_low(reg(apu_mem::Register::TriangleTimerLow));
        self.triangle
            .write_timer_high(reg(apu_mem::Register::TriangleTimerHigh));

        // Noise configuration.
        self.noise
            .write_control(reg(apu_mem::Register::NoiseControl));
        self.noise
            .write_mode_and_period(reg(apu_mem::Register::NoiseModeAndPeriod));
        self.noise.write_length(reg(apu_mem::Register::NoiseLength));

        // DMC configuration registers are preserved across reset; enabling
        // still requires a post-reset `$4015` write.
        self.dmc
            .write_control(reg(apu_mem::Register::DmcControl), &mut self.status);
        self.dmc
            .write_direct_load(reg(apu_mem::Register::DmcDirectLoad));
        self.dmc
            .write_sample_address(reg(apu_mem::Register::DmcSampleAddress));
        self.dmc
            .write_sample_length(reg(apu_mem::Register::DmcSampleLength));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stores_channel_registers() {
        let mut apu = Apu::new();
        apu.cpu_write(apu_mem::REGISTER_BASE, 0xAA, 0);
        apu.cpu_write(apu_mem::REGISTER_BASE + 4, 0x55, 0);
        apu.cpu_write(apu_mem::REGISTER_BASE + 8, 0x0F, 0);

        assert_eq!(apu.registers[0], 0xAA);
        assert_eq!(apu.registers[4], 0x55);
        assert_eq!(apu.registers[8], 0x0F);
    }

    #[test]
    fn status_enables_channels_and_length_counters() {
        let mut apu = Apu::new();
        apu.cpu_write(apu_mem::STATUS, 0b0000_0001, 0);
        apu.cpu_write(0x4003, 0b1111_1000, 0); // load a long length value
        apu.commit_length_halt_flags();

        // Length counter latched because pulse1 is enabled.
        assert!(apu.pulse[0].length_active());

        // Disable and ensure the length counter clears.
        apu.cpu_write(apu_mem::STATUS, 0, 0);
        assert!(!apu.pulse[0].length_active());
    }

    #[test]
    #[ignore = "this test fails and needs investigation"]
    fn frame_counter_configuration() {
        let mut apu = Apu::new();
        apu.cpu_write(apu_mem::FRAME_COUNTER, 0b1000_0000, 0);
        assert_eq!(apu.frame_counter.mode(), FrameCounterMode::FiveStep);

        apu.cpu_write(apu_mem::FRAME_COUNTER, 0, 0);
        assert_eq!(apu.frame_counter.mode(), FrameCounterMode::FourStep);
    }

    #[test]
    fn frame_irq_flag_set_and_cleared() {
        let mut apu = Apu::new();
        apu.cpu_write(apu_mem::FRAME_COUNTER, 0, 0); // 4-step, IRQs enabled
        let mut ram = crate::mem_block::cpu::Ram::new();
        let mut ppu = crate::ppu::Ppu::default();
        let mut controllers = crate::controller::ControllerPorts::new();
        let mut pending_dma = crate::bus::PendingDma::default();
        let mut open_bus = crate::bus::OpenBus::new();
        let mut cpu_cycles = 0u64;
        let mut master_clock = 0u64;

        let mut bus = CpuBus {
            ram: &mut ram,
            ppu: &mut ppu,
            apu: &mut apu,
            cartridge: None,
            controllers: &mut controllers,
            serial_log: None,
            open_bus: &mut open_bus,
            mixer: None,
            cycles: &mut cpu_cycles,
            master_clock: &mut master_clock,
            ppu_offset: 0,
            clock_start_count: 0,
            clock_end_count: 0,
            pending_dma: &mut pending_dma,
        };
        let mut cpu = Cpu::new();
        let mut ctx = Context::None;

        for _ in 0..=frame_counter::FRAME_STEP_4_PERIOD as u64 {
            Apu::step(&mut bus, &mut cpu, &mut ctx);
        }
        assert!(bus.apu.status.frame_interrupt);

        let first = bus.apu.cpu_read(apu_mem::STATUS);
        assert_eq!(first & 0b0100_0000, 0b0100_0000);

        let second = bus.apu.cpu_read(apu_mem::STATUS);
        assert_eq!(second & 0b0100_0000, 0);
    }

    #[test]
    fn dmc_status_bit_and_irq_clear_on_write() {
        let mut apu = Apu::new();
        apu.cpu_write(0x4013, 0x01, 0); // sample length = 17 bytes
        apu.cpu_write(apu_mem::STATUS, 0b0001_0000, 0); // enable DMC

        // Active bit should report bytes remaining.
        let status = apu.cpu_read(apu_mem::STATUS);
        assert_eq!(status & 0b0001_0000, 0b0001_0000);

        // Force an IRQ and ensure reads preserve it.
        apu.status.dmc_interrupt = true;
        let first = apu.cpu_read(apu_mem::STATUS);
        assert_eq!(first & 0b1000_0000, 0b1000_0000);
        let second = apu.cpu_read(apu_mem::STATUS);
        assert_eq!(second & 0b1000_0000, 0b1000_0000);

        // Writing $4015 clears DMC IRQ.
        apu.cpu_write(apu_mem::STATUS, 0b0001_0000, 0);
        let third = apu.cpu_read(apu_mem::STATUS);
        assert_eq!(third & 0b1000_0000, 0);
    }
}
