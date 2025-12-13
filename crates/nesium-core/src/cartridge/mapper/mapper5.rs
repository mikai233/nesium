use std::{borrow::Cow, cell::Cell};

use crate::{
    cartridge::{
        ChrRom, Mapper, PrgRom, TrainerBytes,
        header::{Header, Mirroring},
        mapper::{
            ChrStorage, NametableTarget, PpuVramAccessContext, PpuVramAccessKind,
            allocate_prg_ram_with_trainer, select_chr_storage,
        },
    },
    mem_block::ByteBlock,
    reset_kind::ResetKind,
};

// Mapper 5 – MMC5 with extended PRG/CHR/nametable features.
//
// | Area | Address range     | Behaviour                                              | IRQ/Audio      |
// |------|-------------------|--------------------------------------------------------|----------------|
// | CPU  | `$6000-$7FFF`     | Bankswitched PRG-RAM via `$5113` (when enabled)       | None           |
// | CPU  | `$8000-$FFFF`     | PRG ROM/RAM windows in 8/16/32 KiB modes (`$5100`)    | MMC5 scanline  |
// | CPU  | `$5100-$5117`     | PRG/CHR/ExRAM/nametable control + PRG banking regs    | None           |
// | CPU  | `$5120-$5127`     | CHR bank registers (1/2/4/8 KiB modes)                | None           |
// | CPU  | `$5200-$5206`     | Split-screen, IRQ, multiplier, and status registers   | MMC5 scanline  |
// | CPU  | `$5C00-$5FFF`     | 1 KiB ExRAM CPU window (mode‑dependent behaviour)     | None           |
// | PPU  | `$0000-$1FFF`     | CHR ROM/RAM with flexible banking via `$5120-$5127`   | MMC5 scanline  |
// | PPU  | `$2000-$3EFF`     | Nametable mapping/fill using ExRAM and `$5105-$5107`  | None           |

/// MMC5 PRG bank size (8 KiB).
const PRG_BANK_SIZE_8K: usize = 8 * 1024;

/// MMC5 has 1 KiB of internal extended RAM.
const EXRAM_SIZE: usize = 1024;

/// CPU `$6000-$7FFF`: bankswitched PRG-RAM window controlled by `$5113`.
const MMC5_PRG_RAM_WINDOW_START: u16 = 0x6000;
const MMC5_PRG_RAM_WINDOW_END: u16 = 0x7FFF;

/// CPU `$8000-$FFFF`: bankswitched PRG-ROM/PRG-RAM windows.
const MMC5_PRG_WINDOW_START: u16 = 0x8000;
const MMC5_PRG_WINDOW_END: u16 = 0xFFFF;
/// CPU `$A000`, `$C000`, `$E000`: boundaries between PRG sub-windows.
const MMC5_PRG_WINDOW_A000_START: u16 = 0xA000;
const MMC5_PRG_WINDOW_C000_START: u16 = 0xC000;
const MMC5_PRG_WINDOW_E000_START: u16 = 0xE000;

/// MMC5 control/configuration registers in `$5100-$5107`.
/// - `$5100`: PRG mode (8/16/32 KiB windows).
/// - `$5101`: CHR mode (1/2/4/8 KiB pages).
/// - `$5102/$5103`: PRG-RAM write-protect keys.
/// - `$5104`: ExRAM mode (nametable/attribute/CPU RAM behaviour).
/// - `$5105`: per-nametable mapping control.
/// - `$5106/$5107`: fill tile and attribute for extended nametable modes.
const MMC5_REG_PRG_MODE: u16 = 0x5100;
const MMC5_REG_CHR_MODE: u16 = 0x5101;
const MMC5_REG_PRG_RAM_PROTECT1: u16 = 0x5102;
const MMC5_REG_PRG_RAM_PROTECT2: u16 = 0x5103;
const MMC5_REG_EXRAM_MODE: u16 = 0x5104;
const MMC5_REG_NAMETABLE_MAPPING: u16 = 0x5105;
const MMC5_REG_FILL_TILE: u16 = 0x5106;
const MMC5_REG_FILL_ATTR: u16 = 0x5107;

/// MMC5 PRG banking registers.
/// - `$5113`: PRG-RAM page for `$6000-$7FFF`.
/// - `$5114-$5117`: PRG-ROM/PRG-RAM bank registers for `$8000/$A000/$C000/$E000`.
const MMC5_REG_PRG_BANK_6000_7FFF: u16 = 0x5113;
const MMC5_REG_PRG_BANK_8000: u16 = 0x5114;
const MMC5_REG_PRG_BANK_A000: u16 = 0x5115;
const MMC5_REG_PRG_BANK_C000: u16 = 0x5116;
const MMC5_REG_PRG_BANK_E000: u16 = 0x5117;

/// MMC5 CHR banking registers `$5120-$5127` and upper CHR bank bits `$5130`.
const MMC5_REG_CHR_BANK_FIRST: u16 = 0x5120;
const MMC5_REG_CHR_BANK_LAST: u16 = 0x5127;
const MMC5_REG_CHR_UPPER_BITS: u16 = 0x5130;

