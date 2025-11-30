//! Mapper 78 – Irem 74HC161/32-style boards (e.g., Holy Diver).
//!
//! A single latch at `$8000-$FFFF` controls:
//! - PRG: 16 KiB switchable bank at `$8000-$BFFF` (low nibble).
//! - PRG: 16 KiB fixed to the last bank at `$C000-$FFFF`.
//! - CHR: 8 KiB bank (high nibble) covering `$0000-$1FFF` (ROM or RAM).
//! - Mirroring: bit 2 selects horizontal (1) vs vertical (0) mirroring.
//! Bus conflicts are ignored to match common emulator behaviour.

use std::borrow::Cow;

use crate::{
    cartridge::{
        Mapper, TRAINER_SIZE,
        header::{Header, Mirroring},
        mapper::{ChrStorage, allocate_prg_ram, select_chr_storage, trainer_destination},
    },
    memory::cpu as cpu_mem,
};

/// PRG banking granularity (16 KiB).
const PRG_BANK_SIZE_16K: usize = 16 * 1024;
/// CHR banking granularity (8 KiB).
const CHR_BANK_SIZE_8K: usize = 8 * 1024;

#[derive(Debug, Clone)]
pub struct Mapper78 {
    prg_rom: Box<[u8]>,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,

    prg_bank_count_16k: usize,
    prg_bank: u8,
    chr_bank: u8,

    mirroring: Mirroring,
}

impl Mapper78 {
    pub fn new(header: Header, prg_rom: Box<[u8]>, chr_rom: Box<[u8]>) -> Self {
        Self::with_trainer(header, prg_rom, chr_rom, None)
    }

    pub(crate) fn with_trainer(
        header: Header,
        prg_rom: Box<[u8]>,
        chr_rom: Box<[u8]>,
        trainer: Option<Box<[u8; TRAINER_SIZE]>>,
    ) -> Self {
        let mut prg_ram = allocate_prg_ram(&header);
        if let (Some(trainer), Some(dst)) = (trainer.as_ref(), trainer_destination(&mut prg_ram)) {
            dst.copy_from_slice(trainer.as_ref());
        }

        let chr = select_chr_storage(&header, chr_rom);
        let prg_bank_count_16k = (prg_rom.len() / PRG_BANK_SIZE_16K).max(1);

        Self {
            prg_rom,
            prg_ram,
            chr,
            prg_bank_count_16k,
            prg_bank: 0,
            chr_bank: 0,
            mirroring: header.mirroring,
        }
    }

    #[inline]
    fn prg_bank_index(&self, reg_value: u8) -> usize {
        if self.prg_bank_count_16k == 0 {
            0
        } else {
            (reg_value as usize) % self.prg_bank_count_16k
        }
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }

        let bank = match addr {
            0x8000..=0xBFFF => self.prg_bank_index(self.prg_bank),
            0xC000..=0xFFFF => self.prg_bank_count_16k.saturating_sub(1),
            _ => 0,
        };
        let offset = (addr & 0x3FFF) as usize;
        let base = bank.saturating_mul(PRG_BANK_SIZE_16K);
        self.prg_rom.get(base + offset).copied().unwrap_or(0)
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
        let offset = (addr as usize) % CHR_BANK_SIZE_8K;
        let base = (self.chr_bank as usize) * CHR_BANK_SIZE_8K;
        self.chr.read_indexed(base, offset)
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        let offset = (addr as usize) % CHR_BANK_SIZE_8K;
        let base = (self.chr_bank as usize) * CHR_BANK_SIZE_8K;
        self.chr.write_indexed(base, offset, data);
    }
}

impl Mapper for Mapper78 {
    fn power_on(&mut self) {
        self.prg_bank = 0;
        self.chr_bank = 0;
        // Keep header mirroring until the game sets it.
    }

    fn reset(&mut self) {
        self.power_on();
    }

    fn cpu_read(&self, addr: u16) -> Option<u8> {
        match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.read_prg_ram(addr),
            cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END => Some(self.read_prg_rom(addr)),
            _ => None,
        }
    }

    fn cpu_write(&mut self, addr: u16, data: u8, _cpu_cycle: u64) {
        match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.write_prg_ram(addr, data),
            cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END => {
                self.prg_bank = data & 0x0F;
                self.chr_bank = data >> 4;
                self.mirroring = if (data & 0x04) != 0 {
                    Mirroring::Horizontal
                } else {
                    Mirroring::Vertical
                };
            }
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
        78
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Irem 74HC161/32")
    }
}
