use std::borrow::Cow;

use crate::{
    cartridge::{
        Mapper, TRAINER_SIZE,
        header::{Header, Mirroring},
        mapper::{allocate_prg_ram, trainer_destination},
    },
    memory::cpu as cpu_mem,
};

const PRG_BANK_SIZE_16K: usize = 16 * 1024;
const CHR_BANK_SIZE_4K: usize = 4 * 1024;

#[derive(Debug, Clone)]
pub struct Mapper1 {
    prg_rom: Box<[u8]>,
    prg_ram: Box<[u8]>,
    chr_rom: Box<[u8]>,
    chr_ram: Box<[u8]>,

    prg_bank_count: usize, // number of 16 KiB PRG banks
    chr_bank_count: usize, // number of 4 KiB CHR banks

    control: u8,
    chr_bank0: u8,
    chr_bank1: u8,
    prg_bank: u8,

    shift_reg: u8,
    shift_count: u8,
}

impl Mapper1 {
    fn power_on_init(&mut self) {
        // Power-on defaults per MMC1 documentation:
        // - Control = 0b11xx (16 KiB banking, fixed high bank)
        // - PRG/CHR banks = 0
        // - Shift register cleared with bit4 set so the next 5 writes latch cleanly.
        self.control = 0x0C;
        self.chr_bank0 = 0;
        self.chr_bank1 = 0;
        self.prg_bank = 0;
        self.shift_reg = 0x10;
        self.shift_count = 0;
    }

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

        let chr_rom_present = header.chr_rom_size > 0;
        let chr_ram = if chr_rom_present {
            Vec::new().into_boxed_slice()
        } else {
            allocate_chr_ram(&header)
        };

        let chr_len = if !chr_rom.is_empty() {
            chr_rom.len()
        } else {
            chr_ram.len()
        };

        let chr_bank_count = if chr_len == 0 {
            0
        } else {
            (chr_len / CHR_BANK_SIZE_4K).max(1)
        };

        let prg_bank_count = (prg_rom.len() / PRG_BANK_SIZE_16K).max(1);

        let mut mapper = Self {
            prg_rom,
            prg_ram,
            chr_rom,
            chr_ram,
            prg_bank_count,
            chr_bank_count,
            control: 0,
            chr_bank0: 0,
            chr_bank1: 0,
            prg_bank: 0,
            shift_reg: 0,
            shift_count: 0,
        };

        mapper.power_on_init();
        mapper
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }

        let bank_index = self.prg_bank_for_cpu_addr(addr);
        let bank_offset = (addr as usize - cpu_mem::PRG_ROM_START as usize) % PRG_BANK_SIZE_16K;
        let base = bank_index.saturating_mul(PRG_BANK_SIZE_16K);
        let idx = base.saturating_add(bank_offset);
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

    fn prg_bank_for_cpu_addr(&self, addr: u16) -> usize {
        if self.prg_bank_count == 0 {
            return 0;
        }

        let mode = (self.control >> 2) & 0b11;
        let bank = (self.prg_bank & 0x0F) as usize;

        match mode {
            // 32 KiB mode, ignore low bit of PRG bank.
            0 | 1 => {
                if self.prg_bank_count == 1 {
                    0
                } else {
                    let max_even_bank = self.prg_bank_count.saturating_sub(2);
                    let bank_even = (bank & !1).min(max_even_bank);
                    if addr < 0xC000 {
                        bank_even
                    } else {
                        bank_even + 1
                    }
                }
            }
            // Fix first 16 KiB at $8000, switch 16 KiB at $C000.
            2 => {
                if addr < 0xC000 {
                    0
                } else {
                    bank.min(self.prg_bank_count - 1)
                }
            }
            // Fix last 16 KiB at $C000, switch 16 KiB at $8000.
            _ => {
                if addr < 0xC000 {
                    bank.min(self.prg_bank_count - 1)
                } else {
                    self.prg_bank_count - 1
                }
            }
        }
    }

    fn read_chr(&self, addr: u16) -> u8 {
        let (mem, is_rom) = if !self.chr_rom.is_empty() {
            (self.chr_rom.as_ref(), true)
        } else {
            (self.chr_ram.as_ref(), false)
        };

        if mem.is_empty() {
            return 0;
        }

        let offset_in_bank = (addr as usize) & 0x0FFF;
        let chr_mode_4k = (self.control >> 4) & 0b1 == 1;

        let bank_index = if !chr_mode_4k {
            // 8 KiB CHR mode: ignore low bit of bank 0.
            let base_bank = (self.chr_bank0 & !1) as usize;
            if addr < 0x1000 {
                base_bank
            } else {
                base_bank + 1
            }
        } else if addr < 0x1000 {
            self.chr_bank0 as usize
        } else {
            self.chr_bank1 as usize
        };

        let total_banks = if self.chr_bank_count == 0 {
            1
        } else {
            self.chr_bank_count
        };

        let bank = bank_index % total_banks;
        let base = bank * CHR_BANK_SIZE_4K;
        let idx = base.saturating_add(offset_in_bank);

        if is_rom {
            mem.get(idx).copied().unwrap_or(0)
        } else {
            mem.get(idx).copied().unwrap_or(0)
        }
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        if !self.chr_rom.is_empty() || self.chr_ram.is_empty() {
            return;
        }

        let offset_in_bank = (addr as usize) & 0x0FFF;
        let chr_mode_4k = (self.control >> 4) & 0b1 == 1;

        let bank_index = if !chr_mode_4k {
            let base_bank = (self.chr_bank0 & !1) as usize;
            if addr < 0x1000 {
                base_bank
            } else {
                base_bank + 1
            }
        } else if addr < 0x1000 {
            self.chr_bank0 as usize
        } else {
            self.chr_bank1 as usize
        };

        let total_banks = if self.chr_bank_count == 0 {
            1
        } else {
            self.chr_bank_count
        };

        let bank = bank_index % total_banks;
        let base = bank * CHR_BANK_SIZE_4K;
        let idx = base.saturating_add(offset_in_bank);

        if let Some(byte) = self.chr_ram.get_mut(idx) {
            *byte = data;
        }
    }

    fn write_register(&mut self, addr: u16, data: u8) {
        if data & 0x80 != 0 {
            // Reset shift register and force 16 KiB PRG banking with fixed high bank.
            // PRG/CHR bank registers keep their current values.
            self.shift_reg = 0x10;
            self.shift_count = 0;
            self.control |= 0x0C;
            return;
        }

        let bit = data & 1;
        self.shift_reg >>= 1;
        self.shift_reg |= bit << 4;
        self.shift_count = self.shift_count.saturating_add(1);

        if self.shift_count == 5 {
            let value = self.shift_reg & 0x1F;
            let target = (addr >> 13) & 0b11;

            match target {
                0 => {
                    // Control register: mirroring / PRG / CHR mode.
                    self.control = value;
                }
                1 => {
                    // CHR bank 0 (4 KiB)
                    self.chr_bank0 = value;
                }
                2 => {
                    // CHR bank 1 (4 KiB)
                    self.chr_bank1 = value;
                }
                3 => {
                    // PRG bank select
                    self.prg_bank = value;
                }
                _ => {}
            }

            self.shift_reg = 0;
            self.shift_count = 0;
        }
    }
}

