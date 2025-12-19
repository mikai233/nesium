//! Mapper 90 – J.Y. Company multicart (simplified).
//!
//! This is a pared-down implementation that covers the common banking model:
//! - PRG: four 8 KiB registers at `$8000-$8003` map to the three switchable
//!   slots at `$8000/$A000/$C000` and a fourth slot we treat as fixed (`$E000`).
//! - CHR: eight 1 KiB banks split across `$9000-$9FFF` (low bits) and
//!   `$A000-$AFFF` (high bits). Advanced nametable-as-CHR behaviour is not
//!   modelled; nametable control writes are ignored.
//! - Mirroring: `$D001` low two bits select V/H/one-screen A/B.
//! - IRQ: simple CPU-cycle counter with reload (`$C005`), prescaler (`$C004`),
//!   enable/ack (`$C000/$C002/$C003`), modelled after other VRC-style timers.
//!
//! Known limitations:
//! - Advanced nametable mapping and block/chunk CHR modes are omitted.
//! - IRQ details on real JY hardware differ; this uses a VRC-like approximation.
//!
//! | Area | Address range                             | Behaviour                                          | IRQ/Audio      |
//! |------|-------------------------------------------|----------------------------------------------------|----------------|
//! | CPU  | `$6000-$7FFF`                             | Optional PRG-RAM                                   | None           |
//! | CPU  | `$8000-$9FFF`                             | PRG bank register 0 / 8 KiB PRG slot               | None           |
//! | CPU  | `$A000-$BFFF`                             | PRG bank register 1 / 8 KiB PRG slot               | None           |
//! | CPU  | `$C000-$DFFF`                             | PRG bank register 2 / 8 KiB PRG slot               | None           |
//! | CPU  | `$E000-$FFFF`                             | PRG bank register 3 / 8 KiB PRG slot (often fixed) | None           |
//! | CPU  | `$9000/$A000/$B000/$C000-$C007/$D001`     | CHR, IRQ, mirroring registers                      | JY IRQ timer   |
//! | PPU  | `$0000-$1FFF`                             | Eight 1 KiB CHR banks (split low/high regs)        | None           |

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

/// CPU `$8000-$FFFF`: J.Y. Company mapper 90 register window. Writes in this
/// range select PRG/CHR banks, mirroring, and IRQ configuration.
const JY90_IO_WINDOW_START: u16 = 0x8000;
const JY90_IO_WINDOW_END: u16 = 0xFFFF;

/// CPU-visible JY-90 mapper register set.
///
/// JY-90 exposes four PRG bank registers, CHR low/high registers, nametable
/// control and a small IRQ/feature control area. Grouping them into an enum
/// makes the decoded layout clearer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Jy90CpuRegister {
    /// `$8000-$8003` – four 8 KiB PRG bank registers.
    PrgBank,
    /// `$9000-$9007` – CHR low bytes for eight 1 KiB banks.
    ChrLow,
    /// `$A000-$A007` – CHR high bytes for eight 1 KiB banks.
    ChrHigh,
    /// `$B000-$B007` – nametable/mode control (ignored in this implementation).
    NametableCtrl,
    /// `$C000` – IRQ enable.
    IrqEnable,
    /// `$C001` – IRQ mode (ignored).
    IrqMode,
    /// `$C002` – IRQ disable.
    IrqDisable,
    /// `$C003` – IRQ enable (alternate).
    IrqEnableAlt,
    /// `$C004` – IRQ prescaler value.
    IrqPrescaler,
    /// `$C005` – IRQ reload value.
    IrqReload,
    /// `$C006-$C007` – unused IRQ-related registers.
    IrqUnused,
    /// `$D000` – mode bits (ignored).
    ModeControl,
    /// `$D001` – mirroring control.
    Mirroring,
    /// `$D002-$D003` – unused.
    MiscUnused,
}

