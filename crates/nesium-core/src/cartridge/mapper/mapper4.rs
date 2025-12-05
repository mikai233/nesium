//! Mapper 4 (MMC3) implementation.
//!
//! This mapper powers many of the most popular NES games (e.g. Super Mario
//! Bros. 3, Kirby's Adventure). It provides:
//! - 8 KiB PRG-ROM banking with two switchable windows and two fixed windows.
//! - Fine‑grained CHR banking using 2 KiB + 1 KiB pages with optional A12
//!   inversion for better sprite/background layout.
//! - A scanline IRQ counter driven by PPU A12 rising edges.
//! - Mapper‑controlled mirroring and PRG‑RAM enable/write‑protect bits.
//!
//! Behaviour is modelled against the Nesdev MMC3 documentation and broadly
//! matches the timing used by Mesen2. A few details (such as power‑on state
//! and PRG‑RAM write protection) are approximations that are safe for the
//! majority of licensed games.
//!
//! | Area | Address range     | Behaviour                                       | IRQ/Audio     |
//! |------|-------------------|-------------------------------------------------|---------------|
//! | CPU  | `$6000-$7FFF`     | Optional PRG-RAM with enable/write-protect     | None          |
//! | CPU  | `$8000-$9FFF`     | Switchable 8 KiB PRG (slot 0) + bank select    | MMC3 scanline |
//! | CPU  | `$A000-$BFFF`     | Switchable 8 KiB PRG (slot 1) + mirroring/RAM  | MMC3 scanline |
//! | CPU  | `$C000-$DFFF`     | Switchable/fixed 8 KiB PRG (slot 2) + IRQ regs | MMC3 scanline |
//! | CPU  | `$E000-$FFFF`     | Fixed 8 KiB PRG (last) + IRQ enable/ack        | MMC3 scanline |
//! | PPU  | `$0000-$1FFF`     | 2×2 KiB + 4×1 KiB CHR banks, A12‑aware         | MMC3 scanline |
//! | PPU  | `$2000-$3EFF`     | Mirroring from header or MMC3 register         | None          |

use std::borrow::Cow;

use crate::{
    cartridge::{
        ChrRom, Mapper, PrgRom, TrainerBytes,
        header::{Header, Mirroring},
        mapper::{
            ChrStorage, PpuVramAccessContext, PpuVramAccessKind, allocate_prg_ram_with_trainer,
            select_chr_storage,
        },
    },
    memory::cpu as cpu_mem,
};

use crate::mem_block::ByteBlock;

/// PRG-ROM bank size exposed to the CPU (8 KiB).
const PRG_BANK_SIZE_8K: usize = 8 * 1024;
/// CHR banking granularity (1 KiB).
const CHR_BANK_SIZE_1K: usize = 1024;

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

/// CPU-visible MMC3 register set.
///
/// MMC3 exposes a handful of control registers in the `$8000-$FFFF` range,
/// mapped as even/odd addresses within each 8 KiB PRG window. Grouping these
/// into an enum keeps the CPU-side logic aligned with the documented layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Mmc3CpuRegister {
    /// `$8000/$8001` – bank select and bank data.
    BankSelect,
    BankData,
    /// `$A000/$A001` – mirroring control and PRG-RAM enable/write-protect.
    Mirroring,
    PrgRamProtect,
    /// `$C000/$C001` – IRQ latch value and reload strobe.
    IrqLatch,
    IrqReload,
    /// `$E000/$E001` – IRQ disable/ack and IRQ enable.
    IrqDisable,
    IrqEnable,
}

