//! Mapper 228 – Active Enterprises (Action 52 / Cheetahmen II).
//!
//! Behaviour adapted from the commonly documented board logic:
//! - CPU writes anywhere in `$8000-$FFFF` latch the *address* and *data* to
//!   control banking.
//! - PRG banking (16 KiB granularity):
//!   - Extract `page = (addr >> 7) & 0x3F`; if `page & 0x30 == 0x30`, subtract
//!     `0x10`.
//!   - Base bank = `(page << 1) + (bit6 & bit5 of addr)`.
//!   - `$8000-$BFFF` maps the base bank; `$C000-$FFFF` maps the base bank plus
//!     `(~bit5 & 1)`.
//! - Mirroring toggles based on address bit 13: `A13 = 0` → horizontal,
//!   `A13 = 1` → vertical.
//! - CHR banking (8 KiB granularity):
//!   - CHR bank = `(data & 0x03) | ((addr & 0x0F) << 2)`.
//! - `$5000-$5FFF` is a tiny 4-byte open bus-friendly RAM window (masked to
//!   4 bits per write) used by some test ROMs.
//!
//! No IRQs are present on this board.
//!
//! | Area | Address range     | Behaviour                                          | IRQ/Audio |
//! |------|-------------------|----------------------------------------------------|-----------|
//! | CPU  | `$5000-$5FFF`     | 4-byte mapper RAM window (low 4 bits stored)      | None      |
//! | CPU  | `$6000-$7FFF`     | Optional PRG-RAM                                   | None      |
//! | CPU  | `$8000-$BFFF`     | 16 KiB PRG-ROM bank derived from latched address   | None      |
//! | CPU  | `$C000-$FFFF`     | 16 KiB PRG-ROM bank derived from latched address   | None      |
//! | PPU  | `$0000-$1FFF`     | 8 KiB CHR ROM/RAM bank from address/data combo     | None      |
//! | PPU  | `$2000-$3EFF`     | Mirroring from latched A13 (H/V)                   | None      |

use std::borrow::Cow;

use crate::{
    cartridge::{
        ChrRom, Mapper, PrgRom, TrainerBytes,
        header::{Header, Mirroring},
        mapper::{ChrStorage, allocate_prg_ram_with_trainer, select_chr_storage},
    },
    memory::cpu as cpu_mem,
    reset_kind::ResetKind,
};

use crate::mem_block::ByteBlock;

/// PRG-ROM banking granularity (16 KiB).
const PRG_BANK_SIZE_16K: usize = 16 * 1024;
/// CHR banking granularity (8 KiB).
const CHR_BANK_SIZE_8K: usize = 8 * 1024;

/// CPU `$5000-$5FFF`: 4-byte mapper RAM ("MRAM") window used by some test
/// ROMs. Writes are masked to 4 bits; reads return the stored nibble.
const AE_MRAM_WINDOW_START: u16 = 0x5000;
const AE_MRAM_WINDOW_END: u16 = 0x5FFF;

/// CPU `$8000-$BFFF/$C000-$FFFF`: two 16 KiB PRG windows controlled by the
/// Action 52 banking algorithm.
const AE_PRG_SLOT0_START: u16 = 0x8000;
const AE_PRG_SLOT0_END: u16 = 0xBFFF;
const AE_PRG_SLOT1_START: u16 = 0xC000;
const AE_PRG_SLOT1_END: u16 = 0xFFFF;

/// CPU `$8000-$FFFF`: bank-select write window; writes latch both the address
/// and data to control PRG/CHR/mirroring.
const AE_BANK_WRITE_WINDOW_START: u16 = 0x8000;
const AE_BANK_WRITE_WINDOW_END: u16 = 0xFFFF;

#[derive(Debug, Clone)]
pub struct Mapper228 {
    prg_rom: PrgRom,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,

    prg_bank_count_16k: usize,

    prg_bank_8000: usize,
    prg_bank_c000: usize,
    chr_bank_8k: usize,

    mram: Mapper228Mram,

    mirroring: Mirroring,
}

type Mapper228Mram = ByteBlock<4>;

