//! Mapper 119 – TQROM (MMC3 variant with mixed CHR ROM/RAM).
//!
//! This reuses MMC3 banking and IRQ behaviour but interprets CHR bank bit6 to
//! select CHR RAM instead of ROM. CHR RAM is treated as 8 KiB split into 1 KiB
//! pages; CHR ROM uses the low 6 bits as the 1 KiB page index.
//!
//! | Area | Address range     | Behaviour                                          | IRQ/Audio         |
//! |------|-------------------|----------------------------------------------------|-------------------|
//! | CPU  | `$6000-$7FFF`     | Optional PRG-RAM with enable/write-protect bits   | None              |
//! | CPU  | `$8000-$DFFF`     | MMC3-style 8 KiB PRG windows (bank select at `$8000`)| None           |
//! | CPU  | `$A000-$BFFF`     | Mirroring + PRG-RAM enable/write-protect          | None              |
//! | CPU  | `$C000-$FFFF`     | IRQ latch/reload and enable/ack registers         | MMC3 scanline IRQ |
//! | PPU  | `$0000-$1FFF`     | 1 KiB CHR banks, bit6 selects CHR RAM vs ROM      | None              |
//! | PPU  | `$2000-$3EFF`     | Mirroring from header or TQROM mirroring control  | None              |

use std::borrow::Cow;

use crate::{
    cartridge::{
        ChrRom, Mapper, PrgRom, TrainerBytes,
        header::{Header, Mirroring},
        mapper::{PpuVramAccessContext, PpuVramAccessKind, allocate_prg_ram_with_trainer},
    },
    memory::cpu as cpu_mem,
    reset_kind::ResetKind,
};

use crate::mem_block::ByteBlock;

/// PRG-ROM bank size exposed to the CPU (8 KiB).
const PRG_BANK_SIZE_8K: usize = 8 * 1024;
/// CHR banking granularity (1 KiB).
const CHR_BANK_SIZE_1K: usize = 1024;
/// Fixed CHR-RAM size used by TQROM boards.
const CHR_RAM_SIZE: usize = 8 * 1024;

/// CPU `$8000-$9FFF`: bank select and bank data registers (MMC3-style slot 0).
const TQROM_BANK_SELECT_START: u16 = 0x8000;
const TQROM_BANK_SELECT_END: u16 = 0x9FFF;

/// CPU `$A000-$BFFF`: mirroring mode and PRG-RAM enable/write-protect registers.
const TQROM_MIRROR_PRGRAM_START: u16 = 0xA000;
const TQROM_MIRROR_PRGRAM_END: u16 = 0xBFFF;

/// CPU `$C000-$DFFF`: IRQ latch value and reload control.
const TQROM_IRQ_LATCH_START: u16 = 0xC000;
const TQROM_IRQ_LATCH_END: u16 = 0xDFFF;

/// CPU `$E000-$FFFF`: IRQ enable/disable and acknowledge.
const TQROM_IRQ_ENABLE_START: u16 = 0xE000;
const TQROM_IRQ_ENABLE_END: u16 = 0xFFFF;

/// CPU-visible TQROM (MMC3-style) register set.
///
/// TQROM reuses the MMC3 register layout but adds mixed CHR ROM/RAM banking.
/// Grouping the CPU register windows into an enum keeps the write logic clear.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TqromCpuRegister {
    /// `$8000/$8001` – bank select and bank data.
    BankSelect,
    BankData,
    /// `$A000/$A001` – mirroring control and PRG-RAM enable/write-protect.
    MirrorControl,
    PrgRamControl,
    /// `$C000/$C001` – IRQ latch value and reload strobe.
    IrqLatch,
    IrqReload,
    /// `$E000/$E001` – IRQ disable/ack and enable.
    IrqDisable,
    IrqEnable,
}

