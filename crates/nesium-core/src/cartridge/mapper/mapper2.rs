use std::borrow::Cow;

use crate::{
    cartridge::{
        Mapper, TRAINER_SIZE,
        header::{Header, Mirroring},
        mapper::{ChrStorage, allocate_prg_ram, select_chr_storage, trainer_destination},
    },
    memory::cpu as cpu_mem,
};

const PRG_BANK_SIZE: usize = 16 * 1024;

#[derive(Debug, Clone)]
pub struct Mapper2 {
    prg_rom: Box<[u8]>,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,
    selected_bank: usize,
    bank_count: usize,
    mirroring: Mirroring,
}

impl Mapper2 {
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

        let bank_count = (prg_rom.len() / PRG_BANK_SIZE).max(1);

        Self {
            prg_rom,
            prg_ram,
            chr: select_chr_storage(&header, chr_rom),
            selected_bank: 0,
            bank_count,
            mirroring: header.mirroring,
        }
    }

    fn fixed_bank(&self) -> usize {
        self.bank_count.saturating_sub(1)
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }

        let bank = if addr < 0xC000 {
            self.selected_bank
        } else {
            self.fixed_bank()
        };

        let offset = (addr as usize) & 0x3FFF;
        let base = bank * PRG_BANK_SIZE;
        let idx = base + offset;
        self.prg_rom.get(idx).copied().unwrap_or(0)
    }

    fn read_prg_ram(&self, addr: u16) -> u8 {
        if self.prg_ram.is_empty() {
            return 0;
        }
        let idx = (addr - cpu_mem::PRG_RAM_START) as usize % self.prg_ram.len();
        self.prg_ram[idx]
    }

    fn write_prg_ram(&mut self, addr: u16, data: u8) {
        if self.prg_ram.is_empty() {
            return;
        }
        let idx = (addr - cpu_mem::PRG_RAM_START) as usize % self.prg_ram.len();
        self.prg_ram[idx] = data;
    }

    fn write_bank_select(&mut self, data: u8) {
        if self.bank_count == 0 {
            return;
        }
        self.selected_bank = (data as usize) % self.bank_count;
    }
}

impl Mapper for Mapper2 {
    fn cpu_read(&self, addr: u16) -> u8 {
        match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.read_prg_ram(addr),
            cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END => self.read_prg_rom(addr),
            _ => 0,
        }
    }

    fn cpu_write(&mut self, addr: u16, data: u8) {
        match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.write_prg_ram(addr, data),
            cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END => self.write_bank_select(data),
            _ => {}
        }
    }

    fn ppu_read(&self, addr: u16) -> u8 {
        self.chr.read(addr)
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
        2
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("UxROM")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cartridge::header::{Header, Mirroring, RomFormat, TvSystem};

    fn header(prg_rom_size: usize, prg_ram_size: usize) -> Header {
        Header {
            format: RomFormat::INes,
            mapper: 2,
            submapper: 0,
            mirroring: Mirroring::Horizontal,
            battery_backed_ram: false,
            trainer_present: false,
            prg_rom_size,
            chr_rom_size: 0,
            prg_ram_size,
            prg_nvram_size: 0,
            chr_ram_size: 8 * 1024,
            chr_nvram_size: 0,
            vs_unisystem: false,
            playchoice_10: false,
            tv_system: TvSystem::Ntsc,
        }
    }

    fn cart_with_banks(bank_count: usize) -> Mapper2 {
        let mut prg_rom = vec![0u8; bank_count * PRG_BANK_SIZE];
        for bank in 0..bank_count {
            let start = bank * PRG_BANK_SIZE;
            let end = start + PRG_BANK_SIZE;
            prg_rom[start..end].fill(bank as u8);
        }
        let chr = Vec::new().into_boxed_slice();
        Mapper2::new(
            header(bank_count * PRG_BANK_SIZE, 8 * 1024),
            prg_rom.into_boxed_slice(),
            chr,
        )
    }

    #[test]
    fn switches_upper_bank() {
        let mut cart = cart_with_banks(4);
        let first = cart.cpu_read(cpu_mem::PRG_ROM_START);
        assert_eq!(first, 0);

        cart.cpu_write(cpu_mem::PRG_ROM_START, 0x02);
        let switched = cart.cpu_read(cpu_mem::PRG_ROM_START);
        assert_eq!(switched, 0x02);
    }

    #[test]
    fn fixes_high_bank_to_last() {
        let mut cart = cart_with_banks(4);
        cart.cpu_write(cpu_mem::PRG_ROM_START, 0x00);
        let high = cart.cpu_read(0xC000);
        assert_eq!(high, 0x03);
    }

    #[test]
    fn reads_and_writes_prg_ram() {
        let mut cart = cart_with_banks(4);
        cart.cpu_write(cpu_mem::PRG_RAM_START, 0x99);
        assert_eq!(cart.cpu_read(cpu_mem::PRG_RAM_START), 0x99);
    }
}
