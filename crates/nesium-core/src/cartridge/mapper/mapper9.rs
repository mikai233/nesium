//! Mapper 9 (MMC2) implementation.
//!
//! MMC2 is used by games like Punch-Out!! and provides:
//! - One 8 KiB switchable PRG-ROM bank at `$8000-$9FFF`.
//! - Three 8 KiB PRG-ROM banks fixed to the last three banks at `$A000-$FFFF`.
//! - Two 4 KiB CHR windows (`$0000-$0FFF` and `$1000-$1FFF`), each with two
//!   banks that are selected at run time by CHR "latches".
//! - Mapper-controlled nametable mirroring via writes in the `$F000-$FFFF`
//!   range.
//!
//! The CHR latches are the distinctive feature: when the PPU reads specific
//! pattern table addresses (tiles `$FD` or `$FE`), the MMC2 remembers which
//! tile was seen and uses that to pick one of two 4 KiB CHR banks for the
//! corresponding 4 KiB region. This lets games double the effective CHR tile
//! set during rendering without involving the CPU.
//!
//! Behaviour here is based on the Nesdev MMC2 documentation and mirrors the
//! overall power-on/reset behaviour used by modern emulators like Mesen2.
//!
//! | Area | Address range     | Behaviour                                          | IRQ/Audio |
//! |------|-------------------|----------------------------------------------------|-----------|
//! | CPU  | `$6000-$7FFF`     | Optional PRG-RAM                                   | None      |
//! | CPU  | `$8000-$9FFF`     | Switchable 8 KiB PRG-ROM bank (`$A000` register)   | None      |
//! | CPU  | `$A000-$FFFF`     | Fixed PRG-ROM banks (last three 8 KiB banks)       | None      |
//! | CPU  | `$A000-$EFFF`     | PRG/CHR bank registers and mirroring (`$F000`)     | None      |
//! | PPU  | `$0000-$1FFF`     | Two 4 KiB CHR windows with FD/FE latch switching   | None      |
//! | PPU  | `$0FD8/$0FE8/...` | Tile fetches that update MMC2 CHR latches          | None      |

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

/// PRG-ROM banking granularity (8 KiB).
const PRG_BANK_SIZE_8K: usize = 8 * 1024;
/// CHR banking granularity (4 KiB).
const CHR_BANK_SIZE_4K: usize = 4 * 1024;

/// CPU `$A000-$AFFF`: 8 KiB PRG-ROM bank select register for `$8000-$9FFF`.
const MMC2_PRG_BANK_REG_START: u16 = 0xA000;
const MMC2_PRG_BANK_REG_END: u16 = 0xAFFF;

/// CPU `$B000-$BFFF`: CHR bank for `$0000-$0FFF` when latch 0 is in the `$FD` state.
const MMC2_CHR_FD_0000_REG_START: u16 = 0xB000;
const MMC2_CHR_FD_0000_REG_END: u16 = 0xBFFF;
/// CPU `$C000-$CFFF`: CHR bank for `$0000-$0FFF` when latch 0 is in the `$FE` state.
const MMC2_CHR_FE_0000_REG_START: u16 = 0xC000;
const MMC2_CHR_FE_0000_REG_END: u16 = 0xCFFF;
/// CPU `$D000-$DFFF`: CHR bank for `$1000-$1FFF` when latch 1 is in the `$FD` state.
const MMC2_CHR_FD_1000_REG_START: u16 = 0xD000;
const MMC2_CHR_FD_1000_REG_END: u16 = 0xDFFF;
/// CPU `$E000-$EFFF`: CHR bank for `$1000-$1FFF` when latch 1 is in the `$FE` state.
const MMC2_CHR_FE_1000_REG_START: u16 = 0xE000;
const MMC2_CHR_FE_1000_REG_END: u16 = 0xEFFF;

/// CPU `$F000-$FFFF`: nametable mirroring control register.
const MMC2_MIRRORING_REG_START: u16 = 0xF000;
const MMC2_MIRRORING_REG_END: u16 = 0xFFFF;

/// CPU-visible MMC2 register windows.
///
/// MMC2 exposes a small set of write-only registers used for PRG banking,
/// CHR FD/FE bank selection, and nametable mirroring. Grouping them into an
/// enum keeps the mapper logic close to how CPU/PPU registers are modelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Mmc2CpuRegister {
    /// `$A000-$AFFF` – PRG bank select for `$8000-$9FFF`.
    PrgBank,
    /// `$B000-$BFFF` – CHR bank for `$0000-$0FFF` when latch 0 is `$FD`.
    ChrFd0000,
    /// `$C000-$CFFF` – CHR bank for `$0000-$0FFF` when latch 0 is `$FE`.
    ChrFe0000,
    /// `$D000-$DFFF` – CHR bank for `$1000-$1FFF` when latch 1 is `$FD`.
    ChrFd1000,
    /// `$E000-$EFFF` – CHR bank for `$1000-$1FFF` when latch 1 is `$FE`.
    ChrFe1000,
    /// `$F000-$FFFF` – nametable mirroring control.
    Mirroring,
}

