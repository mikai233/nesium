//! Mapper 10 (MMC4) implementation.
//!
//! MMC4 is a close relative of MMC2 and is used by games such as Fire Emblem.
//! It provides:
//! - One 16 KiB switchable PRG-ROM bank at `$8000-$BFFF`.
//! - One 16 KiB fixed PRG-ROM bank at `$C000-$FFFF` (last bank).
//! - Two 4 KiB CHR windows (`$0000-$0FFF` and `$1000-$1FFF`) controlled by
//!   FD/FE latches, just like MMC2.
//! - Mapper-controlled nametable mirroring via `$F000-$FFFF`.
//!
//! The key difference from MMC2 is that both CHR windows use tile *ranges*
//! to trigger the FD/FE latches (see Nesdev MMC4 docs). We approximate the
//! "latch updates after fetch" behaviour by updating latch state in
//! [`ppu_vram_access`] whenever the PPU performs a rendering fetch from the
//! documented trigger addresses.
//!
//! | Area | Address range     | Behaviour                                          | IRQ/Audio |
//! |------|-------------------|----------------------------------------------------|-----------|
//! | CPU  | `$6000-$7FFF`     | Optional PRG-RAM                                   | None      |
//! | CPU  | `$8000-$BFFF`     | Switchable 16 KiB PRG-ROM bank (`$A000` register)  | None      |
//! | CPU  | `$C000-$FFFF`     | Fixed 16 KiB PRG-ROM bank (last)                   | None      |
//! | CPU  | `$A000-$EFFF`     | CHR FD/FE bank registers (`$B000-$E000` ranges)    | None      |
//! | CPU  | `$F000-$FFFF`     | Nametable mirroring control                        | None      |
//! | PPU  | `$0000-$1FFF`     | Two 4 KiB CHR windows with FD/FE latch switching   | None      |
//! | PPU  | `$0FD8-$0FEF/...` | Tile ranges that update MMC4 CHR latches           | None      |

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
    reset_kind::ResetKind,
};

/// PRG-ROM banking granularity (16 KiB).
const PRG_BANK_SIZE_16K: usize = 16 * 1024;
/// CHR banking granularity (4 KiB).
const CHR_BANK_SIZE_4K: usize = 4 * 1024;

/// CPU `$A000-$AFFF`: 16 KiB PRG-ROM bank select register for `$8000-$BFFF`.
const MMC4_PRG_BANK_REG_START: u16 = 0xA000;
const MMC4_PRG_BANK_REG_END: u16 = 0xAFFF;

/// CPU `$B000-$BFFF`: CHR bank for `$0000-$0FFF` when latch 0 is in the `$FD` state.
const MMC4_CHR_FD_0000_REG_START: u16 = 0xB000;
const MMC4_CHR_FD_0000_REG_END: u16 = 0xBFFF;
/// CPU `$C000-$CFFF`: CHR bank for `$0000-$0FFF` when latch 0 is in the `$FE` state.
const MMC4_CHR_FE_0000_REG_START: u16 = 0xC000;
const MMC4_CHR_FE_0000_REG_END: u16 = 0xCFFF;
/// CPU `$D000-$DFFF`: CHR bank for `$1000-$1FFF` when latch 1 is in the `$FD` state.
const MMC4_CHR_FD_1000_REG_START: u16 = 0xD000;
const MMC4_CHR_FD_1000_REG_END: u16 = 0xDFFF;
/// CPU `$E000-$EFFF`: CHR bank for `$1000-$1FFF` when latch 1 is in the `$FE` state.
const MMC4_CHR_FE_1000_REG_START: u16 = 0xE000;
const MMC4_CHR_FE_1000_REG_END: u16 = 0xEFFF;

/// CPU `$F000-$FFFF`: nametable mirroring control register.
const MMC4_MIRRORING_REG_START: u16 = 0xF000;
const MMC4_MIRRORING_REG_END: u16 = 0xFFFF;

