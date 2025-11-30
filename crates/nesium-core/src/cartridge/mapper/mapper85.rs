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

use std::borrow::Cow;

use crate::{
    cartridge::{
        Mapper, TRAINER_SIZE,
        header::{Header, Mirroring},
        mapper::{ChrStorage, allocate_prg_ram, select_chr_storage, trainer_destination},
    },
    memory::cpu as cpu_mem,
};

/// PRG banking granularity (8 KiB).
const PRG_BANK_SIZE_8K: usize = 8 * 1024;
/// CHR banking granularity (1 KiB).
const CHR_BANK_SIZE_1K: usize = 1 * 1024;

#[derive(Debug, Clone)]
pub struct Mapper85 {
    prg_rom: Box<[u8]>,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,

    prg_bank_count_8k: usize,

    prg_banks: [u8; 3], // $8000, $A000, $C000
    chr_banks: [u8; 8],

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

impl Mapper85 {
    pub fn new(header: Header, prg_rom: Box<[u8]>, chr_rom: Box<[u8]>) -> Self {
        Self::with_trainer(header, prg_rom, chr_rom, None)
    }

    pub(crate) fn with_trainer(
        header: Header,
        prg_rom: Box<[u8]>,
        chr_rom: Box<[u8]>,
        trainer: Option<Box<[u8; TRAINER_SIZE]>>,
    ) -> Self {
        let mut prg_ram = allocate_prg_ram(&header);
        if let (Some(trainer), Some(dst)) = (trainer.as_ref(), trainer_destination(&mut prg_ram)) {
            dst.copy_from_slice(trainer.as_ref());
        }

        let chr = select_chr_storage(&header, chr_rom);
        let prg_bank_count_8k = (prg_rom.len() / PRG_BANK_SIZE_8K).max(1);

        Self {
            prg_rom,
            prg_ram,
            chr,
            prg_bank_count_8k,
            prg_banks: [0; 3],
            chr_banks: [0; 8],
            control: 0,
            mirroring: header.mirroring,
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
            0x8000..=0x9FFF => self.prg_bank_index(self.prg_banks[0]),
            0xA000..=0xBFFF => self.prg_bank_index(self.prg_banks[1]),
            0xC000..=0xDFFF => self.prg_bank_index(self.prg_banks[2]),
            0xE000..=0xFFFF => self.prg_bank_count_8k.saturating_sub(1),
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
        match addr & 0xF038 {
            0x8000 => self.prg_banks[0] = value & 0x3F,
            0x8008 => self.prg_banks[1] = value & 0x3F,
            0x9000 => self.prg_banks[2] = value & 0x3F,
            0xA000 => self.chr_banks[0] = value,
            0xA008 => self.chr_banks[1] = value,
            0xB000 => self.chr_banks[2] = value,
            0xB008 => self.chr_banks[3] = value,
            0xC000 => self.chr_banks[4] = value,
            0xC008 => self.chr_banks[5] = value,
            0xD000 => self.chr_banks[6] = value,
            0xD008 => self.chr_banks[7] = value,
            0xE000 => self.update_control(value),
            0xE008 => {
                self.irq_reload = value;
            }
            0xF000 => {
                self.irq_enabled_after_ack = (value & 0x01) != 0;
                self.irq_enabled = (value & 0x02) != 0;
                self.irq_cycle_mode = (value & 0x04) != 0;
                if self.irq_enabled {
                    self.irq_counter = self.irq_reload;
                    self.irq_prescaler = 341;
                    self.irq_pending = false;
                }
            }
            0xF008 => {
                self.irq_enabled = self.irq_enabled_after_ack;
                self.irq_pending = false;
            }
            // $9010/$9030 audio registers are ignored (muted).
            _ => {}
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
    fn power_on(&mut self) {
        self.prg_banks = [0; 3];
        self.chr_banks = [0; 8];
        self.control = 0;
        self.mirroring = self.mirroring; // keep header mirroring until written

        self.irq_reload = 0;
        self.irq_counter = 0;
        self.irq_prescaler = 0;
        self.irq_enabled = false;
        self.irq_enabled_after_ack = false;
        self.irq_cycle_mode = false;
        self.irq_pending = false;
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
        85
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Konami VRC7 (no audio)")
    }
}
