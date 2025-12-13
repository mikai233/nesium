//! Mapper 6 – Front Fareast Magic Card / F4xxx support.
//!
//! This mapper is used for ROMs converted from the Front Fareast
//! "Magic Card" RAM cartridges (often referred to as F4xxx boards).
//! The hardware is documented on Nesdev under the *Super Magic Card*
//! article and is implemented in Mesen2 as the `FrontFareast` board.
//!
//! Behaviour modelled here (for iNES mapper 6):
//! - CPU `$6000-$7FFF`: optional PRG-RAM (battery-backed or work RAM).
//! - CPU `$8000-$BFFF`: 16 KiB switchable PRG-ROM window (two 8 KiB banks).
//! - CPU `$C000-$FFFF`: 16 KiB fixed PRG-ROM window mapped to the last
//!   16 KiB of the ROM image.
//! - PPU `$0000-$1FFF`: 8 KiB CHR-RAM, switchable in 8 KiB steps.
//! - CPU `$42FE/$42FF`: mirroring + Front Fareast "alt mode" control.
//! - CPU `$4501/$4502/$4503`: 16‑bit CPU‑cycle IRQ counter.
//! - CPU `$8000-$FFFF`: bank select writes for PRG+CHR (mirroring Mesen2).
//!
//! The IRQ counter is a simple up-counter:
//! - `$4502` writes the low 8 bits of the 16‑bit counter.
//! - `$4503` writes the high 8 bits and *enables* counting.
//! - The counter increments every CPU bus write (this is a slight
//!   approximation of the real hardware, which counts every CPU cycle).
//! - When the counter overflows from `$FFFF` to `$0000`, the mapper
//!   asserts the external IRQ line and disables further counting until
//!   the game reinitialises the counter.
//!
//! | Area | Address range     | Behaviour                                     | IRQ/Audio           |
//! |------|-------------------|-----------------------------------------------|---------------------|
//! | CPU  | `$6000-$7FFF`     | Optional PRG-RAM                              | None                |
//! | CPU  | `$42FE/$42FF`     | Front Fareast mirroring + alt-mode control   | None                |
//! | CPU  | `$4501-$4503`     | 16-bit IRQ counter control                   | CPU-write IRQ timer |
//! | CPU  | `$8000-$FFFF`     | Combined PRG (16 KiB) + CHR (8 KiB) bank sel | CPU-write IRQ timer |
//! | PPU  | `$0000-$1FFF`     | 8 KiB CHR-RAM window inside 32 KiB space     | None                |
//! | PPU  | `$2000-$3EFF`     | Mirroring from FFE control registers         | None                |

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

/// Size of a single PRG bank exposed to the CPU (8 KiB).
const PRG_BANK_SIZE_8K: usize = 8 * 1024;
/// Total CHR-RAM size used by the Front Fareast board.
///
/// Nesdev / Mesen2 model this as a 32 KiB CHR-RAM space, regardless of the
/// CHR sizes advertised in the original ROM header.
const CHR_RAM_SIZE: usize = 32 * 1024;

/// CPU `$42FE`: Front Fareast control register (one-screen mirroring base and alt-mode).
const FF_CTRL_MIRROR_ONE_SCREEN_ADDR: u16 = 0x42FE;
/// CPU `$42FF`: Front Fareast control register (vertical vs horizontal mirroring).
const FF_CTRL_MIRROR_ORIENTATION_ADDR: u16 = 0x42FF;

/// CPU `$4501/$4502/$4503`: 16-bit IRQ counter control.
/// - `$4501`: disable/acknowledge IRQ and stop counting.
/// - `$4502`: low byte of IRQ counter.
/// - `$4503`: high byte of IRQ counter and enable counting.
const FF_IRQ_DISABLE_ADDR: u16 = 0x4501;
const FF_IRQ_COUNTER_LOW_ADDR: u16 = 0x4502;
const FF_IRQ_COUNTER_HIGH_ADDR: u16 = 0x4503;

