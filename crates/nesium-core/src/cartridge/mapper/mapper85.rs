//! Mapper 85 – Konami VRC7 (without expansion audio synthesis).
//!
//! This models the banking/IRQ behaviour of the VRC7 used by titles like Lagrange
//! Point. Expansion audio registers are accepted but muted; integrating full VRC7
//! audio would require an OPLL core wired through `ExpansionAudio`.
//!
//! - PRG: three switchable 8 KiB banks at `$8000/$A000/$C000`, fixed last bank
//!   at `$E000`.
//! - CHR: eight 1 KiB banks at `$0000-$1FFF`.
//! - Mirroring: control register `$E000` bits 0‑1 (H/V/screen A/B).
//! - PRG-RAM enable: control register bit 7 (when present via header sizing).
//! - IRQ: VRC-style counter with reload (`$E008`), control (`$F000`), ack
//!   (`$F008`), prescaler clocked by CPU cycles (divide by ~341).
//!
//! | Area | Address range       | Behaviour                                          | IRQ/Audio                          |
//! |------|---------------------|----------------------------------------------------|------------------------------------|
//! | CPU  | `$6000-$7FFF`       | Optional PRG-RAM with enable bit in control       | None                               |
//! | CPU  | `$8000/$A000/$C000` | Three switchable 8 KiB PRG-ROM banks             | None                               |
//! | CPU  | `$E000-$FFFF`       | Fixed 8 KiB PRG-ROM bank (last) + control/IRQ    | VRC7 IRQ (expansion audio muted)   |
//! | CPU  | `$8000-$D008`       | VRC7 PRG/CHR/mirroring/IRQ registers              | VRC7 IRQ (audio registers, muted)  |
//! | PPU  | `$0000-$1FFF`       | Eight 1 KiB CHR banks                             | None                               |
//! | PPU  | `$2000-$3EFF`       | Mirroring from VRC7 control register              | None                               |

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

/// PRG banking granularity (8 KiB).
const PRG_BANK_SIZE_8K: usize = 8 * 1024;
/// CHR banking granularity (1 KiB).
const CHR_BANK_SIZE_1K: usize = 1024;

/// CPU `$8000-$9FFF`: switchable 8 KiB PRG-ROM bank mapped at `$8000`.
const VRC7_PRG_SLOT0_START: u16 = 0x8000;
const VRC7_PRG_SLOT0_END: u16 = 0x9FFF;
/// CPU `$A000-$BFFF`: switchable 8 KiB PRG-ROM bank mapped at `$A000`.
const VRC7_PRG_SLOT1_START: u16 = 0xA000;
const VRC7_PRG_SLOT1_END: u16 = 0xBFFF;
/// CPU `$C000-$DFFF`: switchable 8 KiB PRG-ROM bank mapped at `$C000`.
const VRC7_PRG_SLOT2_START: u16 = 0xC000;
const VRC7_PRG_SLOT2_END: u16 = 0xDFFF;
/// CPU `$E000-$FFFF`: fixed 8 KiB PRG-ROM bank mapped to the last bank.
const VRC7_PRG_FIXED_SLOT_START: u16 = 0xE000;
const VRC7_PRG_FIXED_SLOT_END: u16 = 0xFFFF;

/// Mask used to decode VRC7 register addresses after board-specific mirroring.
const VRC7_REG_ADDR_MASK: u16 = 0xF038;

/// VRC7 PRG bank select registers (after address decoding).
/// - `$8000`: 8 KiB bank for `$8000-$9FFF`.
/// - `$8008`: 8 KiB bank for `$A000-$BFFF`.
/// - `$9000`: 8 KiB bank for `$C000-$DFFF`.
const VRC7_REG_PRG_BANK_8000: u16 = 0x8000;
const VRC7_REG_PRG_BANK_A000: u16 = 0x8008;
const VRC7_REG_PRG_BANK_C000: u16 = 0x9000;

