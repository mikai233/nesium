//! Picture Processing Unit (PPU) scaffolding.
//!
//! The NES PPU exposes eight CPU-facing registers between `$2000` and `$2007`.
//! Rendering is a complex pipeline that mixes nametables, pattern tables,
//! palettes, and sprites. This module intentionally focuses on the register
//! layer so that the CPU/bus plumbing can be built and tested before any pixel
//! level logic exists. Each register mirrors the original hardware behavior and
//! contains thorough documentation describing its purpose.

pub mod palette;

mod registers;

use core::fmt;

use crate::{
    memory::ppu::{self as ppu_mem, Register as PpuRegister},
    ram::ppu::{PaletteRam, Vram},
};
use registers::{Control, Mask, Registers, Status};
const CYCLES_PER_SCANLINE: u16 = 341;
const SCANLINES_PER_FRAME: i16 = 262; // -1 (prerender) + 0..239 visible + vblank lines

/// Entry points for the CPU PPU register mirror.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Ppu {
    /// Collection of CPU visible registers and their helper latches.
    registers: Registers,
    /// Internal VRAM backing store for nametables and pattern tables.
    vram: Vram,
    /// Dedicated palette RAM. Addresses between `$3F00` and `$3FFF` map here.
    palette_ram: PaletteRam,
    /// Current dot (0..=340) within the active scanline.
    cycle: u16,
    /// Current scanline. `-1` is the prerender line, `0..239` are visible.
    scanline: i16,
    /// Total number of frames produced so far.
    frame: u64,
    /// Tracks whether the current frame is odd. Required for the skipped tick logic.
    odd_frame: bool,
}

impl fmt::Debug for Ppu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Ppu")
            .field("registers", &self.registers)
            .field("cycle", &self.cycle)
            .field("scanline", &self.scanline)
            .field("frame", &self.frame)
            .field("odd_frame", &self.odd_frame)
            .finish()
    }
}

impl Default for Ppu {
    fn default() -> Self {
        Self::new()
    }
}

impl Ppu {
    /// Creates a new PPU instance with cleared VRAM and default register values.
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
            vram: Vram::new(),
            palette_ram: PaletteRam::new(),
            cycle: 0,
            scanline: -1,
            frame: 0,
            odd_frame: false,
        }
    }

    /// Restores the device to its power-on state.
    pub fn reset(&mut self) {
        self.registers.reset();
        self.vram.fill(0);
        self.palette_ram.fill(0);
        self.cycle = 0;
        self.scanline = -1;
        self.frame = 0;
        self.odd_frame = false;
    }

    /// Handles CPU writes to the mirrored PPU register space (`$2000-$3FFF`).
    pub fn cpu_write(&mut self, addr: u16, value: u8) {
        match PpuRegister::from_cpu_addr(addr) {
            PpuRegister::Control => self.registers.control = Control::from_bits_retain(value),
            PpuRegister::Mask => self.registers.mask = Mask::from_bits_retain(value),
            PpuRegister::Status => {} // read-only
            PpuRegister::OamAddr => self.registers.oam_addr = value,
            PpuRegister::OamData => self.write_oam_data(value),
            PpuRegister::Scroll => self.registers.scroll.write(value),
            PpuRegister::Addr => self.registers.addr.write(value),
            PpuRegister::Data => self.write_vram_data(value),
        }
    }

    /// Handles CPU reads from the mirrored PPU register space (`$2000-$3FFF`).
    pub fn cpu_read(&mut self, addr: u16) -> u8 {
        match PpuRegister::from_cpu_addr(addr) {
            PpuRegister::Status => self.read_status(),
            PpuRegister::OamData => self.read_oam_data(),
            PpuRegister::Data => self.read_vram_data(),
            _ => 0,
        }
    }

    /// Advances the PPU by a single dot, keeping cycle and frame counters up to date.
    pub fn clock(&mut self) {
        if self.scanline == 241 && self.cycle == 1 {
            self.registers.status.insert(Status::VERTICAL_BLANK);
        }
        if self.scanline == -1 && self.cycle == 1 {
            self.registers.status.remove(Status::VERTICAL_BLANK);
            self.registers
                .status
                .remove(Status::SPRITE_OVERFLOW | Status::SPRITE_ZERO_HIT);
        }

        self.cycle += 1;
        if self.cycle >= CYCLES_PER_SCANLINE {
            self.cycle = 0;
            self.scanline += 1;

            if self.scanline >= SCANLINES_PER_FRAME {
                self.scanline = -1;
                self.frame = self.frame.wrapping_add(1);
                self.odd_frame = !self.odd_frame;
            }
        }
    }

    fn read_status(&mut self) -> u8 {
        let status = self.registers.status.bits();
        self.registers.status.remove(Status::VERTICAL_BLANK);
        self.registers.scroll.reset_latch();
        self.registers.addr.reset_latch();
        status
    }

    fn write_oam_data(&mut self, value: u8) {
        let idx = self.registers.oam_addr as usize;
        if idx < registers::OAM_SIZE {
            self.registers.oam[idx] = value;
            self.registers.oam_addr = self.registers.oam_addr.wrapping_add(1);
        }
    }

    fn read_oam_data(&self) -> u8 {
        let idx = self.registers.oam_addr as usize;
        if idx < registers::OAM_SIZE {
            self.registers.oam[idx]
        } else {
            0
        }
    }

    fn write_vram_data(&mut self, value: u8) {
        let addr = self.registers.addr.addr();
        self.write_vram(addr, value);
        let increment = self.registers.control.vram_increment();
        self.registers.addr.increment(increment);
    }

    fn read_vram_data(&mut self) -> u8 {
        let addr = self.registers.addr.addr();
        let data = self.read_vram(addr);
        let buffered = self.registers.vram_buffer;
        self.registers.vram_buffer = data;
        let increment = self.registers.control.vram_increment();
        self.registers.addr.increment(increment);

        if addr >= ppu_mem::PALETTE_BASE {
            data
        } else {
            buffered
        }
    }

    fn write_vram(&mut self, addr: u16, value: u8) {
        let addr = addr & ppu_mem::VRAM_MIRROR_MASK;
        if addr >= ppu_mem::PALETTE_BASE {
            let index = palette_index(addr);
            self.palette_ram[index] = value;
        } else {
            self.vram[addr as usize] = value;
        }
    }

    fn read_vram(&self, addr: u16) -> u8 {
        let addr = addr & ppu_mem::VRAM_MIRROR_MASK;
        if addr >= ppu_mem::PALETTE_BASE {
            let index = palette_index(addr);
            self.palette_ram[index]
        } else {
            self.vram[addr as usize]
        }
    }
}

