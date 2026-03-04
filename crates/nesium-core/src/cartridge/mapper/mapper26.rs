//! Mapper 26 – Konami VRC6b.
//!
//! This implementation mirrors the PRG/CHR banking and IRQ behaviour of VRC6,
//! following Mesen2's layout, including VRC6 expansion audio (two pulse
//! channels + one saw channel) clocked on CPU cycles.
//!
//! | Area | Address range       | Behaviour                                          | IRQ/Audio                         |
//! |------|---------------------|----------------------------------------------------|-----------------------------------|
//! | CPU  | `$6000-$7FFF`       | Optional PRG-RAM (enabled via banking_mode bit 7)  | None                              |
//! | CPU  | `$8000-$BFFF`       | 16 KiB switchable PRG-ROM window (2×8 KiB)         | None                              |
//! | CPU  | `$C000-$DFFF`       | 8 KiB switchable PRG-ROM window                    | None                              |
//! | CPU  | `$E000-$FFFF`       | 8 KiB fixed PRG-ROM window (last)                  | None                              |
//! | CPU  | `$9000-$B003`       | VRC6 expansion audio registers                      | VRC6 audio                        |
//! | CPU  | `$F000-$F002`       | IRQ control registers                               | VRC6 IRQ                          |
//! | PPU  | `$0000-$1FFF`       | Eight 1 KiB CHR banks with mode‑dependent mapping  | None                              |
//! | PPU  | `$2000-$3EFF`       | Mirroring from VRC6 control (`banking_mode`)       | None                              |

use std::{
    borrow::Cow,
    fs::OpenOptions,
    io::Write,
    sync::{Mutex, OnceLock},
};

