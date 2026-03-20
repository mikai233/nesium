//! Mapper 69 (Sunsoft FME-7 / Sunsoft 5B).
//!
//! This board provides:
//! - 8 KiB PRG banking at `$8000-$DFFF` with a fixed last bank at `$E000-$FFFF`
//! - 1 KiB CHR banking for all 8 pattern slots
//! - Configurable `$6000-$7FFF` window (PRG-ROM or PRG-RAM page)
//! - 16-bit CPU-cycle IRQ counter
//! - Optional Sunsoft 5B expansion audio (3 tone channels)

use std::borrow::Cow;

use crate::cartridge::mapper::{MapperMemoryMut, MapperMemoryRef};

use crate::{
    apu::{
        ExpansionAudio, ExpansionAudioClockContext, ExpansionAudioSink, ExpansionAudioSnapshot,
        Sunsoft5bAudio,
    },
    cartridge::{
        ChrRom, Mapper, PrgRom, TrainerBytes,
        header::{Header, Mirroring},
        mapper::{ChrStorage, allocate_prg_ram_with_trainer, select_chr_storage},
    },
    memory::cpu as cpu_mem,
    reset_kind::ResetKind,
};

const PRG_BANK_SIZE_8K: usize = 8 * 1024;
const CHR_BANK_SIZE_1K: usize = 1024;

const FME7_CMD_SELECT_START: u16 = 0x8000;
const FME7_CMD_SELECT_END: u16 = 0x9FFF;
const FME7_CMD_DATA_START: u16 = 0xA000;
const FME7_CMD_DATA_END: u16 = 0xBFFF;

#[derive(Debug, Clone)]
pub struct Mapper69 {
    prg_rom: PrgRom,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,

    prg_bank_count_8k: usize,

    command: u8,
    chr_banks: [u8; 8],
    prg_bank_8000: u8,
    prg_bank_a000: u8,
    prg_bank_c000: u8,
    prg_map_8000: bool,
    prg_map_a000: bool,
    prg_map_c000: bool,
    ram_select: u8,
    base_mirroring: Mirroring,
    mirroring: Mirroring,

    irq_enabled: bool,
    irq_counter_enabled: bool,
    irq_counter: u16,
    irq_pending: bool,

    audio: Sunsoft5bAudio,
}

impl Mapper69 {
    pub fn new(header: Header, prg_rom: PrgRom, chr_rom: ChrRom, trainer: TrainerBytes) -> Self {
        let prg_ram = allocate_prg_ram_with_trainer(&header, trainer);
        let chr = select_chr_storage(&header, chr_rom);
        let prg_bank_count_8k = (prg_rom.len() / PRG_BANK_SIZE_8K).max(1);

        Self {
            prg_rom,
            prg_ram,
            chr,
            prg_bank_count_8k,
            command: 0,
            chr_banks: [0; 8],
            prg_bank_8000: 0,
            prg_bank_a000: 1,
            prg_bank_c000: 2,
            prg_map_8000: false,
            prg_map_a000: false,
            prg_map_c000: false,
            ram_select: 0,
            base_mirroring: header.mirroring(),
            mirroring: header.mirroring(),
            irq_enabled: false,
            irq_counter_enabled: false,
            irq_counter: 0,
            irq_pending: false,
            audio: Sunsoft5bAudio::new(),
        }
    }

    #[inline]
    fn prg_rom_bank_index(&self, bank: u8) -> usize {
        if self.prg_bank_count_8k == 0 {
            0
        } else {
            (bank as usize) % self.prg_bank_count_8k
        }
    }

    #[inline]
    fn prg_ram_bank_index(&self, bank: u8) -> Option<usize> {
        if self.prg_ram.is_empty() {
            return None;
        }
        let bank_count = (self.prg_ram.len() / PRG_BANK_SIZE_8K).max(1);
        Some((bank as usize) % bank_count)
    }

