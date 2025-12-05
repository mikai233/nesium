//! Mapper 21 – Konami VRC4 (VRC4a/VRC4c) implementation.
//!
//! This mapper provides:
//! - Two switchable 8 KiB PRG banks and two fixed banks (second‑last and last).
//! - Eight 1 KiB CHR banks with split low/high nibble registers.
//! - Mapper‑controlled nametable mirroring.
//! - An IRQ counter modelled after Mesen2's `VrcIrq` (prescaler + reloadable 8‑bit
//!   counter with optional CPU‑cycle mode).
//!
//! | Area | Address range     | Behaviour                                          | IRQ/Audio   |
//! |------|-------------------|----------------------------------------------------|-------------|
//! | CPU  | `$6000-$7FFF`     | Optional PRG-RAM                                   | None        |
//! | CPU  | `$8000-$DFFF`     | Two switchable 8 KiB PRG banks + one fixed window  | None        |
//! | CPU  | `$8000-$E003`     | PRG/CHR/mirroring/IRQ registers (after translation)| VRC4 IRQ    |
//! | CPU  | `$E000-$FFFF`     | Fixed 8 KiB PRG (last) + IRQ control/ack          | VRC4 IRQ    |
//! | PPU  | `$0000-$1FFF`     | Eight 1 KiB CHR banks with split low/high nibbles  | None        |
//! | PPU  | `$2000-$3EFF`     | Mirroring controlled by VRC4 register              | None        |

use std::borrow::Cow;

use crate::{
    cartridge::{
        ChrRom, Mapper, PrgRom, TrainerBytes,
        header::{Header, Mirroring},
        mapper::{ChrStorage, allocate_prg_ram_with_trainer, select_chr_storage},
    },
    memory::cpu as cpu_mem,
};

use crate::mem_block::ByteBlock;

/// PRG-ROM banking granularity (8 KiB).
const PRG_BANK_SIZE_8K: usize = 8 * 1024;
/// CHR banking granularity (1 KiB).
const CHR_BANK_SIZE_1K: usize = 1024;

/// CPU `$8000-$9FFF/$A000-$BFFF/$C000-$DFFF/$E000-$FFFF`: four 8 KiB PRG windows.
const VRC4_PRG_SLOT0_START: u16 = 0x8000;
const VRC4_PRG_SLOT0_END: u16 = 0x9FFF;
const VRC4_PRG_SLOT1_START: u16 = 0xA000;
const VRC4_PRG_SLOT1_END: u16 = 0xBFFF;
const VRC4_PRG_SLOT2_START: u16 = 0xC000;
const VRC4_PRG_SLOT2_END: u16 = 0xDFFF;
const VRC4_PRG_FIXED_SLOT_START: u16 = 0xE000;
const VRC4_PRG_FIXED_SLOT_END: u16 = 0xFFFF;

/// VRC4 register decode addresses after `translate_address` and masking:
/// - `$8000-$8006`: PRG bank for `$8000-$9FFF`.
/// - `$9000-$9001`: mirroring control.
/// - `$9002-$9003`: PRG mode (swap at `$C000`) and IRQ mode bits.
/// - `$A000-$A006`: PRG bank for `$A000-$BFFF`.
/// - `$B000-$E006`: CHR bank low/high nibbles.
/// - `$F000-$F003`: IRQ reload low/high, IRQ control and acknowledge.
const VRC4_REG_PRG_BANK_8000_START: u16 = 0x8000;
const VRC4_REG_PRG_BANK_8000_END: u16 = 0x8006;
const VRC4_REG_MIRRORING_START: u16 = 0x9000;
const VRC4_REG_MIRRORING_END: u16 = 0x9001;
const VRC4_REG_MODE_START: u16 = 0x9002;
const VRC4_REG_MODE_END: u16 = 0x9003;
const VRC4_REG_PRG_BANK_A000_START: u16 = 0xA000;
const VRC4_REG_PRG_BANK_A000_END: u16 = 0xA006;
const VRC4_REG_CHR_BANK_FIRST: u16 = 0xB000;
const VRC4_REG_CHR_BANK_LAST: u16 = 0xE006;
const VRC4_REG_IRQ_RELOAD_LOW: u16 = 0xF000;
const VRC4_REG_IRQ_RELOAD_HIGH: u16 = 0xF001;
const VRC4_REG_IRQ_CONTROL: u16 = 0xF002;
const VRC4_REG_IRQ_ACK: u16 = 0xF003;

/// CPU `$8000-$FFFF`: VRC4 register I/O window.
const VRC4_IO_WINDOW_START: u16 = 0x8000;
const VRC4_IO_WINDOW_END: u16 = 0xFFFF;