/// PPU `$0FD8-$0FDF`: left pattern table (`$0000-$0FFF`) tile `$FD` range; sets latch 0 to `$FD`.
const MMC4_LATCH0_FD_TRIGGER_START: u16 = 0x0FD8;
const MMC4_LATCH0_FD_TRIGGER_END: u16 = 0x0FDF;
/// PPU `$0FE8-$0FEF`: left pattern table tile `$FE` range; sets latch 0 to `$FE`.
const MMC4_LATCH0_FE_TRIGGER_START: u16 = 0x0FE8;
const MMC4_LATCH0_FE_TRIGGER_END: u16 = 0x0FEF;
/// PPU `$1FD8-$1FDF`: right pattern table (`$1000-$1FFF`) tile `$FD` range; sets latch 1 to `$FD`.
const MMC4_LATCH1_FD_TRIGGER_START: u16 = 0x1FD8;
const MMC4_LATCH1_FD_TRIGGER_END: u16 = 0x1FDF;
/// PPU `$1FE8-$1FEF`: right pattern table tile `$FE` range; sets latch 1 to `$FE`.
const MMC4_LATCH1_FE_TRIGGER_START: u16 = 0x1FE8;
const MMC4_LATCH1_FE_TRIGGER_END: u16 = 0x1FEF;

/// CPU-visible MMC4 register windows.
///
/// MMC4 exposes a small set of write-only registers in `$A000-$FFFF`. Grouping
/// them into an enum keeps the CPU side logic aligned with how CPU/PPU
/// registers are modelled elsewhere in the core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Mmc4CpuRegister {
    /// `$A000-$AFFF` – PRG bank select for `$8000-$BFFF`.
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

impl Mmc4CpuRegister {
    fn from_addr(addr: u16) -> Option<Self> {
        use Mmc4CpuRegister::*;

        match addr {
            MMC4_PRG_BANK_REG_START..=MMC4_PRG_BANK_REG_END => Some(PrgBank),
            MMC4_CHR_FD_0000_REG_START..=MMC4_CHR_FD_0000_REG_END => Some(ChrFd0000),
            MMC4_CHR_FE_0000_REG_START..=MMC4_CHR_FE_0000_REG_END => Some(ChrFe0000),
            MMC4_CHR_FD_1000_REG_START..=MMC4_CHR_FD_1000_REG_END => Some(ChrFd1000),
            MMC4_CHR_FE_1000_REG_START..=MMC4_CHR_FE_1000_REG_END => Some(ChrFe1000),
            MMC4_MIRRORING_REG_START..=MMC4_MIRRORING_REG_END => Some(Mirroring),
            _ => None,
        }
    }
}

/// Internal representation of the CHR latch state.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum ChrLatch {
    Fd,
    Fe,
}

impl ChrLatch {
    fn power_on_latch0() -> Self {
        // Power-on default: use `$FD/0000` bank until the game hits a
        // switching tile. Matches the common behaviour used by Mesen2.
        ChrLatch::Fd
    }

    fn power_on_latch1() -> Self {
        // Power-on default: use `$FE/1000` bank for the right pattern table.
        ChrLatch::Fe
    }
}

#[derive(Debug, Clone)]
pub struct Mapper10 {
    prg_rom: PrgRom,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,

    /// Number of 16 KiB PRG-ROM banks.
    prg_bank_count_16k: usize,

    /// Base mirroring mode from the header. Some MMC4 boards use fixed
    /// four-screen VRAM; in that case we ignore `$F000` writes and always
    /// report the header mirroring.
    base_mirroring: Mirroring,
    /// Current effective nametable mirroring.
    mirroring: Mirroring,

