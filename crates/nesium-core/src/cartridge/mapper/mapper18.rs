//! Mapper 18 - Jaleco SS88006.
//!
//! This board provides:
//! - 8 KiB PRG banking at `$8000-$DFFF` with a fixed last bank at `$E000-$FFFF`
//! - Eight 1 KiB CHR banks
//! - Mirroring control via `$F002`
//! - CPU-cycle IRQ counter with selectable 4/8/12/16-bit width

use std::borrow::Cow;

use crate::cartridge::mapper::{MapperMemoryMut, MapperMemoryRef};

use crate::{
    cartridge::{
        ChrRom, Mapper, PrgRom, TrainerBytes,
        header::{Header, Mirroring},
        mapper::{
            ChrStorage, MapperEvent, MapperHookMask, allocate_prg_ram_with_trainer,
            select_chr_storage,
        },
    },
    memory::cpu as cpu_mem,
    reset_kind::ResetKind,
};

const PRG_BANK_SIZE_8K: usize = 8 * 1024;
const CHR_BANK_SIZE_1K: usize = 1024;
const IRQ_MASKS: [u16; 4] = [0xFFFF, 0x0FFF, 0x00FF, 0x000F];

#[derive(Debug, Clone)]
pub struct Mapper18 {
    prg_rom: PrgRom,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,

    prg_bank_count_8k: usize,
    prg_banks: [u8; 3],
    chr_banks: [u8; 8],
    irq_reload_nibbles: [u8; 4],
    irq_counter: u16,
    irq_counter_size: u8,
    irq_enabled: bool,
    irq_pending: bool,
    base_mirroring: Mirroring,
    mirroring: Mirroring,
}

impl Mapper18 {
    pub fn new(header: Header, prg_rom: PrgRom, chr_rom: ChrRom, trainer: TrainerBytes) -> Self {
        let prg_ram = allocate_prg_ram_with_trainer(&header, trainer);
        let chr = select_chr_storage(&header, chr_rom);
        let prg_bank_count_8k = (prg_rom.len() / PRG_BANK_SIZE_8K).max(1);

        Self {
            prg_rom,
            prg_ram,
            chr,
            prg_bank_count_8k,
            prg_banks: [0; 3],
            chr_banks: [0; 8],
            irq_reload_nibbles: [0; 4],
            irq_counter: 0,
            irq_counter_size: 0,
            irq_enabled: false,
            irq_pending: false,
            base_mirroring: header.mirroring(),
            mirroring: header.mirroring(),
        }
    }

    #[inline]
    fn prg_bank_index(&self, bank: u8) -> usize {
        (bank as usize) % self.prg_bank_count_8k
    }

