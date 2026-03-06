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

use crate::cartridge::mapper::{MapperMemoryMut, MapperMemoryRef};

use crate::reset_kind::ResetKind;
use crate::{
    cartridge::{
        ChrRom, Mapper, PrgRom, TrainerBytes,
        header::{Header, Mirroring, RomFormat},
        mapper::{
            ChrStorage, MapperEvent, MapperHookMask, PpuVramAccessContext,
            allocate_prg_ram_with_trainer,
            core::mmc3::{
                MMC3_POWER_ON_BANK_REGS, Mmc3Core, Mmc3CoreResetConfig, Mmc3CpuRegister,
                Mmc3IrqRevision, Mmc3WriteConfig, Mmc3WriteResult, PRG_BANK_SIZE_8K,
                resolve_mmc3_chr_bank,
            },
            select_chr_storage,
        },
    },
    memory::cpu as cpu_mem,
};

#[cfg(feature = "savestate-serde")]
use serde::{Deserialize, Serialize};

/// PRG-ROM bank size exposed to the CPU (8 KiB).
/// CHR banking granularity (1 KiB).
const CHR_BANK_SIZE_1K: usize = 1024;
const MMC3_WRITE_CONFIG: Mmc3WriteConfig = Mmc3WriteConfig {
    bank_select_mask: 0xFF,
    clear_counter_on_reload: true,
};

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
    mmc3: Mmc3Core,
    mmc3_reset: Mmc3CoreResetConfig,
}

#[cfg_attr(feature = "savestate-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mapper4State {
    pub base_mirroring: u8,
    pub mirroring: u8,
    pub bank_select: u8,
    pub bank_regs: [u8; 8],
    pub prg_ram_enable: bool,
    pub prg_ram_write_protect: bool,
    pub irq_latch: u8,
    pub irq_counter: u8,
    pub irq_reload: bool,
    pub irq_enabled: bool,
    pub irq_pending: bool,
    pub irq_revision: u8,
    pub a12_low_start_master_clock: Option<u64>,
}

impl Mapper4 {
    pub fn new(header: Header, prg_rom: PrgRom, chr_rom: ChrRom, trainer: TrainerBytes) -> Self {
        let prg_ram = allocate_prg_ram_with_trainer(&header, trainer);

        let chr = select_chr_storage(&header, chr_rom);
        let prg_bank_count = (prg_rom.len() / PRG_BANK_SIZE_8K).max(1);
        let irq_revision = detect_mmc3_irq_revision(header);
        let mmc3_reset = Mmc3CoreResetConfig {
            bank_select: 0x00,
            bank_regs: MMC3_POWER_ON_BANK_REGS,
            prg_ram_enable: false,
            prg_ram_write_protect: false,
            irq_revision,
        };

        Self {
            prg_rom,
            prg_ram,
            chr,
            prg_bank_count,
            base_mirroring: header.mirroring(),
            mirroring: header.mirroring(),
            mmc3: Mmc3Core::new(mmc3_reset),
            mmc3_reset,
        }
    }

    pub(crate) fn save_state(&self) -> Mapper4State {
        let mut regs = [0u8; 8];
        regs.copy_from_slice(self.mmc3.bank_regs.as_slice());
        Mapper4State {
            base_mirroring: mirroring_to_u8(self.base_mirroring),
            mirroring: mirroring_to_u8(self.mirroring),
            bank_select: self.mmc3.bank_select,
            bank_regs: regs,
            prg_ram_enable: self.mmc3.prg_ram_enable,
            prg_ram_write_protect: self.mmc3.prg_ram_write_protect,
            irq_latch: self.mmc3.irq_latch,
            irq_counter: self.mmc3.irq_counter,
            irq_reload: self.mmc3.irq_reload,
            irq_enabled: self.mmc3.irq_enabled,
            irq_pending: self.mmc3.irq_pending,
            irq_revision: self.mmc3.irq_revision.as_u8(),
            a12_low_start_master_clock: self.mmc3.a12_low_start_master_clock,
        }
    }

    pub(crate) fn load_state(&mut self, state: &Mapper4State) {
        self.base_mirroring = mirroring_from_u8(state.base_mirroring);
        self.mirroring = mirroring_from_u8(state.mirroring);
        self.mmc3.bank_select = state.bank_select;
        self.mmc3
            .bank_regs
            .as_mut_slice()
            .copy_from_slice(&state.bank_regs);
        self.mmc3.prg_ram_enable = state.prg_ram_enable;
        self.mmc3.prg_ram_write_protect = state.prg_ram_write_protect;
        self.mmc3.irq_latch = state.irq_latch;
        self.mmc3.irq_counter = state.irq_counter;
        self.mmc3.irq_reload = state.irq_reload;
        self.mmc3.irq_enabled = state.irq_enabled;
        self.mmc3.irq_pending = state.irq_pending;
        self.mmc3.irq_revision = Mmc3IrqRevision::from_u8(state.irq_revision);
        self.mmc3.a12_low_start_master_clock = state.a12_low_start_master_clock;
    }

