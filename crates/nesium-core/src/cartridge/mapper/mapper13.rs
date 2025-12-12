//! Mapper 13 – CPROM discrete CHR-RAM banking.
//!
//! CPROM is a simple Nintendo board that exposes:
//! - 32 KiB of fixed PRG-ROM mapped at CPU `$8000-$FFFF`.
//! - 16 KiB of CHR-RAM split into two 4 KiB windows on the PPU side:
//!   - `$0000-$0FFF`: fixed to the first 4 KiB page.
//!   - `$1000-$1FFF`: 4 KiB page selected by a 2-bit register.
//! - A single write-only register at `$8000-$FFFF` that selects the
//!   switchable CHR-RAM page:
//!   - `xxxx xxCC` → `CC` chooses one of four 4 KiB CHR-RAM banks for
//!     `$1000-$1FFF`.
//!
//! PRG is not banked; the entire PRG-ROM image is mirrored into the
//! `$8000-$FFFF` window as needed. This behaviour matches both the Nesdev
//! CPROM description and Mesen2's `CpRom` mapper.
//!
//! | Area | Address range     | Behaviour                                        | IRQ/Audio |
//! |------|-------------------|--------------------------------------------------|-----------|
//! | CPU  | `$6000-$7FFF`     | Optional PRG-RAM                                 | None      |
//! | CPU  | `$8000-$FFFF`     | Fixed 32 KiB PRG-ROM (mirrored if smaller)      | None      |
//! | CPU  | `$8000-$FFFF`     | CHR-RAM bank-select register (`CC` bits)        | None      |
//! | PPU  | `$0000-$0FFF`     | Fixed 4 KiB CHR-RAM bank 0                      | None      |
//! | PPU  | `$1000-$1FFF`     | 4 KiB switchable CHR-RAM bank (0-3)             | None      |
//! | PPU  | `$2000-$3EFF`     | Mirroring from header (no mapper-side control)  | None      |

use std::borrow::Cow;

use crate::{
    cartridge::{
        ChrRom, Mapper, PrgRom, TrainerBytes,
        header::{Header, Mirroring},
        mapper::{ChrStorage, allocate_prg_ram_with_trainer},
    },
    memory::cpu as cpu_mem,
    reset_kind::ResetKind,
};

/// Fixed PRG window size (32 KiB).
const PRG_WINDOW_SIZE_32K: usize = 32 * 1024;
/// Size of a single 4 KiB CHR-RAM bank.
const CHR_BANK_SIZE_4K: usize = 4 * 1024;
/// Total CHR-RAM size used by CPROM (4 banks × 4 KiB).
const CHR_RAM_SIZE: usize = 4 * CHR_BANK_SIZE_4K;

/// CPU `$8000-$FFFF`: CPROM CHR-RAM bank-select register. Writes in this range
/// update the 4 KiB bank mapped at PPU `$1000-$1FFF`.
const CPROM_BANK_SELECT_START: u16 = 0x8000;
const CPROM_BANK_SELECT_END: u16 = 0xFFFF;

#[derive(Debug, Clone)]
pub struct Mapper13 {
    prg_rom: PrgRom,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,

    /// Currently selected 4 KiB CHR-RAM bank for `$1000-$1FFF`.
    chr_bank: u8,

    /// CPROM boards are typically wired for vertical mirroring; however, we
    /// respect the mirroring mode encoded in the iNES header so that custom
    /// dumps can override it when needed.
    mirroring: Mirroring,
}

impl Mapper13 {
    pub fn new(header: Header, prg_rom: PrgRom, _chr_rom: ChrRom, trainer: TrainerBytes) -> Self {
        let prg_ram = allocate_prg_ram_with_trainer(&header, trainer);

        // Always allocate 16 KiB of CHR-RAM as per the CPROM spec.
        let chr_ram = vec![0u8; CHR_RAM_SIZE].into_boxed_slice();
        let chr = ChrStorage::Ram(chr_ram);

        Self {
            prg_rom,
            prg_ram,
            chr,
            chr_bank: 0,
            mirroring: header.mirroring,
        }
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }

        // 32 KiB window at $8000-$FFFF with mirroring when the PRG-ROM is
        // smaller than 32 KiB.
        let offset = (addr as usize).saturating_sub(cpu_mem::PRG_ROM_START as usize);
        if self.prg_rom.len() <= PRG_WINDOW_SIZE_32K {
            // Mirror the whole PRG-ROM across the 32 KiB window.
            let len = self.prg_rom.len();
            let idx = offset % len;
            self.prg_rom[idx]
        } else {
            // Use the first 32 KiB window of PRG-ROM; games that rely on
            // larger PRG sizes with CPROM are extremely rare.
            let idx = offset.min(self.prg_rom.len() - 1);
            self.prg_rom[idx]
        }
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

    fn read_chr(&self, addr: u16) -> u8 {
        let a = addr & 0x1FFF;
        let offset = (a & 0x0FFF) as usize;

        if a < 0x1000 {
            // Lower 4 KiB is always bank 0.
            let base = 0;
            self.chr.read_indexed(base, offset)
        } else {
            // Upper 4 KiB maps to the selected bank.
            let bank = (self.chr_bank & 0x03) as usize;
            let base = bank.saturating_mul(CHR_BANK_SIZE_4K);
            self.chr.read_indexed(base, offset)
        }
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        let a = addr & 0x1FFF;
        let offset = (a & 0x0FFF) as usize;

        if a < 0x1000 {
            let base = 0;
            self.chr.write_indexed(base, offset, data);
        } else {
            let bank = (self.chr_bank & 0x03) as usize;
            let base = bank.saturating_mul(CHR_BANK_SIZE_4K);
            self.chr.write_indexed(base, offset, data);
        }
    }

    fn write_chr_bank_select(&mut self, data: u8) {
        // Only the low two bits are used for the CHR-RAM bank index.
        self.chr_bank = data & 0x03;
    }
}

impl Mapper for Mapper13 {
    fn reset(&mut self, _kind: ResetKind) {
        self.chr_bank = 0;
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
            CPROM_BANK_SELECT_START..=CPROM_BANK_SELECT_END => self.write_chr_bank_select(data),
            _ => {}
        }
    }

    fn ppu_read(&self, addr: u16) -> Option<u8> {
        Some(self.read_chr(addr))
    }

    fn ppu_write(&mut self, addr: u16, data: u8) {
        self.write_chr(addr, data);
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
        13
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("CPROM")
    }
}