fn palette_index(addr: u16) -> usize {
    let mut index = ((addr - ppu_mem::PALETTE_BASE) % ppu_mem::PALETTE_STRIDE) as usize;
    if index >= 16 && index % 4 == 0 {
        index -= 16;
    }
    index % ppu_mem::PALETTE_RAM_SIZE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn control_register_helpers() {
        let mut ppu = Ppu::new();
        ppu.cpu_write(PpuRegister::Control.addr(), 0b1000_0100);
        assert!(ppu.registers.control.nmi_enabled());
        assert_eq!(ppu.registers.control.vram_increment(), 32);
        assert_eq!(
            ppu.registers.control.base_nametable_addr(),
            ppu_mem::NAMETABLE_BASE
        );
    }

    #[test]
    fn buffered_ppu_data_read() {
        let mut ppu = Ppu::new();
        // Point to $2000 and write a value.
        ppu.cpu_write(PpuRegister::Addr.addr(), 0x20);
        ppu.cpu_write(PpuRegister::Addr.addr(), 0x00);
        ppu.cpu_write(PpuRegister::Data.addr(), 0x12);

        // Reset VRAM address to read back.
        ppu.cpu_write(PpuRegister::Addr.addr(), 0x20);
        ppu.cpu_write(PpuRegister::Addr.addr(), 0x00);

        let first = ppu.cpu_read(PpuRegister::Data.addr());
        let second = ppu.cpu_read(PpuRegister::Data.addr());
        assert_eq!(first, 0x00, "First read should return buffered value");
        assert_eq!(second, 0x12, "Second read should contain VRAM data");
    }

    #[test]
    fn palette_reads_bypass_buffer() {
        let mut ppu = Ppu::new();
        ppu.cpu_write(PpuRegister::Addr.addr(), 0x3F);
        ppu.cpu_write(PpuRegister::Addr.addr(), 0x00);
        ppu.cpu_write(PpuRegister::Data.addr(), 0x99);

        ppu.cpu_write(PpuRegister::Addr.addr(), 0x3F);
        ppu.cpu_write(PpuRegister::Addr.addr(), 0x00);

        let value = ppu.cpu_read(PpuRegister::Data.addr());
        assert_eq!(value, 0x99);
    }

    #[test]
    fn status_read_resets_scroll_latch() {
        let mut ppu = Ppu::new();
        ppu.cpu_write(PpuRegister::Scroll.addr(), 0x12); // horizontal
        ppu.cpu_write(PpuRegister::Scroll.addr(), 0x34); // vertical
        assert_eq!(ppu.registers.scroll.horizontal(), 0x12);
        assert_eq!(ppu.registers.scroll.vertical(), 0x34);

        // Reading status should clear the write toggle so the next write targets horizontal.
        let _ = ppu.cpu_read(PpuRegister::Status.addr());
        ppu.cpu_write(PpuRegister::Scroll.addr(), 0x56);
        assert_eq!(ppu.registers.scroll.horizontal(), 0x56);
    }

    #[test]
    fn oam_data_auto_increments() {
        let mut ppu = Ppu::new();
        ppu.cpu_write(PpuRegister::OamAddr.addr(), 0x02);
        ppu.cpu_write(PpuRegister::OamData.addr(), 0xAA);
        ppu.cpu_write(PpuRegister::OamData.addr(), 0xBB);
        assert_eq!(ppu.registers.oam[2], 0xAA);
        assert_eq!(ppu.registers.oam[3], 0xBB);
    }

    #[test]
    fn vblank_flag_is_managed_by_clock() {
        let mut ppu = Ppu::new();
        // Run until scanline 241, cycle 1.
        let target_cycles = (241i32 * CYCLES_PER_SCANLINE as i32 + 1) as usize;
        for _ in 0..target_cycles {
            ppu.clock();
        }
        assert!(ppu.registers.status.contains(Status::VERTICAL_BLANK));

        // Continue until prerender line clears the flag.
        while ppu.scanline != -1 || ppu.cycle != 1 {
            ppu.clock();
        }
        assert!(!ppu.registers.status.contains(Status::VERTICAL_BLANK));
    }
}
