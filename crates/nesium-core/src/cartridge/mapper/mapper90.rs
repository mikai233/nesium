//! Mapper 90 - J.Y. Company multicart.
//!
//! This tracks Mesen's mapper-90 core behavior closely for the common
//! multicart banking and IRQ paths:
//! - PRG banking modes from `$D000`, including optional PRG ROM at `$6000`
//! - CHR banking modes/block mode from `$D000/$D003`
//! - Mirroring control from `$D001`
//! - Multiply/register-RAM reads in `$5000-$5FFF`
//! - IRQ modes from `$C000-$C007` (CPU clock / CPU write / PPU render / A12)
//!
//! Advanced nametable-as-CHR behavior remains intentionally omitted for mapper
//! 90 itself; Mesen also excludes mapper 90 from the JY advanced NT path.

use std::borrow::Cow;

use crate::cartridge::mapper::{MapperMemoryMut, MapperMemoryRef};
use crate::cartridge::{
    ChrRom, Mapper, PrgRom, TrainerBytes,
    header::{Header, Mirroring},
    mapper::{
        ChrStorage, CpuBusAccessKind, MapperEvent, MapperHookMask, PpuVramAccessKind,
        allocate_prg_ram_with_trainer, select_chr_storage,
    },
};
use crate::mem_block::ByteBlock;
use crate::memory::cpu as cpu_mem;
use crate::reset_kind::ResetKind;

const PRG_BANK_SIZE_8K: usize = 8 * 1024;
const CHR_BANK_SIZE_1K: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum JyIrqSource {
    CpuClock = 0,
    PpuA12Rise = 1,
    PpuRead = 2,
    CpuWrite = 3,
}

#[derive(Debug, Clone)]
pub struct Mapper90 {
    prg_rom: PrgRom,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,

    prg_bank_count_8k: usize,

    prg_regs: ByteBlock<4>,
    chr_low_regs: ByteBlock<8>,
    chr_high_regs: ByteBlock<8>,
    chr_latch: [u8; 2],

    prg_mode: u8,
    enable_prg_at_6000: bool,

    chr_mode: u8,
    chr_block_mode: bool,
    chr_block: u8,
    mirror_chr: bool,

    mirroring_reg: u8,
    mirroring: Mirroring,
    advanced_nt_control: bool,
    disable_nt_ram: bool,
    nt_ram_select_bit: u8,
    nt_low_regs: ByteBlock<4>,
    nt_high_regs: ByteBlock<4>,

    irq_enabled: bool,
    irq_pending: bool,
    irq_source: JyIrqSource,
    irq_count_direction: u8,
    irq_funky_mode: bool,
    irq_funky_mode_reg: u8,
    irq_small_prescaler: bool,
    irq_prescaler: u8,
    irq_counter: u8,
    irq_xor_reg: u8,
    last_ppu_addr: u16,

    multiply_value1: u8,
    multiply_value2: u8,
    reg_ram_value: u8,
}

impl Mapper90 {
    pub fn new(header: Header, prg_rom: PrgRom, chr_rom: ChrRom, trainer: TrainerBytes) -> Self {
        let prg_ram = allocate_prg_ram_with_trainer(&header, trainer);
        let chr = select_chr_storage(&header, chr_rom);
        let prg_bank_count_8k = (prg_rom.len() / PRG_BANK_SIZE_8K).max(1);

        let mut mapper = Self {
            prg_rom,
            prg_ram,
            chr,
            prg_bank_count_8k,
            prg_regs: ByteBlock::new(),
            chr_low_regs: ByteBlock::new(),
            chr_high_regs: ByteBlock::new(),
            chr_latch: [0, 4],
            prg_mode: 0,
            enable_prg_at_6000: false,
            chr_mode: 0,
            chr_block_mode: false,
            chr_block: 0,
            mirror_chr: false,
            mirroring_reg: 0,
            mirroring: Mirroring::Vertical,
            advanced_nt_control: false,
            disable_nt_ram: false,
            nt_ram_select_bit: 0,
            nt_low_regs: ByteBlock::new(),
            nt_high_regs: ByteBlock::new(),
            irq_enabled: false,
            irq_pending: false,
            irq_source: JyIrqSource::CpuClock,
            irq_count_direction: 0,
            irq_funky_mode: false,
            irq_funky_mode_reg: 0,
            irq_small_prescaler: false,
            irq_prescaler: 0,
            irq_counter: 0,
            irq_xor_reg: 0,
            last_ppu_addr: 0,
            multiply_value1: 0,
            multiply_value2: 0,
            reg_ram_value: 0,
        };
        mapper.apply_state();
        mapper
    }

