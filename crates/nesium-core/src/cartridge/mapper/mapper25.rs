//! Mapper 25 – Konami VRC4b / VRC4d / VRC2c implementation.
//!
//! This mapper family mirrors VRC4 behaviour: two switchable 8 KiB PRG banks,
//! two fixed banks, eight 1 KiB CHR banks with split low/high nibbles,
//! mapper-controlled mirroring, and (for VRC4) an IRQ counter. VRC2c lacks
//! the PRG mode bit and IRQ; address-line permutations differ between VRC4b
//! and VRC4d. Submapper 0 enables a heuristic that ORs both layouts for
//! ambiguous dumps, matching Mesen2.
//!
//! | Area | Address range     | Behaviour                                          | IRQ/Audio              |
//! |------|-------------------|----------------------------------------------------|------------------------|
//! | CPU  | `$6000-$7FFF`     | Optional PRG-RAM                                   | None                   |
//! | CPU  | `$8000-$DFFF`     | Two switchable 8 KiB PRG banks + fixed window      | None                   |
//! | CPU  | `$8000-$FFFF`     | PRG/CHR/mirroring/IRQ registers (after translation)| VRC4 IRQ (VRC4x only) |
//! | PPU  | `$0000-$1FFF`     | Eight 1 KiB CHR banks with split low/high nibbles  | None                   |
//! | PPU  | `$2000-$3EFF`     | Mirroring from VRC4b/VRC4d/VRC2c register          | None                   |

use std::borrow::Cow;

