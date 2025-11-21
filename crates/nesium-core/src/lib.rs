use std::path::Path;

use crate::{
    apu::Apu,
    bus::cpu::CpuBus,
    cartridge::Cartridge,
    controller::{Button, Controller},
    cpu::Cpu,
    error::Error,
    ppu::{
        self as ppu_mod, Ppu,
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
pub struct NES {
    cpu: Cpu,
    ppu: Ppu,
    apu: Apu,
    ram: cpu_ram::Ram,
    cartridge: Option<Cartridge>,
    controllers: [Controller; 2],
    palette: Palette,
    last_frame: u64,
}

impl NES {
    /// Constructs a powered-on NES instance with cleared RAM and default palette.
    pub fn new() -> Self {
        let mut nes = Self {
            cpu: Cpu::new(),
            ppu: Ppu::new(),
            apu: Apu::new(),
            ram: cpu_ram::Ram::new(),
            cartridge: None,
            controllers: [Controller::new(), Controller::new()],
            palette: Palette::default(),
            last_frame: 0,
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

    /// Resets the CPU, RAM, and attached peripherals to their power-on state.
    pub fn reset(&mut self) {
        self.ram.fill(0);
        self.ppu.reset();
        self.apu.reset();
        let mut bus = CpuBus::new(
            &mut self.ram,
            &mut self.ppu,
            &mut self.apu,
            self.cartridge.as_mut(),
            &mut self.controllers,
        );
        self.cpu.reset(&mut bus);
        self.last_frame = bus.ppu().frame_count();
    }

    /// Advances the system by a single CPU tick (three PPU ticks).
    ///
    /// Returns `true` when a new frame has just been produced.
    pub fn clock(&mut self) -> bool {
        let mut bus = CpuBus::new(
            &mut self.ram,
            &mut self.ppu,
            &mut self.apu,
            self.cartridge.as_mut(),
            &mut self.controllers,
        );

        self.cpu.clock(&mut bus);

        // NTSC timing: 3 PPU cycles per CPU cycle.
        bus.clock_ppu();
        bus.clock_ppu();
        bus.clock_ppu();

        // Keep APU counters moving so audio timing stays roughly correct.
        bus.apu_mut().clock();

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
            self.clock();
        }
    }

    /// Palette indices for the latest frame (PPU native format).
    pub fn framebuffer(&self) -> &[u8] {
        self.ppu.framebuffer()
    }

    /// Converts the current framebuffer into RGBA8 pixels using the active palette.
    pub fn frame_rgba(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(ppu_mod::SCREEN_WIDTH * ppu_mod::SCREEN_HEIGHT * 4);
        for index in self.ppu.framebuffer() {
            let color = self.palette.color(*index);
            out.extend_from_slice(&[color.r, color.g, color.b, 0xFF]);
        }
        out
    }

    /// Selects one of the built-in palettes.
    pub fn set_palette_kind(&mut self, kind: PaletteKind) {
        self.palette = kind.palette();
    }

    /// Active palette reference.
    pub fn palette(&self) -> &Palette {
        &self.palette
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

    /// Executes the next instruction (advancing CPU/PPU/APU as needed).
    pub fn step_instruction(&mut self) {
        loop {
            self.clock();
            if !self.cpu.opcode_active() {
                break;
            }
        }
    }

    /// Forces the CPU registers to the provided snapshot (clears in-flight opcode).
    pub fn set_cpu_snapshot(&mut self, snapshot: CpuSnapshot) {
        self.cpu.load_snapshot(snapshot);
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