/// MMC5 split-screen / IRQ / multiplier registers in `$5200-$5206`.
const MMC5_REG_SPLIT_CONTROL: u16 = 0x5200;
const MMC5_REG_SPLIT_SCROLL: u16 = 0x5201;
const MMC5_REG_SPLIT_CHR_BANK: u16 = 0x5202;
/// CPU `$5203`: scanline IRQ target.
const MMC5_REG_IRQ_SCANLINE: u16 = 0x5203;
/// CPU `$5204`: IRQ status (pending + in-frame bits).
const MMC5_REG_IRQ_STATUS: u16 = 0x5204;
/// CPU `$5205/$5206`: 8×8→16 multiplier result low/high bytes.
const MMC5_REG_MULTIPLIER_A: u16 = 0x5205;
const MMC5_REG_MULTIPLIER_B: u16 = 0x5206;

/// CPU `$5C00-$5FFF`: ExRAM CPU window.
const MMC5_EXRAM_CPU_START: u16 = 0x5C00;
const MMC5_EXRAM_CPU_END: u16 = 0x5FFF;

/// CPU-visible MMC5 register set.
///
/// MMC5 exposes a rich set of control registers across `$5100-$5206` as well
/// as CPU-mapped ExRAM and PRG-RAM/PRG-ROM windows. This enum groups the
/// major logical registers so that CPU-side logic can work with names instead
/// of raw addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Mmc5CpuRegister {
    PrgMode,
    ChrMode,
    PrgRamProtect1,
    PrgRamProtect2,
    ExRamMode,
    NametableMapping,
    FillTile,
    FillAttr,
    PrgBank6000,
    PrgBank8000,
    PrgBankA000,
    PrgBankC000,
    PrgBankE000,
    ChrBank,
    ChrUpperBits,
    SplitControl,
    SplitScroll,
    SplitChrBank,
    IrqScanline,
    IrqStatus,
    MultiplierA,
    MultiplierB,
    ExRamCpu,
    PrgRamWindow,
    PrgWindow,
}

