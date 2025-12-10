use std::{path::Path, u64};

use crate::{
    apu::Apu,
    audio::{AudioChannel, CPU_CLOCK_NTSC, NesSoundMixer, SoundMixerBus, bus::AudioBusConfig},
    bus::{Bus, OpenBus, cpu::CpuBus},
    cartridge::{Cartridge, Provider},
    config::region::Region,
    controller::{Button, Controller},
    cpu::Cpu,
    error::Error,
    mem_block::cpu as cpu_ram,
    ppu::{
        Ppu,
        buffer::{ColorFormat, FrameBuffer},
        nmi_debug_state::NmiDebugState,
        palette::{Palette, PaletteKind},
        sprite0_hit_debug::Sprite0HitDebug,
    },
    reset_kind::ResetKind,
};

pub mod apu;
pub mod audio;
pub mod bus;
pub mod cartridge;
pub mod config;
pub mod controller;
pub mod cpu;
pub mod error;
pub mod mem_block;
pub mod memory;
pub mod ppu;
pub mod reset_kind;
pub mod state;

pub use cpu::CpuSnapshot;

#[derive(Debug)]
pub struct Nes {
    pub cpu: Cpu,
    pub ppu: Ppu,
    apu: Apu,
    ram: cpu_ram::Ram,
    cartridge: Option<Cartridge>,
    mapper_provider: Option<Box<dyn Provider>>,
    controllers: [Controller; 2],
    last_frame: u32,
    /// Master PPU dot counter used to drive CPU/PPU/APU in lockstep (3 dots per CPU cycle).
    dot_counter: u64,
    /// Master clock in PPU ticks (4 master clocks per PPU dot, 12 per CPU cycle).
    master_clock: u64,
    /// Phase offset between CPU and PPU master clocks (mirrors Mesen2's default of 1).
    ppu_offset: u8,
    /// Start/End half-cycle lengths in master clocks (NTSC defaults to 6).
    clock_start_count: u8,
    clock_end_count: u8,
    serial_log: controller::SerialLogger,
    /// Pending OAM DMA page written via `$4014` (latched until CPU picks it up).
    oam_dma_request: Option<u8>,
    /// CPU data-bus open-bus latch (mirrors Mesen2's decay model).
    open_bus: OpenBus,
    /// CPU bus access counter (fed into timing-sensitive mappers).
    cycles: u64,
    /// Per-console mixer that produces band-limited PCM at a fixed internal rate.
    mixer: NesSoundMixer,
    /// Global audio bus that resamples from the internal mixer rate to the host sample rate.
    sound_bus: SoundMixerBus,
    audio_sample_rate: u32,
    /// Scratch buffer for a single frame of internal-rate audio from the per-console mixer.
    mixer_frame_buffer: Vec<f32>,
    region: Region,
}

/// Internal mixer output sample rate (matches Mesen2's fixed 96 kHz path).
const INTERNAL_MIXER_SAMPLE_RATE: u32 = 96_000;

impl Nes {
    /// Constructs a powered-on NES instance with cleared RAM and default palette.
    pub fn new(format: ColorFormat) -> Self {
        Self::new_with_sample_rate(format, 48_000)
    }

    /// Constructs a powered-on NES instance with a specified audio sample rate.
    pub fn new_with_sample_rate(format: ColorFormat, sample_rate: u32) -> Self {
        let buffer = FrameBuffer::new_color(format);
        Self::new_with_framebuffer_and_sample_rate(buffer, sample_rate)
    }

    /// Constructs a powered-on NES instance using an explicit framebuffer configuration.
    pub fn new_with_framebuffer(buffer: FrameBuffer) -> Self {
        Self::new_with_framebuffer_and_sample_rate(buffer, 48_000)
    }