impl Mmc2CpuRegister {
    fn from_addr(addr: u16) -> Option<Self> {
        use Mmc2CpuRegister::*;

        match addr {
            MMC2_PRG_BANK_REG_START..=MMC2_PRG_BANK_REG_END => Some(PrgBank),
            MMC2_CHR_FD_0000_REG_START..=MMC2_CHR_FD_0000_REG_END => Some(ChrFd0000),
            MMC2_CHR_FE_0000_REG_START..=MMC2_CHR_FE_0000_REG_END => Some(ChrFe0000),
            MMC2_CHR_FD_1000_REG_START..=MMC2_CHR_FD_1000_REG_END => Some(ChrFd1000),
            MMC2_CHR_FE_1000_REG_START..=MMC2_CHR_FE_1000_REG_END => Some(ChrFe1000),
            MMC2_MIRRORING_REG_START..=MMC2_MIRRORING_REG_END => Some(Mirroring),
            _ => None,
        }
    }
}

/// PPU `$0FD8`: left pattern table (`$0000-$0FFF`) tile `$FD`; sets latch 0 to `$FD`.
const MMC2_LATCH0_FD_ADDR: u16 = 0x0FD8;
/// PPU `$0FE8`: left pattern table tile `$FE`; sets latch 0 to `$FE`.
const MMC2_LATCH0_FE_ADDR: u16 = 0x0FE8;
/// PPU `$1FD8-$1FDF`: right pattern table (`$1000-$1FFF`) tile `$FD` range; sets latch 1 to `$FD`.
const MMC2_LATCH1_FD_TRIGGER_START: u16 = 0x1FD8;
const MMC2_LATCH1_FD_TRIGGER_END: u16 = 0x1FDF;
/// PPU `$1FE8-$1FEF`: right pattern table tile `$FE` range; sets latch 1 to `$FE`.
const MMC2_LATCH1_FE_TRIGGER_START: u16 = 0x1FE8;
const MMC2_LATCH1_FE_TRIGGER_END: u16 = 0x1FEF;

/// Internal representation of the CHR latch state.
///
/// Nesdev describes the latches as storing `$FD` or `$FE`, chosen by pattern
/// table reads from specific addresses. We only need to distinguish the two
/// states, so a small enum keeps the code clearer than raw bytes.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ChrLatch {
    /// Latch is in the `$FD` state.
    Fd,
    /// Latch is in the `$FE` state.
    Fe,
}

impl ChrLatch {
    fn power_on_latch0() -> Self {
        // Most documentation and emulator implementations initialise latch 0
        // to the `$FD` state so that the first sprite fetches use the
        // `$FD/0000` bank until the game explicitly hits a switching tile.
        ChrLatch::Fd
    }

    fn power_on_latch1() -> Self {
        // Latch 1 is typically initialised to `$FE` so that background
        // fetches start from the `$FE/1000` bank. This matches Mesen2 and is
        // compatible with known commercial games.
        ChrLatch::Fe
    }
}

#[derive(Debug, Clone)]
pub struct Mapper9 {
    prg_rom: PrgRom,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,

    /// Number of 8 KiB PRG-ROM banks.
    prg_bank_count: usize,

    /// Base mirroring mode from the header. Some MMC2 boards use fixed
    /// four-screen VRAM; in that case we ignore writes to the mirroring
    /// register and always report the header mirroring.
    base_mirroring: Mirroring,
    /// Current effective mirroring, controlled via `$F000-$FFFF`.
    mirroring: Mirroring,

    /// 8 KiB PRG-ROM bank number for CPU `$8000-$9FFF` (`$A000` writes).
    prg_bank: u8,

    /// CHR bank numbers for the two 4 KiB windows when latch 0/1 are in the
    /// `$FD`/`$FE` states. Each bank number selects a 4 KiB page.
    chr_fd_0000: u8, // `$B000` - left pattern table, latch0 = $FD
    chr_fe_0000: u8, // `$C000` - left pattern table, latch0 = $FE
    chr_fd_1000: u8, // `$D000` - right pattern table, latch1 = $FD
    chr_fe_1000: u8, // `$E000` - right pattern table, latch1 = $FE