impl Mmc5CpuRegister {
    fn from_addr(addr: u16) -> Option<Self> {
        use Mmc5CpuRegister::*;

        match addr {
            MMC5_REG_PRG_MODE => Some(PrgMode),
            MMC5_REG_CHR_MODE => Some(ChrMode),
            MMC5_REG_PRG_RAM_PROTECT1 => Some(PrgRamProtect1),
            MMC5_REG_PRG_RAM_PROTECT2 => Some(PrgRamProtect2),
            MMC5_REG_EXRAM_MODE => Some(ExRamMode),
            MMC5_REG_NAMETABLE_MAPPING => Some(NametableMapping),
            MMC5_REG_FILL_TILE => Some(FillTile),
            MMC5_REG_FILL_ATTR => Some(FillAttr),
            MMC5_REG_PRG_BANK_6000_7FFF => Some(PrgBank6000),
            MMC5_REG_PRG_BANK_8000 => Some(PrgBank8000),
            MMC5_REG_PRG_BANK_A000 => Some(PrgBankA000),
            MMC5_REG_PRG_BANK_C000 => Some(PrgBankC000),
            MMC5_REG_PRG_BANK_E000 => Some(PrgBankE000),
            MMC5_REG_CHR_BANK_FIRST..=MMC5_REG_CHR_BANK_LAST => Some(ChrBank),
            MMC5_REG_CHR_UPPER_BITS => Some(ChrUpperBits),
            MMC5_REG_SPLIT_CONTROL => Some(SplitControl),
            MMC5_REG_SPLIT_SCROLL => Some(SplitScroll),
            MMC5_REG_SPLIT_CHR_BANK => Some(SplitChrBank),
            MMC5_REG_IRQ_SCANLINE => Some(IrqScanline),
            MMC5_REG_IRQ_STATUS => Some(IrqStatus),
            MMC5_REG_MULTIPLIER_A => Some(MultiplierA),
            MMC5_REG_MULTIPLIER_B => Some(MultiplierB),
            MMC5_EXRAM_CPU_START..=MMC5_EXRAM_CPU_END => Some(ExRamCpu),
            MMC5_PRG_RAM_WINDOW_START..=MMC5_PRG_RAM_WINDOW_END => Some(PrgRamWindow),
            MMC5_PRG_WINDOW_START..=MMC5_PRG_WINDOW_END => Some(PrgWindow),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum PrgWindowSize {
    /// 8 KiB CPU window.
    Size8K,
    /// 16 KiB CPU window.
    Size16K,
    /// 32 KiB CPU window.
    Size32K,
}

#[derive(Debug, Clone)]
pub struct Mapper5 {
    prg_rom: PrgRom,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,
    exram: Box<[u8; EXRAM_SIZE]>,

    /// PRG ROM bank count in 8 KiB units.
    prg_bank_count: usize,

    // Configuration registers
    prg_mode: u8,         // $5100 (0-3)
    chr_mode: u8,         // $5101 (0-3)
    prg_ram_protect1: u8, // $5102
    prg_ram_protect2: u8, // $5103
    exram_mode: u8,       // $5104 (0-3)
    nt_mapping: u8,       // $5105
    fill_tile: u8,        // $5106
    fill_attr: u8,        // $5107 (low 2 bits used)

    // PRG banking registers ($5113-$5117).
    prg_bank_6000_7fff: u8, // $5113 (PRG-RAM / PRG-ROM, simplified)
    prg_bank_8000: u8,      // $5114
    prg_bank_a000: u8,      // $5115
    prg_bank_c000: u8,      // $5116
    prg_bank_e000: u8,      // $5117

    // CHR banking registers ($5120-$5127). We ignore the separate BG banks
    // ($5128-$512B) for now and treat these as the unified set.
    // TODO: Implement BG-specific banks once PpuVramAccessContext exposes
    // whether a given fetch is BG or sprite.
    chr_banks: Mapper5ChrBanks,
    chr_upper_bits: u8, // $5130 (upper CHR bank bits)

    // IRQ / scanline registers.
    irq_scanline: u8, // $5203
    irq_enabled: bool,
    irq_pending: Cell<bool>,

    // Vertical split registers ($5200-$5202). We currently only latch these;
    // proper split rendering requires richer PPU context (tile X/Y, BG vs
    // sprite, etc.) exposed from the PPU core.
    split_control: u8,  // $5200
    split_scroll: u8,   // $5201
    split_chr_bank: u8, // $5202

    // Unsigned 8x8->16 multiplier ($5205/$5206).
    mul_a: u8,
    mul_b: u8,
    mul_result: u16,

    // Scanline IRQ / frame tracking state (approximate).
    current_scanline: u8,
    last_scanline_cycle: u64,
    in_frame: bool,
    last_nt_addr: u16,
    nt_addr_repeat_count: u8,
    expect_scanline_on_next_fetch: bool,
}

type Mapper5ChrBanks = ByteBlock<8>;

impl Mapper5 {
    pub fn new(header: Header, prg_rom: PrgRom, chr_rom: ChrRom, trainer: TrainerBytes) -> Self {
        let prg_ram = allocate_prg_ram_with_trainer(&header, trainer);

        // MMC5 boards often have large PRG-RAM; the allocate_prg_ram helper
        // already considers NES 2.0 hints. Games that rely on banking PRG-RAM
        // across CPU windows still work when we treat PRG-RAM as a flat superset.
        let exram = Box::new([0u8; EXRAM_SIZE]);

        let prg_bank_count = (prg_rom.len() / PRG_BANK_SIZE_8K).max(1);

        Self {
            prg_rom,
            prg_ram,
            chr: select_chr_storage(&header, chr_rom),
            exram,
            prg_bank_count,
            prg_mode: 3, // default to 4×8 KiB banking
            chr_mode: 3, // default to 1 KiB CHR pages
            prg_ram_protect1: 0,
            prg_ram_protect2: 0,
            exram_mode: 0,
            nt_mapping: 0,
            fill_tile: 0,
            fill_attr: 0,
            prg_bank_6000_7fff: 0,
            prg_bank_8000: 0,
            prg_bank_a000: 0,
            prg_bank_c000: 0,
            prg_bank_e000: 0,
            chr_banks: Mapper5ChrBanks::new(),
            chr_upper_bits: 0,
            irq_scanline: 0,
            irq_enabled: false,
            irq_pending: Cell::new(false),
            split_control: 0,
            split_scroll: 0,
            split_chr_bank: 0,
            mul_a: 0xFF,
            mul_b: 0xFF,
            // Power-on default $FE01 per MMC5A docs.
            mul_result: 0xFF * 0xFF,
            current_scanline: 0,
            last_scanline_cycle: 0,
            in_frame: false,
            last_nt_addr: 0,
            nt_addr_repeat_count: 0,
            expect_scanline_on_next_fetch: false,
        }
    }

    fn prg_rom_bank(&self, bank: u8) -> usize {
        if self.prg_bank_count == 0 {
            0
        } else {
            (bank as usize) % self.prg_bank_count
        }
    }

    fn prg_ram_enabled(&self) -> bool {
        // Simple implementation: require $5102=0x02 and $5103=0x01 to enable
        // PRG-RAM writes. Reads are allowed whenever RAM is present.
        !self.prg_ram.is_empty()
            && self.prg_ram_protect1 & 0x03 == 0x02
            && self.prg_ram_protect2 & 0x03 == 0x01
    }

    /// Decode the effective 8 KiB PRG-ROM bank for a given register and
    /// window size, following the MMC5 bit layout described on Nesdev.
    fn prg_rom_bank_index(&self, reg: u8, size: PrgWindowSize, addr: u16) -> usize {
        if self.prg_bank_count == 0 {
            return 0;
        }

        // bit7 is RAM/ROM select, address decoding uses only bits 6..0.
        let reg7 = reg & 0x7F;
        let bank = match size {
            PrgWindowSize::Size8K => reg7 as usize,
            PrgWindowSize::Size16K => {
                // Bits 6..1 select A19..A14, CPU A13 selects the low bit.
                let a13 = ((addr >> 13) & 0x01) as usize;
                ((reg7 & 0x7E) as usize) | a13
            }
            PrgWindowSize::Size32K => {
                // Bits 6..2 select A19..A15, CPU A14..A13 provide the low bits.
                let a13 = ((addr >> 13) & 0x01) as usize;
                let a14 = ((addr >> 14) & 0x01) as usize;
                let high = (reg7 & 0x7C) as usize;
                let low = (a14 << 1) | a13;
                high | low
            }
        };

        bank % self.prg_bank_count
    }

    fn read_prg_rom_window(&self, addr: u16, reg: u8, size: PrgWindowSize) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }

        let bank = self.prg_rom_bank_index(reg, size, addr);
        let base = bank.saturating_mul(PRG_BANK_SIZE_8K);
        let offset = (addr as usize) & (PRG_BANK_SIZE_8K - 1);
        let idx = base.saturating_add(offset);
        self.prg_rom.get(idx).copied().unwrap_or(0)
    }

    /// Read PRG-RAM through a bankswitched window.
    fn read_prg_ram_page(&self, addr: u16, reg: u8) -> u8 {
        if self.prg_ram.is_empty() {
            return 0;
        }

        // Use low 3 bits as 8 KiB page index (superset mapping from Nesdev).
        let page = (reg & 0x07) as usize;
        let base = page * PRG_BANK_SIZE_8K;
        let offset = (addr as usize) & (PRG_BANK_SIZE_8K - 1);
        let idx = base + offset;
        let len = self.prg_ram.len();
        self.prg_ram[idx % len]
    }

    /// Write PRG-RAM through a bankswitched window, honoring write protection.
    fn write_prg_ram_page(&mut self, addr: u16, reg: u8, value: u8) {
        if self.prg_ram.is_empty() || !self.prg_ram_enabled() {
            return;
        }

        let page = (reg & 0x07) as usize;
        let base = page * PRG_BANK_SIZE_8K;
        let offset = (addr as usize) & (PRG_BANK_SIZE_8K - 1);
        let idx = base + offset;
        let len = self.prg_ram.len();
        if len != 0 {
            let wrapped = idx % len;
            self.prg_ram[wrapped] = value;
        }
    }

    /// Helper for PRG-ROM/PRG-RAM switchable windows (modes 1–3).
    fn read_prg_window_switchable(&self, addr: u16, reg: u8, size: PrgWindowSize) -> u8 {
        // $5114-$5116: bit7 = 0 => RAM, bit7 = 1 => ROM.
        let use_ram = (reg & 0x80) == 0 && !self.prg_ram.is_empty();
        if use_ram {
            self.read_prg_ram_page(addr, reg)
        } else {
            self.read_prg_rom_window(addr, reg, size)
        }
    }

    fn read_prg(&self, addr: u16) -> Option<u8> {
        match addr {
            MMC5_PRG_RAM_WINDOW_START..=MMC5_PRG_RAM_WINDOW_END => {
                if self.prg_ram.is_empty() {
                    return None;
                }
                // $6000-$7FFF always map PRG-RAM via $5113.
                Some(self.read_prg_ram_page(addr, self.prg_bank_6000_7fff))
            }
            MMC5_PRG_WINDOW_START..=MMC5_PRG_WINDOW_END => {
                let mode = self.prg_mode & 0x03;
                let value = match mode {
                    0 => {
                        // PRG mode 0: one 32 KiB ROM bank at $8000-$FFFF (ROM only).
                        self.read_prg_rom_window(addr, self.prg_bank_e000, PrgWindowSize::Size32K)
                    }
                    1 => {
                        // PRG mode 1:
                        // $8000-$BFFF: 16 KiB switchable ROM/RAM via $5115.
                        // $C000-$FFFF: 16 KiB ROM via $5117.
                        if addr < MMC5_PRG_WINDOW_C000_START {
                            self.read_prg_window_switchable(
                                addr,
                                self.prg_bank_a000,
                                PrgWindowSize::Size16K,
                            )
                        } else {
                            self.read_prg_rom_window(
                                addr,
                                self.prg_bank_e000,
                                PrgWindowSize::Size16K,
                            )
                        }
                    }
                    2 => {
                        // PRG mode 2:
                        // $8000-$BFFF: 16 KiB switchable ROM/RAM via $5115.
                        // $C000-$DFFF: 8 KiB switchable ROM/RAM via $5116.
                        // $E000-$FFFF: 8 KiB ROM via $5117.
                        if addr < MMC5_PRG_WINDOW_C000_START {
                            self.read_prg_window_switchable(
                                addr,
                                self.prg_bank_a000,
                                PrgWindowSize::Size16K,
                            )
                        } else if addr < MMC5_PRG_WINDOW_E000_START {
                            self.read_prg_window_switchable(
                                addr,
                                self.prg_bank_c000,
                                PrgWindowSize::Size8K,
                            )
                        } else {
                            self.read_prg_rom_window(
                                addr,
                                self.prg_bank_e000,
                                PrgWindowSize::Size8K,
                            )
                        }
                    }
                    _ => {
                        // PRG mode 3 (default for most games):
                        // $8000-$9FFF: 8 KiB ROM/RAM via $5114.
                        // $A000-$BFFF: 8 KiB ROM/RAM via $5115.
                        // $C000-$DFFF: 8 KiB ROM/RAM via $5116.
                        // $E000-$FFFF: 8 KiB ROM via $5117.
                        if addr < MMC5_PRG_WINDOW_A000_START {
                            self.read_prg_window_switchable(
                                addr,
                                self.prg_bank_8000,
                                PrgWindowSize::Size8K,
                            )
                        } else if addr < MMC5_PRG_WINDOW_C000_START {
                            self.read_prg_window_switchable(
                                addr,
                                self.prg_bank_a000,
                                PrgWindowSize::Size8K,
                            )
                        } else if addr < MMC5_PRG_WINDOW_E000_START {
                            self.read_prg_window_switchable(
                                addr,
                                self.prg_bank_c000,
                                PrgWindowSize::Size8K,
                            )
                        } else {
                            self.read_prg_rom_window(
                                addr,
                                self.prg_bank_e000,
                                PrgWindowSize::Size8K,
                            )
                        }
                    }
                };
                Some(value)
            }
            _ => None,
        }
    }

    fn write_prg(&mut self, addr: u16, data: u8) {
        match Mmc5CpuRegister::from_addr(addr) {
            // MMC5 control/config registers live in $5100-$51FF and $5200+.
            Some(Mmc5CpuRegister::PrgMode) => self.prg_mode = data & 0x03,
            Some(Mmc5CpuRegister::ChrMode) => self.chr_mode = data & 0x03,
            Some(Mmc5CpuRegister::PrgRamProtect1) => self.prg_ram_protect1 = data,
            Some(Mmc5CpuRegister::PrgRamProtect2) => self.prg_ram_protect2 = data,
            Some(Mmc5CpuRegister::ExRamMode) => self.exram_mode = data & 0x03,
            Some(Mmc5CpuRegister::NametableMapping) => self.nt_mapping = data,
            Some(Mmc5CpuRegister::FillTile) => self.fill_tile = data,
            Some(Mmc5CpuRegister::FillAttr) => self.fill_attr = data & 0x03,
            Some(Mmc5CpuRegister::PrgBank6000) => self.prg_bank_6000_7fff = data,
            Some(Mmc5CpuRegister::PrgBank8000) => self.prg_bank_8000 = data,
            Some(Mmc5CpuRegister::PrgBankA000) => self.prg_bank_a000 = data,
            Some(Mmc5CpuRegister::PrgBankC000) => self.prg_bank_c000 = data,
            Some(Mmc5CpuRegister::PrgBankE000) => self.prg_bank_e000 = data,
            Some(Mmc5CpuRegister::ChrBank) => {
                let idx = (addr - MMC5_REG_CHR_BANK_FIRST) as usize;
                self.chr_banks[idx] = data;
            }
            Some(Mmc5CpuRegister::ChrUpperBits) => self.chr_upper_bits = data & 0x03,
            Some(Mmc5CpuRegister::SplitControl) => {
                // Vertical split control. We only latch the value here; the
                // actual split behaviour is implemented in ppu_vram_access
                // and currently requires more detailed PPU context.
                self.split_control = data;
                // TODO: Use split_control in ppu_vram_access/map_nametable to
                // implement MMC5 vertical split once PpuVramAccessContext
                // exposes tile X/Y and BG vs sprite fetch information.
            }
            Some(Mmc5CpuRegister::SplitScroll) => {
                // Vertical split scroll value.
                self.split_scroll = data;
                // TODO: Honour split_scroll when emulating the split region.
            }
            Some(Mmc5CpuRegister::SplitChrBank) => {
                // Vertical split CHR bank.
                self.split_chr_bank = data;
                // TODO: Use split_chr_bank for BG CHR selection in split area.
            }
            Some(Mmc5CpuRegister::IrqScanline) => {
                self.irq_scanline = data;
                // Writes that modify the compare value also acknowledge a pending IRQ.
                self.irq_pending.set(false);
            }
            Some(Mmc5CpuRegister::IrqStatus) => {
                // Writing with bit7 set enables IRQ, clearing it disables.
                self.irq_enabled = data & 0x80 != 0;
                if !self.irq_enabled {
                    self.irq_pending.set(false);
                }
            }
            Some(Mmc5CpuRegister::MultiplierA) => {
                // Unsigned 8-bit multiplicand.
                self.mul_a = data;
                self.mul_result = (self.mul_a as u16) * (self.mul_b as u16);
            }
            Some(Mmc5CpuRegister::MultiplierB) => {
                // Unsigned 8-bit multiplier.
                self.mul_b = data;
                self.mul_result = (self.mul_a as u16) * (self.mul_b as u16);
            }
            Some(Mmc5CpuRegister::ExRamCpu) => {
                // Internal ExRAM writes. $5104 controls CPU accessibility:
                // modes 0/1 are write-only, mode 2 is read/write, mode 3 is
                // read-only. We only gate writes in mode 3 here; timing
                // restrictions while the PPU is rendering are not modelled.
                let idx = (addr - MMC5_EXRAM_CPU_START) as usize;
                if idx < EXRAM_SIZE {
                    if (self.exram_mode & 0x03) != 0b11 {
                        self.exram[idx] = data;
                    } else {
                        // TODO: In ExRAM mode 3, writes during rendering may
                        // have more complex behaviour (open bus). We simply
                        // ignore them for now.
                    }
                }
            }
            Some(Mmc5CpuRegister::PrgRamWindow) => {
                // $6000-$7FFF always map PRG-RAM via $5113.
                self.write_prg_ram_page(addr, self.prg_bank_6000_7fff, data);
            }
            Some(Mmc5CpuRegister::PrgWindow) => {
                // Some PRG windows in modes 1–3 can be mapped to PRG-RAM.
                if self.prg_ram.is_empty() || !self.prg_ram_enabled() {
                    return;
                }
                let mode = self.prg_mode & 0x03;
                match mode {
                    0 => {
                        // Mode 0 has PRG-ROM only at $8000-$FFFF.
                    }
                    1 => {
                        // $8000-$BFFF: 16 KiB ROM/RAM via $5115.
                        if addr < MMC5_PRG_WINDOW_C000_START && (self.prg_bank_a000 & 0x80) == 0 {
                            self.write_prg_ram_page(addr, self.prg_bank_a000, data);
                        }
                    }
                    2 => {
                        // $8000-$BFFF: 16 KiB ROM/RAM via $5115.
                        // $C000-$DFFF: 8 KiB ROM/RAM via $5116.
                        if addr < MMC5_PRG_WINDOW_C000_START {
                            if (self.prg_bank_a000 & 0x80) == 0 {
                                self.write_prg_ram_page(addr, self.prg_bank_a000, data);
                            }
                        } else if addr < MMC5_PRG_WINDOW_E000_START
                            && (self.prg_bank_c000 & 0x80) == 0
                        {
                            self.write_prg_ram_page(addr, self.prg_bank_c000, data);
                        }
                    }
                    _ => {
                        // Mode 3: three 8 KiB ROM/RAM windows.
                        if addr < MMC5_PRG_WINDOW_A000_START {
                            if (self.prg_bank_8000 & 0x80) == 0 {
                                self.write_prg_ram_page(addr, self.prg_bank_8000, data);
                            }
                        } else if addr < MMC5_PRG_WINDOW_C000_START {
                            if (self.prg_bank_a000 & 0x80) == 0 {
                                self.write_prg_ram_page(addr, self.prg_bank_a000, data);
                            }
                        } else if addr < MMC5_PRG_WINDOW_E000_START
                            && (self.prg_bank_c000 & 0x80) == 0
                        {
                            self.write_prg_ram_page(addr, self.prg_bank_c000, data);
                        }
                        // $E000-$FFFF is ROM-only.
                    }
                }
            }
            _ => {}
        }
    }

    fn chr_bank_for_addr(&self, addr: u16) -> (usize, usize) {
        // Decode which CHR bank register applies to this address based on
        // CHR mode ($5101) and the Nesdev mapping table.
        let mode = self.chr_mode & 0x03;
        let (reg_index, bank_size) = match mode {
            0 => {
                // 8 KiB page: $5127
                (7usize, 0x2000usize)
            }
            1 => {
                // 4 KiB pages: $5123 (0x0000-0x0FFF), $5127 (0x1000-0x1FFF)
                if addr < 0x1000 {
                    (3usize, 0x1000usize)
                } else {
                    (7usize, 0x1000usize)
                }
            }
            2 => {
                // 2 KiB pages: $5121,$5123,$5125,$5127
                match addr {
                    0x0000..=0x07FF => (1usize, 0x0800usize),
                    0x0800..=0x0FFF => (3usize, 0x0800usize),
                    0x1000..=0x17FF => (5usize, 0x0800usize),
                    _ => (7usize, 0x0800usize), // 0x1800-0x1FFF
                }
            }
            _ => {
                // Mode 3: eight 1 KiB pages via $5120-$5127.
                let index = ((addr as usize) >> 10) & 0x07;
                (index, 0x0400usize)
            }
        };

        let bank_val = self.chr_banks[reg_index];
        let upper = (self.chr_upper_bits & 0x03) as usize;
        let bank_index = (upper << 8) | bank_val as usize;
        (bank_index, bank_size)
    }

    fn read_chr(&self, addr: u16) -> u8 {
        let (bank_index, bank_size) = self.chr_bank_for_addr(addr);
        let base = bank_index.saturating_mul(bank_size);
        let offset = (addr as usize) & (bank_size - 1);
        self.chr.read_indexed(base, offset)
    }

    fn write_chr(&mut self, addr: u16, value: u8) {
        let (bank_index, bank_size) = self.chr_bank_for_addr(addr);
        let base = bank_index.saturating_mul(bank_size);
        let offset = (addr as usize) & (bank_size - 1);
        self.chr.write_indexed(base, offset, value);
    }

    fn exram_index_for_nametable(&self, offset: u16) -> usize {
        // ExRAM is a single 1 KiB window mirrored for any nametable that maps
        // to it. Offset is already relative to the nametable (0-0x3FF).
        (offset as usize) & (EXRAM_SIZE - 1)
    }

    fn is_fill_offset(offset: u16) -> bool {
        offset & 0x1000 != 0
    }

    fn decode_fill_offset(offset: u16) -> u16 {
        offset & 0x03FF
    }
}

impl Mapper for Mapper5 {
    fn reset(&mut self, kind: ResetKind) {
        if !matches!(kind, ResetKind::PowerOn) {
            return;
        }

        // Mesen2-style defaults: PRG/CHR 8 KiB/1 KiB modes, ExRAM mode 0, all
        // banks pointing at the start of PRG/CHR.
        self.prg_mode = 3;
        self.chr_mode = 3;
        self.prg_ram_protect1 = 0;
        self.prg_ram_protect2 = 0;
        self.exram_mode = 0;
        self.nt_mapping = 0;
        self.fill_tile = 0;
        self.fill_attr = 0;
        self.prg_bank_6000_7fff = 0;
        self.prg_bank_8000 = 0;
        self.prg_bank_a000 = 1;
        self.prg_bank_c000 = 2;
        self.prg_bank_e000 = (self.prg_bank_count.saturating_sub(1)) as u8;
        self.chr_banks.fill(0);
        self.chr_upper_bits = 0;
        self.irq_scanline = 0;
        self.irq_enabled = false;
        self.irq_pending.set(false);
        self.split_control = 0;
        self.split_scroll = 0;
        self.split_chr_bank = 0;
        self.mul_a = 0xFF;
        self.mul_b = 0xFF;
        self.mul_result = 0xFF * 0xFF; // $FE01
        self.current_scanline = 0;
        self.last_scanline_cycle = 0;
        // TODO: in_frame should be cleared precisely during vertical blank.
        // This requires an explicit vblank/frame signal from the PPU core.
        self.in_frame = false;
        self.last_nt_addr = 0;
        self.nt_addr_repeat_count = 0;
        self.expect_scanline_on_next_fetch = false;
        self.exram.fill(0);
    }

