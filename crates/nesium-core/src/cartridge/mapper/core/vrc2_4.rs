use crate::{
    cartridge::mapper::core::vrc_irq::VrcIrq,
    cartridge::{PrgRom, header::Mirroring, mapper::ChrStorage},
    mem_block::ByteBlock,
    memory::cpu as cpu_mem,
};

const PRG_BANK_SIZE_8K: usize = 8 * 1024;
const CHR_BANK_SIZE_1K: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VrcAddressBits {
    pub a0_shift: u8,
    pub a1_shift: u8,
}

impl VrcAddressBits {
    pub const fn new(a0_shift: u8, a1_shift: u8) -> Self {
        Self { a0_shift, a1_shift }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Vrc2_4AddressConfig {
    pub primary: VrcAddressBits,
    pub heuristic_alt: Option<VrcAddressBits>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Vrc2_4Register {
    PrgBank8000,
    Mirroring,
    Mode,
    ModeOrMirroring,
    PrgBankA000,
    ChrBank,
    IrqReloadLow,
    IrqReloadHigh,
    IrqControl,
    IrqAck,
}

impl Vrc2_4Register {
    pub fn from_addr(addr: u16, mode_overlaps_mirroring: bool) -> Option<Self> {
        use Vrc2_4Register::*;

        match addr {
            0x8000..=0x8006 => Some(PrgBank8000),
            0x9000..=0x9001 => Some(Mirroring),
            0x9002..=0x9003 => Some(if mode_overlaps_mirroring {
                ModeOrMirroring
            } else {
                Mode
            }),
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

pub fn translate_vrc2_4_address(
    addr: u16,
    config: Vrc2_4AddressConfig,
    use_heuristics: bool,
) -> u16 {
    let (a0, a1) = if use_heuristics {
        if let Some(alt) = config.heuristic_alt {
            (
                (read_addr_line(addr, config.primary.a0_shift)
                    | read_addr_line(addr, alt.a0_shift))
                    & 0x01,
                (read_addr_line(addr, config.primary.a1_shift)
                    | read_addr_line(addr, alt.a1_shift))
                    & 0x01,
            )
        } else {
            (
                read_addr_line(addr, config.primary.a0_shift),
                read_addr_line(addr, config.primary.a1_shift),
            )
        }
    } else {
        (
            read_addr_line(addr, config.primary.a0_shift),
            read_addr_line(addr, config.primary.a1_shift),
        )
    };

    (addr & 0xFF00) | (a1 << 1) | a0
}

pub fn read_prg_ram_window(prg_ram: &[u8], addr: u16) -> Option<u8> {
    if prg_ram.is_empty() {
        return None;
    }
    let idx = (addr - cpu_mem::PRG_RAM_START) as usize % prg_ram.len();
    Some(prg_ram[idx])
}

pub fn write_prg_ram_window(prg_ram: &mut [u8], addr: u16, data: u8) {
    if prg_ram.is_empty() {
        return;
    }
    let idx = (addr - cpu_mem::PRG_RAM_START) as usize % prg_ram.len();
    prg_ram[idx] = data;
}

#[derive(Debug, Clone)]
pub struct Vrc2_4Banking {
    prg_bank_count_8k: usize,
    prg_bank_8000: u8,
    prg_bank_a000: u8,
    prg_mode_swap: bool,
    chr_low_regs: ByteBlock<8>,
    chr_high_regs: ByteBlock<8>,
    mirroring: Mirroring,
    base_mirroring: Mirroring,
}

impl Vrc2_4Banking {
    pub fn new(prg_rom: &PrgRom, base_mirroring: Mirroring) -> Self {
        Self {
            prg_bank_count_8k: (prg_rom.len() / PRG_BANK_SIZE_8K).max(1),
            prg_bank_8000: 0,
            prg_bank_a000: 0,
            prg_mode_swap: false,
            chr_low_regs: ByteBlock::new(),
            chr_high_regs: ByteBlock::new(),
            mirroring: base_mirroring,
            base_mirroring,
        }
    }

    pub fn reset(&mut self) {
        self.prg_bank_8000 = 0;
        self.prg_bank_a000 = 1;
        self.prg_mode_swap = false;
        self.chr_low_regs.fill(0);
        self.chr_high_regs.fill(0);
        self.mirroring = self.base_mirroring;
    }

    pub fn set_prg_bank_8000(&mut self, value: u8) {
        self.prg_bank_8000 = value & 0x1F;
    }

    pub fn set_prg_bank_a000(&mut self, value: u8) {
        self.prg_bank_a000 = value & 0x1F;
    }

    pub fn set_prg_mode_swap(&mut self, enabled: bool) {
        self.prg_mode_swap = enabled;
    }

    pub fn set_chr_bank_low(&mut self, bank: usize, value: u8) {
        if bank < 8 {
            self.chr_low_regs[bank] = value & 0x0F;
        }
    }

    pub fn set_chr_bank_high(&mut self, bank: usize, value: u8) {
        if bank < 8 {
            self.chr_high_regs[bank] = value & 0x1F;
        }
    }

    pub fn set_chr_bank_nibble_from_addr(&mut self, addr: u16, value: u8) {
        let reg_number = ((((addr >> 12) & 0x07) - 3) << 1) + ((addr >> 1) & 0x01);
        let idx = reg_number as usize;
        if idx >= 8 {
            return;
        }

        if addr & 0x01 == 0 {
            self.set_chr_bank_low(idx, value);
        } else {
            self.set_chr_bank_high(idx, value);
        }
    }

    pub fn set_mirroring_from_value(&mut self, value: u8, mask: u8) {
        self.mirroring = match value & mask {
            0 => Mirroring::Vertical,
            1 => Mirroring::Horizontal,
            2 => Mirroring::SingleScreenLower,
            _ => Mirroring::SingleScreenUpper,
        };
    }

    pub fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    pub fn read_prg_rom(&self, prg_rom: &PrgRom, addr: u16, allow_prg_mode_swap: bool) -> u8 {
        if prg_rom.is_empty() {
            return 0;
        }

        let bank = self.prg_bank_for_addr(addr, allow_prg_mode_swap);
        let offset = (addr & 0x1FFF) as usize;
        let base = bank.saturating_mul(PRG_BANK_SIZE_8K);
        prg_rom.get(base + offset).copied().unwrap_or(0)
    }

    pub fn read_chr(&self, chr: &ChrStorage, addr: u16) -> u8 {
        let bank = ((addr >> 10) & 0x07) as usize;
        let offset = (addr & 0x03FF) as usize;
        let base = self.chr_page_base(bank);
        chr.read_indexed(base, offset)
    }

    pub fn write_chr(&self, chr: &mut ChrStorage, addr: u16, data: u8) {
        let bank = ((addr >> 10) & 0x07) as usize;
        let offset = (addr & 0x03FF) as usize;
        let base = self.chr_page_base(bank);
        chr.write_indexed(base, offset, data);
    }

    fn prg_bank_index(&self, reg_value: u8) -> usize {
        if self.prg_bank_count_8k == 0 {
            0
        } else {
            (reg_value as usize) % self.prg_bank_count_8k
        }
    }

    fn prg_bank_for_addr(&self, addr: u16, allow_prg_mode_swap: bool) -> usize {
        let last = self.prg_bank_count_8k.saturating_sub(1);
        let second_last = self.prg_bank_count_8k.saturating_sub(2);

        if self.prg_mode_swap && allow_prg_mode_swap {
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

    fn chr_page_base(&self, bank: usize) -> usize {
        let lo = self.chr_low_regs.get(bank).copied().unwrap_or(0) & 0x0F;
        let hi = self.chr_high_regs.get(bank).copied().unwrap_or(0) & 0x1F;
        let page = ((hi as usize) << 4) | lo as usize;
        page * CHR_BANK_SIZE_1K
    }
}

pub fn write_vrc2_4_register(
    banking: &mut Vrc2_4Banking,
    mut irq: Option<&mut VrcIrq>,
    reg: Vrc2_4Register,
    addr: u16,
    value: u8,
    mirroring_mask: u8,
    mode_controls_prg_swap: bool,
) {
    use Vrc2_4Register::*;

    match reg {
        PrgBank8000 => banking.set_prg_bank_8000(value),
        Mirroring => banking.set_mirroring_from_value(value, mirroring_mask),
        Mode => banking.set_prg_mode_swap((value & 0x02) != 0),
        ModeOrMirroring => {
            if mode_controls_prg_swap {
                banking.set_prg_mode_swap((value & 0x02) != 0);
            } else {
                banking.set_mirroring_from_value(value, mirroring_mask);
            }
        }
        PrgBankA000 => banking.set_prg_bank_a000(value),
        ChrBank => banking.set_chr_bank_nibble_from_addr(addr, value),
        IrqReloadLow => {
            if let Some(irq) = irq.as_deref_mut() {
                irq.write_reload_low_nibble(value);
            }
        }
        IrqReloadHigh => {
            if let Some(irq) = irq.as_deref_mut() {
                irq.write_reload_high_nibble(value);
            }
        }
        IrqControl => {
            if let Some(irq) = irq.as_deref_mut() {
                irq.write_control(value);
            }
        }
        IrqAck => {
            if let Some(irq) = irq.as_deref_mut() {
                irq.acknowledge();
            }
        }
    }
}

#[inline]
fn read_addr_line(addr: u16, shift: u8) -> u16 {
    (addr >> shift) & 0x01
}
