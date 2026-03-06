use crate::{
    cartridge::{
        PrgRom,
        header::Mirroring,
        mapper::{ChrStorage, NametableTarget},
    },
    mem_block::ByteBlock,
};

const PRG_BANK_SIZE_8K: usize = 8 * 1024;
const CHR_BANK_SIZE_1K: usize = 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Vrc6Register {
    PrgBank8000_2x,
    ExpansionAudio,
    Control,
    PrgBankC000,
    ChrBankLow,
    ChrBankHigh,
    IrqReload,
    IrqControl,
    IrqAck,
}

impl Vrc6Register {
    pub fn from_addr(addr: u16) -> Option<Self> {
        use Vrc6Register::*;

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
pub struct Vrc6Board {
    prg_bank_count_8k: usize,
    prg_bank_8000_2x: Option<usize>,
    prg_bank_c000: Option<usize>,
    banking_mode: u8,
    prg_ram_gate_initialized: bool,
    chr_regs: ByteBlock<8>,
    mirroring: Mirroring,
    base_mirroring: Mirroring,
}

impl Vrc6Board {
    pub fn new(prg_rom: &PrgRom, base_mirroring: Mirroring) -> Self {
        Self {
            prg_bank_count_8k: (prg_rom.len() / PRG_BANK_SIZE_8K).max(1),
            prg_bank_8000_2x: None,
            prg_bank_c000: None,
            banking_mode: 0,
            prg_ram_gate_initialized: false,
            chr_regs: ByteBlock::new(),
            mirroring: base_mirroring,
            base_mirroring,
        }
    }

    pub fn reset(&mut self) {
        self.prg_bank_8000_2x = None;
        self.prg_bank_c000 = None;
        self.banking_mode = 0;
        self.prg_ram_gate_initialized = false;
        self.chr_regs.fill(0);
        self.mirroring = self.base_mirroring;
    }

    pub fn translate_address(&self, addr: u16) -> u16 {
        (addr & 0xFFFC) | ((addr & 0x0001) << 1) | ((addr & 0x0002) >> 1)
    }

    pub fn prg_ram_enabled(&self, has_prg_ram: bool) -> bool {
        if !has_prg_ram {
            return false;
        }
        if !self.prg_ram_gate_initialized {
            true
        } else {
            (self.banking_mode & 0x80) != 0
        }
    }

    pub fn read_prg_rom(&self, prg_rom: &PrgRom, addr: u16) -> u8 {
        if prg_rom.is_empty() {
            return 0;
        }

        let bank = match addr {
            0x8000..=0x9FFF => self.prg_bank_8000_2x,
            0xA000..=0xBFFF => self.prg_bank_8000_2x.map(|bank| bank.saturating_add(1)),
            0xC000..=0xDFFF => self.prg_bank_c000,
            0xE000..=0xFFFF => Some(self.prg_bank_count_8k.saturating_sub(1)),
            _ => None,
        };

        let Some(bank) = bank else {
            return 0;
        };
        let bank = bank % self.prg_bank_count_8k;
        let offset = (addr & 0x1FFF) as usize;
        let base = bank.saturating_mul(PRG_BANK_SIZE_8K);
        prg_rom.get(base + offset).copied().unwrap_or(0)
    }

    pub fn read_chr(&self, chr: &ChrStorage, addr: u16) -> u8 {
        let (base, offset) = self.resolve_chr_bank_and_offset(addr);
        chr.read_indexed(base, offset)
    }

    pub fn write_chr(&self, chr: &mut ChrStorage, addr: u16, data: u8) {
        let (base, offset) = self.resolve_chr_bank_and_offset(addr);
        chr.write_indexed(base, offset, data);
    }

    pub fn write_prg_bank_8000(&mut self, value: u8) {
        self.prg_bank_8000_2x = Some(((value & 0x0F) as usize) << 1);
    }

    pub fn write_prg_bank_c000(&mut self, value: u8) {
        self.prg_bank_c000 = Some(self.prg_bank_index(value & 0x1F));
    }

    pub fn write_control(&mut self, value: u8) {
        self.banking_mode = value;
        self.prg_ram_gate_initialized = true;
        self.update_mirroring();
    }

    pub fn write_chr_low(&mut self, index: usize, value: u8) {
        if index < 4 {
            self.chr_regs[index] = value;
            self.prg_ram_gate_initialized = true;
        }
    }

    pub fn write_chr_high(&mut self, index: usize, value: u8) {
        if index < 4 {
            self.chr_regs[4 + index] = value;
            self.prg_ram_gate_initialized = true;
        }
    }

    pub fn mirroring(&self) -> Mirroring {
        self.mirroring
    }

    pub fn map_nametable(&self, addr: u16) -> NametableTarget {
        let base = addr & 0x0FFF;
        let nt = ((base >> 10) & 0x03) as usize;
        let within = base & 0x03FF;

        if (self.banking_mode & 0x10) != 0 {
            let page = self
                .chr_nt_page_special(nt)
                .unwrap_or_else(|| self.chr_nt_page_default(nt));
            let offset = ((page as u32) << 10) | u32::from(within);
            return NametableTarget::MapperVram(offset);
        }

        let page = self
            .ciram_nt_page_special(nt)
            .unwrap_or_else(|| self.chr_nt_page_default(nt) & 0x01);

        NametableTarget::Ciram(((page as u16) << 10) | within)
    }

    pub fn mapper_nametable_read(&self, chr: &ChrStorage, offset: u32) -> u8 {
        let page = (offset >> 10) as usize;
        let within = (offset as usize) & 0x03FF;
        chr.read_indexed(page * CHR_BANK_SIZE_1K, within)
    }

    pub fn mapper_nametable_write(&self, chr: &mut ChrStorage, offset: u32, value: u8) {
        let page = (offset >> 10) as usize;
        let within = (offset as usize) & 0x03FF;
        chr.write_indexed(page * CHR_BANK_SIZE_1K, within, value);
    }

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

    fn prg_bank_index(&self, reg_value: u8) -> usize {
        if self.prg_bank_count_8k == 0 {
            0
        } else {
            (reg_value as usize) % self.prg_bank_count_8k
        }
    }

    fn update_mirroring(&mut self) {
        if (self.banking_mode & 0x10) != 0 {
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

    fn ciram_nt_page_special(&self, nt: usize) -> Option<u8> {
        match self.banking_mode & 0x2F {
            0x20 | 0x27 => Some(if nt == 0 || nt == 2 { 0 } else { 1 }),
            0x23 | 0x24 => Some(if nt <= 1 { 0 } else { 1 }),
            0x28 | 0x2F => Some(0),
            0x2B | 0x2C => Some(1),
            _ => None,
        }
    }
}
