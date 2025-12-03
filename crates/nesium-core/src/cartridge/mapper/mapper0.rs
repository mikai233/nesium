//! Mapper 0 (NROM) implementation.
//!
//! NROM is the simplest NES mapper, used by early titles like *Super Mario Bros.*,
//! *Donkey Kong*, and *Excitebike*. It provides no banking capabilities, meaning
//! the CPU sees the entire PRG ROM and the PPU sees the entire CHR ROM/RAM directly.
//!
//! # Memory Layout
//!
//! - **PRG ROM**: 16 KiB or 32 KiB mapped at `$8000-$FFFF`.
//!   - **NROM-128 (16 KiB)**: Mirrored at `$8000-$BFFF` and `$C000-$FFFF`.
//!   - **NROM-256 (32 KiB)**: Occupies the full `$8000-$FFFF` range.
//! - **PRG RAM**: Optional 2 KiB or 4 KiB at `$6000-$7FFF` (Family Basic).
//!   - *Note*: Many emulators (and this implementation) provide 8 KiB of PRG RAM by
//!     default for iNES 1.0 ROMs to support homebrew/hacks, unless specified otherwise.
//! - **CHR**: 8 KiB of ROM or RAM mapped at `$0000-$1FFF` (PPU).
//!
//! # Reference
//! - [NROM on NESdev Wiki](https://www.nesdev.org/wiki/NROM)

use std::borrow::Cow;

use crate::{
    cartridge::{
        ChrRom, Mapper, PrgRom, TrainerBytes,
        header::{Header, Mirroring},
        mapper::{ChrStorage, allocate_prg_ram_with_trainer, select_chr_storage},
    },
    memory::cpu as cpu_mem,
};

/// Mapper 0 (NROM) state.
#[derive(Debug, Clone)]
pub struct Mapper0 {
    /// PRG ROM data (16 KiB or 32 KiB).
    prg_rom: PrgRom,
    /// PRG RAM (Work RAM), typically 8 KiB at `$6000`.
    /// Contains the Trainer if one was present in the header.
    prg_ram: Box<[u8]>,
    /// CHR memory (ROM or RAM) mapped to PPU `$0000`.
    chr: ChrStorage,
    /// Hardwired mirroring mode (Horizontal/Vertical).
    mirroring: Mirroring,
}

impl Mapper0 {
    /// Constructs a new NROM mapper instance.
    ///
    /// This handles the allocation of PRG RAM (including trainer copying)
    /// and selects the appropriate CHR storage backend.
    pub fn new(header: Header, prg_rom: PrgRom, chr_rom: ChrRom, trainer: TrainerBytes) -> Self {
        let prg_ram = allocate_prg_ram_with_trainer(&header, trainer);

        Self {
            prg_rom,
            prg_ram,
            chr: select_chr_storage(&header, chr_rom),
            mirroring: header.mirroring,
        }
    }

    /// Reads from PRG ROM handling the NROM-128 mirroring.
    ///
    /// If the ROM is only 16 KiB (NROM-128), accesses to `$C000-$FFFF` are
    /// mapped back to the first 16 KiB.
    fn read_prg_rom(&self, addr: u16) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }
        // Modulo operator handles the 16 KiB mirroring automatically:
        // - 32 KiB ROM: (offset % 32768) -> linear access.
        // - 16 KiB ROM: (offset % 16384) -> mirrors upper bank to lower.
        let idx = (addr - cpu_mem::PRG_ROM_START) as usize % self.prg_rom.len();
        self.prg_rom[idx]
    }

    /// Reads from PRG RAM (WRAM).
    fn read_prg_ram(&self, addr: u16) -> u8 {
        if self.prg_ram.is_empty() {
            return 0;
        }
        let idx = (addr - cpu_mem::PRG_RAM_START) as usize % self.prg_ram.len();
        self.prg_ram[idx]
    }

    /// Writes to PRG RAM (WRAM).
    fn write_prg_ram(&mut self, addr: u16, data: u8) {
        if self.prg_ram.is_empty() {
            return;
        }
        let idx = (addr - cpu_mem::PRG_RAM_START) as usize % self.prg_ram.len();
        self.prg_ram[idx] = data;
    }
}

