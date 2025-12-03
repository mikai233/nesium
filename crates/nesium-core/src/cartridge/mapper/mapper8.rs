//! Mapper 8 – Front Fareast Magic Card (GUI mode).
//!
//! iNES mapper 8 is used for dumps converted from Front Fareast / Super
//! Magic Card "GUI mode" disks (FFE F3xxx family). Mesen2 models these
//! boards via the `FrontFareast` mapper with `_romInfo.MapperID == 8`.
//!
//! This implementation closely follows that behaviour:
//! - CPU `$6000-$7FFF`: optional PRG-RAM (battery-backed or work RAM).
//! - CPU `$8000-$BFFF`: 16 KiB switchable PRG-ROM window (two 8 KiB banks).
//! - CPU `$C000-$FFFF`: 16 KiB fixed PRG-ROM window mapped to the last
//!   16 KiB of the ROM image.
//! - PPU `$0000-$1FFF`: 8 KiB CHR-RAM window inside a 32 KiB CHR-RAM space.
//! - CPU `$42FE/$42FF`: mirroring control (one-screen A/B and H/V).
//! - CPU `$4501/$4502/$4503`: 16‑bit IRQ counter incremented by CPU activity.
//! - CPU `$8000-$FFFF`: combined PRG/CHR bank select (mirroring Mesen2).
//!
//! PRG/CHR select semantics (mirroring FrontFareast in Mesen2):
//! - Writing `value` to `$8000-$FFFF`:
//!   - PRG 16 KiB bank at `$8000-$BFFF`:
//!     - Compute an 8 KiB page index `p = (value & 0xF8) >> 2`.
//!     - Map `$8000-$9FFF` to page `p` and `$A000-$BFFF` to page `p + 1`.
//!   - CHR 8 KiB bank at `$0000-$1FFF`:
//!     - Use low 3 bits `c = value & 0x07`.
//!     - Map `$0000-$1FFF` to CHR-RAM bytes starting at `c * 8 KiB`.

use std::borrow::Cow;

use crate::{
    cartridge::{
        ChrRom, Mapper, PrgRom, TrainerBytes,
        header::{Header, Mirroring},
        mapper::{ChrStorage, allocate_prg_ram_with_trainer},
    },
    memory::cpu as cpu_mem,
};

/// Size of a single PRG bank exposed to the CPU (8 KiB).
const PRG_BANK_SIZE_8K: usize = 8 * 1024;
/// Total CHR-RAM size used by the Front Fareast board in GUI mode.
const CHR_RAM_SIZE: usize = 32 * 1024;

#[derive(Debug, Clone)]
pub struct Mapper8 {
    prg_rom: PrgRom,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,

    /// Number of 8 KiB PRG-ROM banks available.
    prg_bank_count_8k: usize,

    /// Base 8 KiB page index for the switchable 16 KiB region at
    /// `$8000-$BFFF`. Slot layout:
    /// - `$8000-$9FFF` → `prg_bank_low_2x`
    /// - `$A000-$BFFF` → `prg_bank_low_2x + 1`
    prg_bank_low_2x: usize,

    /// Base 8 KiB page index for the fixed 16 KiB region at `$C000-$FFFF`.
    /// Slot layout:
    /// - `$C000-$DFFF` → `prg_bank_high_2x`
    /// - `$E000-$FFFF` → `prg_bank_high_2x + 1`
    prg_bank_high_2x: usize,

    /// Current 8 KiB CHR-RAM "group" (0‑7). Each group selects an 8 KiB chunk
    /// of the 32 KiB CHR-RAM space.
    chr_bank_group: u8,

    /// 16‑bit IRQ counter incremented once per CPU activity while enabled.
    irq_counter: u16,
    irq_enabled: bool,
    irq_pending: bool,

    /// Current effective nametable mirroring.
    mirroring: Mirroring,
}

impl Mapper8 {
    pub fn new(header: Header, prg_rom: PrgRom, _chr_rom: ChrRom, trainer: TrainerBytes) -> Self {
        let prg_ram = allocate_prg_ram_with_trainer(&header, trainer);

        // Front Fareast carts expose a large CHR-RAM region regardless of the
        // original header; we model this directly instead of using
        // `select_chr_storage`.
        let chr_ram = vec![0u8; CHR_RAM_SIZE].into_boxed_slice();
        let chr = ChrStorage::Ram(chr_ram);

        let prg_bank_count_8k = (prg_rom.len() / PRG_BANK_SIZE_8K).max(1);

        Self {
            prg_rom,
            prg_ram,
            chr,
            prg_bank_count_8k,
            prg_bank_low_2x: 0,
            prg_bank_high_2x: 0, // initialised in `power_on`
            chr_bank_group: 0,
            irq_counter: 0,
            irq_enabled: false,
            irq_pending: false,
            mirroring: header.mirroring,
        }
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }

