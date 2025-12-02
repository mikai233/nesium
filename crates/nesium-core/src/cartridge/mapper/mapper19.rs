//! Mapper 19 – Namco 163 (N163) implementation with basic expansion audio.
//!
//! This mapper powers a number of late Famicom titles (Digital Devil Story:
//! Megami Tensei II, Erika to Satoru no Yume Bouken, etc.). The ASIC supports
//! a flexible 8 KiB PRG banking scheme, fine‑grained 1 KiB CHR banking, an IRQ
//! counter, and optional expansion audio. This implementation focuses on:
//! - PRG banking:
//!   - `$6000-$7FFF`: optional 8 KiB PRG-RAM (when present in the header).
//!   - `$8000-$9FFF`: 8 KiB switchable PRG-ROM bank.
//!   - `$A000-$BFFF`: 8 KiB switchable PRG-ROM bank.
//!   - `$C000-$DFFF`: 8 KiB switchable PRG-ROM bank.
//!   - `$E000-$FFFF`: 8 KiB PRG-ROM bank fixed to the last bank.
//! - CHR banking:
//!   - Eight 1 KiB banks for `$0000-$1FFF` (pattern tables), backed by either
//!     CHR-ROM or CHR-RAM via [`ChrStorage`].
//! - IRQ counter:
//!   - 15‑bit CPU‑cycle counter configured via `$5000`/`$5800`, as per Nesdev.
//!   - When the counter reaches `$7FFF`, an IRQ is latched and counting stops.
//!
//! Nametable‑as‑CHR configurations and some of the more exotic pin behaviours
//! are currently omitted; most commercial games should still behave correctly.

use std::borrow::Cow;

use crate::{
    apu::{ExpansionAudio, expansion::ExpansionSamples},
    cartridge::{
        Mapper, TRAINER_SIZE,
        header::{Header, Mirroring},
        mapper::{ChrStorage, allocate_prg_ram, select_chr_storage, trainer_destination},
    },
    memory::cpu as cpu_mem,
};

use crate::mem_block::{ByteBlock, MemBlock};

/// PRG-ROM banking granularity (8 KiB).
const PRG_BANK_SIZE_8K: usize = 8 * 1024;
/// CHR banking granularity (1 KiB).
const CHR_BANK_SIZE_1K: usize = 1 * 1024;

/// Namco 163 audio state (adapted from Mesen2's `Namco163Audio`).
#[derive(Debug, Clone)]
struct Namco163AudioState {
    internal_ram: Namco163InternalRam,
    channel_output: Namco163ChannelOutput,
    ram_position: u8,
    auto_increment: bool,
    update_counter: u8,
    current_channel: i8,
    last_output: f32,
    disabled: bool,
}

type Namco163InternalRam = ByteBlock<0x80>;
type Namco163ChannelOutput = MemBlock<i16, 8>;
type Namco163ChrBankRegs = ByteBlock<8>;

impl Namco163AudioState {
    fn new() -> Self {
        Self {
            internal_ram: Namco163InternalRam::new(),
            channel_output: Namco163ChannelOutput::new(),
            ram_position: 0,
            auto_increment: false,
            update_counter: 0,
            current_channel: 7,
            last_output: 0.0,
            disabled: false,
        }
    }

    fn num_channels(&self) -> u8 {
        // Nesdev: high nibble of $7F encodes (channels - 1).
        (self.internal_ram[0x7F] >> 4) & 0x07
    }

    fn frequency(&self, channel: usize) -> u32 {
        let base = 0x40 + channel as u8 * 0x08;
        let lo = self.internal_ram[base as usize] as u32;
        let mid = self.internal_ram[base as usize + 2] as u32;
        let hi = (self.internal_ram[base as usize + 4] & 0x03) as u32;
        (hi << 16) | (mid << 8) | lo
    }

    fn phase(&self, channel: usize) -> u32 {
        let base = 0x40 + channel as u8 * 0x08;
        let lo = self.internal_ram[base as usize + 1] as u32;
        let mid = self.internal_ram[base as usize + 3] as u32;
        let hi = self.internal_ram[base as usize + 5] as u32;
        (hi << 16) | (mid << 8) | lo
    }

    fn set_phase(&mut self, channel: usize, phase: u32) {
        let base = 0x40 + channel as u8 * 0x08;
        self.internal_ram[base as usize + 5] = ((phase >> 16) & 0xFF) as u8;
        self.internal_ram[base as usize + 3] = ((phase >> 8) & 0xFF) as u8;
        self.internal_ram[base as usize + 1] = (phase & 0xFF) as u8;
    }