    fn cpu_read(&self, addr: u16) -> Option<u8> {
        match Mmc5CpuRegister::from_addr(addr) {
            Some(Mmc5CpuRegister::IrqStatus) => {
                // IRQ status ($5204). We expose the pending and "in frame" bits,
                // clearing the pending flag on read to match hardware ack
                // semantics. Bit 7 latches when the scanline IRQ triggers and
                // stays set until the CPU polls this register or rewrites the
                // IRQ counter.
                let mut value = 0u8;
                if self.irq_pending.get() {
                    value |= 0x80;
                }
                if self.in_frame {
                    value |= 0x40;
                }
                // Reading $5204 acknowledges a latched IRQ.
                // (Bit 6 remains as-is to reflect in-frame state.)
                // In-frame flag is not cleared here; it follows PPU fetch timing.
                // Source: observed emulator behaviour (Mesen2) and Nesdev docs.
                // Matches NES hardware by deasserting the IRQ level after the
                // CPU observes it.
                self.irq_pending.set(false);
                Some(value)
            }
            Some(Mmc5CpuRegister::MultiplierA) => Some(self.mul_result as u8),
            Some(Mmc5CpuRegister::MultiplierB) => Some((self.mul_result >> 8) as u8),
            Some(Mmc5CpuRegister::ExRamCpu) => {
                // Internal ExRAM CPU reads ($5C00-$5FFF).
                let idx = (addr - MMC5_EXRAM_CPU_START) as usize;
                if idx >= EXRAM_SIZE {
                    return Some(0);
                }
                let mode = self.exram_mode & 0x03;
                match mode {
                    0 | 1 => {
                        // Modes 0/1: CPU writes are allowed but reads behave
                        // like open bus. We approximate open bus as 0 here.
                        Some(0)
                    }
                    _ => {
                        // Modes 2/3: CPU can read ExRAM.
                        Some(self.exram[idx])
                    }
                }
            }
            _ => self.read_prg(addr),
        }
    }

