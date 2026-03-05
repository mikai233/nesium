//! Mapper 26 – Konami VRC6b (with basic VRC6 behaviour, audio stubbed).
//!
//! This implementation mirrors the PRG/CHR banking and IRQ behaviour of VRC6,
//! following Mesen2's layout. VRC6's expansion audio registers are accepted
//! but do not currently generate audio output; this can be extended via the
//! [`ExpansionAudio`] trait in the future.
//!
//! | Area | Address range       | Behaviour                                          | IRQ/Audio                         |
//! |------|---------------------|----------------------------------------------------|-----------------------------------|
//! | CPU  | `$6000-$7FFF`       | Optional PRG-RAM (enabled via banking_mode bit 7)  | None                              |
//! | CPU  | `$8000-$BFFF`       | 16 KiB switchable PRG-ROM window (2×8 KiB)         | None                              |
//! | CPU  | `$C000-$DFFF`       | 8 KiB switchable PRG-ROM window                    | None                              |
//! | CPU  | `$E000-$FFFF`       | 8 KiB fixed PRG-ROM window (last)                  | None                              |
//! | CPU  | `$B003/$F000-$F002` | Banking/mirroring/IRQ control registers           | VRC6 IRQ (audio regs stubbed)     |
//! | PPU  | `$0000-$1FFF`       | Eight 1 KiB CHR banks with mode‑dependent mapping  | None                              |
//! | PPU  | `$2000-$3EFF`       | Mirroring from VRC6 control (`banking_mode`)       | None                              |

use std::borrow::Cow;

use crate::cartridge::mapper::{MapperMemoryMut, MapperMemoryRef};

use crate::{
    cartridge::{
        ChrRom, Mapper, PrgRom, TrainerBytes,
        header::{Header, Mirroring},
        mapper::{
            ChrStorage, MapperEvent, MapperHookMask, NametableTarget,
            allocate_prg_ram_with_trainer, select_chr_storage,
        },
    },
    memory::cpu as cpu_mem,
    reset_kind::ResetKind,
};

use crate::mem_block::ByteBlock;

/// PRG-ROM banking granularity (8 KiB).
const PRG_BANK_SIZE_8K: usize = 8 * 1024;
/// CHR banking granularity (1 KiB).
const CHR_BANK_SIZE_1K: usize = 1024;

/// CPU `$8000-$FFFF`: VRC6b register I/O and PRG banking window. Writes in
/// this range, after address translation, select PRG/CHR/IRQ/mirroring state.
const VRC6_IO_WINDOW_START: u16 = 0x8000;
const VRC6_IO_WINDOW_END: u16 = 0xFFFF;

/// CPU-visible VRC6b register set after address translation.
///
/// VRC6b uses a compact decoded address space (after `translate_address`)
/// where only a handful of masked values represent actual registers. This
/// enum mirrors that layout to make the CPU-side logic easier to follow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Vrc6CpuRegister {
    /// `$8000-$8003` – PRG bank for `$8000-$BFFF` (2×8 KiB window).
    PrgBank8000_2x,
    /// `$9000-$B002` – expansion audio registers (currently ignored).
    ExpansionAudio,
    /// `$B003` – banking/mirroring/CHR mode/PRG-RAM control.
    Control,
    /// `$C000-$C003` – PRG bank for `$C000-$DFFF`.
    PrgBankC000,
    /// `$D000-$D003` – CHR bank registers 0-3.
    ChrBankLow,
    /// `$E000-$E003` – CHR bank registers 4-7.
    ChrBankHigh,
    /// `$F000` – IRQ reload value.
    IrqReload,
    /// `$F001` – IRQ control (enable/mode).
    IrqControl,
    /// `$F002` – IRQ acknowledge / re-enable.
    IrqAck,
}