        let offset = (addr & 0x1FFF) as usize;
        let base_page = match addr {
            0x8000..=0x9FFF => self.prg_bank_low_2x,
            0xA000..=0xBFFF => self.prg_bank_low_2x.saturating_add(1),
            0xC000..=0xDFFF => self.prg_bank_high_2x,
            0xE000..=0xFFFF => self.prg_bank_high_2x.saturating_add(1),
            _ => return 0,
        };

        let page = if self.prg_bank_count_8k == 0 {
            0
        } else {
            base_page % self.prg_bank_count_8k
        };

        let base = page.saturating_mul(PRG_BANK_SIZE_8K);
        let idx = base.saturating_add(offset);
        self.prg_rom.get(idx).copied().unwrap_or(0)
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

    /// Update the 16 KiB PRG window at `$8000-$BFFF` using the same formula
    /// as Mesen2's FrontFareast mapper for iNES ID 8.
    fn update_prg_bank_low(&mut self, data: u8) {
        if self.prg_bank_count_8k < 2 {
            self.prg_bank_low_2x = 0;
            return;
        }

        // Mesen2: SelectPrgPage2x(0, (value & 0xF8) >> 2)
        let mut page = ((data & 0xF8) as usize) >> 2;
        if page + 1 >= self.prg_bank_count_8k {
            page = self.prg_bank_count_8k.saturating_sub(2);
        }
        self.prg_bank_low_2x = page;
    }

    /// Update the 8 KiB CHR-RAM window based on the low three bits of the
    /// written value. This mirrors the `SelectChrPage8x(0, (value & 0x07) << 3)`
    /// behaviour in Mesen2 when modelled as 32 KiB of 1 KiB pages.
    fn update_chr_bank(&mut self, data: u8) {
        self.chr_bank_group = data & 0x07;
    }

    fn read_chr(&self, addr: u16) -> u8 {
        let group = (self.chr_bank_group & 0x07) as usize;
        let base = group * (8 * 1024); // 8 KiB per group.
        let offset = (addr & 0x1FFF) as usize;
        self.chr.read_indexed(base, offset)
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        let group = (self.chr_bank_group & 0x07) as usize;
        let base = group * (8 * 1024);
        let offset = (addr & 0x1FFF) as usize;
        self.chr.write_indexed(base, offset, data);
    }

    fn write_control_42fe(&mut self, data: u8) {
        // Bit 4 selects one-screen mirroring (Screen A vs Screen B).
        match (data >> 4) & 0x01 {
            0 => self.mirroring = Mirroring::SingleScreenLower,
            _ => self.mirroring = Mirroring::SingleScreenUpper,
        }
    }

    fn write_control_42ff(&mut self, data: u8) {
        // Bit 4 selects vertical vs horizontal mirroring.
        self.mirroring = if (data >> 4) & 0x01 == 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };
    }

    fn write_irq_disable_4501(&mut self) {
        self.irq_enabled = false;
        self.irq_pending = false;
    }

    fn write_irq_low_4502(&mut self, data: u8) {
        self.irq_counter = (self.irq_counter & 0xFF00) | (data as u16);
        self.irq_pending = false;
    }

    fn write_irq_high_4503(&mut self, data: u8) {
        self.irq_counter = (self.irq_counter & 0x00FF) | ((data as u16) << 8);
        self.irq_enabled = true;
        self.irq_pending = false;
    }

    fn write_bank_select_8000_plus(&mut self, _addr: u16, data: u8) {
        // Mapper 8 always treats writes in `$8000-$FFFF` as combined PRG/CHR
        // bank selects.
        self.update_prg_bank_low(data);
        self.update_chr_bank(data);
    }
}

impl Mapper for Mapper8 {
    fn power_on(&mut self) {
        // Power-on defaults:
        // - IRQ counter disabled and cleared.
        // - PRG mapping: first 16 KiB at $8000, last 16 KiB at $C000.
        // - CHR mapping: first 8 KiB at $0000.
        self.irq_counter = 0;
        self.irq_enabled = false;
        self.irq_pending = false;
        self.chr_bank_group = 0;

        self.prg_bank_low_2x = 0;
        self.prg_bank_high_2x = if self.prg_bank_count_8k >= 2 {
            self.prg_bank_count_8k - 2
        } else {
            0
        };
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

            0x42FE => self.write_control_42fe(data),
            0x42FF => self.write_control_42ff(data),

            0x4501 => self.write_irq_disable_4501(),
            0x4502 => self.write_irq_low_4502(data),
            0x4503 => self.write_irq_high_4503(data),

            0x8000..=0xFFFF => self.write_bank_select_8000_plus(addr, data),
            _ => {}
        }
    }

    fn cpu_clock(&mut self, _cpu_cycle: u64) {
        if self.irq_enabled {
            self.irq_counter = self.irq_counter.wrapping_add(1);
            if self.irq_counter == 0 {
                self.irq_pending = true;
                self.irq_enabled = false;
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
        8
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Front Fareast GUI (Mapper 8)")
    }
}
