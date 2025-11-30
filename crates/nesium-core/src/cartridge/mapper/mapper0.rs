use std::borrow::Cow;

use crate::{
    cartridge::{
        Mapper, TRAINER_SIZE,
        header::{Header, Mirroring},
        mapper::{ChrStorage, allocate_prg_ram, select_chr_storage, trainer_destination},
    },
    memory::cpu as cpu_mem,
};

#[derive(Debug, Clone)]
pub struct Mapper0 {
    prg_rom: Box<[u8]>,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,
    mirroring: Mirroring,
}

impl Mapper0 {
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

        Self {
            prg_rom,
            prg_ram,
            chr: select_chr_storage(&header, chr_rom),
            mirroring: header.mirroring,
        }
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }
        let idx = (addr - cpu_mem::PRG_ROM_START) as usize % self.prg_rom.len();
        self.prg_rom[idx]
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
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let chr = vec![0; chr_rom_size].into_boxed_slice();
        Mapper0::new(header, prg, chr)
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
}
