//! Mapper 90 – J.Y. Company multicart (simplified).
//!
//! This is a pared-down implementation that covers the common banking model:
//! - PRG: four 8 KiB registers at `$8000-$8003` map to the three switchable
//!   slots at `$8000/$A000/$C000` and a fourth slot we treat as fixed (`$E000`).
//! - CHR: eight 1 KiB banks split across `$9000-$9FFF` (low bits) and
//!   `$A000-$AFFF` (high bits). Advanced nametable-as-CHR behaviour is not
//!   modelled; nametable control writes are ignored.
//! - Mirroring: `$D001` low two bits select V/H/one-screen A/B.
//! - IRQ: simple CPU-cycle counter with reload (`$C005`), prescaler (`$C004`),
//!   enable/ack (`$C000/$C002/$C003`), modelled after other VRC-style timers.
//!
//! Known limitations:
//! - Advanced nametable mapping and block/chunk CHR modes are omitted.
//! - IRQ details on real JY hardware differ; this uses a VRC-like approximation.

use std::borrow::Cow;

use crate::{
    cartridge::{
        ChrRom, Mapper, PrgRom, TrainerBytes,
        header::{Header, Mirroring},
        mapper::{ChrStorage, allocate_prg_ram, select_chr_storage, trainer_destination},
    },
    memory::cpu as cpu_mem,
};

use crate::mem_block::ByteBlock;

/// PRG banking granularity (8 KiB).
const PRG_BANK_SIZE_8K: usize = 8 * 1024;
/// CHR banking granularity (1 KiB).
const CHR_BANK_SIZE_1K: usize = 1 * 1024;

#[derive(Debug, Clone)]
pub struct Mapper90 {
    prg_rom: PrgRom,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,

    prg_bank_count_8k: usize,

    prg_regs: Mapper90PrgRegs,
    chr_low: Mapper90ChrLowRegs,
    chr_high: Mapper90ChrHighRegs,

    mirroring: Mirroring,

    // IRQ (simplified VRC-style)
    irq_reload: u8,
    irq_counter: u8,
    irq_prescaler: i32,
    irq_enabled: bool,
    irq_pending: bool,
}

type Mapper90PrgRegs = ByteBlock<4>;
type Mapper90ChrLowRegs = ByteBlock<8>;
type Mapper90ChrHighRegs = ByteBlock<8>;

impl Mapper90 {
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
        let prg_bank_count_8k = (prg_rom.len() / PRG_BANK_SIZE_8K).max(1);

        Self {
            prg_rom,
            prg_ram,
            chr,
            prg_bank_count_8k,
            prg_regs: Mapper90PrgRegs::new(),
            chr_low: Mapper90ChrLowRegs::new(),
            chr_high: Mapper90ChrHighRegs::new(),
            mirroring: header.mirroring,
            irq_reload: 0,
            irq_counter: 0,
            irq_prescaler: 0,
            irq_enabled: false,
            irq_pending: false,
        }
    }

    #[inline]
    fn prg_bank_index(&self, value: u8) -> usize {
        if self.prg_bank_count_8k == 0 {
            0
        } else {
            (value as usize) % self.prg_bank_count_8k
        }
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }
        let bank = match addr {
            0x8000..=0x9FFF => self.prg_bank_index(self.prg_regs[0]),
            0xA000..=0xBFFF => self.prg_bank_index(self.prg_regs[1]),
            0xC000..=0xDFFF => self.prg_bank_index(self.prg_regs[2]),
            0xE000..=0xFFFF => {
                self.prg_bank_index(self.prg_regs[3].max(self.prg_bank_count_8k as u8 - 1))
            }
            _ => 0,
        };
        let offset = (addr & 0x1FFF) as usize;
        let base = bank.saturating_mul(PRG_BANK_SIZE_8K);
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

    fn chr_page_base(&self, bank: usize) -> usize {
        let lo = self.chr_low.get(bank).copied().unwrap_or(0);
        let hi = self.chr_high.get(bank).copied().unwrap_or(0);
        let page = (lo as usize) | ((hi as usize) << 8);
        page * CHR_BANK_SIZE_1K
    }

    fn read_chr(&self, addr: u16) -> u8 {
        let bank = ((addr >> 10) & 0x07) as usize;
        let offset = (addr & 0x03FF) as usize;
        let base = self.chr_page_base(bank);
        self.chr.read_indexed(base, offset)
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        let bank = ((addr >> 10) & 0x07) as usize;
        let offset = (addr & 0x03FF) as usize;
        let base = self.chr_page_base(bank);
        self.chr.write_indexed(base, offset, data);
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr {
            0x8000..=0x8003 => {
                self.prg_regs[(addr & 0x0003) as usize] = value & 0x7F;
            }
            0x9000..=0x9007 => {
                self.chr_low[(addr & 0x0007) as usize] = value;
            }
            0xA000..=0xA007 => {
                self.chr_high[(addr & 0x0007) as usize] = value & 0x1F;
            }
            0xB000..=0xB007 => {
                // Nametable control ignored in this simplified model.
                let _ = value;
            }
            0xC000 => {
                self.irq_enabled = (value & 0x01) != 0;
                if !self.irq_enabled {
                    self.irq_pending = false;
                }
            }
            0xC001 => {
                let _ = value; // IRQ mode bits ignored.
            }
            0xC002 => {
                self.irq_enabled = false;
                self.irq_pending = false;
            }
            0xC003 => {
                self.irq_enabled = true;
            }
            0xC004 => {
                self.irq_prescaler = (value as i32) & 0xFF;
            }
            0xC005 => {
                self.irq_reload = value;
            }
            0xC006 | 0xC007 => {
                let _ = value; // Unused in this simplified model.
            }
            0xD000 => {
                // Mode bits ignored; base implementation uses direct registers.
                let _ = value;
            }
            0xD001 => {
                self.mirroring = match value & 0x03 {
                    0 => Mirroring::Vertical,
                    1 => Mirroring::Horizontal,
                    2 => Mirroring::SingleScreenLower,
                    _ => Mirroring::SingleScreenUpper,
                };
            }
            0xD002 | 0xD003 => {
                let _ = value; // Ignored
            }
            _ => {}
        }
    }

    fn clock_irq_counter(&mut self) {
        if self.irq_counter == 0xFF {
            self.irq_counter = self.irq_reload;
            self.irq_pending = true;
        } else {
            self.irq_counter = self.irq_counter.wrapping_add(1);
        }
    }
}

impl Mapper for Mapper90 {
    fn power_on(&mut self) {
        self.prg_regs[0] = 0;
        self.prg_regs[1] = 1;
        self.prg_regs[2] = 2;
        self.prg_regs[3] = 0x7F;
        self.chr_low.fill(0);
        self.chr_high.fill(0);
        self.irq_reload = 0;
        self.irq_counter = 0;
        self.irq_prescaler = 0;
        self.irq_enabled = false;
        self.irq_pending = false;
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
            0x8000..=0xFFFF => self.write_register(addr, data),
            _ => {}
        }
    }

    fn cpu_clock(&mut self, _cpu_cycle: u64) {
        if !self.irq_enabled {
            return;
        }
        self.irq_prescaler -= 3;
        if self.irq_prescaler <= 0 {
            self.clock_irq_counter();
            self.irq_prescaler += 341;
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

    fn clear_irq(&mut self) {
        self.irq_pending = false;
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
        90
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("JY Company (simplified)")
    }
}