    fn apply_state(&mut self) {
        self.mirroring = match self.mirroring_reg & 0x03 {
            0 => Mirroring::Vertical,
            1 => Mirroring::Horizontal,
            2 => Mirroring::SingleScreenLower,
            _ => Mirroring::SingleScreenUpper,
        };
    }

    fn prg_bank_index(&self, page: usize) -> usize {
        if self.prg_bank_count_8k == 0 {
            0
        } else {
            page % self.prg_bank_count_8k
        }
    }

    fn inverted_prg_reg(&self, reg: u8) -> u8 {
        if (self.prg_mode & 0x03) != 0x03 {
            return reg & 0x7F;
        }
        ((reg & 0x01) << 6)
            | ((reg & 0x02) << 4)
            | ((reg & 0x04) << 2)
            | ((reg & 0x10) >> 2)
            | ((reg & 0x20) >> 4)
            | ((reg & 0x40) >> 6)
    }

    fn mapped_prg_regs(&self) -> [usize; 4] {
        let regs = [
            self.inverted_prg_reg(self.prg_regs[0]) as usize,
            self.inverted_prg_reg(self.prg_regs[1]) as usize,
            self.inverted_prg_reg(self.prg_regs[2]) as usize,
            self.inverted_prg_reg(self.prg_regs[3]) as usize,
        ];

        match self.prg_mode & 0x03 {
            0 => {
                let base = if (self.prg_mode & 0x04) != 0 {
                    regs[3]
                } else {
                    0x3C
                };
                [base, base + 1, base + 2, base + 3]
            }
            1 => {
                let lo = regs[1] << 1;
                let hi = if (self.prg_mode & 0x04) != 0 {
                    regs[3]
                } else {
                    0x3E
                };
                [lo, lo + 1, hi, hi + 1]
            }
            _ => {
                let hi = if (self.prg_mode & 0x04) != 0 {
                    regs[3]
                } else {
                    0x3F
                };
                [regs[0], regs[1], regs[2], hi]
            }
        }
    }