    fn cpu_write(&mut self, addr: u16, data: u8, _cpu_cycle: u64) {
        self.write_prg(addr, data);
    }

    fn cpu_clock(&mut self, _cpu_cycle: u64) {
        // TODO: Implement MMC5 scanline IRQ timing based on CPU/PPU state when needed.
    }

    fn ppu_read(&self, addr: u16) -> Option<u8> {
        // MMC5 CHR banking only applies to pattern table space.
        if addr < 0x2000 {
            Some(self.read_chr(addr))
        } else {
            None
        }
    }

    fn ppu_write(&mut self, addr: u16, data: u8) {
        if addr < 0x2000 {
            self.write_chr(addr, data);
        }
    }

    fn ppu_vram_access(&mut self, addr: u16, ctx: PpuVramAccessContext) {
        // Only PPU background/sprite rendering fetches are relevant for MMC5
        // scanline IRQ and split/ExGrafix behaviour. CPU-driven $2007 accesses
        // are ignored here.
        if ctx.kind != PpuVramAccessKind::RenderingFetch {
            return;
        }

        // Mark that we are inside a rendered frame for $5204 "in frame" bit.
        // TODO: Clear in_frame precisely at vblank start when the PPU exposes
        // that information; for now this stays set once rendering begins.
        self.in_frame = true;

        // Nesdev: MMC5 detects scanlines by watching three consecutive reads
        // from the same $2xxx nametable address; the following read
        // (regardless of address) is treated as the scanline boundary.
        // We approximate that here using the VRAM address and access kind.
        if !(0x2000..=0x2FFF).contains(&addr) {
            // Leaving nametable space resets the repeat tracking.
            self.nt_addr_repeat_count = 0;
            return;
        }

        if self.last_nt_addr == addr {
            self.nt_addr_repeat_count = self.nt_addr_repeat_count.saturating_add(1);
        } else {
            self.last_nt_addr = addr;
            self.nt_addr_repeat_count = 1;
        }

        if self.nt_addr_repeat_count == 3 {
            // The next rendering fetch is considered the scanline boundary.
            self.expect_scanline_on_next_fetch = true;
            self.nt_addr_repeat_count = 0;
            return;
        }

        if self.expect_scanline_on_next_fetch {
            self.expect_scanline_on_next_fetch = false;

            // Approximate frame boundaries by looking for a large gap in
            // PPU cycles between successive detected scanlines. A real
            // implementation should use explicit PPU vblank/frame signals.
            const SCANLINE_GAP_THRESHOLD: u64 = 2000;
            if self.last_scanline_cycle == 0
                || ctx.ppu_cycle.saturating_sub(self.last_scanline_cycle) > SCANLINE_GAP_THRESHOLD
            {
                // Start of a new frame.
                self.current_scanline = 0;
            } else {
                self.current_scanline = self.current_scanline.wrapping_add(1);
            }
            self.last_scanline_cycle = ctx.ppu_cycle;

            // Generate scanline IRQ when the current scanline matches $5203.
            // Per docs, a compare value of $00 suppresses new IRQs.
            let target = self.irq_scanline;
            if self.irq_enabled && target != 0 && self.current_scanline == target {
                self.irq_pending.set(true);
            }
        }

        // TODO: Use addr/ctx and the vertical split registers ($5200-$5202)
        // plus ExRAM contents to implement MMC5's split-screen mode and
        // extended attribute (ExGrafix) behaviour. This requires additional
        // PPU context (e.g. tile X/Y, BG vs sprite fetch) that is not yet
        // exposed in PpuVramAccessContext.
    }

