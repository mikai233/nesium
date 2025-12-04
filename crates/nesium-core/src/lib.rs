use std::{fs::File, path::Path};

use crate::{
    apu::Apu,
    audio::{AudioChannel, CPU_CLOCK_NTSC, NesSoundMixer, SoundMixerBus, bus::AudioBusConfig},
    bus::{Bus, OpenBus, cpu::CpuBus},
    cartridge::{Cartridge, Provider},
    controller::{Button, Controller},
    cpu::Cpu,
    error::Error,
    mem_block::cpu as cpu_ram,
    ppu::{
        Ppu,
        buffer::{ColorFormat, FrameBuffer},
        palette::{Palette, PaletteKind},
    },
};

pub mod apu;
pub mod audio;
pub mod bus;
pub mod cartridge;
pub mod controller;
pub mod cpu;
pub mod error;
pub mod mem_block;
pub mod memory;
pub mod ppu;
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
    last_frame: u64,
    /// Master PPU dot counter used to drive CPU/PPU/APU in lockstep (3 dots per CPU cycle).
    dot_counter: u64,
    serial_log: controller::SerialLogger,
    /// Pending OAM DMA page written via `$4014` (latched until CPU picks it up).
    oam_dma_request: Option<u8>,
    /// CPU data-bus open-bus latch (mirrors Mesen2's decay model).
    open_bus: OpenBus,
    /// CPU bus access counter (fed into timing-sensitive mappers).
    cpu_bus_cycle: u64,
    /// Per-console mixer that produces band-limited PCM at a fixed internal rate.
    mixer: NesSoundMixer,
    /// Global audio bus that resamples from the internal mixer rate to the host sample rate.
    sound_bus: SoundMixerBus,
    audio_sample_rate: u32,
    /// Scratch buffer for a single frame of internal-rate audio from the per-console mixer.
    mixer_frame_buffer: Vec<f32>,
    /// Optional debug dump of the internal 96 kHz mixer output (for waveform comparison).
    debug_apu_dump: Option<File>,
    debug_apu_frames_written: u64,
    /// Optional debug dump of the post-bus PCM (for waveform comparison).
    debug_bus_dump: Option<File>,
    debug_bus_frames_written: u64,
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
            serial_log: controller::SerialLogger::default(),
            oam_dma_request: None,
            open_bus: OpenBus::new(),
            cpu_bus_cycle: 0,
            mixer: NesSoundMixer::new(CPU_CLOCK_NTSC, INTERNAL_MIXER_SAMPLE_RATE),
            sound_bus: SoundMixerBus::new(INTERNAL_MIXER_SAMPLE_RATE, sample_rate),
            audio_sample_rate: sample_rate,
            mixer_frame_buffer: Vec::new(),
            debug_apu_dump: None,
            debug_apu_frames_written: 0,
            debug_bus_dump: None,
            debug_bus_frames_written: 0,
        };
        nes.ppu.set_palette(PaletteKind::NesdevNtsc.palette());
        // Apply a power-on style reset once at construction time. This matches
        // the console being turned on from a cold state (CPU/PPU/APU and RAM
        // cleared) and is distinct from subsequent warm resets triggered by
        // the user pressing the reset button.
        nes.power_on_reset();
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
        self.power_on_reset();
    }

    /// Ejects the currently inserted cartridge and resets the system.
    pub fn eject_cartridge(&mut self) {
        self.cartridge = None;
        // Treat cartridge removal as a full power cycle from the core's
        // perspective so all state, including RAM, returns to power-on.
        self.power_on_reset();
    }

    /// Applies a power-on style reset: clears CPU RAM and fully reinitializes
    /// CPU/PPU/APU, open bus, mixer state, and any attached cartridge.
    ///
    /// This corresponds to turning the console off and back on.
    fn power_on_reset(&mut self) {
        self.ram.fill(0);
        self.ppu.reset();
        self.apu.power_on_reset();
        if let Some(cart) = self.cartridge.as_mut() {
            cart.power_on();
        }
        self.serial_log.drain();
        self.open_bus.reset();
        self.cpu_bus_cycle = 0;
        self.mixer.reset();
        self.sound_bus.reset();
        self.mixer_frame_buffer.clear();
        let mut bus = CpuBus::new(
            &mut self.ram,
            &mut self.ppu,
            &mut self.apu,
            self.cartridge.as_mut(),
            &mut self.controllers,
            Some(&mut self.serial_log),
            &mut self.oam_dma_request,
            &mut self.open_bus,
            &mut self.cpu_bus_cycle,
        );
        self.cpu.reset(&mut bus);
        self.last_frame = bus.ppu().frame_count();
        self.dot_counter = 0;
    }

    /// Applies a warm console reset: resets CPU/PPU/APU and mapper state while
    /// preserving CPU RAM contents. This mirrors the behaviour expected by
    /// reset-sensitive test ROMs (for example, blargg's `apu_reset` suite),
    /// which store metadata and counters in non-volatile RAM across resets.
    pub fn reset(&mut self) {
        self.ppu.reset();
        self.apu.reset();
        if let Some(cart) = self.cartridge.as_mut() {
            cart.reset();
        }
        self.serial_log.drain();
        self.open_bus.reset();
        self.cpu_bus_cycle = 0;
        self.mixer.reset();
        self.sound_bus.reset();
        self.mixer_frame_buffer.clear();
        let mut bus = CpuBus::new(
            &mut self.ram,
            &mut self.ppu,
            &mut self.apu,
            self.cartridge.as_mut(),
            &mut self.controllers,
            Some(&mut self.serial_log),
            &mut self.oam_dma_request,
            &mut self.open_bus,
            &mut self.cpu_bus_cycle,
        );
        self.cpu.reset(&mut bus);
        self.last_frame = bus.ppu().frame_count();
        self.dot_counter = 0;
    }

    /// Advances the system by a single PPU dot (master tick) and reports whether
    /// CPU/APU were clocked on this dot.
    pub fn step_dot(&mut self) -> ClockResult {
        let mut bus = CpuBus::new(
            &mut self.ram,
            &mut self.ppu,
            &mut self.apu,
            self.cartridge.as_mut(),
            &mut self.controllers,
            Some(&mut self.serial_log),
            &mut self.oam_dma_request,
            &mut self.open_bus,
            &mut self.cpu_bus_cycle,
        );

        // NTSC timing: 1 CPU tick per 3 PPU dots. Run one PPU dot every call,
        // and run CPU/APU after the third dot in each 3-dot group so timing
        // matches the common PPU-first, CPU-later cadence.
        bus.clock_ppu();
        // Run CPU/APU once every 3 PPU dots with a phase that aligns CPU work
        // just after the second PPU dot in each trio, matching common PPU-first cadence.
        let apu_clocked = if (self.dot_counter + 2) % 3 == 0 {
            self.cpu.clock(&mut bus);
            bus.apu_mut().clock();
            true
        } else {
            false
        };

        self.dot_counter = self.dot_counter.wrapping_add(1);

        let frame_count = bus.ppu().frame_count();
        let frame_advanced = if frame_count != self.last_frame {
            self.last_frame = frame_count;
            true
        } else {
            false
        };

        ClockResult {
            frame_advanced,
            apu_clocked,
        }
    }

    /// Advances the system by a single PPU dot while feeding audio deltas into the shared mixer.
    pub fn step_dot_with_audio(&mut self) -> ClockResult {
        // First, run PPU + CPU for this dot using the shared CPU bus.
        let (frame_advanced, apu_clocked, cpu_cycles, expansion_samples) = {
            let mut bus = CpuBus::new(
                &mut self.ram,
                &mut self.ppu,
                &mut self.apu,
                self.cartridge.as_mut(),
                &mut self.controllers,
                Some(&mut self.serial_log),
                &mut self.oam_dma_request,
                &mut self.open_bus,
                &mut self.cpu_bus_cycle,
            );

            bus.clock_ppu();
            let apu_clocked = if (self.dot_counter + 2) % 3 == 0 {
                self.cpu.clock(&mut bus);

                // Expansion audio samples are taken at the same CPU clock as
                // the core APU tick, mirroring Mesen2's behaviour.
                let expansion = bus
                    .cartridge()
                    .and_then(|cart| cart.mapper().as_expansion_audio())
                    .map(|exp| exp.samples());
                (true, expansion)
            } else {
                (false, None)
            };

            self.dot_counter = self.dot_counter.wrapping_add(1);

            let frame_count = bus.ppu().frame_count();
            let frame_advanced = if frame_count != self.last_frame {
                self.last_frame = frame_count;
                true
            } else {
                false
            };

            let cpu_cycles = bus.cpu_cycles();
            (frame_advanced, apu_clocked.0, cpu_cycles, apu_clocked.1)
        };

        // After dropping the bus (and its borrows of the cartridge), run the
        // APU tick and feed deltas into the shared mixer, wiring the DMC
        // sample fetch path directly to the PRG space.
        if apu_clocked {
            use crate::memory::cpu as cpu_mem;

            let mut reader = |addr: u16| {
                if addr < cpu_mem::CARTRIDGE_SPACE_BASE {
                    0
                } else {
                    self.cartridge
                        .as_ref()
                        .and_then(|cart| cart.cpu_read(addr))
                        .unwrap_or(0)
                }
            };
            self.apu
                .clock_with_reader(&mut reader, Some(&mut self.mixer));

            if let Some(samples) = expansion_samples {
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
        }

        ClockResult {
            frame_advanced,
            apu_clocked,
        }
    }

    /// Runs CPU/PPU/APU ticks until the PPU completes the next frame.
    pub fn run_frame(&mut self) {
        let target_frame = self.last_frame.wrapping_add(1);
        while self.ppu.frame_count() < target_frame {
            self.step_dot();
        }
    }

    /// Advances the system by a single PPU dot (debug helper alias for [`step_dot`]).
    pub fn clock_dot(&mut self) -> ClockResult {
        self.step_dot()
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

    /// Run a full frame and emit interleaved stereo PCM samples.
    pub fn run_frame_with_audio(&mut self, out: &mut Vec<f32>) {
        let end_clock = loop {
            let res = self.step_dot_with_audio();
            if res.frame_advanced {
                break self.apu_cycles() as i64;
            }
        };
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

        let _out_start = out.len();
        self.sound_bus.mix_frame(&[&self.mixer_frame_buffer], out);

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
    pub fn sprite0_hit_pos(&self) -> Option<crate::ppu::Sprite0HitDebug> {
        self.ppu.sprite0_hit_pos()
    }

    /// Peek PPU NMI flags and position (tests/debug).
    pub fn ppu_nmi_debug(&self) -> crate::ppu::NmiDebugState {
        self.ppu.debug_nmi_state()
    }

    /// Debug-only: override PPU counters for trace alignment.
    pub fn debug_set_ppu_position(&mut self, scanline: i16, cycle: u16, frame: u64) {
        self.ppu.debug_set_position(scanline, cycle, frame);
    }

    /// Executes the next instruction (advancing CPU/PPU/APU as needed).
    pub fn step_instruction(&mut self) {
        let mut seen_active = false;
        loop {
            self.step_dot();
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
            &mut self.cpu_bus_cycle,
        );
        bus.read(addr)
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
            &mut self.cpu_bus_cycle,
        );
        for (offset, byte) in buffer.iter_mut().enumerate() {
            *byte = bus.read(base.wrapping_add(offset as u16));
        }
    }

    /// Drains any bytes emitted on controller port 1 via the blargg serial protocol.
    pub fn take_serial_output(&mut self) -> Vec<u8> {
        self.serial_log.drain()
    }
}

/// Result of a single dot tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockResult {
    pub frame_advanced: bool,
    pub apu_clocked: bool,
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
