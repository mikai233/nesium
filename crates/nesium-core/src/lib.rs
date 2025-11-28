use std::path::Path;

use crate::{
    apu::Apu,
    bus::{Bus, OpenBus, cpu::CpuBus},
    cartridge::Cartridge,
    controller::{Button, Controller},
    cpu::Cpu,
    error::Error,
    ppu::{
        Ppu,
        buffer::{ColorFormat, FrameBuffer},
        palette::{Palette, PaletteKind},
    },
    ram::cpu as cpu_ram,
};

pub mod apu;
pub mod bus;
pub mod cartridge;
pub mod controller;
pub mod cpu;
pub mod error;
pub mod memory;
pub mod ppu;
pub mod ram;

pub use cpu::CpuSnapshot;

#[derive(Debug)]
pub struct Nes {
    pub cpu: Cpu,
    pub ppu: Ppu,
    apu: Apu,
    ram: cpu_ram::Ram,
    cartridge: Option<Cartridge>,
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
}

impl Nes {
    /// Constructs a powered-on NES instance with cleared RAM and default palette.
    pub fn new(format: ColorFormat) -> Self {
        let buffer = FrameBuffer::new_color(PaletteKind::NesdevNtsc.palette(), format);
        let mut nes = Self {
            cpu: Cpu::new(),
            ppu: Ppu::new(buffer),
            apu: Apu::new(),
            ram: cpu_ram::Ram::new(),
            cartridge: None,
            controllers: [Controller::new(), Controller::new()],
            last_frame: 0,
            dot_counter: 0,
            serial_log: controller::SerialLogger::default(),
            oam_dma_request: None,
            open_bus: OpenBus::new(),
            cpu_bus_cycle: 0,
        };
        nes.reset();
        nes
    }

    /// Loads a cartridge from disk, inserts it, and performs a reset sequence.
    pub fn load_cartridge_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<(), Error> {
        let cartridge = cartridge::load_cartridge_from_file(path)?;
        self.insert_cartridge(cartridge);
        Ok(())
    }

    /// Inserts a cartridge that has already been constructed.
    pub fn insert_cartridge(&mut self, cartridge: Cartridge) {
        self.ppu.attach_cartridge(&cartridge);
        self.cartridge = Some(cartridge);
        self.reset();
    }

    /// Ejects the currently inserted cartridge and resets the system.
    pub fn eject_cartridge(&mut self) {
        self.cartridge = None;
        self.reset();
    }

    /// Resets the CPU, RAM, and attached peripherals to their power-on state.
    pub fn reset(&mut self) {
        self.ram.fill(0);
        self.ppu.reset();
        self.apu.reset();
        self.serial_log.drain();
        self.open_bus.reset();
        self.cpu_bus_cycle = 0;
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

    /// Advances the system by a single PPU dot (master tick).
    ///
    /// Runs PPU every call; runs CPU/APU every 3 dots (NTSC ratio).
    /// Returns `true` when a new frame has just been produced.
    pub fn clock_dot(&mut self) -> bool {
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
        if (self.dot_counter + 2) % 3 == 0 {
            self.cpu.clock(&mut bus);
            bus.apu_mut().clock();
        }

        self.dot_counter = self.dot_counter.wrapping_add(1);

        let frame_count = bus.ppu().frame_count();
        let new_frame = frame_count != self.last_frame;
        if new_frame {
            self.last_frame = frame_count;
        }
        new_frame
    }

    /// Runs CPU/PPU/APU ticks until the PPU completes the next frame.
    pub fn run_frame(&mut self) {
        let target_frame = self.last_frame.wrapping_add(1);
        while self.ppu.frame_count() < target_frame {
            self.clock_dot();
        }
    }

    /// Palette indices for the latest frame (PPU native format).
    pub fn render_buffer(&self) -> &[u8] {
        self.ppu.render_buffer()
    }

    /// Selects one of the built-in palettes.
    pub fn set_palette(&mut self, palette: Palette) {
        self.ppu.framebuffer.set_palette(palette);
    }

    /// Active palette reference.
    pub fn palette(&self) -> &Palette {
        self.ppu.framebuffer.get_palette()
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
            self.clock_dot();
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