use crate::{
    apu::{ExpansionAudio, ExpansionAudioClockContext, ExpansionAudioSink, ExpansionAudioSnapshot},
    audio::AudioChannel,
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
    /// `$9000-$B002` – expansion audio registers.
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

const VRC6_DUTY_TABLE: [[u8; 16]; 8] = [
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // 1/16
    [1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // 2/16
    [1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // 3/16
    [1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // 4/16
    [1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // 5/16
    [1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // 6/16
    [1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0], // 7/16
    [1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0], // 8/16
];

#[derive(Debug, Clone)]
struct Vrc6PulseState {
    volume: u8,
    duty_cycle: u8,
    ignore_duty: bool,
    frequency: u16,
    enabled: bool,
    timer: i32,
    step: u8,
    frequency_shift: u8,
}

impl Vrc6PulseState {
    fn new() -> Self {
        Self {
            volume: 0,
            duty_cycle: 0,
            ignore_duty: false,
            frequency: 1,
            enabled: false,
            timer: 1,
            step: 0,
            frequency_shift: 0,
        }
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0x03 {
            0 => {
                self.volume = value & 0x0F;
                self.duty_cycle = (value >> 4) & 0x07;
                self.ignore_duty = (value & 0x80) != 0;
            }
            1 => {
                self.frequency = (self.frequency & 0x0F00) | value as u16;
            }
            2 => {
                self.frequency = (self.frequency & 0x00FF) | (((value & 0x0F) as u16) << 8);
                self.enabled = (value & 0x80) != 0;
                if !self.enabled {
                    self.step = 0;
                }
            }
            _ => {}
        }
    }

    fn set_frequency_shift(&mut self, shift: u8) {
        self.frequency_shift = shift;
    }

    fn clock(&mut self) {
        if !self.enabled {
            return;
        }

        self.timer -= 1;
        if self.timer <= 0 {
            self.step = (self.step + 1) & 0x0F;
            self.timer = ((self.frequency >> self.frequency_shift) as i32) + 1;
        }
    }

    fn volume(&self) -> u8 {
        if !self.enabled {
            0
        } else if self.ignore_duty {
            self.volume
        } else if VRC6_DUTY_TABLE[self.duty_cycle as usize][self.step as usize] != 0 {
            self.volume
        } else {
            0
        }
    }
}

#[derive(Debug, Clone)]
struct Vrc6SawState {
    accumulator_rate: u8,
    accumulator: u8,
    frequency: u16,
    enabled: bool,
    timer: i32,
    step: u8,
    frequency_shift: u8,
}

impl Vrc6SawState {
    fn new() -> Self {
        Self {
            accumulator_rate: 0,
            accumulator: 0,
            frequency: 1,
            enabled: false,
            timer: 1,
            step: 0,
            frequency_shift: 0,
        }
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0x03 {
            0 => self.accumulator_rate = value & 0x3F,
            1 => self.frequency = (self.frequency & 0x0F00) | value as u16,
            2 => {
                self.frequency = (self.frequency & 0x00FF) | (((value & 0x0F) as u16) << 8);
                self.enabled = (value & 0x80) != 0;
                if !self.enabled {
                    self.accumulator = 0;
                    self.step = 0;
                }
            }
            _ => {}
        }
    }

    fn set_frequency_shift(&mut self, shift: u8) {
        self.frequency_shift = shift;
    }

    fn clock(&mut self) {
        if !self.enabled {
            return;
        }

        self.timer -= 1;
        if self.timer <= 0 {
            self.step = (self.step + 1) % 14;
            self.timer = ((self.frequency >> self.frequency_shift) as i32) + 1;

            if self.step == 0 {
                self.accumulator = 0;
            } else if (self.step & 0x01) == 0 {
                self.accumulator = self.accumulator.wrapping_add(self.accumulator_rate);
            }
        }
    }

    fn volume(&self) -> u8 {
        if self.enabled {
            self.accumulator >> 3
        } else {
            0
        }
    }
}

#[derive(Debug, Clone)]
struct Vrc6AudioState {
    pulse1: Vrc6PulseState,
    pulse2: Vrc6PulseState,
    saw: Vrc6SawState,
    halt_audio: bool,
}

impl Vrc6AudioState {
    fn new() -> Self {
        Self {
            pulse1: Vrc6PulseState::new(),
            pulse2: Vrc6PulseState::new(),
            saw: Vrc6SawState::new(),
            halt_audio: false,
        }
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0xF003 {
            0x9000..=0x9002 => self.pulse1.write_register(addr, value),
            0x9003 => {
                self.halt_audio = (value & 0x01) != 0;
                let shift = if (value & 0x04) != 0 {
                    8
                } else if (value & 0x02) != 0 {
                    4
                } else {
                    0
                };
                self.pulse1.set_frequency_shift(shift);
                self.pulse2.set_frequency_shift(shift);
                self.saw.set_frequency_shift(shift);
            }
            0xA000..=0xA002 => self.pulse2.write_register(addr, value),
            0xB000..=0xB002 => self.saw.write_register(addr, value),
            _ => {}
        }
    }

    fn clock(&mut self) {
        if self.halt_audio {
            return;
        }
        self.pulse1.clock();
        self.pulse2.clock();
        self.saw.clock();
    }

    fn sample(&self) -> f32 {
        // Mesen2 VRC6 path: (pulse1 + pulse2 + saw) * 15
        let level =
            self.pulse1.volume() as i16 + self.pulse2.volume() as i16 + self.saw.volume() as i16;
        (level * 15) as f32
    }
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

    audio: Vrc6AudioState,
    audio_level: f32,
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
            audio: Vrc6AudioState::new(),
            audio_level: 0.0,
        }
    }

    fn translate_address(&self, addr: u16) -> u16 {
        // VRC6b swaps A0/A1 lines.
        (addr & 0xFFFC) | ((addr & 0x0001) << 1) | ((addr & 0x0002) >> 1)
    }

    #[inline]
    fn prg_ram_enabled(&self) -> bool {
        !self.prg_ram.is_empty() && (self.banking_mode & 0x80) != 0
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
        }? % self.prg_bank_count_8k;

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

    /// Map PPU address to CHR bank base + offset according to banking mode.
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

        let bank = match self.banking_mode & 0x03 {
            0 => bank_idx,
            1 => {
                // Banks 0/1,2/3,4/5,6/7 share pairs.
                let pair = bank_idx / 2;
                (pair * 2) | (bank_idx & 1)
            }
            _ => {
                // Mode 2/3: banks 0-3 direct; banks 4/5 mirror reg4; 6/7 mirror reg5.
                if bank_idx < 4 {
                    bank_idx
                } else if bank_idx < 6 {
                    4
                } else {
                    5
                }
            }
        };

        let reg_val = self.chr_regs.get(bank).copied().unwrap_or(0);
        let page = (reg_val & mask) | or_mask;
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
                    self.audio.write_register(addr, value);
                }
                Control => {
                    self.banking_mode = value;
                    self.update_mirroring();
                }
                PrgBankC000 => {
                    self.update_prg_bank_c000(value);
                }
                ChrBankLow => {
                    let idx = (addr & 0x0003) as usize;
                    self.chr_regs[idx] = value;
                }
                ChrBankHigh => {
                    let idx = 4 + (addr & 0x0003) as usize;
                    self.chr_regs[idx] = value;
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
            // CHR ROM nametable modes not modelled; leave mirroring unchanged.
            return;
        }

        self.mirroring = match self.banking_mode & 0x2F {
            0x20 | 0x27 => Mirroring::Vertical,
            0x23 | 0x24 => Mirroring::Horizontal,
            0x28 | 0x2F => Mirroring::SingleScreenLower,
            0x2B | 0x2C => Mirroring::SingleScreenUpper,
            _ => self.base_mirroring,
        };
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

fn mapper26_trace_sink() -> &'static Option<Mutex<std::fs::File>> {
    static TRACE: OnceLock<Option<Mutex<std::fs::File>>> = OnceLock::new();
    TRACE.get_or_init(|| {
        let path = std::env::var("NESIUM_MAPPER26_TRACE_PATH").ok()?;
        if path.trim().is_empty() {
            return None;
        }

        let mut file = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)
            .ok()?;
        let _ = writeln!(
            file,
            "cpu_cycle,addr,translated_addr,value,prg8000_2x,prgC000,banking_mode"
        );
        Some(Mutex::new(file))
    })
}

fn mapper26_trace_write(
    cpu_cycle: u64,
    addr: u16,
    translated_addr: u16,
    value: u8,
    prg_bank_8000_2x: Option<usize>,
    prg_bank_c000: Option<usize>,
    banking_mode: u8,
) {
    let Some(lock) = mapper26_trace_sink().as_ref() else {
        return;
    };
    let prg8000 = prg_bank_8000_2x.map(|v| v as i64).unwrap_or(-1);
    let prgc000 = prg_bank_c000.map(|v| v as i64).unwrap_or(-1);
    if let Ok(mut file) = lock.lock() {
        let _ = writeln!(
            file,
            "{},{:#06X},{:#06X},{:#04X},{},{},{:#04X}",
            cpu_cycle, addr, translated_addr, value, prg8000, prgc000, banking_mode
        );
    }
}

impl Mapper for Mapper26 {
    fn hook_mask(&self) -> MapperHookMask {
        MapperHookMask::CPU_CLOCK
    }

    fn on_mapper_event(&mut self, event: MapperEvent) {
        if let MapperEvent::CpuClock { .. } = event {
            if !self.irq_enabled {
                return;
            }
            self.irq_prescaler -= 3;
            if self.irq_cycle_mode || self.irq_prescaler <= 0 {
                self.clock_irq_counter();
                self.irq_prescaler += 341;
            }
        }
    }

    fn reset(&mut self, _kind: ResetKind) {
        self.prg_bank_8000_2x = None;
        self.prg_bank_c000 = None;
        self.banking_mode = 0;
        self.chr_regs.fill(0);
        self.mirroring = self.base_mirroring;

        self.irq_reload = 0;
        self.irq_counter = 0;
        self.irq_prescaler = 0;
        self.irq_enabled = false;
        self.irq_enabled_after_ack = false;
        self.irq_cycle_mode = false;
        self.irq_pending = false;
        self.audio = Vrc6AudioState::new();
        self.audio_level = 0.0;
    }

    fn cpu_read(&self, addr: u16) -> Option<u8> {
        match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.read_prg_ram(addr),
            cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END => self.read_prg_rom(addr),
            _ => None,
        }
    }

    fn cpu_write(&mut self, addr: u16, data: u8, cpu_cycle: u64) {
        match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.write_prg_ram(addr, data),
            VRC6_IO_WINDOW_START..=VRC6_IO_WINDOW_END => {
                let translated = self.translate_address(addr);
                self.write_register(translated, data);
                mapper26_trace_write(
                    cpu_cycle,
                    addr,
                    translated,
                    data,
                    self.prg_bank_8000_2x,
                    self.prg_bank_c000,
                    self.banking_mode,
                );
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
        26
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Konami VRC6b")
    }

    fn expansion_audio(&self) -> Option<&dyn ExpansionAudio> {
        Some(self)
    }

    fn expansion_audio_mut(&mut self) -> Option<&mut dyn ExpansionAudio> {
        Some(self)
    }
}

impl ExpansionAudio for Mapper26 {
    fn clock_cpu(&mut self, ctx: ExpansionAudioClockContext, sink: &mut dyn ExpansionAudioSink) {
        self.audio.clock();
        let level = self.audio.sample();
        let delta = level - self.audio_level;
        if delta != 0.0 {
            sink.push_delta(AudioChannel::Vrc6, ctx.cpu_cycle, delta);
            self.audio_level = level;
        }
    }

    fn snapshot(&self) -> ExpansionAudioSnapshot {
        ExpansionAudioSnapshot {
            vrc6: self.audio_level,
            ..ExpansionAudioSnapshot::default()
        }
    }
}