/// CPU `$8000-$FFFF`: combined PRG/CHR bank select window.
const FF_BANK_SELECT_START: u16 = 0x8000;
const FF_BANK_SELECT_END: u16 = 0xFFFF;

#[derive(Debug, Clone)]
pub struct Mapper6 {
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

    /// Current 8 KiB CHR-RAM "group" (0‑3). Each group selects an 8 KiB chunk
    /// of the 32 KiB CHR-RAM space:
    /// - group 0 → `$0000-$1FFF` maps CHR bytes `0x0000-0x1FFF`
    /// - group 1 → `$0000-$1FFF` maps CHR bytes `0x2000-0x3FFF`
    ///   etc.
    chr_bank_group: u8,

    /// 16‑bit IRQ counter incremented once per CPU bus write while enabled.
    irq_counter: u16,
    irq_enabled: bool,
    irq_pending: bool,

    /// Front Fareast "alt mode" flag (bit 7 of `$42FE`).
    ///
    /// Mesen2 uses this together with `HasChrRam()` to decide whether PRG
    /// banking is affected by `$8000-$FFFF` writes. For mapper 6 images that
    /// expose CHR-RAM, the condition is always true; we keep the flag for
    /// documentation and future CHR-ROM variants.
    ffe_alt_mode: bool,

    /// Current effective nametable mirroring.
    mirroring: Mirroring,
}

impl Mapper6 {
    pub fn new(header: Header, prg_rom: PrgRom, _chr_rom: ChrRom, trainer: TrainerBytes) -> Self {
        // Allocate PRG-RAM using the shared helper so NES 2.0 save/work RAM
        // sizing is respected and any trainer bytes are copied in.
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
            ffe_alt_mode: true,
            mirroring: header.mirroring,
        }
    }

    #[inline]
    fn has_chr_ram(&self) -> bool {
        matches!(self.chr, ChrStorage::Ram(_))
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }

        // Convert the CPU address into a 13‑bit offset within the selected
        // 8 KiB bank.
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

    /// Update the 16 KiB PRG window at `$8000-$BFFF` based on the value
    /// written by the CPU. This mirrors Mesen2's `SelectPrgPage2x(0, (value
    /// & 0xFC) >> 1)` behaviour.
    fn update_prg_bank_low(&mut self, data: u8) {
        if self.prg_bank_count_8k < 2 {
            self.prg_bank_low_2x = 0;
            return;
        }

        let mut page = ((data & 0xFC) as usize) >> 1;
        // Clamp so that the second page in the 16 KiB pair still falls
        // within the valid bank range.
        if page + 1 >= self.prg_bank_count_8k {
            page = self.prg_bank_count_8k.saturating_sub(2);
        }
        self.prg_bank_low_2x = page;
    }

    /// Update the 8 KiB CHR-RAM window based on the low two bits of the
    /// written value. This mirrors Mesen2's `SelectChrPage8x(0, value << 3)`
    /// with a 1 KiB page size.
    fn update_chr_bank(&mut self, data: u8) {
        self.chr_bank_group = data & 0x03;
    }

    fn read_chr(&self, addr: u16) -> u8 {
        let group = (self.chr_bank_group & 0x03) as usize;
        let base = group * (8 * 1024); // 8 KiB per group
        let offset = (addr & 0x1FFF) as usize;
        self.chr.read_indexed(base, offset)
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        let group = (self.chr_bank_group & 0x03) as usize;
        let base = group * (8 * 1024);
        let offset = (addr & 0x1FFF) as usize;
        self.chr.write_indexed(base, offset, data);
    }

    fn write_ffe_control_42fe(&mut self, data: u8) {
        // Bit 7 controls an alternate mode used when CHR-ROM is present on
        // some FFE boards. For mapper 6 with CHR-RAM this is effectively
        // always true, but we keep it for completeness.
        self.ffe_alt_mode = (data & 0x80) == 0;

        // Bit 4 selects one-screen mirroring (Screen A vs Screen B).
        // 0 → Screen A (nametable 0), 1 → Screen B (nametable 1).
        match (data >> 4) & 0x01 {
            0 => self.mirroring = Mirroring::SingleScreenLower,
            _ => self.mirroring = Mirroring::SingleScreenUpper,
        }
    }

    fn write_ffe_control_42ff(&mut self, data: u8) {
        // Bit 4 selects vertical vs horizontal mirroring.
        self.mirroring = if (data >> 4) & 0x01 == 0 {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };
    }

    fn write_irq_disable_4501(&mut self) {
        // Acknowledges any pending IRQ and disables counting.
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

    fn write_bank_select_8000_plus(&mut self, addr: u16, mut data: u8) {
        let _ = addr;

        // Mapper 6 (FFE Front Fareast) behaviour as implemented in Mesen2:
        // - When CHR-RAM is present or alt-mode is enabled, `$8000-$FFFF`
        //   writes adjust both PRG (16 KiB at $8000) and CHR (8 KiB).
        // - Otherwise, only CHR is affected.
        if self.has_chr_ram() || self.ffe_alt_mode {
            self.update_prg_bank_low(data);
            // High bits used for PRG; low two bits select the CHR group.
            data &= 0x03;
        }
        self.update_chr_bank(data);
    }
}

