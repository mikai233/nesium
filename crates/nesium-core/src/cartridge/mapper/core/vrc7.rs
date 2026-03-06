use crate::{
    cartridge::{PrgRom, header::Mirroring, mapper::ChrStorage},
    mem_block::ByteBlock,
};

const PRG_BANK_SIZE_8K: usize = 8 * 1024;
const CHR_BANK_SIZE_1K: usize = 1024;

const REG_ADDR_MASK: u16 = 0xF038;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Vrc7Register {
    PrgBank8000,
    PrgBankA000,
    PrgBankC000,
    AudioSelect,
    AudioData,
    ChrBank,
    Control,
    IrqReload,
    IrqControl,
    IrqAck,
}

impl Vrc7Register {
    pub fn from_addr(addr: u16) -> Option<Self> {
        use Vrc7Register::*;

        match addr & REG_ADDR_MASK {
            0x8000 => Some(PrgBank8000),
            0x8008 => Some(PrgBankA000),
            0x9000 => Some(PrgBankC000),
            0x9010 => Some(AudioSelect),
            0x9030 => Some(AudioData),
            0xA000 | 0xA008 | 0xB000 | 0xB008 | 0xC000 | 0xC008 | 0xD000 | 0xD008 => Some(ChrBank),
            0xE000 => Some(Control),
            0xE008 => Some(IrqReload),
            0xF000 => Some(IrqControl),
            0xF008 => Some(IrqAck),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Vrc7Board {
    prg_bank_count_8k: usize,
    prg_banks: ByteBlock<3>,
    chr_banks: ByteBlock<8>,
    control: u8,
    mirroring: Mirroring,
    base_mirroring: Mirroring,
}

impl Vrc7Board {
    pub fn new(prg_rom: &PrgRom, base_mirroring: Mirroring) -> Self {
        Self {
            prg_bank_count_8k: (prg_rom.len() / PRG_BANK_SIZE_8K).max(1),
            prg_banks: ByteBlock::new(),
            chr_banks: ByteBlock::new(),
            control: 0,
            mirroring: base_mirroring,
            base_mirroring,
        }
    }

    pub fn reset(&mut self) {
        self.prg_banks.fill(0);
        self.chr_banks.fill(0);
        self.control = 0;
        self.mirroring = self.base_mirroring;
    }

    pub fn translate_address(&self, addr: u16) -> u16 {
        if (addr & 0x10) != 0 && (addr & 0xF010) != 0x9010 {
            (addr | 0x0008) & !0x0010
        } else {
            addr
        }
    }

    pub fn prg_ram_enabled(&self, has_prg_ram: bool) -> bool {
        has_prg_ram && (self.control & 0x80) != 0
    }

    pub fn read_prg_rom(&self, prg_rom: &PrgRom, addr: u16) -> u8 {
        if prg_rom.is_empty() {
            return 0;
        }

        let bank = match addr {
            0x8000..=0x9FFF => self.prg_bank_index(self.prg_banks[0]),
            0xA000..=0xBFFF => self.prg_bank_index(self.prg_banks[1]),
            0xC000..=0xDFFF => self.prg_bank_index(self.prg_banks[2]),
            0xE000..=0xFFFF => self.prg_bank_count_8k.saturating_sub(1),
            _ => 0,
        };

        let offset = (addr & 0x1FFF) as usize;
        let base = bank.saturating_mul(PRG_BANK_SIZE_8K);
        prg_rom.get(base + offset).copied().unwrap_or(0)
    }

    pub fn read_chr(&self, chr: &ChrStorage, addr: u16) -> u8 {
        let bank = ((addr >> 10) & 0x07) as usize;
        let offset = (addr & 0x03FF) as usize;
        chr.read_indexed(self.chr_page_base(bank), offset)
    }

    pub fn write_chr(&self, chr: &mut ChrStorage, addr: u16, data: u8) {
        let bank = ((addr >> 10) & 0x07) as usize;
        let offset = (addr & 0x03FF) as usize;
        chr.write_indexed(self.chr_page_base(bank), offset, data);
    }

    pub fn write_prg_bank(&mut self, slot: usize, value: u8) {
        if slot < 3 {
            self.prg_banks[slot] = value & 0x3F;
        }
    }

    pub fn write_chr_bank_by_addr(&mut self, addr: u16, value: u8) {
        let index = match addr & REG_ADDR_MASK {
            0xA000 => 0,
            0xA008 => 1,
            0xB000 => 2,
            0xB008 => 3,
            0xC000 => 4,
            0xC008 => 5,
            0xD000 => 6,
            0xD008 => 7,
            _ => return,
        };
        self.chr_banks[index] = value;
    }

    pub fn set_control(&mut self, value: u8) {
        self.control = value;
        self.mirroring = match value & 0x03 {
            0 => Mirroring::Vertical,
            1 => Mirroring::Horizontal,
            2 => Mirroring::SingleScreenLower,
            _ => Mirroring::SingleScreenUpper,
        };
    }

    pub fn control(&self) -> u8 {
        self.control
    }

    pub fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    fn prg_bank_index(&self, value: u8) -> usize {
        if self.prg_bank_count_8k == 0 {
            0
        } else {
            (value as usize) % self.prg_bank_count_8k
        }
    }

    fn chr_page_base(&self, bank: usize) -> usize {
        self.chr_banks.get(bank).copied().unwrap_or(0) as usize * CHR_BANK_SIZE_1K
    }
}