    fn read_prg_rom_bank(&self, bank: usize, addr: u16) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }
        let base = bank.saturating_mul(PRG_BANK_SIZE_8K);
        let offset = (addr as usize) & (PRG_BANK_SIZE_8K - 1);
        self.prg_rom.get(base + offset).copied().unwrap_or(0)
    }

    fn read_prg_rom_window(&self, addr: u16) -> Option<u8> {
        let bank = match addr {
            0x8000..=0x9FFF => self.prg_bank_index(self.prg_banks[0]),
            0xA000..=0xBFFF => self.prg_bank_index(self.prg_banks[1]),
            0xC000..=0xDFFF => self.prg_bank_index(self.prg_banks[2]),
            0xE000..=0xFFFF => self.prg_bank_count_8k.saturating_sub(1),
            _ => return None,
        };
        Some(self.read_prg_rom_bank(bank, addr))
    }

    fn read_prg_ram(&self, addr: u16) -> Option<u8> {
        if self.prg_ram.is_empty() {
            return None;
        }
        let offset = ((addr - cpu_mem::PRG_RAM_START) as usize) % self.prg_ram.len();
        Some(self.prg_ram[offset])
    }

    fn write_prg_ram(&mut self, addr: u16, value: u8) {
        if self.prg_ram.is_empty() {
            return;
        }
        let offset = ((addr - cpu_mem::PRG_RAM_START) as usize) % self.prg_ram.len();
        self.prg_ram[offset] = value;
    }

    fn read_chr(&self, addr: u16) -> u8 {
        let slot = ((addr >> 10) & 0x07) as usize;
        let bank = self.chr_banks[slot] as usize;
        let base = bank.saturating_mul(CHR_BANK_SIZE_1K);
        let offset = (addr & 0x03FF) as usize;
        self.chr.read_indexed(base, offset)
    }

    fn write_chr(&mut self, addr: u16, value: u8) {
        let slot = ((addr >> 10) & 0x07) as usize;
        let bank = self.chr_banks[slot] as usize;
        let base = bank.saturating_mul(CHR_BANK_SIZE_1K);
        let offset = (addr & 0x03FF) as usize;
        self.chr.write_indexed(base, offset, value);
    }

    fn update_prg_bank(&mut self, bank_number: usize, value: u8, update_upper_bits: bool) {
        if update_upper_bits {
            self.prg_banks[bank_number] = (self.prg_banks[bank_number] & 0x0F) | (value << 4);
        } else {
            self.prg_banks[bank_number] = (self.prg_banks[bank_number] & 0xF0) | value;
        }
    }

    fn update_chr_bank(&mut self, bank_number: usize, value: u8, update_upper_bits: bool) {
        if update_upper_bits {
            self.chr_banks[bank_number] = (self.chr_banks[bank_number] & 0x0F) | (value << 4);
        } else {
            self.chr_banks[bank_number] = (self.chr_banks[bank_number] & 0xF0) | value;
        }
    }

    fn reload_irq_counter(&mut self) {
        self.irq_counter = self.irq_reload_nibbles[0] as u16
            | ((self.irq_reload_nibbles[1] as u16) << 4)
            | ((self.irq_reload_nibbles[2] as u16) << 8)
            | ((self.irq_reload_nibbles[3] as u16) << 12);
    }

    fn clock_irq_counter(&mut self) {
        if !self.irq_enabled {
            return;
        }

        let mask = IRQ_MASKS[self.irq_counter_size as usize];
        let counter = self.irq_counter & mask;
        let next = counter.wrapping_sub(1) & mask;
        if next == 0 {
            self.irq_pending = true;
        }
        self.irq_counter = (self.irq_counter & !mask) | next;
    }

    fn write_register(&mut self, addr: u16, data: u8) {
        let update_upper_bits = (addr & 0x01) != 0;
        let value = data & 0x0F;

        match addr & 0xF003 {
            0x8000 | 0x8001 => self.update_prg_bank(0, value, update_upper_bits),
            0x8002 | 0x8003 => self.update_prg_bank(1, value, update_upper_bits),
            0x9000 | 0x9001 => self.update_prg_bank(2, value, update_upper_bits),
            0xA000 | 0xA001 => self.update_chr_bank(0, value, update_upper_bits),
            0xA002 | 0xA003 => self.update_chr_bank(1, value, update_upper_bits),
            0xB000 | 0xB001 => self.update_chr_bank(2, value, update_upper_bits),
            0xB002 | 0xB003 => self.update_chr_bank(3, value, update_upper_bits),
            0xC000 | 0xC001 => self.update_chr_bank(4, value, update_upper_bits),
            0xC002 | 0xC003 => self.update_chr_bank(5, value, update_upper_bits),
            0xD000 | 0xD001 => self.update_chr_bank(6, value, update_upper_bits),
            0xD002 | 0xD003 => self.update_chr_bank(7, value, update_upper_bits),
            0xE000 | 0xE001 | 0xE002 | 0xE003 => {
                self.irq_reload_nibbles[(addr & 0x03) as usize] = value;
            }
            0xF000 => {
                self.irq_pending = false;
                self.reload_irq_counter();
            }
            0xF001 => {
                self.irq_pending = false;
                self.irq_enabled = value & 0x01 != 0;
                self.irq_counter_size = if value & 0x08 != 0 {
                    3
                } else if value & 0x04 != 0 {
                    2
                } else if value & 0x02 != 0 {
                    1
                } else {
                    0
                };
            }
            0xF002 => {
                self.mirroring = match value & 0x03 {
                    0 => Mirroring::Horizontal,
                    1 => Mirroring::Vertical,
                    2 => Mirroring::SingleScreenLower,
                    _ => Mirroring::SingleScreenUpper,
                };
            }
            0xF003 => {}
            _ => {}
        }
    }
}

