use core::ffi::c_void;
use std::path::Path;

use crate::{
    apu::Apu,
    audio::{AudioChannel, CPU_CLOCK_NTSC, NesSoundMixer, SoundMixerBus, bus::AudioBusConfig},
    bus::{OpenBus, PendingDma, cpu::CpuBus},
    cartridge::{Cartridge, Provider},
    config::region::Region,
    context::Context,
    controller::{Button, ControllerPorts},
    cpu::Cpu,
    error::Error,
    interceptor::sprite_interceptor::{SpriteInterceptor, SpriteSnapshot},
    interceptor::tile_viewer_interceptor::{TileViewerInterceptor, TileViewerSnapshot},
    interceptor::tilemap_interceptor::{TilemapInterceptor, TilemapSnapshot},
    interceptor::{EmuInterceptor, log_interceptor::LogInterceptor},
    mem_block::cpu as cpu_ram,
    ppu::{
        Ppu,
        buffer::{ColorFormat, FrameBuffer, FrameReadyCallback},
        palette::{Palette, PaletteKind},
    },
    reset_kind::ResetKind,
};

pub mod apu;
pub mod audio;
pub mod bus;
pub mod cartridge;
pub mod config;
pub mod context;
pub mod controller;
pub mod cpu;
pub mod error;
pub mod interceptor;
pub mod mem_block;
pub mod memory;
pub mod ppu;
pub mod reset_kind;
pub mod rng;
pub mod state;

pub use cpu::CpuSnapshot;

#[derive(Debug)]
pub struct Nes {
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub apu: Apu,
    ram: cpu_ram::Ram,
    cartridge: Option<Cartridge>,
    mapper_provider: Option<Box<dyn Provider>>,
    pub controllers: ControllerPorts,
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
    pending_dma: PendingDma,
    /// CPU data-bus open-bus latch (external/internal, Mesen2-style).
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
    pub region: Region,
    pub interceptor: EmuInterceptor,
}

/// Internal mixer output sample rate (matches Mesen2's fixed 96 kHz path).
const INTERNAL_MIXER_SAMPLE_RATE: u32 = 96_000;

/// Builder for configuring and constructing a powered-on NES instance.
///
/// This is primarily a readability/ergonomics helper to avoid long constructor
/// argument chains and to make defaults explicit.
#[derive(Debug)]
pub struct NesBuilder {
    format: Option<ColorFormat>,
    framebuffer: Option<FrameBuffer>,
    sample_rate: u32,
    region: Region,
    interceptor: Option<EmuInterceptor>,
    power_on_reset: bool,
}

impl Default for NesBuilder {
    fn default() -> Self {
        Self {
            format: Some(ColorFormat::Rgb555),
            framebuffer: None,
            sample_rate: 48_000,
            region: Region::Auto,
            interceptor: None,
            power_on_reset: true,
        }
    }
}

impl NesBuilder {
    /// Creates a builder with sensible defaults:
    /// - RGB555 framebuffer
    /// - 48 kHz host audio
    /// - Region::Auto
    /// - power-on style reset performed once after construction
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the framebuffer color format (ignored if `framebuffer()` is provided).
    pub fn format(mut self, format: ColorFormat) -> Self {
        self.format = Some(format);
        self
    }

    /// Provides an explicit framebuffer configuration.
    pub fn framebuffer(mut self, framebuffer: FrameBuffer) -> Self {
        self.framebuffer = Some(framebuffer);
        self
    }

    /// Sets the host audio sample rate.
    pub fn sample_rate(mut self, sample_rate: u32) -> Self {
        self.sample_rate = sample_rate;
        self
    }

    /// Sets the region selection.
    pub fn region(mut self, region: Region) -> Self {
        self.region = region;
        self
    }

    /// Overrides the default interceptor stack.
    pub fn interceptor(mut self, interceptor: EmuInterceptor) -> Self {
        self.interceptor = Some(interceptor);
        self
    }

    /// Enables/disables the initial power-on reset performed after construction.
    ///
    /// Most frontends want this enabled. Tests or special setups may disable it.
    pub fn power_on_reset(mut self, enabled: bool) -> Self {
        self.power_on_reset = enabled;
        self
    }