impl Jy90CpuRegister {
    fn from_addr(addr: u16) -> Option<Self> {
        use Jy90CpuRegister::*;

        match addr {
            0x8000..=0x8003 => Some(PrgBank),
            0x9000..=0x9007 => Some(ChrLow),
            0xA000..=0xA007 => Some(ChrHigh),
            0xB000..=0xB007 => Some(NametableCtrl),
            0xC000 => Some(IrqEnable),
            0xC001 => Some(IrqMode),
            0xC002 => Some(IrqDisable),
            0xC003 => Some(IrqEnableAlt),
            0xC004 => Some(IrqPrescaler),
            0xC005 => Some(IrqReload),
            0xC006 | 0xC007 => Some(IrqUnused),
            0xD000 => Some(ModeControl),
            0xD001 => Some(Mirroring),
            0xD002 | 0xD003 => Some(MiscUnused),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mapper90 {
    prg_rom: PrgRom,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,

    prg_bank_count_8k: usize,

    prg_regs: Mapper90PrgRegs,
    chr_low: Mapper90ChrLowRegs,
    chr_high: Mapper90ChrHighRegs,

    mirroring: Mirroring,

    // IRQ (simplified VRC-style)
    irq_reload: u8,
    irq_counter: u8,
    irq_prescaler: i32,
    irq_enabled: bool,
    irq_pending: bool,
}

type Mapper90PrgRegs = ByteBlock<4>;
type Mapper90ChrLowRegs = ByteBlock<8>;
type Mapper90ChrHighRegs = ByteBlock<8>;

impl Mapper90 {
    pub fn new(header: Header, prg_rom: PrgRom, chr_rom: ChrRom, trainer: TrainerBytes) -> Self {
        let prg_ram = allocate_prg_ram_with_trainer(&header, trainer);

        let chr = select_chr_storage(&header, chr_rom);
        let prg_bank_count_8k = (prg_rom.len() / PRG_BANK_SIZE_8K).max(1);

        Self {
            prg_rom,
            prg_ram,
            chr,
            prg_bank_count_8k,
            prg_regs: Mapper90PrgRegs::new(),
            chr_low: Mapper90ChrLowRegs::new(),
            chr_high: Mapper90ChrHighRegs::new(),
            mirroring: header.mirroring(),
            irq_reload: 0,
            irq_counter: 0,
            irq_prescaler: 0,
            irq_enabled: false,
            irq_pending: false,
        }
    }

    #[inline]
    fn prg_bank_index(&self, value: u8) -> usize {
        if self.prg_bank_count_8k == 0 {
            0
        } else {
            (value as usize) % self.prg_bank_count_8k
        }
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }
        let bank = match addr {
            0x8000..=0x9FFF => self.prg_bank_index(self.prg_regs[0]),
            0xA000..=0xBFFF => self.prg_bank_index(self.prg_regs[1]),
            0xC000..=0xDFFF => self.prg_bank_index(self.prg_regs[2]),
            0xE000..=0xFFFF => {
                self.prg_bank_index(self.prg_regs[3].max(self.prg_bank_count_8k as u8 - 1))
            }
            _ => 0,
        };
        let offset = (addr & 0x1FFF) as usize;
        let base = bank.saturating_mul(PRG_BANK_SIZE_8K);
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

    fn chr_page_base(&self, bank: usize) -> usize {
        let lo = self.chr_low.get(bank).copied().unwrap_or(0);
        let hi = self.chr_high.get(bank).copied().unwrap_or(0);
        let page = (lo as usize) | ((hi as usize) << 8);
        page * CHR_BANK_SIZE_1K
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
        if let Some(reg) = Jy90CpuRegister::from_addr(addr) {
            use Jy90CpuRegister::*;

            match reg {
                PrgBank => {
                    self.prg_regs[(addr & 0x0003) as usize] = value & 0x7F;
                }
                ChrLow => {
                    self.chr_low[(addr & 0x0007) as usize] = value;
                }
                ChrHigh => {
                    self.chr_high[(addr & 0x0007) as usize] = value & 0x1F;
                }
                NametableCtrl => {
                    // Nametable control ignored in this simplified model.
                    let _ = value;
                }
                IrqEnable => {
                    self.irq_enabled = (value & 0x01) != 0;
                    if !self.irq_enabled {
                        self.irq_pending = false;
                    }
                }
                IrqMode => {
                    let _ = value; // IRQ mode bits ignored.
                }
                IrqDisable => {
                    self.irq_enabled = false;
                    self.irq_pending = false;
                }
                IrqEnableAlt => {
                    self.irq_enabled = true;
                }
                IrqPrescaler => {
                    self.irq_prescaler = (value as i32) & 0xFF;
                }
                IrqReload => {
                    self.irq_reload = value;
                }
                IrqUnused => {
                    let _ = value; // Unused in this simplified model.
                }
                ModeControl => {
                    // Mode bits ignored; base implementation uses direct registers.
                    let _ = value;
                }
                Mirroring => {
                    self.mirroring = match value & 0x03 {
                        0 => crate::cartridge::header::Mirroring::Vertical,
                        1 => crate::cartridge::header::Mirroring::Horizontal,
                        2 => crate::cartridge::header::Mirroring::SingleScreenLower,
                        _ => crate::cartridge::header::Mirroring::SingleScreenUpper,
                    };
                }
                MiscUnused => {
                    let _ = value; // Ignored
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

impl Mapper for Mapper90 {
    fn reset(&mut self, _kind: ResetKind) {
        self.prg_regs[0] = 0;
        self.prg_regs[1] = 1;
        self.prg_regs[2] = 2;
        self.prg_regs[3] = 0x7F;
        self.chr_low.fill(0);
        self.chr_high.fill(0);
        self.irq_reload = 0;
        self.irq_counter = 0;
        self.irq_prescaler = 0;
        self.irq_enabled = false;
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
            JY90_IO_WINDOW_START..=JY90_IO_WINDOW_END => self.write_register(addr, data),
            _ => {}
        }
    }

    fn cpu_clock(&mut self, _cpu_cycle: u64) {
        if !self.irq_enabled {
            return;
        }
        self.irq_prescaler -= 3;
        if self.irq_prescaler <= 0 {
            self.clock_irq_counter();
            self.irq_prescaler += 341;
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
        90
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("JY Company (simplified)")
    }
}