impl Mapper for Mapper0 {
    fn cpu_read(&self, addr: u16) -> Option<u8> {
        let value = match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => {
                if self.prg_ram.is_empty() {
                    return None;
                }
                self.read_prg_ram(addr)
            }
            cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END => self.read_prg_rom(addr),
            _ => return None,
        };
        Some(value)
    }

    fn cpu_write(&mut self, addr: u16, data: u8, _cpu_cycle: u64) {
        if (cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END).contains(&addr) {
            self.write_prg_ram(addr, data);
        }
    }

    fn ppu_read(&self, addr: u16) -> Option<u8> {
        Some(self.chr.read(addr))
    }

    fn ppu_write(&mut self, addr: u16, data: u8) {
        self.chr.write(addr, data);
    }

    fn prg_rom(&self) -> Option<&[u8]> {
        Some(self.prg_rom.as_ref())
    }

    fn prg_ram(&self) -> Option<&[u8]> {
        if self.prg_ram.is_empty() {
            None
        } else {
            Some(self.prg_ram.as_ref())
        }
    }

    fn prg_ram_mut(&mut self) -> Option<&mut [u8]> {
        if self.prg_ram.is_empty() {
            None
        } else {
            Some(self.prg_ram.as_mut())
        }
    }

    fn prg_save_ram(&self) -> Option<&[u8]> {
        self.prg_ram()
    }

    fn prg_save_ram_mut(&mut self) -> Option<&mut [u8]> {
        self.prg_ram_mut()
    }

    fn chr_rom(&self) -> Option<&[u8]> {
        self.chr.as_rom()
    }

    fn chr_ram(&self) -> Option<&[u8]> {
        self.chr.as_ram()
    }

    fn chr_ram_mut(&mut self) -> Option<&mut [u8]> {
        self.chr.as_ram_mut()
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn mapper_id(&self) -> u16 {
        0
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("NROM")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cartridge::header::{Header, Mirroring, RomFormat, TvSystem};

    fn header(prg_rom_size: usize, prg_ram_size: usize, chr_rom_size: usize) -> Header {
        Header {
            format: RomFormat::INes,
            mapper: 0,
            submapper: 0,
            mirroring: Mirroring::Horizontal,
            battery_backed_ram: false,
            trainer_present: false,
            prg_rom_size,
            chr_rom_size,
            prg_ram_size,
            prg_nvram_size: 0,
            chr_ram_size: if chr_rom_size == 0 { 8 * 1024 } else { 0 },
            chr_nvram_size: 0,
            vs_unisystem: false,
            playchoice_10: false,
            tv_system: TvSystem::Ntsc,
        }
    }

    fn new_mapper0(prg_rom_size: usize, prg_ram_size: usize, chr_rom_size: usize) -> Mapper0 {
        let header = header(prg_rom_size, prg_ram_size, chr_rom_size);
        let prg = (0..prg_rom_size)
            .map(|value| (value & 0xFF) as u8)
            .collect::<Vec<_>>();
        let chr = vec![0; chr_rom_size];
        Mapper0::new(header, prg.into(), chr.into(), None)
    }

    #[test]
    fn mirrors_prg_rom_when_16k() {
        let cart = new_mapper0(0x4000, 0x2000, 0);
        let a = cart.cpu_read(cpu_mem::PRG_ROM_START).unwrap();
        let b = cart.cpu_read(cpu_mem::PRG_ROM_START + 0x4000).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn reads_and_writes_prg_ram() {
        let mut cart = new_mapper0(0x4000, 0x2000, 0);
        cart.cpu_write(cpu_mem::PRG_RAM_START, 0x42, 0);
        assert_eq!(cart.cpu_read(cpu_mem::PRG_RAM_START), Some(0x42));
    }

    #[test]
    fn writes_to_chr_ram() {
        let mut cart = new_mapper0(0x8000, 0, 0);
        cart.ppu_write(0x0010, 0x77);
        assert_eq!(cart.ppu_read(0x0010), Some(0x77));
    }

    #[test]
    fn defaults_to_8k_prg_ram_for_ines1_0_roms_with_0_prg_ram() {
        let cart = new_mapper0(0x4000, 0, 0);
        assert_eq!(cart.prg_ram.len(), 8192);
    }
}