    fn read_prg_rom_bank(&self, bank: usize, addr: u16) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }
        let base = bank.saturating_mul(PRG_BANK_SIZE_8K);
        let offset = (addr as usize) & (PRG_BANK_SIZE_8K - 1);
        self.prg_rom.get(base + offset).copied().unwrap_or(0)
    }

    fn read_prg_ram_bank(&self, bank: usize, addr: u16) -> u8 {
        if self.prg_ram.is_empty() {
            return 0;
        }
        let base = bank.saturating_mul(PRG_BANK_SIZE_8K);
        let offset = (addr as usize) & (PRG_BANK_SIZE_8K - 1);
        self.prg_ram[(base + offset) % self.prg_ram.len()]
    }

    fn write_prg_ram_bank(&mut self, bank: usize, addr: u16, value: u8) {
        if self.prg_ram.is_empty() {
            return;
        }
        let base = bank.saturating_mul(PRG_BANK_SIZE_8K);
        let offset = (addr as usize) & (PRG_BANK_SIZE_8K - 1);
        let index = (base + offset) % self.prg_ram.len();
        self.prg_ram[index] = value;
    }

    #[inline]
    fn ram_mode(&self) -> bool {
        self.ram_select & 0x40 != 0
    }

    #[inline]
    fn ram_enabled(&self) -> bool {
        self.ram_select & 0x80 != 0
    }

    fn read_lower_window(&self, addr: u16) -> Option<u8> {
        let bank = self.ram_select & 0x3F;

        if self.ram_mode() {
            if !self.ram_enabled() {
                return None;
            }
            let bank = self.prg_ram_bank_index(bank)?;
            return Some(self.read_prg_ram_bank(bank, addr));
        }

        let bank = self.prg_rom_bank_index(bank);
        Some(self.read_prg_rom_bank(bank, addr))
    }

    fn write_lower_window(&mut self, addr: u16, value: u8) {
        if !self.ram_mode() || !self.ram_enabled() {
            return;
        }

        if let Some(bank) = self.prg_ram_bank_index(self.ram_select & 0x3F) {
            self.write_prg_ram_bank(bank, addr, value);
        }
    }

    fn read_prg_rom_window(&self, addr: u16) -> Option<u8> {
        let bank = match addr {
            0x8000..=0x9FFF if self.prg_map_8000 => self.prg_rom_bank_index(self.prg_bank_8000),
            0xA000..=0xBFFF if self.prg_map_a000 => self.prg_rom_bank_index(self.prg_bank_a000),
            0xC000..=0xDFFF if self.prg_map_c000 => self.prg_rom_bank_index(self.prg_bank_c000),
            0xE000..=0xFFFF => self.prg_bank_count_8k.saturating_sub(1),
            _ => return None,
        };
        Some(self.read_prg_rom_bank(bank, addr))
    }

    fn read_chr(&self, addr: u16) -> u8 {
        let a = addr & 0x1FFF;
        let slot = ((a >> 10) & 0x07) as usize;
        let bank = self.chr_banks[slot] as usize;
        let base = bank.saturating_mul(CHR_BANK_SIZE_1K);
        let offset = (a & 0x03FF) as usize;
        self.chr.read_indexed(base, offset)
    }

    fn write_chr(&mut self, addr: u16, value: u8) {
        let a = addr & 0x1FFF;
        let slot = ((a >> 10) & 0x07) as usize;
        let bank = self.chr_banks[slot] as usize;
        let base = bank.saturating_mul(CHR_BANK_SIZE_1K);
        let offset = (a & 0x03FF) as usize;
        self.chr.write_indexed(base, offset, value);
    }

    fn write_command_data(&mut self, value: u8) {
        match self.command {
            0x00..=0x07 => {
                self.chr_banks[self.command as usize] = value;
            }
            0x08 => {
                self.ram_select = value;
            }
            0x09 => {
                self.prg_bank_8000 = value & 0x3F;
                self.prg_map_8000 = true;
            }
            0x0A => {
                self.prg_bank_a000 = value & 0x3F;
                self.prg_map_a000 = true;
            }
            0x0B => {
                self.prg_bank_c000 = value & 0x3F;
                self.prg_map_c000 = true;
            }
            0x0C => {
                self.mirroring = match value & 0x03 {
                    0 => Mirroring::Vertical,
                    1 => Mirroring::Horizontal,
                    2 => Mirroring::SingleScreenLower,
                    _ => Mirroring::SingleScreenUpper,
                };
            }
            0x0D => {
                self.irq_enabled = value & 0x01 != 0;
                self.irq_counter_enabled = value & 0x80 != 0;
                self.irq_pending = false;
            }
            0x0E => {
                self.irq_counter = (self.irq_counter & 0xFF00) | (value as u16);
            }
            0x0F => {
                self.irq_counter = (self.irq_counter & 0x00FF) | ((value as u16) << 8);
            }
            _ => {}
        }
    }
}

impl Mapper for Mapper69 {
    fn reset(&mut self, _kind: ResetKind) {
        self.command = 0;
        self.chr_banks = [0; 8];
        self.prg_bank_8000 = 0;
        self.prg_bank_a000 = 1;
        self.prg_bank_c000 = 2;
        self.prg_map_8000 = false;
        self.prg_map_a000 = false;
        self.prg_map_c000 = false;
        self.ram_select = 0;
        self.mirroring = self.base_mirroring;
        self.irq_enabled = false;
        self.irq_counter_enabled = false;
        self.irq_counter = 0;
        self.irq_pending = false;
        self.audio = Sunsoft5bAudio::new();
    }