impl Mapper for Mapper6 {
    fn reset(&mut self, _kind: ResetKind) {
        // Front Fareast power-on defaults:
        // - IRQ counter disabled and cleared.
        // - Alt mode enabled.
        // - PRG mapping: first 16 KiB at $8000, last 16 KiB at $C000.
        self.irq_counter = 0;
        self.irq_enabled = false;
        self.irq_pending = false;
        self.ffe_alt_mode = true;
        self.chr_bank_group = 0;

        // Low 16 KiB starts at the first two 8 KiB banks.
        self.prg_bank_low_2x = 0;
        // High 16 KiB fixed to the last pair of 8 KiB banks.
        self.prg_bank_high_2x = self.prg_bank_count_8k.saturating_sub(2);
    }

    fn cpu_read(&self, addr: u16) -> Option<u8> {
        let value = match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => return self.read_prg_ram(addr),
            cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END => self.read_prg_rom(addr),
            // Registers in `$42xx`/`$45xx` are write-only; reads behave as
            // open bus, so we return `None` here.
            _ => return None,
        };
        Some(value)
    }

    fn cpu_write(&mut self, addr: u16, data: u8, _cpu_cycle: u64) {
        match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.write_prg_ram(addr, data),

            // Front Fareast control registers.
            FF_CTRL_MIRROR_ONE_SCREEN_ADDR => self.write_ffe_control_42fe(data),
            FF_CTRL_MIRROR_ORIENTATION_ADDR => self.write_ffe_control_42ff(data),

            // IRQ control and 16‑bit counter.
            FF_IRQ_DISABLE_ADDR => self.write_irq_disable_4501(),
            FF_IRQ_COUNTER_LOW_ADDR => self.write_irq_low_4502(data),
            FF_IRQ_COUNTER_HIGH_ADDR => self.write_irq_high_4503(data),

            // PRG/CHR banking via `$8000-$FFFF`.
            FF_BANK_SELECT_START..=FF_BANK_SELECT_END => {
                self.write_bank_select_8000_plus(addr, data)
            }

            _ => {}
        }
    }

    fn cpu_clock(&mut self, _cpu_cycle: u64) {
        // Approximate the Front Fareast IRQ timer: increment the counter
        // once per CPU bus write. On real hardware, the counter increments
        // every CPU cycle, but this approximation matches the intent for
        // most games using mapper 6 dumps.
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
        6
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Front Fareast Magic Card")
    }
}