    /// Returns true when CHR A12 inversion is active (bank select bit7 set).
    #[inline]
    fn chr_invert(&self) -> bool {
        self.mmc3.chr_invert()
    }

    fn read_prg_ram(&self, addr: u16) -> Option<u8> {
        self.mmc3.read_prg_ram(&self.prg_ram, addr)
    }

    fn write_prg_ram(&mut self, addr: u16, data: u8) {
        self.mmc3.write_prg_ram(&mut self.prg_ram, addr, data);
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }
        let Some(bank) = self.mmc3.resolve_prg_rom_bank(self.prg_bank_count, addr) else {
            return 0;
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
        let (bank, inner) =
            resolve_mmc3_chr_bank(self.mmc3.bank_regs.as_slice(), self.chr_invert(), a);
        let base = bank as usize * CHR_BANK_SIZE_1K;

        self.chr.read_indexed(base, inner)
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        let a = addr & 0x1FFF;
        let (bank, inner) =
            resolve_mmc3_chr_bank(self.mmc3.bank_regs.as_slice(), self.chr_invert(), a);
        let base = bank as usize * CHR_BANK_SIZE_1K;

        self.chr.write_indexed(base, inner, data);
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

    /// Observe a PPU VRAM access and detect MMC3-qualified A12 rising edges.
    fn observe_ppu_vram_access(&mut self, addr: u16, ctx: PpuVramAccessContext) {
        self.mmc3.observe_ppu_vram_access(addr, ctx);
    }
}

impl Mapper for Mapper4 {
    fn hook_mask(&self) -> MapperHookMask {
        MapperHookMask::PPU_BUS_ADDRESS
    }

    fn on_mapper_event(&mut self, event: MapperEvent) {
        if let MapperEvent::PpuBusAddress { addr, ctx } = event {
            self.observe_ppu_vram_access(addr, ctx);
        }
    }

    fn reset(&mut self, _kind: ResetKind) {
        self.mmc3.reset(self.mmc3_reset);
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

        if let Some(reg) = Mmc3CpuRegister::from_addr(addr) {
            match self.mmc3.write_register(reg, data, MMC3_WRITE_CONFIG) {
                Mmc3WriteResult::Handled => {}
                Mmc3WriteResult::Mirroring(data) => self.write_mirroring(data),
            }
        }
    }

    fn ppu_read(&self, addr: u16) -> Option<u8> {
        Some(self.read_chr(addr))
    }

    fn ppu_write(&mut self, addr: u16, data: u8) {
        self.write_chr(addr, data);
    }

    fn irq_pending(&self) -> bool {
        self.mmc3.irq_pending
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
        4
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("MMC3")
    }
}

fn detect_mmc3_irq_revision(header: Header) -> Mmc3IrqRevision {
    if let Some(override_revision) = parse_mmc3_irq_revision_override() {
        return override_revision;
    }

    // NES 2.0 submapper carries board-level identity that can distinguish
    // MMC6-style boards. Legacy iNES mapper-4 dumps do not have enough data
    // to disambiguate reliably, so default to RevB-like behavior.
    if header.format() == RomFormat::Nes20 && header.submapper() == 1 {
        Mmc3IrqRevision::RevA
    } else {
        Mmc3IrqRevision::RevB
    }
}

fn parse_mmc3_irq_revision_override() -> Option<Mmc3IrqRevision> {
    let value = std::env::var("NESIUM_MMC3_IRQ_REV").ok()?;
    let normalized = value.trim().to_ascii_uppercase();

    match normalized.as_str() {
        "A" | "REVA" | "REV_A" | "MMC3A" | "MMC6" | "MMC6_STYLE" => Some(Mmc3IrqRevision::RevA),
        "B" | "REVB" | "REV_B" | "MMC3B" => Some(Mmc3IrqRevision::RevB),
        "AUTO" | "" => None,
        _ => None,
    }
}

fn mirroring_to_u8(m: Mirroring) -> u8 {
    match m {
        Mirroring::Horizontal => 0,
        Mirroring::Vertical => 1,
        Mirroring::FourScreen => 2,
        Mirroring::SingleScreenLower => 3,
        Mirroring::SingleScreenUpper => 4,
        Mirroring::MapperControlled => 5,
    }
}

fn mirroring_from_u8(v: u8) -> Mirroring {
    match v {
        0 => Mirroring::Horizontal,
        1 => Mirroring::Vertical,
        2 => Mirroring::FourScreen,
        3 => Mirroring::SingleScreenLower,
        4 => Mirroring::SingleScreenUpper,
        5 => Mirroring::MapperControlled,
        _ => Mirroring::Horizontal,
    }
}
