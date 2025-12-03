//! Mapper 34 – Irem BNROM / NINA-001 style simple 32 KiB PRG banking.
//!
//! Modelled after Mesen2's `BnRom`: a single 32 KiB switchable PRG window
//! mapped to `$8000-$FFFF`, with CHR provided directly by the cartridge
//! storage (typically CHR-RAM). Writes to `$8000-$FFFF` latch the PRG bank
//! number; bus conflicts are ignored.

use std::borrow::Cow;

use crate::{
    cartridge::{
        ChrRom, Mapper, PrgRom, TrainerBytes,
        header::{Header, Mirroring},
        mapper::{ChrStorage, allocate_prg_ram_with_trainer, select_chr_storage},
    },
    memory::cpu as cpu_mem,
};

/// PRG-ROM banking granularity (32 KiB).
const PRG_BANK_SIZE_32K: usize = 32 * 1024;

#[derive(Debug, Clone)]
pub struct Mapper34 {
    prg_rom: PrgRom,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,

    /// Number of 32 KiB PRG-ROM banks.
    prg_bank_count_32k: usize,
    /// Currently selected 32 KiB PRG bank.
    prg_bank: u8,

    mirroring: Mirroring,
}

impl Mapper34 {
    pub fn new(header: Header, prg_rom: PrgRom, chr_rom: ChrRom, trainer: TrainerBytes) -> Self {
        let prg_ram = allocate_prg_ram_with_trainer(&header, trainer);

        let chr = select_chr_storage(&header, chr_rom);
        let prg_bank_count_32k = (prg_rom.len() / PRG_BANK_SIZE_32K).max(1);

        Self {
            prg_rom,
            prg_ram,
            chr,
            prg_bank_count_32k,
            prg_bank: 0,
            mirroring: header.mirroring,
        }
    }

    #[inline]
    fn prg_bank_index(&self, reg_value: u8) -> usize {
        if self.prg_bank_count_32k == 0 {
            0
        } else {
            (reg_value as usize) % self.prg_bank_count_32k
        }
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }

        let bank = self.prg_bank_index(self.prg_bank);
        let base = bank.saturating_mul(PRG_BANK_SIZE_32K);
        let offset = (addr.saturating_sub(cpu_mem::PRG_ROM_START) as usize) % PRG_BANK_SIZE_32K;
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
}

impl Mapper for Mapper34 {
    fn power_on(&mut self) {
        self.prg_bank = 0;
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
                // Latch 32 KiB PRG bank; bus conflicts ignored.
                self.prg_bank = data;
            }
            _ => {}
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
        34
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Irem BNROM / NINA-001")
    }
}