/// CPU-visible VRC4 register set after address translation.
///
/// VRC4 exposes a compact set of control registers spread across the
/// `$8000-$FFFF` window. After `translate_address` has normalised the wiring
/// differences, these logical registers behave consistently across variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Vrc4CpuRegister {
    /// `$8000-$8006` – PRG bank for `$8000-$9FFF`.
    PrgBank8000,
    /// `$9000-$9001` – nametable mirroring control.
    Mirroring,
    /// `$9002-$9003` – PRG mode / IRQ mode bits.
    Mode,
    /// `$A000-$A006` – PRG bank for `$A000-$BFFF`.
    PrgBankA000,
    /// `$B000-$E006` – CHR bank low/high nibble registers.
    ChrBank,
    /// `$F000` – IRQ reload low nibble.
    IrqReloadLow,
    /// `$F001` – IRQ reload high nibble.
    IrqReloadHigh,
    /// `$F002` – IRQ control (enable/mode).
    IrqControl,
    /// `$F003` – IRQ acknowledge / re-enable.
    IrqAck,
}

impl Vrc4CpuRegister {
    fn from_addr(addr: u16) -> Option<Self> {
        use Vrc4CpuRegister::*;

        match addr {
            VRC4_REG_PRG_BANK_8000_START..=VRC4_REG_PRG_BANK_8000_END => Some(PrgBank8000),
            VRC4_REG_MIRRORING_START..=VRC4_REG_MIRRORING_END => Some(Mirroring),
            VRC4_REG_MODE_START..=VRC4_REG_MODE_END => Some(Mode),
            VRC4_REG_PRG_BANK_A000_START..=VRC4_REG_PRG_BANK_A000_END => Some(PrgBankA000),
            VRC4_REG_CHR_BANK_FIRST..=VRC4_REG_CHR_BANK_LAST => Some(ChrBank),
            VRC4_REG_IRQ_RELOAD_LOW => Some(IrqReloadLow),
            VRC4_REG_IRQ_RELOAD_HIGH => Some(IrqReloadHigh),
            VRC4_REG_IRQ_CONTROL => Some(IrqControl),
            VRC4_REG_IRQ_ACK => Some(IrqAck),
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Vrc4Variant {
    /// Standard VRC4a address wiring (mapper 21 submapper 0/1).
    Vrc4a,
    /// VRC4c address wiring (mapper 21 submapper 2).
    Vrc4c,
}

#[derive(Debug, Clone)]
pub struct Mapper21 {
    prg_rom: PrgRom,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,

    /// Number of 8 KiB PRG-ROM banks.
    prg_bank_count_8k: usize,

    /// Switchable 8 KiB PRG bank mapped at `$8000-$9FFF` (or `$C000-$DFFF`
    /// when `prg_mode_swap` is true).
    prg_bank_8000: u8,
    /// Switchable 8 KiB PRG bank mapped at `$A000-$BFFF`.
    prg_bank_a000: u8,
    /// PRG mode bit (0: swap at `$8000`, 1: swap at `$C000`), matching MMC3/VRC4.
    prg_mode_swap: bool,

    /// Low 4 bits of each 1 KiB CHR bank register.
    chr_low_regs: Mapper21ChrLowRegs,
    /// High 5 bits of each 1 KiB CHR bank register.
    chr_high_regs: Mapper21ChrHighRegs,

    /// Current nametable mirroring configuration.
    mirroring: Mirroring,
    /// Power-on mirroring from the header so reset can restore it.
    base_mirroring: Mirroring,

    // IRQ state ------------------------------------------------------------
    irq_reload: u8,
    irq_counter: u8,
    irq_prescaler: i32,
    irq_enabled: bool,
    irq_enabled_after_ack: bool,
    irq_cycle_mode: bool,
    irq_pending: bool,

    /// Address decoding variant and optional heuristic mode that ORs both
    /// VRC4a/VRC4c address line layouts (Mesen2 behaviour when submapper == 0).
    variant: Vrc4Variant,
    use_heuristics: bool,
}

type Mapper21ChrLowRegs = ByteBlock<8>;
type Mapper21ChrHighRegs = ByteBlock<8>;

impl Mapper21 {
    pub fn new(header: Header, prg_rom: PrgRom, chr_rom: ChrRom, trainer: TrainerBytes) -> Self {
        let prg_ram = allocate_prg_ram_with_trainer(&header, trainer);

        let chr = select_chr_storage(&header, chr_rom);
        let prg_bank_count_8k = (prg_rom.len() / PRG_BANK_SIZE_8K).max(1);

        let variant = match header.submapper {
            2 => Vrc4Variant::Vrc4c,
            _ => Vrc4Variant::Vrc4a,
        };
        let use_heuristics = header.submapper == 0;

        Self {
            prg_rom,
            prg_ram,
            chr,
            prg_bank_count_8k,
            prg_bank_8000: 0,
            prg_bank_a000: 0,
            prg_mode_swap: false,
            chr_low_regs: Mapper21ChrLowRegs::new(),
            chr_high_regs: Mapper21ChrHighRegs::new(),
            mirroring: header.mirroring,
            base_mirroring: header.mirroring,
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

    /// Translate the CPU address into the VRC4 register layout, emulating the
    /// A0/A1 pin permutations documented on Nesdev and mirrored in Mesen2.
    fn translate_address(&self, addr: u16) -> u16 {
        let (mut a0, mut a1) = if self.use_heuristics {
            // Heuristic mode ORs both possible wirings to maximise compatibility
            // for submapper 0 ROMs.
            let base_a0 = (addr >> 1) & 0x01;
            let base_a1 = (addr >> 2) & 0x01;
            match self.variant {
                Vrc4Variant::Vrc4a | Vrc4Variant::Vrc4c => {
                    let alt_a0 = (addr >> 6) & 0x01;
                    let alt_a1 = (addr >> 7) & 0x01;
                    (base_a0 | alt_a0, base_a1 | alt_a1)
                }
            }
        } else {
            match self.variant {
                Vrc4Variant::Vrc4a => ((addr >> 1) & 0x01, (addr >> 2) & 0x01),
                Vrc4Variant::Vrc4c => ((addr >> 6) & 0x01, (addr >> 7) & 0x01),
            }
        };

        a0 &= 0x01;
        a1 &= 0x01;
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

        match (self.prg_mode_swap, addr) {
            // Mode 0: switchable at $8000/$A000, fixed at $C000(second last)/$E000(last).
            (false, VRC4_PRG_SLOT0_START..=VRC4_PRG_SLOT0_END) => {
                self.prg_bank_index(self.prg_bank_8000)
            }
            (false, VRC4_PRG_SLOT1_START..=VRC4_PRG_SLOT1_END) => {
                self.prg_bank_index(self.prg_bank_a000)
            }
            (false, VRC4_PRG_SLOT2_START..=VRC4_PRG_SLOT2_END) => second_last,
            (false, _) => last,

            // Mode 1: fixed second-last at $8000, switchable $A000 + $C000 swapped.
            (true, VRC4_PRG_SLOT0_START..=VRC4_PRG_SLOT0_END) => second_last,
            (true, VRC4_PRG_SLOT1_START..=VRC4_PRG_SLOT1_END) => {
                self.prg_bank_index(self.prg_bank_a000)
            }
            (true, VRC4_PRG_SLOT2_START..=VRC4_PRG_SLOT2_END) => {
                self.prg_bank_index(self.prg_bank_8000)
            }
            (true, _) => last,
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
        self.mirroring = match value & 0x03 {
            0 => Mirroring::Vertical,
            1 => Mirroring::Horizontal,
            2 => Mirroring::SingleScreenLower,
            _ => Mirroring::SingleScreenUpper,
        };
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        if let Some(reg) = Vrc4CpuRegister::from_addr(addr) {
            match reg {
                Vrc4CpuRegister::PrgBank8000 => {
                    self.prg_bank_8000 = value & 0x1F;
                }
                Vrc4CpuRegister::Mirroring => {
                    self.set_mirroring_from_value(value);
                }
                Vrc4CpuRegister::Mode => {
                    self.prg_mode_swap = (value & 0x02) != 0;
                }
                Vrc4CpuRegister::PrgBankA000 => {
                    self.prg_bank_a000 = value & 0x1F;
                }
                Vrc4CpuRegister::ChrBank => {
                    // Eight 1 KiB CHR banks spread across B/C/D/E regions.
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
                Vrc4CpuRegister::IrqReloadLow => {
                    self.irq_reload = (self.irq_reload & 0xF0) | (value & 0x0F);
                }
                Vrc4CpuRegister::IrqReloadHigh => {
                    self.irq_reload = (self.irq_reload & 0x0F) | ((value & 0x0F) << 4);
                }
                Vrc4CpuRegister::IrqControl => {
                    self.irq_enabled_after_ack = (value & 0x01) != 0;
                    self.irq_enabled = (value & 0x02) != 0;
                    self.irq_cycle_mode = (value & 0x04) != 0;
                    if self.irq_enabled {
                        self.irq_counter = self.irq_reload;
                        self.irq_prescaler = 341;
                        self.irq_pending = false;
                    }
                }
                Vrc4CpuRegister::IrqAck => {
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

impl Mapper for Mapper21 {
    fn power_on(&mut self) {
        // Basic VRC4 defaults: mirroring from header, PRG mode 0, IRQ disabled.
        self.prg_mode_swap = false;
        self.prg_bank_8000 = 0;
        self.prg_bank_a000 = 1;
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

        // Keep the header-provided mirroring until the game selects otherwise.
        // (Some dumps ship with single-screen headers even when they later
        // configure mirroring via $9000.)
        // Mirroring is left unchanged here.
    }

    fn reset(&mut self) {
        self.power_on();
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
            VRC4_IO_WINDOW_START..=VRC4_IO_WINDOW_END => {
                let translated = self.translate_address(addr) & 0xF00F;
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
        21
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Konami VRC4")
    }
}