impl TqromCpuRegister {
    fn from_addr(addr: u16) -> Option<Self> {
        use TqromCpuRegister::*;

        match addr {
            TQROM_BANK_SELECT_START..=TQROM_BANK_SELECT_END => {
                if addr & 0x0001 == 0 {
                    Some(BankSelect)
                } else {
                    Some(BankData)
                }
            }
            TQROM_MIRROR_PRGRAM_START..=TQROM_MIRROR_PRGRAM_END => {
                if addr & 0x0001 == 0 {
                    Some(MirrorControl)
                } else {
                    Some(PrgRamControl)
                }
            }
            TQROM_IRQ_LATCH_START..=TQROM_IRQ_LATCH_END => {
                if addr & 0x0001 == 0 {
                    Some(IrqLatch)
                } else {
                    Some(IrqReload)
                }
            }
            TQROM_IRQ_ENABLE_START..=TQROM_IRQ_ENABLE_END => {
                if addr & 0x0001 == 0 {
                    Some(IrqDisable)
                } else {
                    Some(IrqEnable)
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mapper119 {
    prg_rom: PrgRom,
    prg_ram: Box<[u8]>,
    chr_rom: ChrRom,
    chr_ram: Box<[u8]>,

    /// Number of 8 KiB PRG-ROM banks.
    prg_bank_count: usize,

    base_mirroring: Mirroring,
    mirroring: Mirroring,

    // Banking registers (MMC3 style) ----------------------------
    bank_select: u8,
    bank_regs: Mapper119BankRegs, // 0-5 CHR, 6-7 PRG

    prg_ram_enable: bool,
    prg_ram_write_protect: bool,

    // IRQ state (MMC3 style) ------------------------------------
    irq_latch: u8,
    irq_counter: u8,
    irq_reload: bool,
    irq_enabled: bool,
    irq_pending: bool,

    // PPU A12 edge detection ------------------------------------
    last_a12_high: bool,
    last_a12_rise_ppu_cycle: u64,
}

type Mapper119BankRegs = ByteBlock<8>;

impl Mapper119 {
    pub fn new(header: Header, prg_rom: PrgRom, chr_rom: ChrRom, trainer: TrainerBytes) -> Self {
        let prg_ram = allocate_prg_ram_with_trainer(&header, trainer);

        let chr_ram = vec![0u8; CHR_RAM_SIZE].into_boxed_slice();
        let prg_bank_count = (prg_rom.len() / PRG_BANK_SIZE_8K).max(1);

        Self {
            prg_rom,
            prg_ram,
            chr_rom,
            chr_ram,
            prg_bank_count,
            base_mirroring: header.mirroring,
            mirroring: header.mirroring,
            bank_select: 0,
            bank_regs: Mapper119BankRegs::new(),
            prg_ram_enable: false,
            prg_ram_write_protect: true,
            irq_latch: 0,
            irq_counter: 0,
            irq_reload: false,
            irq_enabled: false,
            irq_pending: false,
            last_a12_high: false,
            last_a12_rise_ppu_cycle: 0,
        }
    }

    #[inline]
    fn chr_invert(&self) -> bool {
        self.bank_select & 0x80 != 0
    }

    #[inline]
    fn prg_swap_at_c000(&self) -> bool {
        self.bank_select & 0x40 != 0
    }

    #[inline]
    fn prg_ram_enabled(&self) -> bool {
        !self.prg_ram.is_empty() && self.prg_ram_enable
    }

    fn read_prg_ram(&self, addr: u16) -> Option<u8> {
        if !self.prg_ram_enabled() {
            return None;
        }
        let idx = (addr - cpu_mem::PRG_RAM_START) as usize % self.prg_ram.len();
        Some(self.prg_ram[idx])
    }

    fn write_prg_ram(&mut self, addr: u16, data: u8) {
        if !self.prg_ram_enabled() || self.prg_ram_write_protect {
            return;
        }
        let idx = (addr - cpu_mem::PRG_RAM_START) as usize % self.prg_ram.len();
        self.prg_ram[idx] = data;
    }

    #[inline]
    fn prg_bank_index(&self, reg_value: u8) -> usize {
        if self.prg_bank_count == 0 {
            0
        } else {
            (reg_value as usize) % self.prg_bank_count
        }
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }

        let bank_slot = match addr {
            TQROM_BANK_SELECT_START..=TQROM_BANK_SELECT_END => 0,
            TQROM_MIRROR_PRGRAM_START..=TQROM_MIRROR_PRGRAM_END => 1,
            TQROM_IRQ_LATCH_START..=TQROM_IRQ_LATCH_END => 2,
            TQROM_IRQ_ENABLE_START..=TQROM_IRQ_ENABLE_END => 3,
            _ => return 0,
        };

        let last_bank = self.prg_bank_count.saturating_sub(1);
        let second_last_bank = self.prg_bank_count.saturating_sub(2);

        let bank = if !self.prg_swap_at_c000() {
            match bank_slot {
                0 => self.prg_bank_index(self.bank_regs[6]),
                1 => self.prg_bank_index(self.bank_regs[7]),
                2 => second_last_bank,
                _ => last_bank,
            }
        } else {
            match bank_slot {
                0 => second_last_bank,
                1 => self.prg_bank_index(self.bank_regs[7]),
                2 => self.prg_bank_index(self.bank_regs[6]),
                _ => last_bank,
            }
        };

        let offset = (addr & 0x1FFF) as usize;
        let base = bank.saturating_mul(PRG_BANK_SIZE_8K);
        self.prg_rom.get(base + offset).copied().unwrap_or(0)
    }

    fn chr_bank_base(&self, bank_reg: u8) -> (bool, usize) {
        // Bit6 selects CHR RAM; lower bits select page.
        let use_ram = bank_reg & 0x40 != 0;
        if use_ram {
            let page = (bank_reg & 0x07) as usize;
            (true, page * CHR_BANK_SIZE_1K)
        } else {
            let page = (bank_reg & 0x3F) as usize;
            (false, page * CHR_BANK_SIZE_1K)
        }
    }

    fn read_chr(&self, addr: u16) -> u8 {
        let (bank_idx, offset) = self.resolve_chr_bank_offset(addr);
        let (use_ram, base) = self.chr_bank_base(bank_idx);
        if use_ram {
            let len = self.chr_ram.len().max(1);
            self.chr_ram[(base + offset) % len]
        } else {
            let len = self.chr_rom.len().max(1);
            self.chr_rom[(base + offset) % len]
        }
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        let (bank_idx, offset) = self.resolve_chr_bank_offset(addr);
        let (use_ram, base) = self.chr_bank_base(bank_idx);
        if !use_ram || self.chr_ram.is_empty() {
            return;
        }
        let len = self.chr_ram.len();
        let idx = (base + offset) % len;
        self.chr_ram[idx] = data;
    }

    fn resolve_chr_bank_offset(&self, addr: u16) -> (u8, usize) {
        // Mimics MMC3 CHR mapping (1 KiB granularity with optional inversion).
        let bank = if !self.chr_invert() {
            match addr {
                0x0000..=0x07FF => self.bank_regs[0] & !0x01,
                0x0800..=0x0FFF => self.bank_regs[0] | 0x01,
                0x1000..=0x13FF => self.bank_regs[2],
                0x1400..=0x17FF => self.bank_regs[3],
                0x1800..=0x1BFF => self.bank_regs[4],
                _ => self.bank_regs[5],
            }
        } else {
            match addr {
                0x0000..=0x03FF => self.bank_regs[2],
                0x0400..=0x07FF => self.bank_regs[3],
                0x0800..=0x0BFF => self.bank_regs[4],
                0x0C00..=0x0FFF => self.bank_regs[5],
                0x1000..=0x17FF => self.bank_regs[0] & !0x01,
                _ => self.bank_regs[0] | 0x01,
            }
        };
        let offset = (addr & 0x03FF) as usize;
        (bank, offset)
    }

    fn observe_ppu_vram_access(&mut self, addr: u16, ctx: PpuVramAccessContext) {
        if addr >= 0x2000 {
            return;
        }

        let a12_high = addr & 0x1000 != 0;
        if a12_high && !self.last_a12_high {
            // Debounce: ignore rises that occur too soon after the last one.
            let delta = ctx.ppu_cycle.saturating_sub(self.last_a12_rise_ppu_cycle);
            if delta >= 8 {
                if self.irq_reload {
                    self.irq_counter = self.irq_latch;
                    self.irq_reload = false;
                } else if self.irq_counter == 0 {
                    self.irq_counter = self.irq_latch;
                } else {
                    self.irq_counter = self.irq_counter.saturating_sub(1);
                }

                if self.irq_counter == 0 && self.irq_enabled {
                    self.irq_pending = true;
                }
            }
            self.last_a12_rise_ppu_cycle = ctx.ppu_cycle;
        }
        self.last_a12_high = a12_high;
    }
}

impl Mapper for Mapper119 {
    fn reset(&mut self, _kind: ResetKind) {
        self.bank_select = 0;
        self.bank_regs.fill(0);
        self.prg_ram_enable = false;
        self.prg_ram_write_protect = true;
        self.irq_latch = 0;
        self.irq_counter = 0;
        self.irq_reload = false;
        self.irq_enabled = false;
        self.irq_pending = false;
        self.last_a12_high = false;
        self.last_a12_rise_ppu_cycle = 0;
        self.mirroring = self.base_mirroring;
    }

    fn cpu_read(&self, addr: u16) -> Option<u8> {
        let value = match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => return self.read_prg_ram(addr),
            cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END => self.read_prg_rom(addr),
            _ => return None,
        };
        Some(value)
    }

    fn cpu_write(&mut self, addr: u16, data: u8, _cpu_cycle: u64) {
        if (cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END).contains(&addr) {
            self.write_prg_ram(addr, data);
            return;
        }

        if let Some(reg) = TqromCpuRegister::from_addr(addr) {
            use TqromCpuRegister::*;

            match reg {
                BankSelect => {
                    self.bank_select = data & 0xC7;
                }
                BankData => {
                    let target = (self.bank_select & 0x07) as usize;
                    if target < self.bank_regs.len() {
                        self.bank_regs[target] = data;
                    }
                }
                MirrorControl => {
                    self.mirroring = if data & 0x01 == 0 {
                        self.base_mirroring
                    } else {
                        match self.base_mirroring {
                            Mirroring::Horizontal => Mirroring::Vertical,
                            Mirroring::Vertical => Mirroring::Horizontal,
                            other => other,
                        }
                    };
                }
                PrgRamControl => {
                    self.prg_ram_write_protect = data & 0x40 != 0;
                    self.prg_ram_enable = data & 0x80 != 0;
                }
                IrqLatch => {
                    self.irq_latch = data;
                }
                IrqReload => {
                    self.irq_reload = true;
                }
                IrqDisable => {
                    self.irq_enabled = false;
                    self.irq_pending = false;
                }
                IrqEnable => {
                    self.irq_enabled = true;
                }
            }
        }
    }

    fn cpu_clock(&mut self, _cpu_cycle: u64) {}

    fn ppu_read(&self, addr: u16) -> Option<u8> {
        Some(self.read_chr(addr))
    }

    fn ppu_write(&mut self, addr: u16, data: u8) {
        self.write_chr(addr, data);
    }

    fn ppu_vram_access(&mut self, addr: u16, ctx: PpuVramAccessContext) {
        if ctx.kind == PpuVramAccessKind::RenderingFetch {
            self.observe_ppu_vram_access(addr, ctx);
        }
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
        Some(self.chr_rom.as_ref())
    }

    fn chr_ram(&self) -> Option<&[u8]> {
        Some(self.chr_ram.as_ref())
    }

    fn chr_ram_mut(&mut self) -> Option<&mut [u8]> {
        Some(self.chr_ram.as_mut())
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn mapper_id(&self) -> u16 {
        119
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("TQROM (MMC3 variant)")
    }
}
