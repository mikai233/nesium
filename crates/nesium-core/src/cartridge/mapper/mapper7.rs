use std::borrow::Cow;

use crate::{
    cartridge::{
        Mapper, TRAINER_SIZE,
        header::{Header, Mirroring},
        mapper::{ChrStorage, allocate_prg_ram, select_chr_storage, trainer_destination},
    },
    memory::cpu as cpu_mem,
};

const PRG_BANK_SIZE: usize = 32 * 1024;

#[derive(Debug, Clone)]
pub struct Mapper7 {
    prg_rom: Box<[u8]>,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,
    selected_bank: usize,
    mirroring: Mirroring,
    bank_count: usize,
}

impl Mapper7 {
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
            mirroring: header.mirroring,
            bank_count,
        }
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }
        let bank = self.selected_bank % self.bank_count;
        let base = bank * PRG_BANK_SIZE;
        let offset = (addr as usize - cpu_mem::PRG_ROM_START as usize) % PRG_BANK_SIZE;
        self.prg_rom
            .get(base + offset)
            .copied()
            .unwrap_or_else(|| self.prg_rom[offset % self.prg_rom.len()])
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
        self.selected_bank = (data & 0b0001_1111) as usize;
        // AxROM uses single-screen mirroring, selecting either the lower
        // (`$2000`) or upper (`$2400`) nametable. Bit 4 chooses the target.
        self.mirroring = if data & 0b0001_0000 == 0 {
            Mirroring::SingleScreenLower
        } else {
            Mirroring::SingleScreenUpper
        };
    }
}

impl Mapper for Mapper7 {
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
        7
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("AxROM")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cartridge::header::{Header, RomFormat, TvSystem};

    fn header(prg_rom_size: usize) -> Header {
        Header {
            format: RomFormat::INes,
            mapper: 7,
            submapper: 0,
            mirroring: Mirroring::Horizontal,
            battery_backed_ram: false,
            trainer_present: false,
            prg_rom_size,
            chr_rom_size: 0,
            prg_ram_size: 8 * 1024,
            prg_nvram_size: 0,
            chr_ram_size: 8 * 1024,
            chr_nvram_size: 0,
            vs_unisystem: false,
            playchoice_10: false,
            tv_system: TvSystem::Ntsc,
        }
    }

    fn cart(prg_banks: usize) -> Mapper7 {
        let mut prg = vec![0u8; prg_banks * PRG_BANK_SIZE];
        for bank in 0..prg_banks {
            let start = bank * PRG_BANK_SIZE;
            let end = start + PRG_BANK_SIZE;
            prg[start..end].fill(bank as u8);
        }

        Mapper7::new(
            header(prg.len()),
            prg.into_boxed_slice(),
            vec![0; 0].into_boxed_slice(),
        )
    }

    #[test]
    fn switches_prg_rom_banks() {
        let mut cart = cart(4);
        assert_eq!(cart.cpu_read(cpu_mem::PRG_ROM_START), Some(0));

        cart.cpu_write(cpu_mem::PRG_ROM_START, 0x02, 0);
        assert_eq!(cart.cpu_read(cpu_mem::PRG_ROM_START), Some(0x02));
    }

    #[test]
    fn writes_prg_ram() {
        let mut cart = cart(2);
        cart.cpu_write(cpu_mem::PRG_RAM_START, 0x55, 0);
        assert_eq!(cart.cpu_read(cpu_mem::PRG_RAM_START), Some(0x55));
    }

    #[test]
    fn updates_mirroring_flag() {
        let mut cart = cart(2);
        cart.cpu_write(cpu_mem::PRG_ROM_START, 0b0001_0000, 0);
        assert_eq!(cart.mirroring, Mirroring::SingleScreenUpper);

        cart.cpu_write(cpu_mem::PRG_ROM_START, 0, 1);
        assert_eq!(cart.mirroring, Mirroring::SingleScreenLower);
    }
}