impl Mapper for Mapper1 {
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
            cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END => self.write_register(addr, data),
            _ => {}
        }
    }

    fn ppu_read(&self, addr: u16) -> u8 {
        self.read_chr(addr)
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
        match self.control & 0b11 {
            0 => Mirroring::SingleScreenLower,
            1 => Mirroring::SingleScreenUpper,
            2 => Mirroring::Vertical,
            _ => Mirroring::Horizontal,
        }
    }

    fn mapper_id(&self) -> u16 {
        1
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("MMC1")
    }
}

fn allocate_chr_ram(header: &Header) -> Box<[u8]> {
    let size = header.chr_ram_size.max(header.chr_nvram_size);
    if size == 0 {
        Vec::new().into_boxed_slice()
    } else {
        vec![0; size].into_boxed_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cartridge::header::{Header, Mirroring, RomFormat, TvSystem};

    fn header(prg_rom_size: usize, chr_rom_size: usize, chr_ram_size: usize) -> Header {
        Header {
            format: RomFormat::INes,
            mapper: 1,
            submapper: 0,
            mirroring: Mirroring::Horizontal,
            battery_backed_ram: false,
            trainer_present: false,
            prg_rom_size,
            chr_rom_size,
            prg_ram_size: 8 * 1024,
            prg_nvram_size: 0,
            chr_ram_size,
            chr_nvram_size: 0,
            vs_unisystem: false,
            playchoice_10: false,
            tv_system: TvSystem::Ntsc,
        }
    }

    fn cart_with_prg_banks(banks_16k: usize) -> Mapper1 {
        let mut prg = vec![0u8; banks_16k * PRG_BANK_SIZE_16K];
        for bank in 0..banks_16k {
            let start = bank * PRG_BANK_SIZE_16K;
            let end = start + PRG_BANK_SIZE_16K;
            prg[start..end].fill(bank as u8);
        }

        let chr_rom = Vec::new().into_boxed_slice();

        Mapper1::new(
            header(prg.len(), 0, 8 * 1024),
            prg.into_boxed_slice(),
            chr_rom,
        )
    }

    fn write_serial_reg(mapper: &mut Mapper1, addr: u16, value: u8) {
        for i in 0..5 {
            let bit = (value >> i) & 1;
            mapper.cpu_write(addr, bit);
        }
    }

    #[test]
    fn default_prg_banking_mode_is_fixed_last_bank() {
        let cart = cart_with_prg_banks(4);
        // Control defaults to 0x0C: 16 KiB banking with fixed last bank at $C000.
        assert_eq!(cart.cpu_read(cpu_mem::PRG_ROM_START), 0);
        assert_eq!(cart.cpu_read(0xC000), 3);
    }

    #[test]
    fn switches_prg_bank_in_mode3() {
        let mut cart = cart_with_prg_banks(4);
        // Select bank 2 at $8000 in mode 3 (control already 0x0C).
        write_serial_reg(&mut cart, 0xE000, 0x02);
        assert_eq!(cart.cpu_read(cpu_mem::PRG_ROM_START), 2);
        // High bank should remain fixed to last bank.
        assert_eq!(cart.cpu_read(0xC000), 3);
    }
}