    /// Current latch states controlling which CHR bank is visible in each
    /// 4 KiB region.
    latch0: ChrLatch,
    latch1: ChrLatch,
}

impl Mapper9 {
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
            prg_bank: 0,
            chr_fd_0000: 0,
            chr_fe_0000: 0,
            chr_fd_1000: 0,
            chr_fe_1000: 0,
            latch0: ChrLatch::power_on_latch0(),
            latch1: ChrLatch::power_on_latch1(),
        }
    }

    #[inline]
    fn prg_ram_present(&self) -> bool {
        !self.prg_ram.is_empty()
    }

    fn read_prg_ram(&self, addr: u16) -> Option<u8> {
        if !self.prg_ram_present() {
            return None;
        }
        let idx = (addr - cpu_mem::PRG_RAM_START) as usize % self.prg_ram.len();
        Some(self.prg_ram[idx])
    }

    fn write_prg_ram(&mut self, addr: u16, data: u8) {
        if !self.prg_ram_present() {
            return;
        }
        let idx = (addr - cpu_mem::PRG_RAM_START) as usize % self.prg_ram.len();
        self.prg_ram[idx] = data;
    }

    /// Resolve an 8 KiB PRG-ROM bank index, wrapping safely when the ROM is
    /// smaller than expected.
    #[inline]
    fn prg_bank_index(&self, bank: u8) -> usize {
        if self.prg_bank_count == 0 {
            0
        } else {
            (bank as usize) % self.prg_bank_count
        }
    }

    /// Return the index of the Nth bank from the end (1 = last, 2 = second
    /// last, etc.), saturating gracefully for very small ROM sizes.
    fn prg_bank_from_end(&self, n: usize) -> usize {
        if self.prg_bank_count == 0 {
            0
        } else { self.prg_bank_count.saturating_sub(n) }
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }

        // Nesdev: CPU mapping
        // - $8000-$9FFF: 8 KB bank selected by $A000.
        // - $A000-$FFFF: 3× 8 KB banks fixed to the last three PRG banks.
        let bank = match addr {
            0x8000..=0x9FFF => self.prg_bank_index(self.prg_bank),
            0xA000..=0xBFFF => self.prg_bank_from_end(3),
            0xC000..=0xDFFF => self.prg_bank_from_end(2),
            0xE000..=0xFFFF => self.prg_bank_from_end(1),
            _ => return 0,
        };

        let base = bank.saturating_mul(PRG_BANK_SIZE_8K);
        let offset = (addr as usize - cpu_mem::PRG_ROM_START as usize) & (PRG_BANK_SIZE_8K - 1);
        let idx = base.saturating_add(offset);
        self.prg_rom.get(idx).copied().unwrap_or(0)
    }

    /// Resolve the active 4 KiB CHR bank and offset for a given PPU address.
    ///
    /// The PPU CHR space is split into two 4 KiB windows:
    /// - `$0000-$0FFF`: controlled by latch 0 and the `$B000/$C000` registers.
    /// - `$1000-$1FFF`: controlled by latch 1 and the `$D000/$E000` registers.
    fn chr_window_for_addr(&self, addr: u16) -> (usize, usize) {
        let a = addr & 0x1FFF;
        let offset = (a & 0x0FFF) as usize;

        if a < 0x1000 {
            // Left pattern table: choose between the FD/FE banks based on latch 0.
            let bank = match self.latch0 {
                ChrLatch::Fd => self.chr_fd_0000,
                ChrLatch::Fe => self.chr_fe_0000,
            } as usize;
            (bank * CHR_BANK_SIZE_4K, offset)
        } else {
            // Right pattern table: choose between the FD/FE banks based on latch 1.
            let bank = match self.latch1 {
                ChrLatch::Fd => self.chr_fd_1000,
                ChrLatch::Fe => self.chr_fe_1000,
            } as usize;
            (bank * CHR_BANK_SIZE_4K, offset)
        }
    }

    fn read_chr(&self, addr: u16) -> u8 {
        let (base, offset) = self.chr_window_for_addr(addr);
        self.chr.read_indexed(base, offset)
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        let (base, offset) = self.chr_window_for_addr(addr);
        self.chr.write_indexed(base, offset, data);
    }

    /// Update latch 0/1 after a CHR read, following the rules from Nesdev:
    ///
    /// - PPU reads $0FD8: latch 0 := $FD
    /// - PPU reads $0FE8: latch 0 := $FE
    /// - PPU reads $1FD8-$1FDF: latch 1 := $FD
    /// - PPU reads $1FE8-$1FEF: latch 1 := $FE
    ///
    /// The latch is updated *after* the pattern data is fetched, so the
    /// switching tile itself is drawn using the old bank. We respect this by
    /// calling `update_latches_after_read` only after fetching CHR data.
    fn update_latches_after_read(&mut self, addr: u16) {
        let a = addr & 0x1FFF;
        match a {
            MMC2_LATCH0_FD_ADDR => {
                self.latch0 = ChrLatch::Fd;
            }
            MMC2_LATCH0_FE_ADDR => {
                self.latch0 = ChrLatch::Fe;
            }
            MMC2_LATCH1_FD_TRIGGER_START..=MMC2_LATCH1_FD_TRIGGER_END => {
                self.latch1 = ChrLatch::Fd;
            }
            MMC2_LATCH1_FE_TRIGGER_START..=MMC2_LATCH1_FE_TRIGGER_END => {
                self.latch1 = ChrLatch::Fe;
            }
            _ => {}
        }
    }

    fn write_prg_bank(&mut self, data: u8) {
        // Nesdev: only the low 4 bits are used (`PPPP`). We keep the full
        // byte for completeness but mask when converting to an index.
        self.prg_bank = data & 0x0F;
    }

    fn write_chr_fd_0000(&mut self, data: u8) {
        self.chr_fd_0000 = data & 0x1F;
    }

    fn write_chr_fe_0000(&mut self, data: u8) {
        self.chr_fe_0000 = data & 0x1F;
    }

    fn write_chr_fd_1000(&mut self, data: u8) {
        self.chr_fd_1000 = data & 0x1F;
    }

    fn write_chr_fe_1000(&mut self, data: u8) {
        self.chr_fe_1000 = data & 0x1F;
    }

    fn write_mirroring(&mut self, data: u8) {
        // Some MMC2 boards use fixed four-screen mirroring; in that case we
        // ignore the register and always report the header mirroring.
        if self.base_mirroring == Mirroring::FourScreen {
            return;
        }
        self.mirroring = if data & 0x01 == 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };
    }
}