    /// Builds the NES instance.
    pub fn build(self) -> Nes {
        let buffer = match self.framebuffer {
            Some(buf) => buf,
            None => {
                let format = self.format.unwrap_or(ColorFormat::Rgb555);
                FrameBuffer::new(format)
            }
        };

        let interceptor = self.interceptor.unwrap_or_else(Nes::build_interceptor);

        let mut nes = Nes {
            cpu: Cpu::new(),
            ppu: Ppu::new(buffer),
            apu: Apu::new(),
            ram: cpu_ram::Ram::new(),
            cartridge: None,
            mapper_provider: None,
            controllers: ControllerPorts::new(),
            last_frame: 0,
            dot_counter: 0,
            master_clock: 0,
            ppu_offset: 1,
            clock_start_count: 6,
            clock_end_count: 6,
            serial_log: controller::SerialLogger::default(),
            pending_dma: PendingDma::default(),
            open_bus: OpenBus::new(),
            cycles: u64::MAX,
            mixer: NesSoundMixer::new(CPU_CLOCK_NTSC, INTERNAL_MIXER_SAMPLE_RATE),
            sound_bus: SoundMixerBus::new(INTERNAL_MIXER_SAMPLE_RATE, self.sample_rate),
            audio_sample_rate: self.sample_rate,
            mixer_frame_buffer: Vec::new(),
            region: self.region,
            interceptor,
        };

        nes.ppu.set_palette(PaletteKind::NesdevNtsc.palette());

        if self.power_on_reset {
            // Apply a power-on style reset once at construction time. This matches
            // the console being turned on from a cold state.
            nes.reset(ResetKind::PowerOn);
        }

        nes
    }
}

macro_rules! nes_cpu_bus {
    ($nes:ident, mixer: $with_mixer:expr, serial: $with_serial:expr) => {{
        let __with_mixer = $with_mixer;
        let __with_serial = $with_serial;
        let __serial_log = if __with_serial {
            Some(&mut $nes.serial_log)
        } else {
            None
        };
        let __mixer = if __with_mixer {
            Some(&mut $nes.mixer)
        } else {
            None
        };
        CpuBus {
            ram: &mut $nes.ram,
            ppu: &mut $nes.ppu,
            apu: &mut $nes.apu,
            cartridge: $nes.cartridge.as_mut(),
            controllers: &mut $nes.controllers,
            serial_log: __serial_log,
            open_bus: &mut $nes.open_bus,
            mixer: __mixer,
            cycles: &mut $nes.cycles,
            master_clock: &mut $nes.master_clock,
            ppu_offset: $nes.ppu_offset,
            clock_start_count: $nes.clock_start_count,
            clock_end_count: $nes.clock_end_count,
            pending_dma: &mut $nes.pending_dma,
        }
    }};
}

impl Nes {
    /// Creates a [`NesBuilder`] with defaults.
    pub fn builder() -> NesBuilder {
        NesBuilder::new()
    }
    /// Constructs a powered-on NES instance with cleared RAM and default palette.
    pub fn new(format: ColorFormat) -> Self {
        Self::builder().format(format).build()
    }

    /// Constructs a powered-on NES instance with a specified audio sample rate.
    pub fn new_with_sample_rate(format: ColorFormat, sample_rate: u32) -> Self {
        Self::builder()
            .format(format)
            .sample_rate(sample_rate)
            .build()
    }

    /// Constructs a powered-on NES instance using an explicit framebuffer configuration.
    pub fn new_with_framebuffer(buffer: FrameBuffer) -> Self {
        Self::builder().framebuffer(buffer).build()
    }