    /// 16 KiB PRG-ROM bank for `$8000-$BFFF` (`$A000` writes).
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

impl Mapper10 {
    pub fn new(header: Header, prg_rom: PrgRom, chr_rom: ChrRom, trainer: TrainerBytes) -> Self {
        let prg_ram = allocate_prg_ram_with_trainer(&header, trainer);

        let chr = select_chr_storage(&header, chr_rom);
        let prg_bank_count_16k = (prg_rom.len() / PRG_BANK_SIZE_16K).max(1);

        Self {
            prg_rom,
            prg_ram,
            chr,
            prg_bank_count_16k,
            base_mirroring: header.mirroring(),
            mirroring: header.mirroring(),
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

    #[inline]
    fn prg_bank_index(&self, bank: u8) -> usize {
        if self.prg_bank_count_16k == 0 {
            0
        } else {
            (bank as usize) % self.prg_bank_count_16k
        }
    }

    /// Index of the last 16 KiB PRG bank.
    fn last_prg_bank_index(&self) -> usize {
        if self.prg_bank_count_16k == 0 {
            0
        } else {
            self.prg_bank_count_16k - 1
        }
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }

        // Nesdev: CPU mapping
        // - $8000-$BFFF: 16 KiB bank selected by $A000.
        // - $C000-$FFFF: last 16 KiB bank.
        let bank = match addr {
            0x8000..=0xBFFF => self.prg_bank_index(self.prg_bank),
            0xC000..=0xFFFF => self.last_prg_bank_index(),
            _ => return 0,
        };

        let base = bank.saturating_mul(PRG_BANK_SIZE_16K);
        let offset = (addr as usize - cpu_mem::PRG_ROM_START as usize) & (PRG_BANK_SIZE_16K - 1);
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
            let bank = match self.latch0 {
                ChrLatch::Fd => self.chr_fd_0000,
                ChrLatch::Fe => self.chr_fe_0000,
            } as usize;
            (bank * CHR_BANK_SIZE_4K, offset)
        } else {
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

    fn write_prg_bank(&mut self, data: u8) {
        // Nesdev: only low 4 bits are used (`PPPP`); we keep the full
        // byte for completeness but mask when mapping.
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
        if self.base_mirroring == Mirroring::FourScreen {
            // Boards with four-screen VRAM ignore $F000 mirroring writes.
            return;
        }
        self.mirroring = if data & 0x01 == 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };
    }

    /// Update latch state in response to a PPU rendering fetch from the
    /// documented trigger addresses. For MMC4, *both* pattern tables use
    /// address ranges:
    ///
    /// - $0FD8-$0FDF: latch0 := $FD
    /// - $0FE8-$0FEF: latch0 := $FE
    /// - $1FD8-$1FDF: latch1 := $FD
    /// - $1FE8-$1FEF: latch1 := $FE
    fn update_latches_on_vram_access(&mut self, addr: u16, ctx: PpuVramAccessContext) {
        if ctx.kind != PpuVramAccessKind::RenderingFetch {
            return;
        }
        if addr >= 0x2000 {
            // Only pattern table fetches ($0000-$1FFF) affect the latches.
            return;
        }

        let a = addr & 0x1FFF;
        match a {
            MMC4_LATCH0_FD_TRIGGER_START..=MMC4_LATCH0_FD_TRIGGER_END => {
                self.latch0 = ChrLatch::Fd;
            }
            MMC4_LATCH0_FE_TRIGGER_START..=MMC4_LATCH0_FE_TRIGGER_END => {
                self.latch0 = ChrLatch::Fe;
            }
            MMC4_LATCH1_FD_TRIGGER_START..=MMC4_LATCH1_FD_TRIGGER_END => {
                self.latch1 = ChrLatch::Fd;
            }
            MMC4_LATCH1_FE_TRIGGER_START..=MMC4_LATCH1_FE_TRIGGER_END => {
                self.latch1 = ChrLatch::Fe;
            }
            _ => {}
        }
    }
}

impl Mapper for Mapper10 {
    fn reset(&mut self, _kind: ResetKind) {
        // Power-on defaults:
        // - PRG bank at $8000 defaults to 0.
        // - CHR FD/FE banks default to 0.
        // - Latches initialised to FD/FE as described above.
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

        if let Some(reg) = Mmc4CpuRegister::from_addr(addr) {
            match reg {
                Mmc4CpuRegister::PrgBank => self.write_prg_bank(data),
                Mmc4CpuRegister::ChrFd0000 => self.write_chr_fd_0000(data),
                Mmc4CpuRegister::ChrFe0000 => self.write_chr_fe_0000(data),
                Mmc4CpuRegister::ChrFd1000 => self.write_chr_fd_1000(data),
                Mmc4CpuRegister::ChrFe1000 => self.write_chr_fe_1000(data),
                Mmc4CpuRegister::Mirroring => self.write_mirroring(data),
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
        self.update_latches_on_vram_access(addr, ctx);
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
        10
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("MMC4")
    }
}