    fn map_nametable(&self, addr: u16) -> NametableTarget {
        // Derive nametable index (0-3) from PPU address.
        if !(0x2000..0x3000).contains(&addr) {
            return NametableTarget::Ciram(addr & 0x07FF);
        }
        let nt = ((addr - 0x2000) / 0x0400) as u8; // 0..3
        let offset = (addr - 0x2000) & 0x03FF;
        let sel_bits = (self.nt_mapping >> (nt * 2)) & 0x03;
        match sel_bits {
            0 => {
                // CIRAM page 0
                NametableTarget::Ciram(offset)
            }
            1 => {
                // CIRAM page 1
                NametableTarget::Ciram(0x0400 | offset)
            }
            2 => {
                // Internal ExRAM.
                NametableTarget::MapperVram(offset)
            }
            3 => {
                // Fill mode: encode using high bit so mapper_nametable_* can
                // distinguish it from ExRAM-backed nametables.
                NametableTarget::MapperVram(0x1000 | offset)
            }
            _ => NametableTarget::Ciram(offset),
        }
    }

    fn mapper_nametable_read(&self, offset: u16) -> u8 {
        if Self::is_fill_offset(offset) {
            let rel = Self::decode_fill_offset(offset);
            // Fill-mode tile vs attribute behaviour depends on the offset.
            if rel < 0x03C0 {
                // Nametable entries replaced by fill-tile byte.
                self.fill_tile
            } else {
                // Attribute bytes replaced by fill-color replicated into all 4 quads.
                let bits = self.fill_attr & 0x03;
                // Replicate two bits across the byte: b1b0 b1b0 b1b0 b1b0.
                bits * 0x55
            }
        } else {
            // ExRAM-backed nametable. When $5104 is %10 or %11, the
            // nametable reads back as all zeros instead of exposing the
            // underlying RAM (per Nesdev). We still allow CPU access to
            // ExRAM via $5C00-$5FFF regardless of this.
            if (self.exram_mode & 0x03) >= 0b10 {
                0
            } else {
                let idx = self.exram_index_for_nametable(offset);
                self.exram[idx]
            }
        }
    }

    fn mapper_nametable_write(&mut self, offset: u16, value: u8) {
        if Self::is_fill_offset(offset) {
            // Writes to fill-mode nametables are ignored; only $5106/$5107 matter.
            let _ = (offset, value);
        } else {
            let idx = self.exram_index_for_nametable(offset);
            self.exram[idx] = value;
        }
    }

    fn irq_pending(&self) -> bool {
        self.irq_pending.get()
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

    fn mapper_ram(&self) -> Option<&[u8]> {
        Some(self.exram.as_ref())
    }

    fn mapper_ram_mut(&mut self) -> Option<&mut [u8]> {
        Some(self.exram.as_mut())
    }

    fn mirroring(&self) -> Mirroring {
        // MMC5 nametable mapping is fully controlled by $5105; advertise
        // mapper-controlled mirroring to the rest of the system.
        Mirroring::MapperControlled
    }

    fn mapper_id(&self) -> u16 {
        5
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("MMC5")
    }
}