    /// Constructs a powered-on NES instance with a provided framebuffer and sample rate.
    pub fn new_with_framebuffer_and_sample_rate(buffer: FrameBuffer, sample_rate: u32) -> Self {
        let mut nes = Self {
            cpu: Cpu::new(),
            ppu: Ppu::new(buffer),
            apu: Apu::new(),
            ram: cpu_ram::Ram::new(),
            cartridge: None,
            mapper_provider: None,
            controllers: [Controller::new(), Controller::new()],
            last_frame: 0,
            dot_counter: 0,
            master_clock: 0,
            ppu_offset: 1,
            clock_start_count: 6,
            clock_end_count: 6,
            serial_log: controller::SerialLogger::default(),
            oam_dma_request: None,
            open_bus: OpenBus::new(),
            cycles: u64::MAX,
            mixer: NesSoundMixer::new(CPU_CLOCK_NTSC, INTERNAL_MIXER_SAMPLE_RATE),
            sound_bus: SoundMixerBus::new(INTERNAL_MIXER_SAMPLE_RATE, sample_rate),
            audio_sample_rate: sample_rate,
            mixer_frame_buffer: Vec::new(),
            region: Region::Auto,
        };
        nes.ppu.set_palette(PaletteKind::NesdevNtsc.palette());
        // Apply a power-on style reset once at construction time. This matches
        // the console being turned on from a cold state (CPU/PPU/APU and RAM
        // cleared) and is distinct from subsequent warm resets triggered by
        // the user pressing the reset button.
        nes.reset(ResetKind::PowerOn); // TODO: Should I remove this? Because insert cartridge also reset the console.
        nes
    }

    /// Replaces the mapper provider used when loading cartridges.
    pub fn set_mapper_provider(&mut self, provider: Option<Box<dyn Provider>>) {
        self.mapper_provider = provider;
    }

    /// Current mapper provider, when one has been supplied.
    pub fn mapper_provider(&self) -> Option<&dyn Provider> {
        self.mapper_provider.as_deref()
    }