impl Mmc3CpuRegister {
    fn from_addr(addr: u16) -> Option<Self> {
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

#[derive(Debug, Clone)]
pub struct Mapper4 {
    prg_rom: crate::cartridge::PrgRom,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,

    /// Number of 8 KiB PRG-ROM banks.
    prg_bank_count: usize,

    /// Base mirroring mode from the header. Some MMC3 boards use fixed
    /// four‑screen mirroring and ignore $A000 writes entirely.
    base_mirroring: Mirroring,
    /// Current effective mirroring (may be overridden by $A000).
    mirroring: Mirroring,

    // Banking registers ----------------------------------------------------
    /// Bank select register ($8000). Layout:
    /// - Bits 0-2: select target bank register (0‑7).
    /// - Bit 6: PRG mode (0: swap at $8000, 1: swap at $C000).
    /// - Bit 7: CHR A12 inversion (0: 2 KiB banks at $0000/$0800,
    ///   1: 2 KiB banks at $1000/$1800).
    bank_select: u8,
    /// Bank data registers ($8001 writes). Index 0‑5 control CHR, 6‑7 control
    /// the two switchable PRG banks.
    bank_regs: Mapper4BankRegs,

    /// PRG‑RAM enable flag from $A001 bit7.
    prg_ram_enable: bool,
    /// PRG‑RAM write protection flag derived from $A001 bit6.
    ///
    /// Nesdev MMC3 doc: bit6 = 1 denies writes, 0 allows writes. We model
    /// that directly: when `prg_ram_write_protect` is true, writes are
    /// blocked even if PRG‑RAM is enabled.
    prg_ram_write_protect: bool,

    // IRQ registers --------------------------------------------------------
    /// IRQ latch value ($C000).
    irq_latch: u8,
    /// Internal down counter.
    irq_counter: u8,
    /// Reload flag set by $C001; causes the next A12 clock to reload from
    /// `irq_latch` instead of decrementing.
    irq_reload: bool,
    /// Whether IRQ output is enabled ($E001 / $E000).
    irq_enabled: bool,
    /// Latched IRQ line visible to the CPU core.
    irq_pending: bool,

    // PPU A12 edge detection -----------------------------------------------
    /// Last observed value of PPU address line A12.
    last_a12_high: bool,
    /// PPU cycle of the last A12 rising edge we acted on. Used as a simple
    /// debounce so that tightly spaced pattern fetches do not clock the
    /// counter multiple times within a single scanline.
    last_a12_rise_ppu_cycle: u64,
}

type Mapper4BankRegs = ByteBlock<8>;

impl Mapper4 {
    pub fn new(header: Header, prg_rom: PrgRom, chr_rom: ChrRom, trainer: TrainerBytes) -> Self {
        let prg_ram = allocate_prg_ram_with_trainer(&header, trainer);

        let chr = select_chr_storage(&header, chr_rom);
        let prg_bank_count = (prg_rom.len() / PRG_BANK_SIZE_8K).max(1);

        Self {
            prg_rom,
            prg_ram,
            chr,
            prg_bank_count,
            base_mirroring: header.mirroring,
            mirroring: header.mirroring,
            bank_select: 0,
            bank_regs: Mapper4BankRegs::new(),
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

    /// Returns true when CHR A12 inversion is active (bank select bit7 set).
    #[inline]
    fn chr_invert(&self) -> bool {
        self.bank_select & 0x80 != 0
    }

    /// Returns the current PRG banking mode (bank select bit6).
    ///
    /// false => mode 0: swap at $8000.
    /// true  => mode 1: swap at $C000.
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

    /// Resolve an 8 KiB PRG-ROM bank index, wrapping to the available ROM size.
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

        // Determine which 8 KiB slot this address falls into.
        let bank_slot = match addr {
            MMC3_PRG_SLOT0_START..=MMC3_PRG_SLOT0_END => 0,
            MMC3_PRG_SLOT1_START..=MMC3_PRG_SLOT1_END => 1,
            MMC3_PRG_SLOT2_START..=MMC3_PRG_SLOT2_END => 2,
            MMC3_PRG_FIXED_SLOT_START..=MMC3_PRG_FIXED_SLOT_END => 3,
            _ => return 0,
        };

        let last_bank = self.prg_bank_count.saturating_sub(1);
        let second_last_bank = self.prg_bank_count.saturating_sub(2);

        // PRG mode controls whether the first or third 8 KiB window is fixed.
        let bank = if !self.prg_swap_at_c000() {
            // Mode 0:
            //   $8000-$9FFF: bank 6 (switchable)
            //   $A000-$BFFF: bank 7 (switchable)
            //   $C000-$DFFF: second last bank (fixed)
            //   $E000-$FFFF: last bank (fixed)
            match bank_slot {
                0 => self.prg_bank_index(self.bank_regs[6]),
                1 => self.prg_bank_index(self.bank_regs[7]),
                2 => second_last_bank,
                _ => last_bank,
            }
        } else {
            // Mode 1:
            //   $8000-$9FFF: second last bank (fixed)
            //   $A000-$BFFF: bank 7 (switchable)
            //   $C000-$DFFF: bank 6 (switchable)
            //   $E000-$FFFF: last bank (fixed)
            match bank_slot {
                0 => second_last_bank,
                1 => self.prg_bank_index(self.bank_regs[7]),
                2 => self.prg_bank_index(self.bank_regs[6]),
                _ => last_bank,
            }
        };

        let base = bank.saturating_mul(PRG_BANK_SIZE_8K);
        let offset = (addr as usize - cpu_mem::PRG_ROM_START as usize) & (PRG_BANK_SIZE_8K - 1);
        let idx = base.saturating_add(offset);
        self.prg_rom.get(idx).copied().unwrap_or(0)
    }

    /// Resolve a CHR byte for the given PPU address, applying the current
    /// banking mode and A12 inversion. Both CHR ROM and CHR RAM cartridges are
    /// supported via the shared [`ChrStorage`] helper.
    fn read_chr(&self, addr: u16) -> u8 {
        let a = addr & 0x1FFF;
        let offset = a as usize;

        let (base, inner) = if !self.chr_invert() {
            // Normal layout:
            //   R0: 2 KiB at $0000-$07FF
            //   R1: 2 KiB at $0800-$0FFF
            //   R2: 1 KiB at $1000-$13FF
            //   R3: 1 KiB at $1400-$17FF
            //   R4: 1 KiB at $1800-$1BFF
            //   R5: 1 KiB at $1C00-$1FFF
            match a {
                0x0000..=0x07FF => {
                    let bank = (self.bank_regs[0] & !1) as usize;
                    (bank * CHR_BANK_SIZE_1K, offset)
                }
                0x0800..=0x0FFF => {
                    let bank = (self.bank_regs[1] & !1) as usize;
                    (bank * CHR_BANK_SIZE_1K, offset - 0x0800)
                }
                0x1000..=0x13FF => {
                    let bank = self.bank_regs[2] as usize;
                    (bank * CHR_BANK_SIZE_1K, offset - 0x1000)
                }
                0x1400..=0x17FF => {
                    let bank = self.bank_regs[3] as usize;
                    (bank * CHR_BANK_SIZE_1K, offset - 0x1400)
                }
                0x1800..=0x1BFF => {
                    let bank = self.bank_regs[4] as usize;
                    (bank * CHR_BANK_SIZE_1K, offset - 0x1800)
                }
                _ => {
                    let bank = self.bank_regs[5] as usize;
                    (bank * CHR_BANK_SIZE_1K, offset - 0x1C00)
                }
            }
        } else {
            // Inverted layout:
            //   R2: 1 KiB at $0000-$03FF
            //   R3: 1 KiB at $0400-$07FF
            //   R4: 1 KiB at $0800-$0BFF
            //   R5: 1 KiB at $0C00-$0FFF
            //   R0: 2 KiB at $1000-$17FF
            //   R1: 2 KiB at $1800-$1FFF
            match a {
                0x0000..=0x03FF => {
                    let bank = self.bank_regs[2] as usize;
                    (bank * CHR_BANK_SIZE_1K, offset)
                }
                0x0400..=0x07FF => {
                    let bank = self.bank_regs[3] as usize;
                    (bank * CHR_BANK_SIZE_1K, offset - 0x0400)
                }
                0x0800..=0x0BFF => {
                    let bank = self.bank_regs[4] as usize;
                    (bank * CHR_BANK_SIZE_1K, offset - 0x0800)
                }
                0x0C00..=0x0FFF => {
                    let bank = self.bank_regs[5] as usize;
                    (bank * CHR_BANK_SIZE_1K, offset - 0x0C00)
                }
                0x1000..=0x17FF => {
                    let bank = (self.bank_regs[0] & !1) as usize;
                    (bank * CHR_BANK_SIZE_1K, offset - 0x1000)
                }
                _ => {
                    let bank = (self.bank_regs[1] & !1) as usize;
                    (bank * CHR_BANK_SIZE_1K, offset - 0x1800)
                }
            }
        };

        self.chr.read_indexed(base, inner)
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        let a = addr & 0x1FFF;
        let offset = a as usize;

        let (base, inner) = if !self.chr_invert() {
            match a {
                0x0000..=0x07FF => {
                    let bank = (self.bank_regs[0] & !1) as usize;
                    (bank * CHR_BANK_SIZE_1K, offset)
                }
                0x0800..=0x0FFF => {
                    let bank = (self.bank_regs[1] & !1) as usize;
                    (bank * CHR_BANK_SIZE_1K, offset - 0x0800)
                }
                0x1000..=0x13FF => {
                    let bank = self.bank_regs[2] as usize;
                    (bank * CHR_BANK_SIZE_1K, offset - 0x1000)
                }
                0x1400..=0x17FF => {
                    let bank = self.bank_regs[3] as usize;
                    (bank * CHR_BANK_SIZE_1K, offset - 0x1400)
                }
                0x1800..=0x1BFF => {
                    let bank = self.bank_regs[4] as usize;
                    (bank * CHR_BANK_SIZE_1K, offset - 0x1800)
                }
                _ => {
                    let bank = self.bank_regs[5] as usize;
                    (bank * CHR_BANK_SIZE_1K, offset - 0x1C00)
                }
            }
        } else {
            match a {
                0x0000..=0x03FF => {
                    let bank = self.bank_regs[2] as usize;
                    (bank * CHR_BANK_SIZE_1K, offset)
                }
                0x0400..=0x07FF => {
                    let bank = self.bank_regs[3] as usize;
                    (bank * CHR_BANK_SIZE_1K, offset - 0x0400)
                }
                0x0800..=0x0BFF => {
                    let bank = self.bank_regs[4] as usize;
                    (bank * CHR_BANK_SIZE_1K, offset - 0x0800)
                }
                0x0C00..=0x0FFF => {
                    let bank = self.bank_regs[5] as usize;
                    (bank * CHR_BANK_SIZE_1K, offset - 0x0C00)
                }
                0x1000..=0x17FF => {
                    let bank = (self.bank_regs[0] & !1) as usize;
                    (bank * CHR_BANK_SIZE_1K, offset - 0x1000)
                }
                _ => {
                    let bank = (self.bank_regs[1] & !1) as usize;
                    (bank * CHR_BANK_SIZE_1K, offset - 0x1800)
                }
            }
        };

        self.chr.write_indexed(base, inner, data);
    }

    fn write_bank_select(&mut self, data: u8) {
        // Only bits 0-2, 6, and 7 are documented; keep other bits unchanged
        // to avoid surprising games that accidentally rely on them.
        self.bank_select = data;
    }

    fn write_bank_data(&mut self, data: u8) {
        let index = (self.bank_select & 0x07) as usize;
        if index >= self.bank_regs.len() {
            return;
        }

        // Nesdev: For R0/R1 (2 KiB CHR banks) the low bit is ignored by the
        // hardware because A10 is forced to 0. We keep the original value
        // around and mask when decoding the CHR address so that test ROMs can
        // still observe the written value if necessary.
        self.bank_regs[index] = data;
    }

    fn write_mirroring(&mut self, data: u8) {
        // Boards that use four‑screen VRAM typically ignore $A000 mirroring
        // writes and keep their fixed layout, so preserve that behaviour.
        if self.base_mirroring == Mirroring::FourScreen {
            return;
        }

        self.mirroring = if data & 0x01 == 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };
    }

    fn write_prg_ram_protect(&mut self, data: u8) {
        // Nesdev PRG RAM protect ($A001):
        // Bit 7: PRG RAM chip enable (0: disable; 1: enable)
        // Bit 6: write protection (0: allow writes; 1: deny writes)
        self.prg_ram_enable = data & 0x80 != 0;
        self.prg_ram_write_protect = data & 0x40 != 0;
        // NOTE: Some emulators choose to ignore these bits (or invert bit 6)
        // to approximate MMC6 behaviour under iNES mapper 4. We keep MMC3's
        // documented semantics here; MMC6 should be modelled as a separate
        // mapper or via NES 2.0 submappers.
    }

    fn write_irq_latch(&mut self, data: u8) {
        self.irq_latch = data;
    }

    fn write_irq_reload(&mut self) {
        // The next A12 rising edge reloads the counter from the latch.
        self.irq_reload = true;
    }

    fn write_irq_disable(&mut self) {
        // Writes to $E000 disable further IRQs and also acknowledge any
        // pending one, matching the behaviour described on Nesdev.
        self.irq_enabled = false;
        self.irq_pending = false;
    }

    fn write_irq_enable(&mut self) {
        self.irq_enabled = true;
    }

    /// Called when a debounced PPU A12 rising edge is detected during
    /// rendering. This clocks the internal IRQ counter in the usual MMC3
    /// manner.
    fn clock_irq_counter(&mut self) {
        if self.irq_reload || self.irq_counter == 0 {
            self.irq_counter = self.irq_latch;
            self.irq_reload = false;
        } else {
            self.irq_counter = self.irq_counter.wrapping_sub(1);
        }

        // When the counter transitions to 0, an IRQ is requested on the next
        // CPU instruction boundary (the CPU core observes `irq_pending`).
        if self.irq_counter == 0 && self.irq_enabled {
            self.irq_pending = true;
        }
    }

    /// Observe a PPU VRAM access and detect A12 rising edges during rendering.
    fn observe_ppu_vram_access(&mut self, addr: u16, ctx: PpuVramAccessContext) {
        // IRQ counter is clocked by PPU A12 rising edges while rendering.
        if ctx.kind != PpuVramAccessKind::RenderingFetch {
            return;
        }

        if addr >= 0x2000 {
            // Only pattern table accesses ($0000-$1FFF) affect the counter.
            return;
        }

        let a12_high = addr & 0x1000 != 0;

        // Nesdev: MMC3 requires A12 to be low for a period (~8 PPU cycles)
        // before a new rising edge is recognised. We approximate this by
        // enforcing a minimum PPU cycle distance between edges.
        if a12_high && !self.last_a12_high {
            let delta = ctx.ppu_cycle.saturating_sub(self.last_a12_rise_ppu_cycle);
            if delta >= 8 {
                self.last_a12_rise_ppu_cycle = ctx.ppu_cycle;
                self.clock_irq_counter();
            }
        }

        self.last_a12_high = a12_high;
    }
}

impl Mapper for Mapper4 {
    fn power_on(&mut self) {
        // Power-on defaults chosen to match common emulator behaviour:
        // - PRG mode = 1 (swap at $C000) so that the last bank appears at
        //   $E000-$FFFF and the second last at $8000-$9FFF.
        // - CHR A12 inversion disabled.
        // - PRG-RAM disabled until the game explicitly enables it via $A001.
        self.bank_select = 0x40;
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

    fn reset(&mut self) {
        // Many games program the MMC3 fully on reset, but keeping behaviour
        // roughly consistent with power-on improves compatibility with test
        // ROMs that probe initial state.
        self.power_on();
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

        if let Some(reg) = Mmc3CpuRegister::from_addr(addr) {
            use Mmc3CpuRegister::*;

            match reg {
                BankSelect => self.write_bank_select(data),
                BankData => self.write_bank_data(data),
                Mirroring => self.write_mirroring(data),
                PrgRamProtect => self.write_prg_ram_protect(data),
                IrqLatch => self.write_irq_latch(data),
                IrqReload => self.write_irq_reload(),
                IrqDisable => self.write_irq_disable(),
                IrqEnable => self.write_irq_enable(),
            }
        }
    }

    fn ppu_read(&self, addr: u16) -> Option<u8> {
        Some(self.read_chr(addr))
    }

    fn ppu_write(&mut self, addr: u16, data: u8) {
        self.write_chr(addr, data);
    }

    fn ppu_vram_access(&mut self, addr: u16, ctx: PpuVramAccessContext) {
        self.observe_ppu_vram_access(addr, ctx);
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
        4
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("MMC3")
    }
}