impl Vrc6CpuRegister {
    fn from_addr(addr: u16) -> Option<Self> {
        use Vrc6CpuRegister::*;

        match addr & 0xF003 {
            0x8000..=0x8003 => Some(PrgBank8000_2x),
            0x9000..=0x9003 => Some(ExpansionAudio),
            0xA000..=0xA003 => Some(ExpansionAudio),
            0xB000..=0xB002 => Some(ExpansionAudio),
            0xB003 => Some(Control),
            0xC000..=0xC003 => Some(PrgBankC000),
            0xD000..=0xD003 => Some(ChrBankLow),
            0xE000..=0xE003 => Some(ChrBankHigh),
            0xF000 => Some(IrqReload),
            0xF001 => Some(IrqControl),
            0xF002 => Some(IrqAck),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mapper26 {
    prg_rom: PrgRom,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,

    prg_bank_count_8k: usize,

    /// Base 16 KiB window at `$8000-$BFFF` (expressed as an 8 KiB index).
    /// `None` means this window is currently unmapped (open bus).
    prg_bank_8000_2x: Option<usize>,
    /// 8 KiB bank at `$C000-$DFFF`.
    /// `None` means this window is currently unmapped (open bus).
    prg_bank_c000: Option<usize>,
    /// Control bits written via `$B003` (banking/mirroring/CHR mode/PRG-RAM).
    banking_mode: u8,
    /// Mirrors Mesen2 startup behavior: $6000-$7FFF starts RW until the first
    /// banking update path (B003/Dxxx/Exxx) reapplies bit7 gating.
    prg_ram_gate_initialized: bool,

    /// Eight 8-bit CHR registers.
    chr_regs: Mapper26ChrRegs,

    mirroring: Mirroring,
    base_mirroring: Mirroring,

    // IRQ state (VRC6 uses the same style counter as VRC4).
    irq_reload: u8,
    irq_counter: u8,
    irq_prescaler: i32,
    irq_enabled: bool,
    irq_enabled_after_ack: bool,
    irq_cycle_mode: bool,
    irq_pending: bool,
}

type Mapper26ChrRegs = ByteBlock<8>;

impl Mapper26 {
    pub fn new(header: Header, prg_rom: PrgRom, chr_rom: ChrRom, trainer: TrainerBytes) -> Self {
        let prg_ram = allocate_prg_ram_with_trainer(&header, trainer);

        let chr = select_chr_storage(&header, chr_rom);
        let prg_bank_count_8k = (prg_rom.len() / PRG_BANK_SIZE_8K).max(1);

        Self {
            prg_rom,
            prg_ram,
            chr,
            prg_bank_count_8k,
            prg_bank_8000_2x: None,
            prg_bank_c000: None,
            banking_mode: 0,
            prg_ram_gate_initialized: false,
            chr_regs: Mapper26ChrRegs::new(),
            mirroring: header.mirroring(),
            base_mirroring: header.mirroring(),
            irq_reload: 0,
            irq_counter: 0,
            irq_prescaler: 0,
            irq_enabled: false,
            irq_enabled_after_ack: false,
            irq_cycle_mode: false,
            irq_pending: false,
        }
    }

    fn translate_address(&self, addr: u16) -> u16 {
        // VRC6b swaps A0/A1 lines.
        (addr & 0xFFFC) | ((addr & 0x0001) << 1) | ((addr & 0x0002) >> 1)
    }

    #[inline]
    fn prg_ram_enabled(&self) -> bool {
        if self.prg_ram.is_empty() {
            return false;
        }

        if !self.prg_ram_gate_initialized {
            true
        } else {
            (self.banking_mode & 0x80) != 0
        }
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

    fn read_prg_rom(&self, addr: u16) -> Option<u8> {
        if self.prg_rom.is_empty() {
            return Some(0);
        }

        let bank = match addr {
            0x8000..=0x9FFF => self.prg_bank_8000_2x,
            0xA000..=0xBFFF => self.prg_bank_8000_2x.map(|bank| bank.saturating_add(1)),
            0xC000..=0xDFFF => self.prg_bank_c000,
            0xE000..=0xFFFF => Some(self.prg_bank_count_8k.saturating_sub(1)),
            _ => None,
        };

        let Some(bank) = bank else {
            // Match Mesen2 BaseMapper behavior: unmapped PRG slots read as 0.
            return Some(0);
        };
        let bank = bank % self.prg_bank_count_8k;

        let offset = (addr & 0x1FFF) as usize;
        let base = bank.saturating_mul(PRG_BANK_SIZE_8K);
        Some(self.prg_rom.get(base + offset).copied().unwrap_or(0))
    }

    fn chr_page_base(&self, bank: usize) -> usize {
        self.chr_regs.get(bank).copied().unwrap_or(0) as usize * CHR_BANK_SIZE_1K
    }

    fn read_chr(&self, addr: u16) -> u8 {
        let (bank, offset) = self.resolve_chr_bank_and_offset(addr);
        self.chr.read_indexed(bank, offset)
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        let (bank, offset) = self.resolve_chr_bank_and_offset(addr);
        self.chr.write_indexed(bank, offset, data);
    }

    /// Map PPU address to CHR bank base + offset according to VRC6 banking mode.
    fn resolve_chr_bank_and_offset(&self, addr: u16) -> (usize, usize) {
        let bank_idx = ((addr >> 10) & 0x07) as usize;
        let offset = (addr & 0x03FF) as usize;
        let mask = if (self.banking_mode & 0x20) != 0 {
            0xFE
        } else {
            0xFF
        };
        let or_mask = if (self.banking_mode & 0x20) != 0 {
            1
        } else {
            0
        };

        let page = match self.banking_mode & 0x03 {
            0 => self.chr_regs.get(bank_idx).copied().unwrap_or(0),
            1 => {
                let reg = self.chr_regs.get(bank_idx / 2).copied().unwrap_or(0);
                if (bank_idx & 0x01) == 0 {
                    reg & mask
                } else {
                    (reg & mask) | or_mask
                }
            }
            _ => {
                // Mode 2/3: first 4 banks direct, last 4 use reg4/reg5 pairs.
                if bank_idx < 4 {
                    self.chr_regs.get(bank_idx).copied().unwrap_or(0)
                } else {
                    let reg = if bank_idx < 6 {
                        self.chr_regs.get(4).copied().unwrap_or(0)
                    } else {
                        self.chr_regs.get(5).copied().unwrap_or(0)
                    };
                    if (bank_idx & 0x01) == 0 {
                        reg & mask
                    } else {
                        (reg & mask) | or_mask
                    }
                }
            }
        };

        (page as usize * CHR_BANK_SIZE_1K, offset)
    }

    fn update_prg_bank_8000(&mut self, value: u8) {
        self.prg_bank_8000_2x = Some(((value & 0x0F) as usize) << 1);
    }

    fn update_prg_bank_c000(&mut self, value: u8) {
        self.prg_bank_c000 = Some(self.prg_bank_index(value & 0x1F));
    }

    #[inline]
    fn prg_bank_index(&self, reg_value: u8) -> usize {
        if self.prg_bank_count_8k == 0 {
            0
        } else {
            (reg_value as usize) % self.prg_bank_count_8k
        }
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        if let Some(reg) = Vrc6CpuRegister::from_addr(addr) {
            use Vrc6CpuRegister::*;

            match reg {
                PrgBank8000_2x => {
                    self.update_prg_bank_8000(value);
                }
                ExpansionAudio => {
                    // Expansion audio registers ($9000-$B002) are accepted but
                    // ignored for now; integration with an ExpansionAudio
                    // implementation can extend this in the future.
                }
                Control => {
                    self.banking_mode = value;
                    self.prg_ram_gate_initialized = true;
                    self.update_mirroring();
                }
                PrgBankC000 => {
                    self.update_prg_bank_c000(value);
                }
                ChrBankLow => {
                    let idx = (addr & 0x0003) as usize;
                    self.chr_regs[idx] = value;
                    self.prg_ram_gate_initialized = true;
                }
                ChrBankHigh => {
                    let idx = 4 + (addr & 0x0003) as usize;
                    self.chr_regs[idx] = value;
                    self.prg_ram_gate_initialized = true;
                }
                IrqReload => {
                    self.irq_reload = value;
                }
                IrqControl => {
                    self.irq_enabled_after_ack = (value & 0x01) != 0;
                    self.irq_enabled = (value & 0x02) != 0;
                    self.irq_cycle_mode = (value & 0x04) != 0;
                    if self.irq_enabled {
                        self.irq_counter = self.irq_reload;
                        self.irq_prescaler = 341;
                    }
                    self.irq_pending = false;
                }
                IrqAck => {
                    self.irq_enabled = self.irq_enabled_after_ack;
                    self.irq_pending = false;
                }
            }
        }
    }

    fn update_mirroring(&mut self) {
        if (self.banking_mode & 0x10) != 0 {
            // CHR-ROM nametable mode is mapper-controlled.
            self.mirroring = Mirroring::MapperControlled;
            return;
        }

        self.mirroring = match self.banking_mode & 0x2F {
            0x20 | 0x27 => Mirroring::Vertical,
            0x23 | 0x24 => Mirroring::Horizontal,
            0x28 | 0x2F => Mirroring::SingleScreenLower,
            0x2B | 0x2C => Mirroring::SingleScreenUpper,
            _ => Mirroring::MapperControlled,
        };
    }

    #[inline]
    fn chr_nt_page_default(&self, nt: usize) -> u8 {
        match self.banking_mode & 0x07 {
            0 | 6 | 7 => {
                if nt < 2 {
                    self.chr_regs.get(6).copied().unwrap_or(0)
                } else {
                    self.chr_regs.get(7).copied().unwrap_or(0)
                }
            }
            1 | 5 => match nt {
                0 => self.chr_regs.get(4).copied().unwrap_or(0),
                1 => self.chr_regs.get(5).copied().unwrap_or(0),
                2 => self.chr_regs.get(6).copied().unwrap_or(0),
                _ => self.chr_regs.get(7).copied().unwrap_or(0),
            },
            _ => {
                if nt == 1 || nt == 3 {
                    self.chr_regs.get(7).copied().unwrap_or(0)
                } else {
                    self.chr_regs.get(6).copied().unwrap_or(0)
                }
            }
        }
    }

    #[inline]
    fn chr_nt_page_special(&self, nt: usize) -> Option<u8> {
        let reg6 = self.chr_regs.get(6).copied().unwrap_or(0);
        let reg7 = self.chr_regs.get(7).copied().unwrap_or(0);
        let reg6_even = reg6 & 0xFE;
        let reg7_even = reg7 & 0xFE;

        match self.banking_mode & 0x2F {
            0x20 | 0x27 => Some(match nt {
                0 => reg6_even,
                1 => reg6_even | 1,
                2 => reg7_even,
                _ => reg7_even | 1,
            }),
            0x23 | 0x24 => Some(match nt {
                0 => reg6_even,
                1 => reg7_even,
                2 => reg6_even | 1,
                _ => reg7_even | 1,
            }),
            0x28 | 0x2F => Some(match nt {
                0 | 1 => reg6_even,
                _ => reg7_even,
            }),
            0x2B | 0x2C => Some(match nt {
                0 | 2 => reg6_even | 1,
                _ => reg7_even | 1,
            }),
            _ => None,
        }
    }

    #[inline]
    fn ciram_nt_page_special(&self, nt: usize) -> Option<u8> {
        match self.banking_mode & 0x2F {
            0x20 | 0x27 => Some(if nt == 0 || nt == 2 { 0 } else { 1 }),
            0x23 | 0x24 => Some(if nt <= 1 { 0 } else { 1 }),
            0x28 | 0x2F => Some(0),
            0x2B | 0x2C => Some(1),
            _ => None,
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

impl Mapper for Mapper26 {
    fn hook_mask(&self) -> MapperHookMask {
        MapperHookMask::CPU_CLOCK
    }

    fn on_mapper_event(&mut self, event: MapperEvent) {
        if !matches!(event, MapperEvent::CpuClock { .. }) {
            return;
        }

        if self.irq_enabled {
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
    }

    fn reset(&mut self, _kind: ResetKind) {
        self.prg_bank_8000_2x = None;
        self.prg_bank_c000 = None;
        self.banking_mode = 0;
        self.prg_ram_gate_initialized = false;
        self.chr_regs.fill(0);
        self.mirroring = self.base_mirroring;

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
            cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END => self.read_prg_rom(addr),
            _ => None,
        }
    }

    fn cpu_write(&mut self, addr: u16, data: u8, _cpu_cycle: u64) {
        match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.write_prg_ram(addr, data),
            VRC6_IO_WINDOW_START..=VRC6_IO_WINDOW_END => {
                let translated = self.translate_address(addr);
                self.write_register(translated, data);
            }
            _ => {}
        }
    }

    fn ppu_read(&self, addr: u16) -> Option<u8> {
        Some(self.read_chr(addr))
    }

    fn ppu_write(&mut self, addr: u16, data: u8) {
        self.write_chr(addr, data);
    }

    fn map_nametable(&self, addr: u16) -> NametableTarget {
        let base = addr & 0x0FFF;
        let nt = ((base >> 10) & 0x03) as usize;
        let within = base & 0x03FF;

        if (self.banking_mode & 0x10) != 0 {
            let page = self
                .chr_nt_page_special(nt)
                .unwrap_or_else(|| self.chr_nt_page_default(nt));
            let offset = ((page as u16) << 10) | within;
            return NametableTarget::MapperVram(offset);
        }

        let page = self
            .ciram_nt_page_special(nt)
            .unwrap_or_else(|| self.chr_nt_page_default(nt) & 0x01);

        NametableTarget::Ciram(((page as u16) << 10) | within)
    }

    fn mapper_nametable_read(&self, offset: u16) -> u8 {
        let page = ((offset >> 10) & 0x3F) as usize;
        let within = (offset & 0x03FF) as usize;
        self.chr.read_indexed(page * CHR_BANK_SIZE_1K, within)
    }

    fn mapper_nametable_write(&mut self, offset: u16, value: u8) {
        let page = ((offset >> 10) & 0x3F) as usize;
        let within = (offset & 0x03FF) as usize;
        self.chr
            .write_indexed(page * CHR_BANK_SIZE_1K, within, value);
    }

    fn irq_pending(&self) -> bool {
        self.irq_pending
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
        26
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Konami VRC6b")
    }
}