    fn wave_address(&self, channel: usize) -> u8 {
        let base = 0x40 + channel as u8 * 0x08;
        self.internal_ram[base as usize + 6]
    }

    fn wave_length(&self, channel: usize) -> u16 {
        let base = 0x40 + channel as u8 * 0x08;
        let raw = self.internal_ram[base as usize + 4] & 0xFC;
        256u16.saturating_sub(raw as u16).max(1)
    }

    fn volume(&self, channel: usize) -> u8 {
        let base = 0x40 + channel as u8 * 0x08;
        self.internal_ram[base as usize + 7] & 0x0F
    }

    fn update_channel(&mut self, channel: usize) {
        let freq = self.frequency(channel);
        let mut phase = self.phase(channel);
        let length = self.wave_length(channel) as u32;
        let offset = self.wave_address(channel);
        let vol = self.volume(channel) as i16;

        // Advance phase within the waveform length.
        phase = phase.wrapping_add(freq) % (length << 16);

        let sample_pos = ((phase >> 16) as u8).wrapping_add(offset);
        let byte = self.internal_ram[(sample_pos >> 1) as usize];
        let nibble = if sample_pos & 1 != 0 {
            (byte >> 4) & 0x0F
        } else {
            byte & 0x0F
        };
        let sample = (nibble as i16) - 8;

        self.channel_output[channel] = sample * vol;
        self.set_phase(channel, phase);
        self.update_output_level();
    }

    fn update_output_level(&mut self) {
        let n = self.num_channels();
        let active = n as i16 + 1;
        let mut sum: i16 = 0;
        for i in (7 - n as i8)..=7 {
            let idx = i as usize;
            sum = sum.saturating_add(self.channel_output[idx]);
        }
        let avg = sum as f32 / active as f32;
        // Roughly normalise into a small range; further scaling happens in the mixer.
        self.last_output = avg / 512.0;
    }

    fn clock(&mut self) {
        if self.disabled {
            return;
        }
        self.update_counter = self.update_counter.wrapping_add(1);
        if self.update_counter == 15 {
            let ch = self.current_channel.clamp(0, 7) as usize;
            self.update_channel(ch);
            self.update_counter = 0;

            self.current_channel -= 1;
            if self.current_channel < 7 - self.num_channels() as i8 {
                self.current_channel = 7;
            }
        }
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0xF800 {
            0x4800 => {
                self.internal_ram[self.ram_position as usize] = value;
                if self.auto_increment {
                    self.ram_position = (self.ram_position + 1) & 0x7F;
                }
            }
            0xE000 => {
                // Bit 6 disables sound; other bits handled by PRG banking.
                self.disabled = (value & 0x40) != 0;
            }
            0xF800 => {
                self.ram_position = value & 0x7F;
                self.auto_increment = (value & 0x80) != 0;
            }
            _ => {}
        }
    }

    fn read_register(&mut self, addr: u16) -> u8 {
        match addr & 0xF800 {
            0x4800 => {
                let val = self.internal_ram[self.ram_position as usize];
                if self.auto_increment {
                    self.ram_position = (self.ram_position + 1) & 0x7F;
                }
                val
            }
            _ => 0,
        }
    }

    fn sample(&self) -> f32 {
        self.last_output
    }
}

#[derive(Debug, Clone)]
pub struct Mapper19 {
    prg_rom: Box<[u8]>,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,

    /// Number of 8 KiB PRG-ROM banks available.
    prg_bank_count_8k: usize,

    /// PRG bank registers for the three switchable 8 KiB windows.
    prg_bank_8000: u8,
    prg_bank_a000: u8,
    prg_bank_c000: u8,

    /// Eight 1 KiB CHR bank registers backing `$0000-$1FFF`.
    chr_banks: Namco163ChrBankRegs,

    /// Namco 163 expansion audio generator state, when this board revision
    /// includes the N163 audio block.
    audio: Option<Namco163AudioState>,

    /// IRQ counter (bit15 is the enable flag; bits 0‑14 are the 15‑bit count),
    /// matching the representation used in Mesen2 and the Nesdev description.
    irq_counter: u16,
    irq_pending: bool,

    /// Current nametable mirroring mode. Namco 163 can repurpose nametable RAM
    /// as CHR, but the basic nametable wiring still follows the header unless
    /// a more complete implementation overrides it.
    mirroring: Mirroring,
}