impl Mapper for Mapper18 {
    fn cpu_read(&self, addr: u16, _open_bus: u8) -> Option<u8> {
        match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.read_prg_ram(addr),
            cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END => self.read_prg_rom_window(addr),
            _ => None,
        }
    }

    fn cpu_write(&mut self, addr: u16, data: u8, _cpu_cycle: u64) {
        match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.write_prg_ram(addr, data),
            cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END => self.write_register(addr, data),
            _ => {}
        }
    }

    fn hook_mask(&self) -> MapperHookMask {
        MapperHookMask::CPU_CLOCK
    }

    fn on_mapper_event(&mut self, event: MapperEvent) {
        if let MapperEvent::CpuClock { .. } = event {
            self.clock_irq_counter();
        }
    }

    fn reset(&mut self, _kind: ResetKind) {
        self.prg_banks = [0; 3];
        self.chr_banks = [0; 8];
        self.irq_reload_nibbles = [0; 4];
        self.irq_counter = 0;
        self.irq_counter_size = 0;
        self.irq_enabled = false;
        self.irq_pending = false;
        self.mirroring = self.base_mirroring;
    }

    fn ppu_read(&self, addr: u16) -> Option<u8> {
        Some(self.read_chr(addr))
    }

    fn ppu_write(&mut self, addr: u16, data: u8) {
        self.write_chr(addr, data);
    }

    fn irq_pending(&self) -> bool {
        self.irq_pending
    }

    fn memory_ref(&self) -> MapperMemoryRef<'_> {
        MapperMemoryRef {
            prg_rom: Some(self.prg_rom.as_ref()),
            prg_ram: (!self.prg_ram.is_empty()).then_some(self.prg_ram.as_ref()),
            prg_work_ram: None,
            mapper_ram: None,
            chr_rom: self.chr.as_rom(),
            chr_ram: self.chr.as_ram(),
            chr_battery_ram: None,
        }
    }

    fn memory_mut(&mut self) -> MapperMemoryMut<'_> {
        MapperMemoryMut {
            prg_ram: (!self.prg_ram.is_empty()).then_some(self.prg_ram.as_mut()),
            prg_work_ram: None,
            mapper_ram: None,
            chr_ram: self.chr.as_ram_mut(),
            chr_battery_ram: None,
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn mapper_id(&self) -> u16 {
        18
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Jaleco SS88006")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::cartridge::header::Header;

    fn test_header(prg_16k_units: u8, chr_8k_units: u8) -> Header {
        let mut rom = [0u8; 16];
        rom[0..4].copy_from_slice(b"NES\x1A");
        rom[4] = prg_16k_units;
        rom[5] = chr_8k_units;
        // Mapper 18 = 0x12.
        rom[6] = 0x20;
        rom[7] = 0x10;
        Header::parse(&rom).expect("valid iNES header")
    }

    fn test_mapper(prg_banks_8k: usize, chr_banks_1k: usize) -> Mapper18 {
        let header = test_header((prg_banks_8k / 2) as u8, (chr_banks_1k / 8) as u8);
        let mut prg = vec![0u8; prg_banks_8k * PRG_BANK_SIZE_8K];
        for (bank, chunk) in prg.chunks_exact_mut(PRG_BANK_SIZE_8K).enumerate() {
            chunk.fill(bank as u8);
        }
        let mut chr = vec![0u8; chr_banks_1k * CHR_BANK_SIZE_1K];
        for (bank, chunk) in chr.chunks_exact_mut(CHR_BANK_SIZE_1K).enumerate() {
            chunk.fill(bank as u8);
        }

        Mapper18::new(header, prg.into(), chr.into(), None)
    }

    #[test]
    fn prg_banking_and_fixed_last_bank() {
        let mut mapper = test_mapper(8, 32);
        mapper.reset(ResetKind::PowerOn);

        assert_eq!(mapper.cpu_read(0xE000, 0), Some(7));

        mapper.cpu_write(0x8000, 0x03, 0);
        mapper.cpu_write(0x8001, 0x01, 0);
        mapper.cpu_write(0x8002, 0x04, 0);
        mapper.cpu_write(0x9000, 0x05, 0);

        assert_eq!(mapper.cpu_read(0x8000, 0), Some(3));
        assert_eq!(mapper.cpu_read(0xA000, 0), Some(4));
        assert_eq!(mapper.cpu_read(0xC000, 0), Some(5));
        assert_eq!(mapper.cpu_read(0xE000, 0), Some(7));
        assert_eq!(mapper.prg_banks[0], 0x13);
    }

    #[test]
    fn chr_bank_nibbles_select_1k_pages() {
        let mut mapper = test_mapper(8, 32);
        mapper.reset(ResetKind::PowerOn);

        mapper.cpu_write(0xA000, 0x02, 0);
        mapper.cpu_write(0xA001, 0x01, 0);
        mapper.cpu_write(0xD002, 0x0F, 0);

        assert_eq!(mapper.ppu_read(0x0000), Some(18));
        assert_eq!(mapper.ppu_read(0x1C00), Some(15));
    }

    #[test]
    fn lower_window_uses_prg_ram_when_present() {
        let mut mapper = test_mapper(8, 32);
        mapper.reset(ResetKind::PowerOn);

        mapper.cpu_write(0x6000, 0x5A, 0);
        mapper.cpu_write(0x7FFF, 0xA5, 0);

        assert_eq!(mapper.cpu_read(0x6000, 0), Some(0x5A));
        assert_eq!(mapper.cpu_read(0x7FFF, 0), Some(0xA5));
    }

    #[test]
    fn mirroring_control_matches_ss88006_modes() {
        let mut mapper = test_mapper(8, 32);
        mapper.reset(ResetKind::PowerOn);

        mapper.cpu_write(0xF002, 0x00, 0);
        assert_eq!(mapper.mirroring(), Mirroring::Horizontal);

        mapper.cpu_write(0xF002, 0x01, 0);
        assert_eq!(mapper.mirroring(), Mirroring::Vertical);

        mapper.cpu_write(0xF002, 0x02, 0);
        assert_eq!(mapper.mirroring(), Mirroring::SingleScreenLower);

        mapper.cpu_write(0xF002, 0x03, 0);
        assert_eq!(mapper.mirroring(), Mirroring::SingleScreenUpper);
    }

    #[test]
    fn irq_reload_and_counter_widths_follow_register_bits() {
        let mut mapper = test_mapper(8, 32);
        mapper.reset(ResetKind::PowerOn);

        mapper.cpu_write(0xE000, 0x04, 0);
        mapper.cpu_write(0xE001, 0x03, 0);
        mapper.cpu_write(0xE002, 0x02, 0);
        mapper.cpu_write(0xE003, 0x01, 0);
        mapper.cpu_write(0xF000, 0x00, 0);
        assert_eq!(mapper.irq_counter, 0x1234);

        mapper.cpu_write(0xF001, 0x09, 0);
        assert_eq!(mapper.irq_counter_size, 3);

        mapper.irq_reload_nibbles = [1, 0, 0, 0];
        mapper.cpu_write(0xF000, 0x00, 0);
        mapper.on_mapper_event(MapperEvent::CpuClock {
            cpu_cycle: 0,
            master_clock: 0,
        });
        assert!(mapper.irq_pending());
    }
}