    fn read_prg_rom_slot(&self, page: usize, offset: usize) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }
        let base = self.prg_bank_index(page) * PRG_BANK_SIZE_8K;
        self.prg_rom.get(base + offset).copied().unwrap_or(0)
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        let offset = (addr & 0x1FFF) as usize;
        if (0x6000..=0x7FFF).contains(&addr) && self.enable_prg_at_6000 {
            let regs = self.mapped_prg_regs();
            let page = match self.prg_mode & 0x03 {
                0 => regs[3],
                1 => regs[3],
                _ => self.inverted_prg_reg(self.prg_regs[3]) as usize,
            };
            return self.read_prg_rom_slot(page, offset);
        }

        let slot = ((addr - 0x8000) >> 13) as usize;
        let page = self.mapped_prg_regs()[slot];
        self.read_prg_rom_slot(page, offset)
    }

    fn chr_reg(&self, mut index: usize) -> usize {
        if self.chr_mode >= 2 && self.mirror_chr && matches!(index, 2 | 3) {
            index -= 2;
        }

        if self.chr_block_mode {
            let (mask, shift) = match self.chr_mode {
                0 => (0x1F, 5),
                1 => (0x3F, 6),
                2 => (0x7F, 7),
                _ => (0xFF, 8),
            };
            ((self.chr_low_regs[index] & mask) as usize) | ((self.chr_block as usize) << shift)
        } else {
            self.chr_low_regs[index] as usize | ((self.chr_high_regs[index] as usize) << 8)
        }
    }

    fn chr_page(&self, bank: usize) -> usize {
        let regs = [
            self.chr_reg(0),
            self.chr_reg(1),
            self.chr_reg(2),
            self.chr_reg(3),
            self.chr_reg(4),
            self.chr_reg(5),
            self.chr_reg(6),
            self.chr_reg(7),
        ];

        match self.chr_mode {
            0 => (regs[0] << 3) + bank,
            1 => {
                let index = if bank < 4 {
                    self.chr_latch[0] as usize
                } else {
                    self.chr_latch[1] as usize
                };
                (regs[index] << 2) + (bank & 0x03)
            }
            2 => {
                let index = match bank / 2 {
                    0 => 0,
                    1 => 2,
                    2 => 4,
                    _ => 6,
                };
                (regs[index] << 1) + (bank & 0x01)
            }
            _ => regs[bank],
        }
    }

    fn read_chr(&self, addr: u16) -> u8 {
        let bank = ((addr >> 10) & 0x07) as usize;
        let offset = (addr & 0x03FF) as usize;
        self.chr
            .read_indexed(self.chr_page(bank) * CHR_BANK_SIZE_1K, offset)
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        let bank = ((addr >> 10) & 0x07) as usize;
        let offset = (addr & 0x03FF) as usize;
        self.chr
            .write_indexed(self.chr_page(bank) * CHR_BANK_SIZE_1K, offset, data);
    }

    fn read_low_register(&self, addr: u16) -> Option<u8> {
        match addr & 0xF803 {
            0x5000 => Some(0),
            0x5800 => Some(self.multiply_value1.wrapping_mul(self.multiply_value2)),
            0x5801 => Some(
                ((u16::from(self.multiply_value1) * u16::from(self.multiply_value2)) >> 8) as u8,
            ),
            0x5803 => Some(self.reg_ram_value),
            _ => None,
        }
    }

    fn write_low_register(&mut self, addr: u16, value: u8) -> bool {
        match addr & 0xF803 {
            0x5800 => self.multiply_value1 = value,
            0x5801 => self.multiply_value2 = value,
            0x5803 => self.reg_ram_value = value,
            _ => return false,
        }
        true
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0xF007 {
            0x8000..=0x8007 => self.prg_regs[(addr & 0x0003) as usize] = value & 0x7F,
            0x9000..=0x9007 => self.chr_low_regs[(addr & 0x0007) as usize] = value,
            0xA000..=0xA007 => self.chr_high_regs[(addr & 0x0007) as usize] = value,
            0xB000..=0xB003 => self.nt_low_regs[(addr & 0x0003) as usize] = value,
            0xB004..=0xB007 => self.nt_high_regs[(addr & 0x0003) as usize] = value,
            0xC000 => {
                self.irq_enabled = (value & 0x01) != 0;
                if !self.irq_enabled {
                    self.irq_pending = false;
                }
            }
            0xC001 => {
                self.irq_count_direction = (value >> 6) & 0x03;
                self.irq_funky_mode = (value & 0x08) != 0;
                self.irq_small_prescaler = ((value >> 2) & 0x01) != 0;
                self.irq_source = match value & 0x03 {
                    0 => JyIrqSource::CpuClock,
                    1 => JyIrqSource::PpuA12Rise,
                    2 => JyIrqSource::PpuRead,
                    _ => JyIrqSource::CpuWrite,
                };
            }
            0xC002 => {
                self.irq_enabled = false;
                self.irq_pending = false;
            }
            0xC003 => self.irq_enabled = true,
            0xC004 => self.irq_prescaler = value ^ self.irq_xor_reg,
            0xC005 => {
                self.irq_counter = value ^ self.irq_xor_reg;
                self.irq_pending = false;
            }
            0xC006 => self.irq_xor_reg = value,
            0xC007 => self.irq_funky_mode_reg = value,
            0xD000 => {
                self.prg_mode = value & 0x07;
                self.chr_mode = (value >> 3) & 0x03;
                self.advanced_nt_control = (value & 0x20) != 0;
                self.disable_nt_ram = (value & 0x40) != 0;
                self.enable_prg_at_6000 = (value & 0x80) != 0;
            }
            0xD001 => self.mirroring_reg = value & 0x03,
            0xD002 => self.nt_ram_select_bit = value & 0x80,
            0xD003 => {
                self.mirror_chr = (value & 0x80) != 0;
                self.chr_block_mode = (value & 0x20) == 0;
                self.chr_block = ((value & 0x18) >> 2) | (value & 0x01);
            }
            _ => return,
        }

        self.apply_state();
    }

    fn tick_irq_counter(&mut self) {
        let _ = self.irq_funky_mode;
        let _ = self.irq_funky_mode_reg;
        let _ = self.advanced_nt_control;
        let _ = self.disable_nt_ram;
        let _ = self.nt_ram_select_bit;

        let mut clock_irq_counter = false;
        let mask = if self.irq_small_prescaler { 0x07 } else { 0xFF };
        let mut prescaler = self.irq_prescaler & mask;

        if self.irq_count_direction == 0x01 {
            prescaler = prescaler.wrapping_add(1);
            if (prescaler & mask) == 0 {
                clock_irq_counter = true;
            }
        } else if self.irq_count_direction == 0x02 {
            prescaler = prescaler.wrapping_sub(1);
            if prescaler == 0 {
                clock_irq_counter = true;
            }
        }

        self.irq_prescaler = (self.irq_prescaler & !mask) | (prescaler & mask);

        if !clock_irq_counter {
            return;
        }

        if self.irq_count_direction == 0x01 {
            self.irq_counter = self.irq_counter.wrapping_add(1);
            if self.irq_counter == 0 && self.irq_enabled {
                self.irq_pending = true;
            }
        } else if self.irq_count_direction == 0x02 {
            self.irq_counter = self.irq_counter.wrapping_sub(1);
            if self.irq_counter == 0xFF && self.irq_enabled {
                self.irq_pending = true;
            }
        }
    }

    fn on_ppu_bus_address(&mut self, addr: u16, kind: PpuVramAccessKind) {
        if self.irq_source == JyIrqSource::PpuRead && kind == PpuVramAccessKind::RenderingFetch {
            self.tick_irq_counter();
        }

        if self.irq_source == JyIrqSource::PpuA12Rise
            && (addr & 0x1000) != 0
            && (self.last_ppu_addr & 0x1000) == 0
        {
            self.tick_irq_counter();
        }

        self.last_ppu_addr = addr;
    }
}