impl Mapper19 {
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

        // Submapper 2 corresponds to boards without expansion sound; other
        // submappers include the N163 audio generator. Treat unknown/legacy
        // values as having audio to remain compatible with older dumps.
        let audio = if header.submapper == 2 {
            None
        } else {
            Some(Namco163AudioState::new())
        };

        Self {
            prg_rom,
            prg_ram,
            chr,
            prg_bank_count_8k,
            prg_bank_8000: 0,
            prg_bank_a000: 1,
            prg_bank_c000: 2,
            chr_banks: Namco163ChrBankRegs::new(),
            audio,
            irq_counter: 0,
            irq_pending: false,
            mirroring: header.mirroring,
        }
    }

    fn prg_bank_index(&self, reg: u8) -> usize {
        if self.prg_bank_count_8k == 0 {
            0
        } else {
            (reg as usize) % self.prg_bank_count_8k
        }
    }

    fn last_prg_bank_index(&self) -> usize {
        if self.prg_bank_count_8k == 0 {
            0
        } else {
            self.prg_bank_count_8k - 1
        }
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }

        let bank = match addr {
            0x8000..=0x9FFF => self.prg_bank_index(self.prg_bank_8000),
            0xA000..=0xBFFF => self.prg_bank_index(self.prg_bank_a000),
            0xC000..=0xDFFF => self.prg_bank_index(self.prg_bank_c000),
            0xE000..=0xFFFF => self.last_prg_bank_index(),
            _ => return 0,
        };

        let base = bank.saturating_mul(PRG_BANK_SIZE_8K);
        let offset = (addr as usize).saturating_sub(cpu_mem::PRG_ROM_START as usize)
            & (PRG_BANK_SIZE_8K - 1);
        let idx = base.saturating_add(offset);
        self.prg_rom.get(idx).copied().unwrap_or(0)
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

    fn write_audio_register(&mut self, addr: u16, value: u8) {
        if let Some(audio) = &mut self.audio {
            audio.write_register(addr, value);
        }
    }

    fn read_audio_register(&self, addr: u16) -> u8 {
        if let Some(mut audio) = self.audio.clone() {
            // Reads can auto-increment the internal address; cloning here keeps
            // the implementation simple at the cost of ignoring that side
            // effect for now. Games rarely rely on read-side auto-increment.
            return audio.read_register(addr);
        }
        0
    }

    fn chr_bank_for_addr(&self, addr: u16) -> (usize, usize) {
        let a = addr & 0x1FFF;
        let index = ((a >> 10) & 0x07) as usize; // 1 KiB slot 0-7
        let offset = (a & 0x03FF) as usize;
        let bank = self.chr_banks[index] as usize;
        (bank * CHR_BANK_SIZE_1K, offset)
    }

    fn read_chr(&self, addr: u16) -> u8 {
        let (base, offset) = self.chr_bank_for_addr(addr);
        self.chr.read_indexed(base, offset)
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        let (base, offset) = self.chr_bank_for_addr(addr);
        self.chr.write_indexed(base, offset, data);
    }

    fn write_chr_bank(&mut self, addr: u16, value: u8) {
        // Nesdev: $8000-$BFFF grouped in 0x800 ranges select 1 KiB CHR banks
        // for $0000-$1FFF. We ignore the nametable‑as‑CHR feature for now and
        // treat all values as CHR ROM/RAM page indices.
        if !(0x8000..=0xBFFF).contains(&addr) {
            return;
        }
        let index = ((addr - 0x8000) >> 11) as usize; // 0..7
        if index < self.chr_banks.len() {
            self.chr_banks[index] = value;
        }
    }

    fn write_prg_select_8000(&mut self, value: u8) {
        // AMPP PPPP → P bits select PRG bank at $8000-$9FFF; we ignore the
        // audio disable and pin reflection bits.
        self.prg_bank_8000 = value & 0x3F;
    }

    fn write_prg_select_a000(&mut self, value: u8) {
        // HLPP PPPP → P bits select PRG bank at $A000-$BFFF; we ignore H/L
        // CHR-RAM/NTRAM flags for now.
        self.prg_bank_a000 = value & 0x3F;
    }

    fn write_prg_select_c000(&mut self, value: u8) {
        // CDPP PPPP → P bits select PRG bank at $C000-$DFFF; we ignore C/D pin
        // related behaviour and the special $3F debug bank.
        self.prg_bank_c000 = value & 0x3F;
    }

    fn write_irq_low(&mut self, value: u8) {
        // Low 8 bits of the 15‑bit IRQ counter; writing also acknowledges
        // pending IRQs.
        self.irq_counter = (self.irq_counter & 0xFF00) | (value as u16);
        self.irq_pending = false;
    }

    fn write_irq_high(&mut self, value: u8) {
        // High bits plus enable flag (bit7).
        self.irq_counter = (self.irq_counter & 0x00FF) | ((value as u16) << 8);
        self.irq_pending = false;
    }

    fn read_irq_low(&self) -> u8 {
        (self.irq_counter & 0x00FF) as u8
    }

    fn read_irq_high(&self) -> u8 {
        (self.irq_counter >> 8) as u8
    }
}

