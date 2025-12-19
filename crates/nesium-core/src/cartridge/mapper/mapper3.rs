use std::borrow::Cow;

use crate::{
    cartridge::{
        ChrRom, Mapper, PrgRom, TrainerBytes,
        header::{Header, Mirroring},
        mapper::allocate_prg_ram_with_trainer,
    },
    memory::cpu as cpu_mem,
};

// Mapper 3 – CnROM 8 KiB CHR banking.
//
// | Area | Address range     | Behaviour                                  | IRQ/Audio |
// |------|-------------------|--------------------------------------------|-----------|
// | CPU  | `$6000-$7FFF`     | Optional PRG-RAM                           | None      |
// | CPU  | `$8000-$FFFF`     | Fixed 32 KiB PRG-ROM (mirrored if smaller) | None      |
// | PPU  | `$0000-$1FFF`     | 8 KiB CHR ROM/RAM, banked via `$8000-$FFFF`| None      |
// | PPU  | `$2000-$3EFF`     | Mirroring from header (no mapper control)  | None      |

const CHR_BANK_SIZE: usize = 8 * 1024;

#[derive(Debug, Clone)]
pub struct Mapper3 {
    prg_rom: PrgRom,
    prg_ram: Box<[u8]>,
    chr_rom: ChrRom,
    chr_ram: Box<[u8]>,
    chr_bank: usize,
    chr_bank_count: usize,
    mirroring: Mirroring,
}

impl Mapper3 {
    pub fn new(header: Header, prg_rom: PrgRom, chr_rom: ChrRom, trainer: TrainerBytes) -> Self {
        let prg_ram = allocate_prg_ram_with_trainer(&header, trainer);

        let chr_bank_count = if chr_rom.is_empty() {
            0
        } else {
            (chr_rom.len() / CHR_BANK_SIZE).max(1)
        };

        Self {
            prg_rom,
            prg_ram,
            chr_rom,
            chr_ram: allocate_chr_ram(&header),
            chr_bank: 0,
            chr_bank_count,
            mirroring: header.mirroring(),
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

    fn read_chr(&self, addr: u16) -> u8 {
        let offset = (addr as usize) & 0x1FFF;
        if !self.chr_rom.is_empty() && self.chr_bank_count > 0 {
            let bank = self.chr_bank % self.chr_bank_count;
            let start = bank * CHR_BANK_SIZE;
            let idx = start + (offset % CHR_BANK_SIZE);
            return self.chr_rom.get(idx).copied().unwrap_or(0);
        }

        if self.chr_ram.is_empty() {
            return 0;
        }
        self.chr_ram[offset % self.chr_ram.len()]
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        if !self.chr_rom.is_empty() {
            return;
        }
        if self.chr_ram.is_empty() {
            return;
        }
        let offset = (addr as usize) & 0x1FFF;
        let idx = offset % self.chr_ram.len();
        self.chr_ram[idx] = data;
    }

    fn write_chr_bank(&mut self, data: u8) {
        if self.chr_bank_count == 0 {
            return;
        }
        self.chr_bank = (data as usize) % self.chr_bank_count;
    }
}

impl Mapper for Mapper3 {
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
            cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END => self.write_chr_bank(data),
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
        if self.chr_rom.is_empty() {
            None
        } else {
            Some(self.chr_rom.as_ref())
        }
    }

    fn chr_ram(&self) -> Option<&[u8]> {
        if self.chr_ram.is_empty() {
            None
        } else {
            Some(self.chr_ram.as_ref())
        }
    }

    fn chr_ram_mut(&mut self) -> Option<&mut [u8]> {
        if self.chr_ram.is_empty() {
            None
        } else {
            Some(self.chr_ram.as_mut())
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn mapper_id(&self) -> u16 {
        3
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("CnROM")
    }
}

fn allocate_chr_ram(header: &Header) -> Box<[u8]> {
    let size = header.chr_ram_size().max(header.chr_nvram_size());
    if size == 0 {
        Vec::new().into_boxed_slice()
    } else {
        vec![0; size].into_boxed_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cartridge::header::Header;

    fn header(prg_rom_size: usize, chr_rom_size: usize, chr_ram_size: usize) -> Header {
        let _ = chr_ram_size; // iNES 1.0 does not encode CHR RAM size beyond "present/absent".

        let prg_rom_units = (prg_rom_size / (16 * 1024)) as u8;
        let chr_rom_units = (chr_rom_size / (8 * 1024)) as u8;

        let flags6 = 0x30; // mapper 3 + horizontal mirroring
        let prg_ram_units = 1; // 8 KiB
        let header_bytes = [
            b'N',
            b'E',
            b'S',
            0x1A,
            prg_rom_units,
            chr_rom_units,
            flags6,
            0,
            prg_ram_units,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ];

        Header::parse(&header_bytes).expect("header parses")
    }

    fn rom_cart(prg_banks: usize, chr_banks: usize) -> Mapper3 {
        let mut prg = vec![0u8; prg_banks * 16 * 1024];
        for (i, byte) in prg.iter_mut().enumerate() {
            *byte = (i & 0xFF) as u8;
        }
        let mut chr = vec![0u8; chr_banks * CHR_BANK_SIZE];
        for bank in 0..chr_banks {
            let start = bank * CHR_BANK_SIZE;
            let end = start + CHR_BANK_SIZE;
            chr[start..end].fill(bank as u8);
        }

        Mapper3::new(
            header(prg.len(), chr.len(), 0),
            prg.into(),
            chr.into(),
            None,
        )
    }

    #[test]
    fn switches_chr_banks() {
        let mut cart = rom_cart(2, 4);
        assert_eq!(cart.ppu_read(0x0000), Some(0));

        cart.cpu_write(cpu_mem::PRG_ROM_START, 0x02, 0);
        assert_eq!(cart.ppu_read(0x0000), Some(0x02));
    }

    #[test]
    fn prg_rom_mirrors() {
        let cart = rom_cart(1, 2);
        let a = cart.cpu_read(cpu_mem::PRG_ROM_START).unwrap();
        let b = cart.cpu_read(cpu_mem::PRG_ROM_START + 0x4000).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn falls_back_to_chr_ram() {
        let header = header(0x8000, 0, 8 * 1024);
        let prg = vec![0; 0x8000].into();
        let chr = Vec::new().into();
        let mut cart = Mapper3::new(header, prg, chr, None);

        cart.ppu_write(0x0010, 0x77);
        assert_eq!(cart.ppu_read(0x0010), Some(0x77));
    }
}
