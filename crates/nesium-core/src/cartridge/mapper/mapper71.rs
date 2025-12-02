//! Mapper 71 – Camerica / Codemasters.
//!
//! Behaviour modelled after the common Camerica/Codemasters boards used by
//! Micro Machines and similar titles:
//! - CPU `$8000-$BFFF`: switchable 16 KiB PRG-ROM bank.
//! - CPU `$C000-$FFFF`: fixed to the last 16 KiB PRG-ROM bank.
//! - PPU `$0000-$1FFF`: 8 KiB CHR (typically CHR-RAM, though CHR-ROM is
//!   supported via the header).
//! - CPU `$8000-$FFFF` writes latch both the PRG bank (low nibble) and the
//!   mirroring bit (bit 4: 0 = single-screen lower, 1 = single-screen upper).
//! - Optional 8 KiB PRG-RAM at `$6000-$7FFF` respects NES 2.0 sizing.

use std::borrow::Cow;

use crate::{
    cartridge::{
        ChrRom, Mapper, PrgRom, TrainerBytes,
        header::{Header, Mirroring},
        mapper::{ChrStorage, allocate_prg_ram, select_chr_storage, trainer_destination},
    },
    memory::cpu as cpu_mem,
};

/// PRG-ROM banking granularity (16 KiB).
const PRG_BANK_SIZE_16K: usize = 16 * 1024;

#[derive(Debug, Clone)]
pub struct Mapper71 {
    prg_rom: PrgRom,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,

    /// Number of 16 KiB PRG-ROM banks.
    prg_bank_count_16k: usize,
    /// Switchable PRG bank mapped at `$8000-$BFFF`.
    prg_bank: u8,

    mirroring: Mirroring,
}

impl Mapper71 {
    pub fn new(header: Header, prg_rom: PrgRom, chr_rom: ChrRom) -> Self {
        Self::with_trainer(header, prg_rom, chr_rom, None)
    }

    pub(crate) fn with_trainer(
        header: Header,
        prg_rom: PrgRom,
        chr_rom: ChrRom,
        trainer: TrainerBytes,
    ) -> Self {
        let mut prg_ram = allocate_prg_ram(&header);
        if let (Some(trainer), Some(dst)) = (trainer, trainer_destination(&mut prg_ram)) {
            dst.copy_from_slice(trainer);
        }

        let chr = select_chr_storage(&header, chr_rom);
        let prg_bank_count_16k = (prg_rom.len() / PRG_BANK_SIZE_16K).max(1);

        Self {
            prg_rom,
            prg_ram,
            chr,
            prg_bank_count_16k,
            prg_bank: 0,
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
}

impl Mapper for Mapper71 {
    fn power_on(&mut self) {
        self.prg_bank = 0;
        // Keep header mirroring until the game writes a mirroring bit.
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
                self.mirroring = if (data & 0x10) != 0 {
                    Mirroring::SingleScreenUpper
                } else {
                    Mirroring::SingleScreenLower
                };
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
        71
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Camerica / Codemasters")
    }
}