impl ExpansionAudio for Mapper19 {
    fn clock_audio(&mut self) {
        if let Some(audio) = &mut self.audio {
            audio.clock();
        }
    }

    fn samples(&self) -> ExpansionSamples {
        let namco163 = self.audio.as_ref().map_or(0.0, |a| a.sample());
        ExpansionSamples {
            namco163,
            ..ExpansionSamples::default()
        }
    }
}

impl Mapper for Mapper19 {
    fn power_on(&mut self) {
        self.prg_bank_8000 = 0;
        self.prg_bank_a000 = 1.min(self.prg_bank_count_8k.saturating_sub(1) as u8);
        self.prg_bank_c000 = 2.min(self.prg_bank_count_8k.saturating_sub(1) as u8);
        self.chr_banks.fill(0);
        self.irq_counter = 0;
        self.irq_pending = false;
    }

    fn reset(&mut self) {
        self.power_on();
    }

    fn cpu_read(&self, addr: u16) -> Option<u8> {
        let value = match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => return self.read_prg_ram(addr),
            0x4800..=0x4FFF => self.read_audio_register(addr),
            0x5000..=0x57FF => self.read_irq_low(),
            0x5800..=0x5FFF => self.read_irq_high(),
            cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END => self.read_prg_rom(addr),
            _ => return None,
        };
        Some(value)
    }

    fn cpu_write(&mut self, addr: u16, data: u8, _cpu_cycle: u64) {
        match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.write_prg_ram(addr, data),

            // Chip RAM / audio data port.
            0x4800..=0x4FFF => self.write_audio_register(addr, data),

            // IRQ counter low/high (both readable and writable).
            0x5000..=0x57FF => self.write_irq_low(data),
            0x5800..=0x5FFF => self.write_irq_high(data),

            // CHR bank selects for 1 KiB pattern table windows.
            0x8000..=0xBFFF => self.write_chr_bank(addr, data),

            // PRG bank selects for the 3 switchable 8 KiB windows.
            0xE000..=0xE7FF => {
                self.write_prg_select_8000(data);
                self.write_audio_register(addr, data);
            }
            0xE800..=0xEFFF => self.write_prg_select_a000(data),
            0xF000..=0xF7FF => self.write_prg_select_c000(data),

            // Write-protect / audio RAM address port.
            0xF800..=0xFFFF => self.write_audio_register(addr, data),
            _ => {}
        }
    }

    fn cpu_clock(&mut self, _cpu_cycle: u64) {
        // Nesdev: IRQ is a 15‑bit CPU cycle up‑counter. $5000/$5800 provide
        // direct access to the counter; bit15 acts as an enable flag.
        //
        // When (counter & 0x8000) != 0 and the low 15 bits are not yet $7FFF,
        // increment on each CPU cycle. When the low 15 bits reach $7FFF,
        // latch an IRQ and stop counting until the value is changed.
        if (self.irq_counter & 0x8000) != 0 && (self.irq_counter & 0x7FFF) != 0x7FFF {
            self.irq_counter = self.irq_counter.wrapping_add(1);
            if (self.irq_counter & 0x7FFF) == 0x7FFF {
                self.irq_pending = true;
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
        19
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Namco 163 (no expansion audio)")
    }

    fn as_expansion_audio(&self) -> Option<&dyn ExpansionAudio> {
        if self.audio.is_some() {
            Some(self)
        } else {
            None
        }
    }

    fn as_expansion_audio_mut(&mut self) -> Option<&mut dyn ExpansionAudio> {
        if self.audio.is_some() {
            Some(self)
        } else {
            None
        }
    }
}