use crate::{
    cartridge::{
        ChrRom, Mapper, PrgRom, TrainerBytes,
        header::{Header, Mirroring},
        mapper::{
            ChrStorage, MapperEvent, MapperHookMask, allocate_prg_ram_with_trainer,
            select_chr_storage,
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

/// CPU `$8000-$FFFF`: VRC4b/VRC4d/VRC2c register I/O window. Writes in this
/// range, after address translation, hit PRG/CHR/mirroring/IRQ registers.
const VRC25_IO_WINDOW_START: u16 = 0x8000;
const VRC25_IO_WINDOW_END: u16 = 0xFFFF;

/// CPU-visible VRC4b/VRC4d/VRC2c register set after address translation.
///
/// This closely mirrors the VRC4 layout: VRC2c simply lacks the IRQ/control
/// behaviour. Using an enum makes it easier to see which logical register a
/// given translated address targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Vrc25CpuRegister {
    /// `$8000-$8006` – PRG bank for `$8000-$9FFF`.
    PrgBank8000,
    /// `$9000-$9001` – nametable mirroring control.
    Mirroring,
    /// `$9002-$9003` – PRG mode / IRQ mode bits (VRC4b/VRC4d) or mirroring (VRC2c).
    ModeOrMirroring,
    /// `$A000-$A006` – PRG bank for `$A000-$BFFF`.
    PrgBankA000,
    /// `$B000-$E006` – CHR bank low/high nibbles.
    ChrBank,
    /// `$F000` – IRQ reload low nibble (VRC4 only).
    IrqReloadLow,
    /// `$F001` – IRQ reload high nibble (VRC4 only).
    IrqReloadHigh,
    /// `$F002` – IRQ control (enable/mode) (VRC4 only).
    IrqControl,
    /// `$F003` – IRQ acknowledge / re-enable (VRC4 only).
    IrqAck,
}

impl Vrc25CpuRegister {
    fn from_addr(addr: u16) -> Option<Self> {
        use Vrc25CpuRegister::*;

        match addr {
            0x8000..=0x8006 => Some(PrgBank8000),
            0x9000..=0x9001 => Some(Mirroring),
            0x9002..=0x9003 => Some(ModeOrMirroring),
            0xA000..=0xA006 => Some(PrgBankA000),
            0xB000..=0xE006 => Some(ChrBank),
            0xF000 => Some(IrqReloadLow),
            0xF001 => Some(IrqReloadHigh),
            0xF002 => Some(IrqControl),
            0xF003 => Some(IrqAck),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Variant {
    /// Mapper 25 submapper 3: VRC2c (no IRQ, no PRG mode).
    Vrc2c,
    /// Mapper 25 submapper 0/1: VRC4b (IRQ + PRG mode).
    Vrc4b,
    /// Mapper 25 submapper 2: VRC4d (IRQ + PRG mode, different address lines).
    Vrc4d,
}

#[derive(Debug, Clone)]
pub struct Mapper25 {
    prg_rom: PrgRom,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,

    prg_bank_count_8k: usize,

    prg_bank_8000: u8,
    prg_bank_a000: u8,
    prg_mode_swap: bool,

    chr_low_regs: Mapper25ChrLowRegs,
    chr_high_regs: Mapper25ChrHighRegs,

    mirroring: Mirroring,
    base_mirroring: Mirroring,

    // IRQ state (VRC4 only).
    irq_reload: u8,
    irq_counter: u8,
    irq_prescaler: i32,
    irq_enabled: bool,
    irq_enabled_after_ack: bool,
    irq_cycle_mode: bool,
    irq_pending: bool,

    variant: Variant,
    /// Submapper 0 heuristic: OR both VRC4b/VRC4d address layouts.
    use_heuristics: bool,
}

type Mapper25ChrLowRegs = ByteBlock<8>;
type Mapper25ChrHighRegs = ByteBlock<8>;

impl Mapper25 {
    pub fn new(header: Header, prg_rom: PrgRom, chr_rom: ChrRom, trainer: TrainerBytes) -> Self {
        let prg_ram = allocate_prg_ram_with_trainer(&header, trainer);

        let chr = select_chr_storage(&header, chr_rom);
        let prg_bank_count_8k = (prg_rom.len() / PRG_BANK_SIZE_8K).max(1);

        let variant = match header.submapper() {
            3 => Variant::Vrc2c,
            2 => Variant::Vrc4d,
            _ => Variant::Vrc4b,
        };
        let use_heuristics = header.submapper() == 0;

        Self {
            prg_rom,
            prg_ram,
            chr,
            prg_bank_count_8k,
            prg_bank_8000: 0,
            prg_bank_a000: 0,
            prg_mode_swap: false,
            chr_low_regs: Mapper25ChrLowRegs::new(),
            chr_high_regs: Mapper25ChrHighRegs::new(),
            mirroring: header.mirroring(),
            base_mirroring: header.mirroring(),
            irq_reload: 0,
            irq_counter: 0,
            irq_prescaler: 0,
            irq_enabled: false,
            irq_enabled_after_ack: false,
            irq_cycle_mode: false,
            irq_pending: false,
            variant,
            use_heuristics,
        }
    }

    fn has_irq(&self) -> bool {
        !matches!(self.variant, Variant::Vrc2c)
    }

    fn translate_address(&self, addr: u16) -> u16 {
        // VRC4b/VRC2c: A0=addr>>1, A1=addr bit0
        // VRC4d: A0=addr>>3, A1=addr>>2
        // Heuristic mode ORs both layouts.
        let (a0, a1) = if self.use_heuristics {
            let base_a0 = (addr >> 1) & 0x01;
            let base_a1 = addr & 0x01;
            let alt_a0 = (addr >> 3) & 0x01;
            let alt_a1 = (addr >> 2) & 0x01;
            ((base_a0 | alt_a0) & 0x01, (base_a1 | alt_a1) & 0x01)
        } else {
            match self.variant {
                Variant::Vrc2c | Variant::Vrc4b => ((addr >> 1) & 0x01, addr & 0x01),
                Variant::Vrc4d => ((addr >> 3) & 0x01, (addr >> 2) & 0x01),
            }
        };
        (addr & 0xFF00) | (a1 << 1) | a0
    }

    #[inline]
    fn prg_bank_index(&self, reg_value: u8) -> usize {
        if self.prg_bank_count_8k == 0 {
            0
        } else {
            (reg_value as usize) % self.prg_bank_count_8k
        }
    }

    fn prg_bank_for_addr(&self, addr: u16) -> usize {
        let last = self.prg_bank_count_8k.saturating_sub(1);
        let second_last = self.prg_bank_count_8k.saturating_sub(2);

        if self.prg_mode_swap && self.has_irq() {
            match addr {
                0x8000..=0x9FFF => second_last,
                0xA000..=0xBFFF => self.prg_bank_index(self.prg_bank_a000),
                0xC000..=0xDFFF => self.prg_bank_index(self.prg_bank_8000),
                _ => last,
            }
        } else {
            match addr {
                0x8000..=0x9FFF => self.prg_bank_index(self.prg_bank_8000),
                0xA000..=0xBFFF => self.prg_bank_index(self.prg_bank_a000),
                0xC000..=0xDFFF => second_last,
                _ => last,
            }
        }
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }
        let bank = self.prg_bank_for_addr(addr);
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
        let lo = self.chr_low_regs.get(bank).copied().unwrap_or(0) & 0x0F;
        let hi = self.chr_high_regs.get(bank).copied().unwrap_or(0) & 0x1F;
        let page = ((hi as usize) << 4) | lo as usize;
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

    fn set_mirroring_from_value(&mut self, value: u8) {
        let mask = if matches!(self.variant, Variant::Vrc2c) && !self.use_heuristics {
            0x01
        } else {
            0x03
        };
        self.mirroring = match value & mask {
            0 => Mirroring::Vertical,
            1 => Mirroring::Horizontal,
            2 => Mirroring::SingleScreenLower,
            _ => Mirroring::SingleScreenUpper,
        };
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        if let Some(reg) = Vrc25CpuRegister::from_addr(addr) {
            use Vrc25CpuRegister::*;

            match reg {
                PrgBank8000 => {
                    self.prg_bank_8000 = value & 0x1F;
                }
                Mirroring => {
                    self.set_mirroring_from_value(value);
                }
                ModeOrMirroring => {
                    if self.has_irq() {
                        self.prg_mode_swap = (value & 0x02) != 0;
                    } else {
                        self.set_mirroring_from_value(value);
                    }
                }
                PrgBankA000 => {
                    self.prg_bank_a000 = value & 0x1F;
                }
                ChrBank => {
                    let reg_number = ((((addr >> 12) & 0x07) - 3) << 1) + ((addr >> 1) & 0x01);
                    let idx = reg_number as usize;
                    if idx < 8 {
                        if addr & 0x01 == 0 {
                            self.chr_low_regs[idx] = value & 0x0F;
                        } else {
                            self.chr_high_regs[idx] = value & 0x1F;
                        }
                    }
                }
                IrqReloadLow => {
                    if self.has_irq() {
                        self.irq_reload = (self.irq_reload & 0xF0) | (value & 0x0F);
                    }
                }
                IrqReloadHigh => {
                    if self.has_irq() {
                        self.irq_reload = (self.irq_reload & 0x0F) | ((value & 0x0F) << 4);
                    }
                }
                IrqControl => {
                    if self.has_irq() {
                        self.irq_enabled_after_ack = (value & 0x01) != 0;
                        self.irq_enabled = (value & 0x02) != 0;
                        self.irq_cycle_mode = (value & 0x04) != 0;
                        if self.irq_enabled {
                            self.irq_counter = self.irq_reload;
                            self.irq_prescaler = 341;
                            self.irq_pending = false;
                        }
                    }
                }
                IrqAck => {
                    if self.has_irq() {
                        self.irq_enabled = self.irq_enabled_after_ack;
                        self.irq_pending = false;
                    }
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

impl Mapper for Mapper25 {
    fn hook_mask(&self) -> MapperHookMask {
        MapperHookMask::CPU_BUS_ACCESS
    }

    fn on_mapper_event(&mut self, event: MapperEvent) {
        if let MapperEvent::CpuBusAccess { .. } = event {
            if !self.has_irq() || !self.irq_enabled {
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
    }

    fn reset(&mut self, _kind: ResetKind) {
        self.prg_bank_8000 = 0;
        self.prg_bank_a000 = 1;
        self.prg_mode_swap = false;
        self.chr_low_regs.fill(0);
        self.chr_high_regs.fill(0);
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
            cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END => Some(self.read_prg_rom(addr)),
            _ => None,
        }
    }

    fn cpu_write(&mut self, addr: u16, data: u8, _cpu_cycle: u64) {
        match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.write_prg_ram(addr, data),
            VRC25_IO_WINDOW_START..=VRC25_IO_WINDOW_END => {
                let translated = self.translate_address(addr) & 0xF00F;
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

    fn irq_pending(&self) -> bool {
        self.has_irq() && self.irq_pending
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
        25
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Konami VRC4b/VRC4d/VRC2c")
    }
}
