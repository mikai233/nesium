use crate::{
    cartridge::mapper::{PpuVramAccessContext, PpuVramAccessKind},
    mem_block::ByteBlock,
    memory::cpu as cpu_mem,
};

/// PRG-ROM bank size exposed to the CPU (8 KiB).
pub(crate) const PRG_BANK_SIZE_8K: usize = 8 * 1024;
/// MMC3 A12 low-time qualifier in CPU cycles.
pub(crate) const MMC3_A12_LOW_MIN_CPU_CYCLES: u64 = 3;
/// One CPU cycle equals 12 master clocks (NTSC timing model in this core).
pub(crate) const MASTER_CLOCKS_PER_CPU_CYCLE: u64 = 12;
/// Mesen2-compatible MMC3 power-on register defaults (R0..R7).
pub(crate) const MMC3_POWER_ON_BANK_REGS: [u8; 8] = [0, 2, 4, 5, 6, 7, 0, 1];

/// CPU `$8000-$9FFF`: first 8 KiB PRG-ROM window and MMC3 bank select/data registers.
const MMC3_PRG_SLOT0_START: u16 = 0x8000;
const MMC3_PRG_SLOT0_END: u16 = 0x9FFF;
/// CPU `$A000-$BFFF`: second 8 KiB PRG-ROM window and mirroring/PRG-RAM control registers.
const MMC3_PRG_SLOT1_START: u16 = 0xA000;
const MMC3_PRG_SLOT1_END: u16 = 0xBFFF;
/// CPU `$C000-$DFFF`: third 8 KiB PRG-ROM window and IRQ latch/reload registers.
const MMC3_PRG_SLOT2_START: u16 = 0xC000;
const MMC3_PRG_SLOT2_END: u16 = 0xDFFF;
/// CPU `$E000-$FFFF`: fixed 8 KiB PRG-ROM window and IRQ enable/ack registers.
const MMC3_PRG_FIXED_SLOT_START: u16 = 0xE000;
const MMC3_PRG_FIXED_SLOT_END: u16 = 0xFFFF;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Mmc3CpuRegister {
    BankSelect,
    BankData,
    Mirroring,
    PrgRamProtect,
    IrqLatch,
    IrqReload,
    IrqDisable,
    IrqEnable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Mmc3WriteConfig {
    pub(crate) bank_select_mask: u8,
    pub(crate) clear_counter_on_reload: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Mmc3WriteResult {
    Handled,
    Mirroring(u8),
}

impl Mmc3CpuRegister {
    pub(crate) fn from_addr(addr: u16) -> Option<Self> {
        use Mmc3CpuRegister::*;

        match addr {
            MMC3_PRG_SLOT0_START..=MMC3_PRG_SLOT0_END => {
                if addr & 1 == 0 {
                    Some(BankSelect)
                } else {
                    Some(BankData)
                }
            }
            MMC3_PRG_SLOT1_START..=MMC3_PRG_SLOT1_END => {
                if addr & 1 == 0 {
                    Some(Mirroring)
                } else {
                    Some(PrgRamProtect)
                }
            }
            MMC3_PRG_SLOT2_START..=MMC3_PRG_SLOT2_END => {
                if addr & 1 == 0 {
                    Some(IrqLatch)
                } else {
                    Some(IrqReload)
                }
            }
            MMC3_PRG_FIXED_SLOT_START..=MMC3_PRG_FIXED_SLOT_END => {
                if addr & 1 == 0 {
                    Some(IrqDisable)
                } else {
                    Some(IrqEnable)
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Mmc3IrqRevision {
    RevA,
    RevB,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Mmc3CoreResetConfig {
    pub(crate) bank_select: u8,
    pub(crate) bank_regs: [u8; 8],
    pub(crate) prg_ram_enable: bool,
    pub(crate) prg_ram_write_protect: bool,
    pub(crate) irq_revision: Mmc3IrqRevision,
}

#[derive(Debug, Clone)]
pub(crate) struct Mmc3Core {
    pub(crate) bank_select: u8,
    pub(crate) bank_regs: ByteBlock<8>,
    pub(crate) prg_ram_enable: bool,
    pub(crate) prg_ram_write_protect: bool,
    pub(crate) irq_latch: u8,
    pub(crate) irq_counter: u8,
    pub(crate) irq_reload: bool,
    pub(crate) irq_enabled: bool,
    pub(crate) irq_pending: bool,
    pub(crate) irq_revision: Mmc3IrqRevision,
    pub(crate) a12_low_start_master_clock: Option<u64>,
}

impl Mmc3Core {
    pub(crate) fn new(reset: Mmc3CoreResetConfig) -> Self {
        let mut core = Self {
            bank_select: 0,
            bank_regs: ByteBlock::new(),
            prg_ram_enable: false,
            prg_ram_write_protect: false,
            irq_latch: 0,
            irq_counter: 0,
            irq_reload: false,
            irq_enabled: false,
            irq_pending: false,
            irq_revision: reset.irq_revision,
            a12_low_start_master_clock: None,
        };
        core.reset(reset);
        core
    }

    pub(crate) fn reset(&mut self, reset: Mmc3CoreResetConfig) {
        self.bank_select = reset.bank_select;
        self.bank_regs
            .as_mut_slice()
            .copy_from_slice(&reset.bank_regs);
        self.prg_ram_enable = reset.prg_ram_enable;
        self.prg_ram_write_protect = reset.prg_ram_write_protect;
        self.irq_latch = 0;
        self.irq_counter = 0;
        self.irq_reload = false;
        self.irq_enabled = false;
        self.irq_pending = false;
        self.irq_revision = reset.irq_revision;
        self.a12_low_start_master_clock = None;
    }

    #[inline]
    pub(crate) fn chr_invert(&self) -> bool {
        self.bank_select & 0x80 != 0
    }

    #[inline]
    pub(crate) fn prg_swap_at_c000(&self) -> bool {
        self.bank_select & 0x40 != 0
    }

    #[inline]
    pub(crate) fn prg_ram_enabled(&self, prg_ram: &[u8]) -> bool {
        !prg_ram.is_empty() && self.prg_ram_enable
    }

    pub(crate) fn read_prg_ram(&self, prg_ram: &[u8], addr: u16) -> Option<u8> {
        if !self.prg_ram_enabled(prg_ram) {
            return None;
        }
        let idx = (addr - cpu_mem::PRG_RAM_START) as usize % prg_ram.len();
        Some(prg_ram[idx])
    }

    pub(crate) fn write_prg_ram(&self, prg_ram: &mut [u8], addr: u16, data: u8) {
        if !self.prg_ram_enabled(prg_ram) || self.prg_ram_write_protect {
            return;
        }
        let idx = (addr - cpu_mem::PRG_RAM_START) as usize % prg_ram.len();
        prg_ram[idx] = data;
    }

    #[inline]
    pub(crate) fn prg_bank_index(&self, prg_bank_count: usize, reg_value: u8) -> usize {
        if prg_bank_count == 0 {
            0
        } else {
            (reg_value as usize) % prg_bank_count
        }
    }

    pub(crate) fn resolve_prg_rom_bank(&self, prg_bank_count: usize, addr: u16) -> Option<usize> {
        let bank_slot = match addr {
            MMC3_PRG_SLOT0_START..=MMC3_PRG_SLOT0_END => 0,
            MMC3_PRG_SLOT1_START..=MMC3_PRG_SLOT1_END => 1,
            MMC3_PRG_SLOT2_START..=MMC3_PRG_SLOT2_END => 2,
            MMC3_PRG_FIXED_SLOT_START..=MMC3_PRG_FIXED_SLOT_END => 3,
            _ => return None,
        };

        let last_bank = prg_bank_count.saturating_sub(1);
        let second_last_bank = prg_bank_count.saturating_sub(2);

        let bank = if !self.prg_swap_at_c000() {
            match bank_slot {
                0 => self.prg_bank_index(prg_bank_count, self.bank_regs[6]),
                1 => self.prg_bank_index(prg_bank_count, self.bank_regs[7]),
                2 => second_last_bank,
                _ => last_bank,
            }
        } else {
            match bank_slot {
                0 => second_last_bank,
                1 => self.prg_bank_index(prg_bank_count, self.bank_regs[7]),
                2 => self.prg_bank_index(prg_bank_count, self.bank_regs[6]),
                _ => last_bank,
            }
        };

        Some(bank)
    }

    pub(crate) fn write_bank_select(&mut self, data: u8) {
        self.bank_select = data;
    }

    pub(crate) fn write_bank_data(&mut self, data: u8) {
        let index = (self.bank_select & 0x07) as usize;
        if index < self.bank_regs.len() {
            self.bank_regs[index] = data;
        }
    }

    pub(crate) fn write_prg_ram_protect(&mut self, data: u8) {
        self.prg_ram_enable = data & 0x80 != 0;
        self.prg_ram_write_protect = data & 0x40 != 0;
    }

    pub(crate) fn write_irq_latch(&mut self, data: u8) {
        self.irq_latch = data;
    }

    pub(crate) fn write_irq_reload(&mut self, clear_counter: bool) {
        if clear_counter {
            self.irq_counter = 0;
        }
        self.irq_reload = true;
    }

    pub(crate) fn write_irq_disable(&mut self) {
        self.irq_enabled = false;
        self.irq_pending = false;
    }

    pub(crate) fn write_irq_enable(&mut self) {
        self.irq_enabled = true;
    }

    pub(crate) fn write_register(
        &mut self,
        reg: Mmc3CpuRegister,
        data: u8,
        config: Mmc3WriteConfig,
    ) -> Mmc3WriteResult {
        match reg {
            Mmc3CpuRegister::BankSelect => {
                self.write_bank_select(data & config.bank_select_mask);
                Mmc3WriteResult::Handled
            }
            Mmc3CpuRegister::BankData => {
                self.write_bank_data(data);
                Mmc3WriteResult::Handled
            }
            Mmc3CpuRegister::Mirroring => Mmc3WriteResult::Mirroring(data),
            Mmc3CpuRegister::PrgRamProtect => {
                self.write_prg_ram_protect(data);
                Mmc3WriteResult::Handled
            }
            Mmc3CpuRegister::IrqLatch => {
                self.write_irq_latch(data);
                Mmc3WriteResult::Handled
            }
            Mmc3CpuRegister::IrqReload => {
                self.write_irq_reload(config.clear_counter_on_reload);
                Mmc3WriteResult::Handled
            }
            Mmc3CpuRegister::IrqDisable => {
                self.write_irq_disable();
                Mmc3WriteResult::Handled
            }
            Mmc3CpuRegister::IrqEnable => {
                self.write_irq_enable();
                Mmc3WriteResult::Handled
            }
        }
    }

    pub(crate) fn observe_ppu_vram_access(&mut self, addr: u16, ctx: PpuVramAccessContext) {
        if !matches!(
            ctx.kind,
            PpuVramAccessKind::RenderingFetch
                | PpuVramAccessKind::CpuRead
                | PpuVramAccessKind::CpuWrite
        ) {
            return;
        }

        if self.is_a12_rising_edge(addr, ctx.ppu_master_clock) {
            self.clock_irq_counter();
        }
    }

    #[inline]
    fn is_a12_rising_edge(&mut self, addr: u16, ppu_master_clock: u64) -> bool {
        let a12_high = addr & 0x1000 != 0;

        if a12_high {
            let low_min_master = MMC3_A12_LOW_MIN_CPU_CYCLES * MASTER_CLOCKS_PER_CPU_CYCLE;
            let is_rise = self
                .a12_low_start_master_clock
                .map(|low_start| ppu_master_clock.saturating_sub(low_start) >= low_min_master)
                .unwrap_or(false);
            self.a12_low_start_master_clock = None;
            return is_rise;
        }

        if self.a12_low_start_master_clock.is_none() {
            self.a12_low_start_master_clock = Some(ppu_master_clock);
        }
        false
    }

    fn clock_irq_counter(&mut self) {
        let counter_before = self.irq_counter;
        let reload_before = self.irq_reload;

        if reload_before || counter_before == 0 {
            self.irq_counter = self.irq_latch;
            self.irq_reload = false;
        } else {
            self.irq_counter = counter_before.wrapping_sub(1);
        }

        if self.irq_counter == 0 && self.irq_enabled {
            match self.irq_revision {
                Mmc3IrqRevision::RevA => {
                    if counter_before > 0 || reload_before {
                        self.irq_pending = true;
                    }
                }
                Mmc3IrqRevision::RevB => {
                    self.irq_pending = true;
                }
            }
        }
    }
}

pub(crate) fn resolve_mmc3_chr_bank(bank_regs: &[u8], chr_invert: bool, addr: u16) -> (u8, usize) {
    let bank = if !chr_invert {
        match addr {
            0x0000..=0x03FF => bank_regs[0] & !0x01,
            0x0400..=0x07FF => bank_regs[0] | 0x01,
            0x0800..=0x0BFF => bank_regs[1] & !0x01,
            0x0C00..=0x0FFF => bank_regs[1] | 0x01,
            0x1000..=0x13FF => bank_regs[2],
            0x1400..=0x17FF => bank_regs[3],
            0x1800..=0x1BFF => bank_regs[4],
            _ => bank_regs[5],
        }
    } else {
        match addr {
            0x0000..=0x03FF => bank_regs[2],
            0x0400..=0x07FF => bank_regs[3],
            0x0800..=0x0BFF => bank_regs[4],
            0x0C00..=0x0FFF => bank_regs[5],
            0x1000..=0x13FF => bank_regs[0] & !0x01,
            0x1400..=0x17FF => bank_regs[0] | 0x01,
            0x1800..=0x1BFF => bank_regs[1] & !0x01,
            _ => bank_regs[1] | 0x01,
        }
    };

    let offset = (addr & 0x03FF) as usize;
    (bank, offset)
}