impl Mapper228 {
    pub fn new(header: Header, prg_rom: PrgRom, chr_rom: ChrRom, trainer: TrainerBytes) -> Self {
        let prg_ram = allocate_prg_ram_with_trainer(&header, trainer);

        let chr = select_chr_storage(&header, chr_rom);
        let prg_bank_count_16k = (prg_rom.len() / PRG_BANK_SIZE_16K).max(1);

        Self {
            prg_rom,
            prg_ram,
            chr,
            prg_bank_count_16k,
            prg_bank_8000: 0,
            prg_bank_c000: prg_bank_count_16k.saturating_sub(1),
            chr_bank_8k: 0,
            mram: Mapper228Mram::new(),
            mirroring: header.mirroring,
        }
    }

    fn sync_from_write(&mut self, addr: u16, value: u8) {
        // PRG banking
        let mut page = ((addr >> 7) & 0x3F) as usize;
        if (page & 0x30) == 0x30 {
            page = page.saturating_sub(0x10);
        }

        let bit5 = (addr >> 5) & 1;
        let bit6 = (addr >> 6) & 1;
        let base = (page << 1) + ((bit6 & bit5) as usize);
        let high = base + ((bit5 ^ 1) as usize);

        self.prg_bank_8000 = base % self.prg_bank_count_16k;
        self.prg_bank_c000 = high % self.prg_bank_count_16k;

        // CHR banking
        let chr_bank = ((value as usize) & 0x03) | (((addr as usize) & 0x0F) << 2);
        self.chr_bank_8k = chr_bank;

        // Mirroring from A13 (inverted in the FCEUX logic).
        self.mirroring = if (addr >> 13) & 1 == 0 {
            Mirroring::Horizontal
        } else {
            Mirroring::Vertical
        };
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }
        let bank = match addr {
            AE_PRG_SLOT0_START..=AE_PRG_SLOT0_END => self.prg_bank_8000,
            AE_PRG_SLOT1_START..=AE_PRG_SLOT1_END => self.prg_bank_c000,
            _ => 0,
        };
        let offset = (addr & 0x3FFF) as usize;
        let base = bank.saturating_mul(PRG_BANK_SIZE_16K);
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
}

impl Mapper for Mapper228 {
    fn reset(&mut self, _kind: ResetKind) {
        self.mram.fill(0);
        self.sync_from_write(0x8000, 0);
    }

    fn cpu_read(&self, addr: u16) -> Option<u8> {
        match addr {
            AE_MRAM_WINDOW_START..=AE_MRAM_WINDOW_END => Some(self.mram[(addr & 0x0003) as usize]),
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.read_prg_ram(addr),
            cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END => Some(self.read_prg_rom(addr)),
            _ => None,
        }
    }

    fn cpu_write(&mut self, addr: u16, data: u8, _cpu_cycle: u64) {
        match addr {
            AE_MRAM_WINDOW_START..=AE_MRAM_WINDOW_END => {
                self.mram[(addr & 0x0003) as usize] = data & 0x0F;
            }
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.write_prg_ram(addr, data),
            AE_BANK_WRITE_WINDOW_START..=AE_BANK_WRITE_WINDOW_END => {
                self.sync_from_write(addr, data)
            }
            _ => {}
        }
    }

    fn ppu_read(&self, addr: u16) -> Option<u8> {
        if self.chr_bank_8k == 0 && matches!(self.chr, ChrStorage::None) {
            return Some(0);
        }
        let offset = (addr as usize) % CHR_BANK_SIZE_8K;
        let base = self.chr_bank_8k.saturating_mul(CHR_BANK_SIZE_8K);
        Some(self.chr.read_indexed(base, offset))
    }

    fn ppu_write(&mut self, addr: u16, data: u8) {
        let offset = (addr as usize) % CHR_BANK_SIZE_8K;
        let base = self.chr_bank_8k.saturating_mul(CHR_BANK_SIZE_8K);
        self.chr.write_indexed(base, offset, data);
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
        228
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Action 52 / Cheetahmen II")
    }
}
