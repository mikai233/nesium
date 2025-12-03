//! Mapper 11 – Color Dreams discrete mapper.
//!
//! This mapper is used by the Color Dreams / Wisdom Tree family of boards.
//! It is functionally very close to Nintendo's GxROM (mapper 66) but uses a
//! slightly different bit layout in its single bank-select register:
//!
//! - CPU `$8000-$FFFF`: one 32 KiB switchable PRG-ROM bank.
//! - PPU `$0000-$1FFF`: one 8 KiB switchable CHR-ROM/RAM bank.
//! - CPU writes anywhere in `$8000-$FFFF` update both PRG and CHR banks.
//!
//! Register layout (per Nesdev "Color Dreams"):
//! ```text
//! 7  bit  0
//! ---- ----
//! CCCC LLPP
//! |||| ||||
//! |||| ||++- Select 32 KB PRG ROM bank for CPU $8000-$FFFF
//! |||| ++--- Used for lockout defeat (ignored here)
//! ++++------ Select 8 KB CHR ROM bank for PPU $0000-$1FFF
//! ```
//!
//! Lockout defeat bits are not modelled here because they do not affect the
//! CPU/PPU address mapping in an emulator.

use std::borrow::Cow;

use crate::{
    cartridge::{
        ChrRom, Mapper, PrgRom, TrainerBytes,
        header::{Header, Mirroring},
        mapper::{ChrStorage, allocate_prg_ram_with_trainer, select_chr_storage},
    },
    memory::cpu as cpu_mem,
};

/// Size of a single PRG bank exposed to the CPU (32 KiB).
const PRG_BANK_SIZE_32K: usize = 32 * 1024;
/// Size of a single CHR bank exposed to the PPU (8 KiB).
const CHR_BANK_SIZE_8K: usize = 8 * 1024;

#[derive(Debug, Clone)]
pub struct Mapper11 {
    prg_rom: PrgRom,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,

    /// Currently selected 32 KiB PRG bank (`PP` bits).
    prg_bank: u8,
    /// Currently selected 8 KiB CHR bank (`CCCC` bits).
    chr_bank: u8,

    /// Nametable mirroring is fixed by the board wiring and encoded in the
    /// iNES header; there is no mapper-controlled mirroring register.
    mirroring: Mirroring,
}

impl Mapper11 {
    pub fn new(
        header: Header,
        prg_rom: PrgRom,
        chr_rom: ChrRom,
        trainer: TrainerBytes,
    ) -> Self {
        let prg_ram = allocate_prg_ram_with_trainer(&header, trainer);

        let chr = select_chr_storage(&header, chr_rom);

        Self {
            prg_rom,
            prg_ram,
            chr,
            prg_bank: 0,
            chr_bank: 0,
            mirroring: header.mirroring,
        }
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }

        // 32 KiB window mapped at $8000-$FFFF, selected by `prg_bank`.
        let bank = (self.prg_bank & 0x03) as usize;
        let bank_size = PRG_BANK_SIZE_32K;

        let offset_within_bank =
            (addr as usize).saturating_sub(cpu_mem::PRG_ROM_START as usize) & (bank_size - 1);
        let base = bank.saturating_mul(bank_size);

        // Wrap safely in case the ROM is smaller than expected.
        let len = self.prg_rom.len();
        let idx = (base + offset_within_bank) % len;
        self.prg_rom[idx]
    }

    fn read_prg_ram(&self, addr: u16) -> Option<u8> {
        if self.prg_ram.is_empty() {
            return None;
        }
        let idx = (addr - cpu_mem::PRG_RAM_START) as usize % self.prg_ram.len();
        Some(self.prg_ram[idx])
    }

    fn write_prg_ram(&mut self, addr: u16, data: u8) {
        if self.prg_ram.is_empty() {
            return;
        }
        let idx = (addr - cpu_mem::PRG_RAM_START) as usize % self.prg_ram.len();
        self.prg_ram[idx] = data;
    }

    fn read_chr(&self, addr: u16) -> u8 {
        // 8 KiB CHR window selected by `chr_bank` (`CCCC` bits).
        let bank = (self.chr_bank & 0x0F) as usize;
        let base = bank.saturating_mul(CHR_BANK_SIZE_8K);
        let offset = (addr & 0x1FFF) as usize;
        self.chr.read_indexed(base, offset)
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        let bank = (self.chr_bank & 0x0F) as usize;
        let base = bank.saturating_mul(CHR_BANK_SIZE_8K);
        let offset = (addr & 0x1FFF) as usize;
        self.chr.write_indexed(base, offset, data);
    }

    /// Update both PRG and CHR bank registers from a CPU write in the
    /// `$8000-$FFFF` range.
    fn write_bank_select(&mut self, data: u8) {
        // Low 2 bits select the 32 KiB PRG bank.
        self.prg_bank = data & 0x03;
        // High 4 bits select the 8 KiB CHR bank.
        self.chr_bank = (data >> 4) & 0x0F;
    }
}

impl Mapper for Mapper11 {
    fn power_on(&mut self) {
        // Power-on defaults: both PRG and CHR banks start at 0, mirroring
        // follows the header's wiring description.
        self.prg_bank = 0;
        self.chr_bank = 0;
    }

    fn reset(&mut self) {
        self.power_on();
    }

    fn cpu_read(&self, addr: u16) -> Option<u8> {
        let value = match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => return self.read_prg_ram(addr),
            cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END => self.read_prg_rom(addr),
            _ => return None,
        };
        Some(value)
    }

    fn cpu_write(&mut self, addr: u16, data: u8, _cpu_cycle: u64) {
        match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.write_prg_ram(addr, data),
            0x8000..=0xFFFF => self.write_bank_select(data),
            _ => {}
        }
    }

    fn ppu_read(&self, addr: u16) -> Option<u8> {
        Some(self.read_chr(addr))
    }

    fn ppu_write(&mut self, addr: u16, data: u8) {
        self.write_chr(addr, data);
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
        11
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Color Dreams")
    }
}