    fn cpu_read(&self, addr: u16, _open_bus: u8) -> Option<u8> {
        match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => return self.read_lower_window(addr),
            cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END => self.read_prg_rom_window(addr),
            _ => return None,
        }
    }

    fn cpu_write(&mut self, addr: u16, data: u8, _cpu_cycle: u64) {
        match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.write_lower_window(addr, data),
            FME7_CMD_SELECT_START..=FME7_CMD_SELECT_END => {
                self.command = data & 0x0F;
            }
            FME7_CMD_DATA_START..=FME7_CMD_DATA_END => {
                self.write_command_data(data);
            }
            0xC000..=0xDFFF | 0xE000..=0xFFFF => {
                self.audio.write_register(addr, data);
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
        self.irq_pending
    }

    fn expansion_audio(&self) -> Option<&dyn ExpansionAudio> {
        Some(self)
    }

    fn expansion_audio_mut(&mut self) -> Option<&mut dyn ExpansionAudio> {
        Some(self)
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
        69
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Sunsoft FME-7 / 5B")
    }
}

impl ExpansionAudio for Mapper69 {
    fn clock_cpu(&mut self, ctx: ExpansionAudioClockContext, sink: &mut dyn ExpansionAudioSink) {
        if self.irq_counter_enabled {
            self.irq_counter = self.irq_counter.wrapping_sub(1);
            if self.irq_counter == 0xFFFF && self.irq_enabled {
                self.irq_pending = true;
            }
        }
        self.audio.clock_cpu(ctx, sink);
    }

    fn snapshot(&self) -> ExpansionAudioSnapshot {
        self.audio.snapshot()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        apu::{ExpansionAudioClockContext, NullExpansionAudioSink},
        cartridge::header::Header,
    };

    fn test_header(prg_16k_units: u8, chr_8k_units: u8) -> Header {
        let mut rom = [0u8; 16];
        rom[0..4].copy_from_slice(b"NES\x1A");
        rom[4] = prg_16k_units;
        rom[5] = chr_8k_units;
        // Mapper 69 = 0x45.
        rom[6] = 0x50;
        rom[7] = 0x40;
        Header::parse(&rom).expect("valid iNES header")
    }

    fn test_mapper(prg_banks_8k: usize) -> Mapper69 {
        let header = test_header((prg_banks_8k / 2) as u8, 1);
        let mut prg = vec![0u8; prg_banks_8k * PRG_BANK_SIZE_8K];
        for (bank, chunk) in prg.chunks_exact_mut(PRG_BANK_SIZE_8K).enumerate() {
            chunk.fill(bank as u8);
        }
        Mapper69::new(header, prg.into(), vec![0u8; 8 * 1024].into(), None)
    }

    #[test]
    fn prg_banking_and_fixed_last_bank() {
        let mut mapper = test_mapper(8);
        mapper.reset(ResetKind::PowerOn);

        assert_eq!(mapper.cpu_read(0xE000, 0), Some(7));

        mapper.cpu_write(0x8000, 0x09, 0);
        mapper.cpu_write(0xA000, 0x03, 0);
        assert_eq!(mapper.cpu_read(0x8000, 0), Some(3));
    }

    #[test]
    fn lower_window_switches_between_rom_and_ram() {
        let mut mapper = test_mapper(8);
        mapper.reset(ResetKind::PowerOn);

        mapper.cpu_write(0x8000, 0x08, 0);
        mapper.cpu_write(0xA000, 0x03, 0);
        assert_eq!(mapper.cpu_read(0x6000, 0), Some(3));

        mapper.cpu_write(0xA000, 0xC0, 0); // RAM mode + enabled, bank 0
        mapper.cpu_write(0x6000, 0x5A, 0);
        assert_eq!(mapper.cpu_read(0x6000, 0), Some(0x5A));

        mapper.cpu_write(0xA000, 0x40, 0); // RAM mode + disabled
        assert_eq!(mapper.cpu_read(0x6000, 0), None);
    }

    #[test]
    fn irq_triggers_on_underflow_when_enabled() {
        let mut mapper = test_mapper(8);
        mapper.reset(ResetKind::PowerOn);

        mapper.cpu_write(0x8000, 0x0E, 0);
        mapper.cpu_write(0xA000, 0x00, 0);
        mapper.cpu_write(0x8000, 0x0F, 0);
        mapper.cpu_write(0xA000, 0x00, 0);
        mapper.cpu_write(0x8000, 0x0D, 0);
        mapper.cpu_write(0xA000, 0x81, 0); // irq_enabled + counter_enabled

        let mut sink = NullExpansionAudioSink;
        mapper.clock_cpu(
            ExpansionAudioClockContext {
                cpu_cycle: 0,
                apu_cycle: 0,
                master_clock: 0,
            },
            &mut sink,
        );

        assert!(mapper.irq_pending());
    }
}
