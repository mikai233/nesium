//! Mapper 66 – GxROM / GNROM: simple PRG/CHR bank switch.
//!
//! - CPU `$8000-$FFFF`: single register, bits 4-5 select 32 KiB PRG bank,
//!   bits 0-1 select 8 KiB CHR bank. Other bits ignored.
//! - CPU `$6000-$7FFF`: optional PRG-RAM (battery-backed or work RAM).
//! - PPU `$0000-$1FFF`: 8 KiB CHR bank (ROM or RAM) selected via the register.
//!
//! Bus conflicts on PRG writes are ignored here, matching the approach in
//! Mesen2's `GxRom` implementation.

use std::borrow::Cow;

use crate::{
    cartridge::{
        Mapper, TRAINER_SIZE,
        header::{Header, Mirroring},
        mapper::{ChrStorage, allocate_prg_ram, select_chr_storage, trainer_destination},
    },
    memory::cpu as cpu_mem,
};

/// PRG banking granularity (32 KiB).
const PRG_BANK_SIZE_32K: usize = 32 * 1024;
/// CHR banking granularity (8 KiB).
const CHR_BANK_SIZE_8K: usize = 8 * 1024;

#[derive(Debug, Clone)]
pub struct Mapper66 {
    prg_rom: Box<[u8]>,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,

    /// Number of 32 KiB PRG-ROM banks.
    prg_bank_count_32k: usize,

    /// Latched PRG bank (bits 4-5 of the last write).
    prg_bank: u8,
    /// Latched CHR bank (bits 0-1 of the last write).
    chr_bank: u8,

    mirroring: Mirroring,
}

impl Mapper66 {
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
        let prg_bank_count_32k = (prg_rom.len() / PRG_BANK_SIZE_32K).max(1);

        Self {
            prg_rom,
            prg_ram,
            chr,
            prg_bank_count_32k,
            prg_bank: 0,
            chr_bank: 0,
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

    fn read_chr(&self, addr: u16) -> u8 {
        let offset = (addr as usize) % CHR_BANK_SIZE_8K;
        let base = (self.chr_bank as usize % 4) * CHR_BANK_SIZE_8K;
        self.chr.read_indexed(base, offset)
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        let offset = (addr as usize) % CHR_BANK_SIZE_8K;
        let base = (self.chr_bank as usize % 4) * CHR_BANK_SIZE_8K;
        self.chr.write_indexed(base, offset, data);
    }
}

impl Mapper for Mapper66 {
    fn power_on(&mut self) {
        self.prg_bank = 0;
        self.chr_bank = 0;
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
                // Bits 4-5 select PRG bank; bits 0-1 select CHR bank.
                self.prg_bank = (data >> 4) & 0x03;
                self.chr_bank = data & 0x03;
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
        66
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("GxROM / GNROM")
    }
}