/// VRC7 CHR bank registers (`$A000-$D008` – eight 1 KiB slots).
const VRC7_REG_CHR_BANK_0: u16 = 0xA000;
const VRC7_REG_CHR_BANK_1: u16 = 0xA008;
const VRC7_REG_CHR_BANK_2: u16 = 0xB000;
const VRC7_REG_CHR_BANK_3: u16 = 0xB008;
const VRC7_REG_CHR_BANK_4: u16 = 0xC000;
const VRC7_REG_CHR_BANK_5: u16 = 0xC008;
const VRC7_REG_CHR_BANK_6: u16 = 0xD000;
const VRC7_REG_CHR_BANK_7: u16 = 0xD008;

/// VRC7 control and IRQ registers.
/// - `$E000`: mirroring / PRG-RAM enable control.
/// - `$E008`: IRQ reload value.
/// - `$F000`: IRQ control (enable/mode).
/// - `$F008`: IRQ acknowledge / re-enable.
const VRC7_REG_CONTROL: u16 = 0xE000;
const VRC7_REG_IRQ_RELOAD: u16 = 0xE008;
const VRC7_REG_IRQ_CONTROL: u16 = 0xF000;
const VRC7_REG_IRQ_ACK: u16 = 0xF008;

/// CPU-visible VRC7 register set after address decoding.
///
/// Similar to VRC4, VRC7 maps its control registers into `$8000-$FFFF` after
/// some address-line mixing. Using a dedicated enum keeps the logical
/// registers clear and close to the documented layout.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Vrc7CpuRegister {
    /// `$8000` – PRG bank for `$8000-$9FFF`.
    PrgBank8000,
    /// `$8008` – PRG bank for `$A000-$BFFF`.
    PrgBankA000,
    /// `$9000` – PRG bank for `$C000-$DFFF`.
    PrgBankC000,
    /// `$A000-$D008` – eight 1 KiB CHR bank registers.
    ChrBank,
    /// `$E000` – mirroring / PRG-RAM control.
    Control,
    /// `$E008` – IRQ reload value.
    IrqReload,
    /// `$F000` – IRQ control.
    IrqControl,
    /// `$F008` – IRQ acknowledge / re-enable.
    IrqAck,
}