    /// Constructs a powered-on NES instance with a provided framebuffer and sample rate.
    pub fn new_with_framebuffer_and_sample_rate(buffer: FrameBuffer, sample_rate: u32) -> Self {
        Self::builder()
            .framebuffer(buffer)
            .sample_rate(sample_rate)
            .build()
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

    pub fn get_cartridge(&self) -> Option<&Cartridge> {
        self.cartridge.as_ref()
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
                    cart.reset(kind);
                }
            }
            ResetKind::Soft => {
                // Warm reset: preserve CPU RAM contents but reset PPU/APU and
                // mapper-visible state. This is what reset-sensitive test ROMs
                // like blargg's `apu_reset` expect.
                self.ppu.reset(kind);
                self.apu.reset(kind);
                if let Some(cart) = self.cartridge.as_mut() {
                    cart.reset(kind);
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
        let mut bus = nes_cpu_bus!(
            self,
            mixer: matches!(kind, ResetKind::PowerOn),
            serial: true
        );
        let mut ctx = Context::Some {
            interceptor: &mut self.interceptor,
        };
        self.cpu.reset(&mut bus, kind, &mut ctx);
        self.last_frame = bus.devices().ppu.frame_count();
        self.dot_counter = 0;
    }

    pub fn step_cpu_cycle(&mut self, emit_audio: bool) -> ClockResult {
        let frame_before = self.ppu.frame_count();
        let apu_clocked = true;
        let (cpu_cycles, expansion_samples, opcode_active) = {
            let mut bus = nes_cpu_bus!(self, mixer: emit_audio, serial: true);
            let mut ctx = Context::Some {
                interceptor: &mut self.interceptor,
            };

            // Advance one CPU cycle (and implicitly PPU/APU).
            self.cpu.step(&mut bus, &mut ctx);
            let cycles = bus.cpu_cycles();
            let expansion = bus
                .cartridge()
                .and_then(|cart| cart.mapper().as_expansion_audio())
                .map(|exp| exp.samples());

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
    pub fn run_frame(&mut self, emit_audio: bool) -> Vec<f32> {
        let mut samples = vec![];
        let target_frame = self.ppu.frame_count().wrapping_add(1);
        while self.ppu.frame_count() < target_frame {
            let _ = self.step_cpu_cycle(emit_audio);
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

    /// Enables a simple "integer FPS" audio stretch.
    ///
    /// When a frontend runs the emulator at an integer display FPS (e.g. 60Hz) rather than
    /// the NES's exact NTSC FPS (~60.0988Hz), the emulator produces slightly fewer audio
    /// samples per wall-clock second. To avoid audio underruns without changing the output
    /// device rate, we time-stretch the resampler input rate by `scale`.
    ///
    /// - `scale < 1.0` stretches audio (more output samples per input chunk).
    /// - `scale > 1.0` compresses audio (fewer output samples).
    pub fn set_audio_integer_fps_scale(&mut self, scale: f64) {
        let scale = scale.clamp(0.25, 4.0);
        let input = (INTERNAL_MIXER_SAMPLE_RATE as f64 * scale).round();
        let input = input.clamp(1.0, u32::MAX as f64) as u32;
        self.sound_bus.set_resample_input_rate(input);
    }

    /// Disables integer-FPS audio stretching and restores the default resampler input rate.
    pub fn reset_audio_integer_fps_scale(&mut self) {
        self.sound_bus.reset_resample_input_rate();
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

    pub fn render_index_buffer(&self) -> &[u8] {
        self.ppu.render_index_buffer()
    }

    /// Copies the current front buffer pixels into the provided destination slice.
    pub fn copy_render_buffer(&mut self, dst: &mut [u8]) {
        self.ppu.copy_render_buffer(dst);
    }

    /// Copies the current front index buffer into the provided destination slice.
    pub fn copy_render_index_buffer(&self, dst: &mut [u8]) {
        self.ppu.copy_render_index_buffer(dst);
    }

    pub fn set_frame_ready_callback(
        &mut self,
        cb: Option<FrameReadyCallback>,
        user_data: *mut c_void,
    ) {
        self.ppu.set_frame_ready_callback(cb, user_data);
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

    /// Master clock counter (total master ticks since power-on).
    pub fn master_clock(&self) -> u64 {
        self.master_clock
    }

    /// CPU cycle counter (CPU cycles since power-on).
    pub fn cpu_cycles(&self) -> u64 {
        self.cycles
    }

    /// Executes the next instruction (advancing CPU/PPU/APU as needed).
    pub fn step_instruction(&mut self) {
        let mut seen_active = false;
        loop {
            self.step_cpu_cycle(false);
            if self.cpu.opcode_active() {
                seen_active = true;
            } else if seen_active {
                break;
            }
        }
    }

    /// Reads a byte from the CPU address space without mutating CPU state.
    pub fn peek_cpu_byte(&mut self, addr: u16) -> u8 {
        let mut bus = nes_cpu_bus!(self, mixer: false, serial: true);
        let mut ctx = Context::Some {
            interceptor: &mut self.interceptor,
        };
        bus.peek(addr, &mut self.cpu, &mut ctx)
    }

    /// Reads a contiguous range of CPU-visible bytes into `buffer`, starting at `base`.
    pub fn peek_cpu_slice(&mut self, base: u16, buffer: &mut [u8]) {
        let mut bus = nes_cpu_bus!(self, mixer: false, serial: true);
        let mut ctx = Context::Some {
            interceptor: &mut self.interceptor,
        };
        for (offset, byte) in buffer.iter_mut().enumerate() {
            *byte = bus.peek(base.wrapping_add(offset as u16), &mut self.cpu, &mut ctx);
        }
    }

    /// Drains any bytes emitted on controller port 1 via the blargg serial protocol.
    pub fn take_serial_output(&mut self) -> Vec<u8> {
        self.serial_log.drain()
    }

    /// Returns a snapshot of CPU and PPU state for debugging.
    pub fn debug_state(&self) -> CpuSnapshot {
        CpuSnapshot {
            pc: self.cpu.pc,
            a: self.cpu.a,
            x: self.cpu.x,
            y: self.cpu.y,
            s: self.cpu.s,
            p: self.cpu.p.bits(),
        }
    }

    /// Returns a detailed PPU state snapshot for debugging.
    pub fn ppu_debug_state(&self) -> (i16, u16, u32, u8, u8, u8, u8, u16, u16, u8) {
        (
            self.ppu.scanline,
            self.ppu.cycle,
            self.ppu.frame,
            self.ppu.registers.control.bits(),
            self.ppu.registers.mask.bits(),
            self.ppu.registers.status.bits(),
            self.ppu.registers.oam_addr,
            self.ppu.registers.vram.v.raw(),
            self.ppu.registers.vram.t.raw(),
            self.ppu.registers.vram.x,
        )
    }

    // =========================================================================
    // Tilemap capture point / snapshot
    // =========================================================================

    pub fn set_tilemap_capture_point(
        &mut self,
        point: crate::interceptor::tilemap_interceptor::CapturePoint,
    ) {
        if let Some(layer) = self.interceptor.layer_mut::<TilemapInterceptor>() {
            layer.set_capture_point(point);
        }
    }

    pub fn take_tilemap_snapshot(&mut self) -> Option<TilemapSnapshot> {
        self.interceptor
            .layer_mut::<TilemapInterceptor>()
            .and_then(|layer| layer.take_snapshot())
    }

    // =========================================================================
    // Tile viewer (CHR) capture point / snapshot
    // =========================================================================

    pub fn set_tile_viewer_capture_point(
        &mut self,
        point: crate::interceptor::tile_viewer_interceptor::CapturePoint,
    ) {
        if let Some(layer) = self.interceptor.layer_mut::<TileViewerInterceptor>() {
            layer.set_capture_point(point);
        }
    }

    pub fn take_tile_viewer_snapshot(&mut self) -> Option<TileViewerSnapshot> {
        self.interceptor
            .layer_mut::<TileViewerInterceptor>()
            .and_then(|layer| layer.take_snapshot())
    }

    // =========================================================================
    // Sprite capture point / snapshot
    // =========================================================================

    pub fn set_sprite_capture_point(
        &mut self,
        point: crate::interceptor::sprite_interceptor::CapturePoint,
    ) {
        if let Some(layer) = self.interceptor.layer_mut::<SpriteInterceptor>() {
            layer.set_capture_point(point);
        }
    }

    pub fn take_sprite_snapshot(&mut self) -> Option<SpriteSnapshot> {
        self.interceptor
            .layer_mut::<SpriteInterceptor>()
            .and_then(|layer| layer.take_snapshot())
    }

    fn build_interceptor() -> EmuInterceptor {
        let mut interceptor = EmuInterceptor::new();
        interceptor.add(LogInterceptor);
        interceptor.add(TilemapInterceptor::new());
        interceptor.add(TileViewerInterceptor::new());
        interceptor.add(SpriteInterceptor::new());
        interceptor
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