    /// Loads a cartridge from disk, inserts it, and performs a reset sequence.
    pub fn load_cartridge_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        let cartridge = cartridge::load_cartridge_from_file_with_provider(
            path,
            self.mapper_provider.as_deref(),
        )?;
        self.insert_cartridge(cartridge);
        Ok(())
    }

    /// Inserts a cartridge that has already been constructed.
    pub fn insert_cartridge(&mut self, cartridge: Cartridge) {
        self.ppu.attach_cartridge(&cartridge);
        self.cartridge = Some(cartridge);
        // Inserting a new cartridge is effectively a power cycle for the
        // console, so apply a full power-on reset rather than a warm reset.
        self.reset(ResetKind::PowerOn);
    }

    /// Ejects the currently inserted cartridge and resets the system.
    pub fn eject_cartridge(&mut self) {
        self.cartridge = None;
        // Treat cartridge removal as a full power cycle from the core's
        // perspective so all state, including RAM, returns to power-on.
        self.reset(ResetKind::PowerOn);
    }

    /// Internal helper that applies either a power-on style reset or a warm reset
    /// depending on `kind`. This drives CPU/PPU/APU, RAM, mixer, and mapper state
    /// in a way that mirrors Mesen2's reset sequencing.
    pub fn reset(&mut self, kind: ResetKind) {
        match kind {
            ResetKind::PowerOn => {
                // Full console power cycle: clear CPU RAM, fully reinitialize APU
                // and cartridge, and treat this as a cold boot.
                self.ram.fill(0);
                self.ppu.reset(kind);
                self.apu.reset(kind);
                if let Some(cart) = self.cartridge.as_mut() {
                    cart.power_on();
                }
            }
            ResetKind::Soft => {
                // Warm reset: preserve CPU RAM contents but reset PPU/APU and
                // mapper-visible state. This is what reset-sensitive test ROMs
                // like blargg's `apu_reset` expect.
                self.ppu.reset(kind);
                self.apu.reset(kind);
                if let Some(cart) = self.cartridge.as_mut() {
                    cart.reset();
                }
            }
        }

        // State that is common to both reset kinds.
        self.serial_log.drain();
        self.open_bus.reset();
        self.cycles = u64::MAX;
        self.mixer.reset();
        self.sound_bus.reset();
        self.mixer_frame_buffer.clear();
        self.master_clock = (self.clock_start_count + self.clock_end_count) as u64;

        // Wire up a temporary CPU bus and run the CPU reset sequence, passing
        // down the reset kind so the CPU can distinguish power-on vs soft
        // reset semantics (register init vs preserving A/X/Y and PS).
        let mut bus = CpuBus::new(
            &mut self.ram,
            &mut self.ppu,
            &mut self.apu,
            self.cartridge.as_mut(),
            &mut self.controllers,
            Some(&mut self.serial_log),
            &mut self.oam_dma_request,
            &mut self.open_bus,
            // On power-on we allow the CPU reset sequence to feed the mixer; for
            // warm resets this is omitted, matching the previous behaviour.
            if matches!(kind, ResetKind::PowerOn) {
                Some(&mut self.mixer)
            } else {
                None
            },
            &mut self.cycles,
            &mut self.master_clock,
            self.ppu_offset,
            self.clock_start_count,
            self.clock_end_count,
        );
        self.cpu.reset(&mut bus, kind);
        self.last_frame = bus.ppu().frame_count();
        self.dot_counter = 0;
    }

    pub fn clock_cpu_cycle(&mut self, audio: bool) -> ClockResult {
        let frame_before = self.ppu.frame_count();
        let apu_clocked = true;
        let (cpu_cycles, expansion_samples, opcode_active) = {
            let mut bus = CpuBus::new(
                &mut self.ram,
                &mut self.ppu,
                &mut self.apu,
                self.cartridge.as_mut(),
                &mut self.controllers,
                Some(&mut self.serial_log),
                &mut self.oam_dma_request,
                &mut self.open_bus,
                if audio { Some(&mut self.mixer) } else { None },
                &mut self.cycles,
                &mut self.master_clock,
                self.ppu_offset,
                self.clock_start_count,
                self.clock_end_count,
            );

            // Advance one CPU cycle (and implicitly PPU/APU).
            self.cpu.clock(&mut bus);
            let cycles = bus.cpu_cycles();
            let expansion = bus
                .cartridge()
                .and_then(|cart| cart.mapper().as_expansion_audio())
                .map(|exp| exp.samples());

            if let Some((stall_cycles, dma_addr)) = bus.take_pending_dmc_stall()
                && stall_cycles > 0
                && self.apply_dmc_stall(stall_cycles, dma_addr)
            {
                // Stall may advance PPU/frame; accounted for below.
            }

            let opcode_active = self.cpu_opcode_active();
            (cycles, expansion, opcode_active)
        };

        self.dot_counter = self.ppu.total_dots();

        // Feed expansion audio into the mixer at the CPU clock edge.
        if apu_clocked && let Some(samples) = expansion_samples {
            let clock = cpu_cycles as i64;
            self.mixer
                .set_channel_level(AudioChannel::Fds, clock, samples.fds);
            self.mixer
                .set_channel_level(AudioChannel::Mmc5, clock, samples.mmc5);
            self.mixer
                .set_channel_level(AudioChannel::Namco163, clock, samples.namco163);
            self.mixer
                .set_channel_level(AudioChannel::Sunsoft5B, clock, samples.sunsoft5b);
            self.mixer
                .set_channel_level(AudioChannel::Vrc6, clock, samples.vrc6);
            self.mixer
                .set_channel_level(AudioChannel::Vrc7, clock, samples.vrc7);
        }

        let frame_after = self.ppu.frame_count();
        self.last_frame = frame_after;

        ClockResult {
            frame_advanced: frame_after != frame_before,
            apu_clocked,
            opcode_active,
        }
    }

    /// Runs CPU/PPU/APU ticks until the PPU completes the next frame.
    pub fn run_frame(&mut self, audio: bool) -> Vec<f32> {
        let mut samples = vec![];
        let target_frame = self.ppu.frame_count().wrapping_add(1);
        while self.ppu.frame_count() < target_frame {
            let _ = self.clock_cpu_cycle(audio);
        }
        let end_clock = self.apu_cycles() as i64;
        self.mixer_frame_buffer.clear();
        self.mixer
            .end_frame(end_clock, &mut self.mixer_frame_buffer);

        // Optional debug path: dump the internal 96 kHz mixer output to a raw
        // float file for waveform comparison against Mesen2. This writes a
        // small header once, followed by interleaved f32 stereo samples:
        //
        //   magic: [u8;4] = b\"APU0\"
        //   sample_rate: u32 (little-endian) = 96_000
        //   channels: u16 (little-endian) = 2
        //   reserved: u16 (little-endian) = 0
        //   then repeated { left: f32 LE, right: f32 LE } samples.
        //
        // The dump is limited to roughly 60 seconds to avoid unbounded files.
        // const DEBUG_MAX_SECONDS: u64 = 60;
        // let max_debug_frames = INTERNAL_MIXER_SAMPLE_RATE as u64 * DEBUG_MAX_SECONDS;
        // if self.debug_apu_frames_written < max_debug_frames {
        //     if self.debug_apu_dump.is_none() {
        //         if let Ok(mut file) = File::create("apu_debug.raw") {
        //             let _ = file.write_all(b"APU0");
        //             let _ = file.write_all(&INTERNAL_MIXER_SAMPLE_RATE.to_le_bytes());
        //             let channels: u16 = 2;
        //             let _ = file.write_all(&channels.to_le_bytes());
        //             let reserved: u16 = 0;
        //             let _ = file.write_all(&reserved.to_le_bytes());
        //             self.debug_apu_dump = Some(file);
        //         }
        //     }
        //     if let Some(file) = self.debug_apu_dump.as_mut() {
        //         let mut buf = Vec::with_capacity(self.mixer_frame_buffer.len() * 4);
        //         for &s in &self.mixer_frame_buffer {
        //             buf.extend_from_slice(&s.to_le_bytes());
        //         }
        //         let _ = file.write_all(&buf);
        //         self.debug_apu_frames_written += (self.mixer_frame_buffer.len() / 2) as u64;
        //     }
        // }

        self.sound_bus
            .mix_frame(&[&self.mixer_frame_buffer], &mut samples);

        // Optional debug path: dump the post-bus PCM at the host sample rate
        // (after resampling/EQ/reverb/crossfeed/master volume) to a raw float
        // file. The format matches `apu_debug.raw`, only the sample rate
        // differs:
        //
        //   magic: [u8;4] = b\"APU0\"
        //   sample_rate: u32 (little-endian) = self.audio_sample_rate()
        //   channels: u16 (little-endian) = 2
        //   reserved: u16 (little-endian) = 0
        //   then repeated { left: f32 LE, right: f32 LE } samples.
        //
        // Also limited to ~60s to keep file size reasonable.
        // const DEBUG_BUS_MAX_SECONDS: u64 = 60;
        // let max_bus_frames = self.audio_sample_rate as u64 * DEBUG_BUS_MAX_SECONDS;
        // let new_samples = &out[out_start..];
        // let new_frames = (new_samples.len() / 2) as u64;
        // if self.debug_bus_frames_written < max_bus_frames && new_frames > 0 {
        //     if self.debug_bus_dump.is_none() {
        //         if let Ok(mut file) = File::create("bus_debug.raw") {
        //             let _ = file.write_all(b"APU0");
        //             let _ = file.write_all(&self.audio_sample_rate.to_le_bytes());
        //             let channels: u16 = 2;
        //             let _ = file.write_all(&channels.to_le_bytes());
        //             let reserved: u16 = 0;
        //             let _ = file.write_all(&reserved.to_le_bytes());
        //             self.debug_bus_dump = Some(file);
        //         }
        //     }
        //     if let Some(file) = self.debug_bus_dump.as_mut() {
        //         // Clamp to remaining budget.
        //         let frames_to_write =
        //             (max_bus_frames - self.debug_bus_frames_written).min(new_frames) as usize;
        //         let samples_to_write = frames_to_write * 2;
        //         let slice = &new_samples[..samples_to_write];
        //         let mut buf = Vec::with_capacity(slice.len() * 4);
        //         for &s in slice {
        //             buf.extend_from_slice(&s.to_le_bytes());
        //         }
        //         let _ = file.write_all(&buf);
        //         self.debug_bus_frames_written += frames_to_write as u64;
        //     }
        // }

        self.last_frame = self.ppu.frame_count();
        self.dot_counter = self.ppu.total_dots();
        samples
    }

    /// Latest audio sample from the APU mixer plus any cartridge expansion audio.
    pub fn audio_sample(&self) -> f32 {
        let base = self.apu.sample();
        let expansion = self
            .cartridge
            .as_ref()
            .and_then(|cart| cart.mapper().as_expansion_audio())
            .map(|exp| exp.samples())
            .unwrap_or_default();

        // Collapse expansion into a single scalar matching the mixer weights.
        let expansion_mono = expansion.fds
            + expansion.mmc5
            + expansion.namco163
            + expansion.sunsoft5b
            + expansion.vrc6
            + expansion.vrc7;

        base + expansion_mono
    }

    /// Update the mixer to a new host sample rate (resets mixer state).
    pub fn set_audio_sample_rate(&mut self, sample_rate: u32) {
        self.audio_sample_rate = sample_rate;
        self.sound_bus.set_output_rate(sample_rate);
    }

    /// Apply per-channel mixer settings (volume / panning).
    pub fn set_mixer_settings(&mut self, settings: &crate::audio::MixerSettings) {
        self.mixer.apply_mixer_settings(settings);
    }

    /// Current audio sample rate used by the internal mixer.
    pub fn audio_sample_rate(&self) -> u32 {
        self.audio_sample_rate
    }

    /// Updates the audio bus configuration (master volume and attenuation).
    pub fn set_audio_bus_config(&mut self, config: AudioBusConfig) {
        self.sound_bus.set_config(config);
    }

    /// Current APU cycle counter (CPU-rate ticks since power-on/reset).
    pub fn apu_cycles(&self) -> u64 {
        self.apu.cycle_count()
    }

    /// Palette indices for the latest frame (PPU native format).
    pub fn render_buffer(&self) -> &[u8] {
        self.ppu.render_buffer()
    }

    /// Selects one of the built-in palettes.
    pub fn set_palette(&mut self, palette: Palette) {
        self.ppu.set_palette(palette);
    }

    /// Active palette reference.
    pub fn palette(&self) -> &Palette {
        self.ppu.palette()
    }

    /// Updates the pressed state of a controller button (0 = port 1).
    pub fn set_button(&mut self, pad: usize, button: Button, pressed: bool) {
        if let Some(ctrl) = self.controllers.get_mut(pad) {
            ctrl.set_button(button, pressed);
        }
    }

    /// Snapshot of the current CPU registers for tracing/debugging.
    pub fn cpu_snapshot(&self) -> CpuSnapshot {
        self.cpu.snapshot()
    }

    /// Returns `true` when the CPU is mid-instruction (opcode + micro-ops still in flight).
    pub fn cpu_opcode_active(&self) -> bool {
        self.cpu.opcode_active()
    }

    /// Internal timing counter (PPU dots since power-on). Exposed for tests/debug.
    pub fn dot_counter(&self) -> u64 {
        self.dot_counter
    }

    /// Peek first sprite-0 hit position for the current frame (debug).
    pub fn sprite0_hit_pos(&self) -> Option<Sprite0HitDebug> {
        self.ppu.sprite0_hit_pos()
    }

    /// Peek PPU NMI flags and position (tests/debug).
    pub fn ppu_nmi_debug(&self) -> NmiDebugState {
        self.ppu.debug_nmi_state()
    }

    /// Debug-only: override PPU counters for trace alignment.
    pub fn debug_set_ppu_position(&mut self, scanline: i16, cycle: u16, frame: u32) {
        self.ppu.debug_set_position(scanline, cycle, frame);
    }

    /// Executes the next instruction (advancing CPU/PPU/APU as needed).
    pub fn step_instruction(&mut self) {
        let mut seen_active = false;
        loop {
            self.clock_cpu_cycle(false);
            if self.cpu.opcode_active() {
                seen_active = true;
            } else if seen_active {
                break;
            }
        }
    }

    /// Forces the CPU registers to the provided snapshot (clears in-flight opcode).
    pub fn set_cpu_snapshot(&mut self, snapshot: CpuSnapshot) {
        self.cpu.load_snapshot(snapshot);
    }

    /// Reads a byte from the CPU address space without mutating CPU state.
    pub fn peek_cpu_byte(&mut self, addr: u16) -> u8 {
        let mut bus = CpuBus::new(
            &mut self.ram,
            &mut self.ppu,
            &mut self.apu,
            self.cartridge.as_mut(),
            &mut self.controllers,
            Some(&mut self.serial_log),
            &mut self.oam_dma_request,
            &mut self.open_bus,
            None,
            &mut self.cycles,
            &mut self.master_clock,
            self.ppu_offset,
            self.clock_start_count,
            self.clock_end_count,
        );
        bus.peek(&mut self.cpu, addr)
    }

    /// Reads a contiguous range of CPU-visible bytes into `buffer`, starting at `base`.
    pub fn peek_cpu_slice(&mut self, base: u16, buffer: &mut [u8]) {
        let mut bus = CpuBus::new(
            &mut self.ram,
            &mut self.ppu,
            &mut self.apu,
            self.cartridge.as_mut(),
            &mut self.controllers,
            Some(&mut self.serial_log),
            &mut self.oam_dma_request,
            &mut self.open_bus,
            None,
            &mut self.cycles,
            &mut self.master_clock,
            self.ppu_offset,
            self.clock_start_count,
            self.clock_end_count,
        );
        for (offset, byte) in buffer.iter_mut().enumerate() {
            *byte = bus.peek(&mut self.cpu, base.wrapping_add(offset as u16));
        }
    }

    /// Drains any bytes emitted on controller port 1 via the blargg serial protocol.
    pub fn take_serial_output(&mut self) -> Vec<u8> {
        self.serial_log.drain()
    }

    /// DMC DMA stall: freeze CPU for the specified cycles and perform a single
    /// PRG read so mappers can observe the DMA.
    ///
    /// Each DMC DMA is 4 CPU cycles; if initiated on an odd cycle an extra
    /// cycle precedes the DMA to align to even. Only one PRG read occurs per
    /// DMA (not per stall cycle).
    fn apply_dmc_stall(&mut self, stall_cycles: u8, dma_addr: Option<u16>) -> bool {
        if stall_cycles == 0 {
            return false;
        }

        // Helper to advance one CPU cycle (with optional PRG read) and keep the
        // PPU in lockstep. The bus read path increments `cpu_bus_cycle`
        // internally; idle cycles advance it explicitly and still clock the
        // mapper/open-bus decay to match hardware.
        let mut frame_advanced = false;
        let run_cycle = |nes: &mut Nes, read_addr: Option<u16>, frame_advanced: &mut bool| {
            if let Some(addr) = read_addr {
                let byte = {
                    let mut bus = CpuBus::new(
                        &mut nes.ram,
                        &mut nes.ppu,
                        &mut nes.apu,
                        nes.cartridge.as_mut(),
                        &mut nes.controllers,
                        Some(&mut nes.serial_log),
                        &mut nes.oam_dma_request,
                        &mut nes.open_bus,
                        None,
                        &mut nes.cycles,
                        &mut nes.master_clock,
                        nes.ppu_offset,
                        nes.clock_start_count,
                        nes.clock_end_count,
                    );
                    bus.mem_read(&mut nes.cpu, addr)
                };
                nes.apu.finish_dma_fetch(byte);
                nes.open_bus.latch(byte);
            } else {
                nes.cycles = nes.cycles.wrapping_add(1);
                if let Some(cart) = nes.cartridge.as_mut() {
                    cart.cpu_clock(nes.cycles);
                }
                nes.open_bus.step();
            }
            // Even though the CPU core is stalled, advance its cycle
            // counter (and pause any in-progress OAM DMA) so alignment/parity
            // stays correct.
            nes.cpu.account_dma_cycle();

            // Advance three PPU dots (one CPU cycle worth) to keep alignment.
            for _ in 0..3 {
                let mut bus = CpuBus::new(
                    &mut nes.ram,
                    &mut nes.ppu,
                    &mut nes.apu,
                    nes.cartridge.as_mut(),
                    &mut nes.controllers,
                    Some(&mut nes.serial_log),
                    &mut nes.oam_dma_request,
                    &mut nes.open_bus,
                    None,
                    &mut nes.cycles,
                    &mut nes.master_clock,
                    nes.ppu_offset,
                    nes.clock_start_count,
                    nes.clock_end_count,
                );
                bus.clock_ppu();
            }
            nes.dot_counter = nes.ppu.total_dots();
            let frame_count = nes.ppu.frame_count();
            if frame_count != nes.last_frame {
                nes.last_frame = frame_count;
                *frame_advanced = true;
            }
        };

        // Align to even CPU cycle when requested stall is non-zero. The first
        // cycle is idle when starting on an odd CPU cycle; the PRG read occurs
        // on the following (even) cycle.
        if (self.cycles & 1) == 1 {
            run_cycle(self, None, &mut frame_advanced);
        }

        // Apply the DMA cycles; the PRG read is performed on the first DMA
        // cycle only.
        for i in 0..stall_cycles {
            let read_addr = if i == 0 { dma_addr } else { None };
            run_cycle(self, read_addr, &mut frame_advanced);
        }

        frame_advanced
    }
}

/// Result of a single dot tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockResult {
    pub frame_advanced: bool,
    pub apu_clocked: bool,
    pub opcode_active: bool,
}

impl Default for Nes {
    fn default() -> Self {
        Self::new(ColorFormat::Rgb555)
    }
}

#[cfg(test)]
mod tests {
    use ctor::ctor;
    use tracing::Level;
    use tracing_subscriber::FmtSubscriber;

    pub(crate) const TEST_COUNT: usize = 1000;

    #[ctor]
    fn init_tracing() {
        let subscriber = FmtSubscriber::builder()
            .with_file(true)
            .with_line_number(true)
            .with_max_level(Level::DEBUG)
            .pretty()
            .finish();
        tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
    }
}
