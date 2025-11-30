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

use std::borrow::Cow;

use crate::{
    cartridge::{
        Mapper, TRAINER_SIZE,
        header::{Header, Mirroring},
        mapper::{
            ChrStorage, PpuVramAccessContext, PpuVramAccessKind, allocate_prg_ram,
            select_chr_storage, trainer_destination,
        },
    },
    memory::cpu as cpu_mem,
};

/// PRG-ROM banking granularity (16 KiB).
const PRG_BANK_SIZE_16K: usize = 16 * 1024;
/// CHR banking granularity (4 KiB).
const CHR_BANK_SIZE_4K: usize = 4 * 1024;

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
    prg_rom: Box<[u8]>,
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
    pub fn new(header: Header, prg_rom: Box<[u8]>, chr_rom: Box<[u8]>) -> Self {
        Self::with_trainer(header, prg_rom, chr_rom, None)
    }

    pub(crate) fn with_trainer(
        header: Header,
        prg_rom: Box<[u8]>,
        chr_rom: Box<[u8]>,
        trainer: Option<Box<[u8; TRAINER_SIZE]>>,
    ) -> Self {
        let mut prg_ram = allocate_prg_ram(&header);
        if let (Some(trainer), Some(dst)) = (trainer.as_ref(), trainer_destination(&mut prg_ram)) {
            dst.copy_from_slice(trainer.as_ref());
        }

        let chr = select_chr_storage(&header, chr_rom);
        let prg_bank_count_16k = (prg_rom.len() / PRG_BANK_SIZE_16K).max(1);

        Self {
            prg_rom,
            prg_ram,
            chr,
            prg_bank_count_16k,
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
            0x0FD8..=0x0FDF => {
                self.latch0 = ChrLatch::Fd;
            }
            0x0FE8..=0x0FEF => {
                self.latch0 = ChrLatch::Fe;
            }
            0x1FD8..=0x1FDF => {
                self.latch1 = ChrLatch::Fd;
            }
            0x1FE8..=0x1FEF => {
                self.latch1 = ChrLatch::Fe;
            }
            _ => {}
        }
    }
}

impl Mapper for Mapper10 {
    fn power_on(&mut self) {
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

    fn reset(&mut self) {
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
        match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.write_prg_ram(addr, data),
            0xA000..=0xAFFF => self.write_prg_bank(data),
            0xB000..=0xBFFF => self.write_chr_fd_0000(data),
            0xC000..=0xCFFF => self.write_chr_fe_0000(data),
            0xD000..=0xDFFF => self.write_chr_fd_1000(data),
            0xE000..=0xEFFF => self.write_chr_fe_1000(data),
            0xF000..=0xFFFF => self.write_mirroring(data),
            _ => {}
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