impl Mapper for Mapper9 {
    fn power_on(&mut self) {
        // Reset state roughly matches the typical behaviour described on
        // Nesdev and implemented by Mesen2:
        // - PRG bank at $8000 defaults to 0.
        // - CHR FD/FE banks default to 0.
        // - Latch 0 starts in the $FD state; latch 1 starts in the $FE state.
        // - Mirroring follows the header until the game writes to $F000-$FFFF.
        self.prg_bank = 0;
        self.chr_fd_0000 = 0;
        self.chr_fe_0000 = 0;
        self.chr_fd_1000 = 0;
        self.chr_fe_1000 = 0;
        self.latch0 = ChrLatch::power_on_latch0();
        self.latch1 = ChrLatch::power_on_latch1();
        self.mirroring = self.base_mirroring;
    }

    fn reset(&mut self) {
        // Treat console reset like a fresh power-on for this mapper; commercial
        // games reinitialise MMC2 state after reset.
        self.power_on();
    }

    fn cpu_read(&self, addr: u16) -> Option<u8> {
        if (cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END).contains(&addr) {
            return self.read_prg_ram(addr);
        }
        if (cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END).contains(&addr) {
            return Some(self.read_prg_rom(addr));
        }
        None
    }

    fn cpu_write(&mut self, addr: u16, data: u8, _cpu_cycle: u64) {
        if (cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END).contains(&addr) {
            self.write_prg_ram(addr, data);
            return;
        }

        if let Some(reg) = Mmc2CpuRegister::from_addr(addr) {
            match reg {
                Mmc2CpuRegister::PrgBank => self.write_prg_bank(data),
                Mmc2CpuRegister::ChrFd0000 => self.write_chr_fd_0000(data),
                Mmc2CpuRegister::ChrFe0000 => self.write_chr_fe_0000(data),
                Mmc2CpuRegister::ChrFd1000 => self.write_chr_fd_1000(data),
                Mmc2CpuRegister::ChrFe1000 => self.write_chr_fe_1000(data),
                Mmc2CpuRegister::Mirroring => self.write_mirroring(data),
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
        // Update MMC2 latches when the PPU performs a rendering fetch from
        // the documented trigger addresses. This approximates the hardware
        // behaviour where the latch flips just after fetching the tile.
        if addr < 0x2000 && ctx.kind == PpuVramAccessKind::RenderingFetch {
            self.update_latches_after_read(addr);
        }
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
        9
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("MMC2")
    }
}