impl Mapper for Mapper90 {
    fn hook_mask(&self) -> MapperHookMask {
        MapperHookMask::CPU_BUS_ACCESS | MapperHookMask::PPU_BUS_ADDRESS | MapperHookMask::CPU_CLOCK
    }

    fn on_mapper_event(&mut self, event: MapperEvent) {
        match event {
            MapperEvent::CpuBusAccess { kind, .. } => {
                if self.irq_source == JyIrqSource::CpuWrite && kind == CpuBusAccessKind::Write {
                    self.tick_irq_counter();
                }
            }
            MapperEvent::PpuBusAddress { addr, ctx } => self.on_ppu_bus_address(addr, ctx.kind),
            MapperEvent::CpuClock { .. } => {
                if self.irq_source == JyIrqSource::CpuClock {
                    self.tick_irq_counter();
                }
            }
        }
    }

    fn reset(&mut self, _kind: ResetKind) {
        self.prg_regs.fill(0);
        self.chr_low_regs.fill(0);
        self.chr_high_regs.fill(0);
        self.chr_latch = [0, 4];
        self.prg_mode = 0;
        self.enable_prg_at_6000 = false;
        self.chr_mode = 0;
        self.chr_block_mode = false;
        self.chr_block = 0;
        self.mirror_chr = false;
        self.mirroring_reg = 0;
        self.advanced_nt_control = false;
        self.disable_nt_ram = false;
        self.nt_ram_select_bit = 0;
        self.nt_low_regs.fill(0);
        self.nt_high_regs.fill(0);
        self.irq_enabled = false;
        self.irq_pending = false;
        self.irq_source = JyIrqSource::CpuClock;
        self.irq_count_direction = 0;
        self.irq_funky_mode = false;
        self.irq_funky_mode_reg = 0;
        self.irq_small_prescaler = false;
        self.irq_prescaler = 0;
        self.irq_counter = 0;
        self.irq_xor_reg = 0;
        self.last_ppu_addr = 0;
        self.multiply_value1 = 0;
        self.multiply_value2 = 0;
        self.reg_ram_value = 0;
        self.apply_state();
    }

    fn cpu_read(&self, addr: u16) -> Option<u8> {
        match addr {
            0x5000..=0x5FFF => self.read_low_register(addr),
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => {
                if self.enable_prg_at_6000 {
                    Some(self.read_prg_rom(addr))
                } else {
                    None
                }
            }
            cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END => Some(self.read_prg_rom(addr)),
            _ => None,
        }
    }

    fn cpu_write(&mut self, addr: u16, data: u8, _cpu_cycle: u64) {
        match addr {
            0x5000..=0x5FFF => {
                let _ = self.write_low_register(addr, data);
            }
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => {}
            0x8000..=0xFFFF => self.write_register(addr, data),
            _ => {}
        }
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
        90
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("JY Company")
    }
}
