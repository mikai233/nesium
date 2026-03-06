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
    apu::{ExpansionAudio, ExpansionAudioClockContext, ExpansionAudioSink, ExpansionAudioSnapshot},
    audio::AudioChannel,
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

/// Minimal Sunsoft 5B tone-generator model aligned with Mesen2's mapper path.
#[derive(Debug, Clone)]
struct Sunsoft5bAudioState {
    volume_lut: [u8; 0x10],
    current_register: u8,
    registers: [u8; 0x10],
    last_output: f32,
    timer: [i16; 3],
    tone_step: [u8; 3],
    process_tick: bool,
}

impl Sunsoft5bAudioState {
    fn new() -> Self {
        let mut volume_lut = [0u8; 0x10];
        volume_lut[0] = 0;

        let mut output = 1.0f64;
        for item in volume_lut.iter_mut().skip(1) {
            // +3.0 dB per volume step (1.5 dB * 2).
            output *= 1.188_502_227_437_018_4;
            output *= 1.188_502_227_437_018_4;
            *item = output as u8;
        }

        Self {
            volume_lut,
            current_register: 0,
            registers: [0; 0x10],
            last_output: 0.0,
            timer: [0; 3],
            tone_step: [0; 3],
            process_tick: false,
        }
    }

    #[inline]
    fn period(&self, channel: usize) -> i16 {
        let lo = self.registers[channel * 2] as u16;
        let hi = self.registers[channel * 2 + 1] as u16;
        (lo | (hi << 8)) as i16
    }

    #[inline]
    fn volume(&self, channel: usize) -> u8 {
        self.volume_lut[(self.registers[8 + channel] & 0x0F) as usize]
    }

    #[inline]
    fn tone_enabled(&self, channel: usize) -> bool {
        ((self.registers[7] >> channel) & 0x01) == 0
    }

    fn update_channel(&mut self, channel: usize) {
        self.timer[channel] = self.timer[channel].saturating_sub(1);
        if self.timer[channel] <= 0 {
            self.timer[channel] = self.period(channel);
            self.tone_step[channel] = (self.tone_step[channel] + 1) & 0x0F;
        }
    }

    fn update_output_level(&mut self) {
        let mut summed = 0u16;
        for ch in 0..3 {
            if self.tone_enabled(ch) && self.tone_step[ch] < 8 {
                summed = summed.saturating_add(self.volume(ch) as u16);
            }
        }
        self.last_output = summed as f32;
    }

    fn clock(&mut self) {
        if self.process_tick {
            for channel in 0..3 {
                self.update_channel(channel);
            }
            self.update_output_level();
        }
        self.process_tick = !self.process_tick;
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0xE000 {
            0xC000 => {
                self.current_register = value;
            }
            0xE000 => {
                if self.current_register <= 0x0F {
                    self.registers[self.current_register as usize] = value;
                }
            }
            _ => {}
        }
    }

    #[inline]
    fn sample(&self) -> f32 {
        self.last_output
    }
}

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

    audio: Sunsoft5bAudioState,
    audio_level: f32,
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
            audio: Sunsoft5bAudioState::new(),
            audio_level: 0.0,
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

impl ExpansionAudio for Mapper69 {
    fn clock_cpu(&mut self, ctx: ExpansionAudioClockContext, sink: &mut dyn ExpansionAudioSink) {
        if self.irq_counter_enabled {
            self.irq_counter = self.irq_counter.wrapping_sub(1);
            if self.irq_counter == 0xFFFF && self.irq_enabled {
                self.irq_pending = true;
            }
        }

        self.audio.clock();
        let level = self.audio.sample();
        let delta = level - self.audio_level;
        if delta != 0.0 {
            sink.push_delta(AudioChannel::Sunsoft5B, ctx.apu_cycle, delta);
            self.audio_level = level;
        }
    }

    fn snapshot(&self) -> ExpansionAudioSnapshot {
        ExpansionAudioSnapshot {
            sunsoft5b: self.audio_level,
            ..ExpansionAudioSnapshot::default()
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
        self.audio = Sunsoft5bAudioState::new();
        self.audio_level = 0.0;
    }

    fn cpu_read(&self, addr: u16) -> Option<u8> {
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

        assert_eq!(mapper.cpu_read(0xE000), Some(7));

        mapper.cpu_write(0x8000, 0x09, 0);
        mapper.cpu_write(0xA000, 0x03, 0);
        assert_eq!(mapper.cpu_read(0x8000), Some(3));
    }

    #[test]
    fn lower_window_switches_between_rom_and_ram() {
        let mut mapper = test_mapper(8);
        mapper.reset(ResetKind::PowerOn);

        mapper.cpu_write(0x8000, 0x08, 0);
        mapper.cpu_write(0xA000, 0x03, 0);
        assert_eq!(mapper.cpu_read(0x6000), Some(3));

        mapper.cpu_write(0xA000, 0xC0, 0); // RAM mode + enabled, bank 0
        mapper.cpu_write(0x6000, 0x5A, 0);
        assert_eq!(mapper.cpu_read(0x6000), Some(0x5A));

        mapper.cpu_write(0xA000, 0x40, 0); // RAM mode + disabled
        assert_eq!(mapper.cpu_read(0x6000), None);
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