impl Vrc7CpuRegister {
    fn from_addr(addr: u16) -> Option<Self> {
        use Vrc7CpuRegister::*;

        match addr & VRC7_REG_ADDR_MASK {
            VRC7_REG_PRG_BANK_8000 => Some(PrgBank8000),
            VRC7_REG_PRG_BANK_A000 => Some(PrgBankA000),
            VRC7_REG_PRG_BANK_C000 => Some(PrgBankC000),
            VRC7_REG_CHR_BANK_0 | VRC7_REG_CHR_BANK_1 | VRC7_REG_CHR_BANK_2
            | VRC7_REG_CHR_BANK_3 | VRC7_REG_CHR_BANK_4 | VRC7_REG_CHR_BANK_5
            | VRC7_REG_CHR_BANK_6 | VRC7_REG_CHR_BANK_7 => Some(ChrBank),
            VRC7_REG_CONTROL => Some(Control),
            VRC7_REG_IRQ_RELOAD => Some(IrqReload),
            VRC7_REG_IRQ_CONTROL => Some(IrqControl),
            VRC7_REG_IRQ_ACK => Some(IrqAck),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mapper85 {
    prg_rom: crate::cartridge::PrgRom,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,

    prg_bank_count_8k: usize,

    prg_banks: Mapper85PrgBanks, // $8000, $A000, $C000
    chr_banks: Mapper85ChrBanks,

    control: u8,
    mirroring: Mirroring,

    irq_reload: u8,
    irq_counter: u8,
    irq_prescaler: i32,
    irq_enabled: bool,
    irq_enabled_after_ack: bool,
    irq_cycle_mode: bool,
    irq_pending: bool,
}

type Mapper85PrgBanks = ByteBlock<3>;
type Mapper85ChrBanks = ByteBlock<8>;

impl Mapper85 {
    pub fn new(header: Header, prg_rom: PrgRom, chr_rom: ChrRom, trainer: TrainerBytes) -> Self {
        let prg_ram = allocate_prg_ram_with_trainer(&header, trainer);

        let chr = select_chr_storage(&header, chr_rom);
        let prg_bank_count_8k = (prg_rom.len() / PRG_BANK_SIZE_8K).max(1);

        Self {
            prg_rom,
            prg_ram,
            chr,
            prg_bank_count_8k,
            prg_banks: Mapper85PrgBanks::new(),
            chr_banks: Mapper85ChrBanks::new(),
            control: 0,
            mirroring: header.mirroring(),
            irq_reload: 0,
            irq_counter: 0,
            irq_prescaler: 0,
            irq_enabled: false,
            irq_enabled_after_ack: false,
            irq_cycle_mode: false,
            irq_pending: false,
        }
    }

    #[inline]
    fn prg_ram_enabled(&self) -> bool {
        !self.prg_ram.is_empty() && (self.control & 0x80) != 0
    }

    #[inline]
    fn prg_bank_index(&self, value: u8) -> usize {
        if self.prg_bank_count_8k == 0 {
            0
        } else {
            (value as usize) % self.prg_bank_count_8k
        }
    }

    fn translate_address(&self, addr: u16) -> u16 {
        // VRC7 specific address line mixing (mirrors Mesen2 logic).
        if (addr & 0x10) != 0 && (addr & 0xF010) != 0x9010 {
            (addr | 0x0008) & !0x0010
        } else {
            addr
        }
    }

    fn update_control(&mut self, value: u8) {
        self.control = value;
        self.mirroring = match value & 0x03 {
            0 => Mirroring::Vertical,
            1 => Mirroring::Horizontal,
            2 => Mirroring::SingleScreenLower,
            _ => Mirroring::SingleScreenUpper,
        };
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }

        let bank = match addr {
            VRC7_PRG_SLOT0_START..=VRC7_PRG_SLOT0_END => self.prg_bank_index(self.prg_banks[0]),
            VRC7_PRG_SLOT1_START..=VRC7_PRG_SLOT1_END => self.prg_bank_index(self.prg_banks[1]),
            VRC7_PRG_SLOT2_START..=VRC7_PRG_SLOT2_END => self.prg_bank_index(self.prg_banks[2]),
            VRC7_PRG_FIXED_SLOT_START..=VRC7_PRG_FIXED_SLOT_END => {
                self.prg_bank_count_8k.saturating_sub(1)
            }
            _ => 0,
        };
        let offset = (addr & 0x1FFF) as usize;
        let base = bank.saturating_mul(PRG_BANK_SIZE_8K);
        self.prg_rom.get(base + offset).copied().unwrap_or(0)
    }

    fn read_prg_ram(&self, addr: u16) -> Option<u8> {
        if !self.prg_ram_enabled() {
            return None;
        }
        let idx = (addr - cpu_mem::PRG_RAM_START) as usize % self.prg_ram.len();
        Some(self.prg_ram[idx])
    }

    fn write_prg_ram(&mut self, addr: u16, data: u8) {
        if !self.prg_ram_enabled() {
            return;
        }
        let idx = (addr - cpu_mem::PRG_RAM_START) as usize % self.prg_ram.len();
        self.prg_ram[idx] = data;
    }

    fn chr_page_base(&self, bank: usize) -> usize {
        self.chr_banks.get(bank).copied().unwrap_or(0) as usize * CHR_BANK_SIZE_1K
    }

    fn read_chr(&self, addr: u16) -> u8 {
        let bank = ((addr >> 10) & 0x07) as usize;
        let offset = (addr & 0x03FF) as usize;
        let base = self.chr_page_base(bank);
        self.chr.read_indexed(base, offset)
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        let bank = ((addr >> 10) & 0x07) as usize;
        let offset = (addr & 0x03FF) as usize;
        let base = self.chr_page_base(bank);
        self.chr.write_indexed(base, offset, data);
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        if let Some(reg) = Vrc7CpuRegister::from_addr(addr) {
            match reg {
                Vrc7CpuRegister::PrgBank8000 => self.prg_banks[0] = value & 0x3F,
                Vrc7CpuRegister::PrgBankA000 => self.prg_banks[1] = value & 0x3F,
                Vrc7CpuRegister::PrgBankC000 => self.prg_banks[2] = value & 0x3F,
                Vrc7CpuRegister::ChrBank => match addr & VRC7_REG_ADDR_MASK {
                    VRC7_REG_CHR_BANK_0 => self.chr_banks[0] = value,
                    VRC7_REG_CHR_BANK_1 => self.chr_banks[1] = value,
                    VRC7_REG_CHR_BANK_2 => self.chr_banks[2] = value,
                    VRC7_REG_CHR_BANK_3 => self.chr_banks[3] = value,
                    VRC7_REG_CHR_BANK_4 => self.chr_banks[4] = value,
                    VRC7_REG_CHR_BANK_5 => self.chr_banks[5] = value,
                    VRC7_REG_CHR_BANK_6 => self.chr_banks[6] = value,
                    VRC7_REG_CHR_BANK_7 => self.chr_banks[7] = value,
                    _ => {}
                },
                Vrc7CpuRegister::Control => self.update_control(value),
                Vrc7CpuRegister::IrqReload => {
                    self.irq_reload = value;
                }
                Vrc7CpuRegister::IrqControl => {
                    self.irq_enabled_after_ack = (value & 0x01) != 0;
                    self.irq_enabled = (value & 0x02) != 0;
                    self.irq_cycle_mode = (value & 0x04) != 0;
                    if self.irq_enabled {
                        self.irq_counter = self.irq_reload;
                        self.irq_prescaler = 341;
                        self.irq_pending = false;
                    }
                }
                Vrc7CpuRegister::IrqAck => {
                    self.irq_enabled = self.irq_enabled_after_ack;
                    self.irq_pending = false;
                }
            }
        }
    }

    fn clock_irq_counter(&mut self) {
        if self.irq_counter == 0xFF {
            self.irq_counter = self.irq_reload;
            self.irq_pending = true;
        } else {
            self.irq_counter = self.irq_counter.wrapping_add(1);
        }
    }
}

impl Mapper for Mapper85 {
    fn reset(&mut self, _kind: ResetKind) {
        self.prg_banks.fill(0);
        self.chr_banks.fill(0);
        self.control = 0;
        self.irq_reload = 0;
        self.irq_counter = 0;
        self.irq_prescaler = 0;
        self.irq_enabled = false;
        self.irq_enabled_after_ack = false;
        self.irq_cycle_mode = false;
        self.irq_pending = false;
    }

    fn cpu_read(&self, addr: u16) -> Option<u8> {
        match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.read_prg_ram(addr),
            cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END => Some(self.read_prg_rom(addr)),
            _ => None,
        }
    }

    fn cpu_write(&mut self, addr: u16, data: u8, _cpu_cycle: u64) {
        match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.write_prg_ram(addr, data),
            0x8000..=0xFFFF => {
                let translated = self.translate_address(addr);
                self.write_register(translated, data);
            }
            _ => {}
        }
    }

    fn cpu_clock(&mut self, _cpu_cycle: u64) {
        if !self.irq_enabled {
            return;
        }
        if self.irq_cycle_mode {
            self.clock_irq_counter();
        } else {
            self.irq_prescaler -= 3;
            if self.irq_prescaler <= 0 {
                self.clock_irq_counter();
                self.irq_prescaler += 341;
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
        85
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Konami VRC7 (no audio)")
    }
}
