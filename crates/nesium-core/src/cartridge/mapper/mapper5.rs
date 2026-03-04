use std::{borrow::Cow, cell::Cell};

use crate::{
    apu::{ExpansionAudio, ExpansionAudioClockContext, ExpansionAudioSink, ExpansionAudioSnapshot},
    audio::{AudioChannel, CPU_CLOCK_NTSC},
    cartridge::{
        ChrRom, Mapper, PrgRom, TrainerBytes,
        header::{Header, Mirroring},
        mapper::{
            ChrStorage, CpuBusAccessKind, MapperEvent, MapperHookMask, NametableTarget,
            PpuRenderFetchTarget, PpuRenderFetchType, PpuVramAccessContext, PpuVramAccessKind,
            allocate_prg_ram_with_trainer, select_chr_storage,
        },
    },
    reset_kind::ResetKind,
};

// Mapper 5 – MMC5 with extended PRG/CHR/nametable features.
//
// | Area | Address range     | Behaviour                                              | IRQ/Audio      |
// |------|-------------------|--------------------------------------------------------|----------------|
// | CPU  | `$6000-$7FFF`     | Bankswitched PRG-RAM via `$5113` (when enabled)       | None           |
// | CPU  | `$8000-$FFFF`     | PRG ROM/RAM windows in 8/16/32 KiB modes (`$5100`)    | MMC5 scanline  |
// | CPU  | `$5100-$5117`     | PRG/CHR/ExRAM/nametable control + PRG banking regs    | None           |
// | CPU  | `$5120-$5127`     | CHR bank registers (1/2/4/8 KiB modes)                | None           |
// | CPU  | `$5200-$5206`     | Split-screen, IRQ, multiplier, and status registers   | MMC5 scanline  |
// | CPU  | `$5C00-$5FFF`     | 1 KiB ExRAM CPU window (mode‑dependent behaviour)     | None           |
// | PPU  | `$0000-$1FFF`     | CHR ROM/RAM with flexible banking via `$5120-$5127`   | MMC5 scanline  |
// | PPU  | `$2000-$3EFF`     | Nametable mapping/fill using ExRAM and `$5105-$5107`  | None           |

/// MMC5 PRG bank size (8 KiB).
const PRG_BANK_SIZE_8K: usize = 8 * 1024;

/// MMC5 has 1 KiB of internal extended RAM.
const EXRAM_SIZE: usize = 1024;

/// CPU `$6000-$7FFF`: bankswitched PRG-RAM window controlled by `$5113`.
const MMC5_PRG_RAM_WINDOW_START: u16 = 0x6000;
const MMC5_PRG_RAM_WINDOW_END: u16 = 0x7FFF;

/// CPU `$8000-$FFFF`: bankswitched PRG-ROM/PRG-RAM windows.
const MMC5_PRG_WINDOW_START: u16 = 0x8000;
const MMC5_PRG_WINDOW_END: u16 = 0xFFFF;
/// CPU `$A000`, `$C000`, `$E000`: boundaries between PRG sub-windows.
const MMC5_PRG_WINDOW_A000_START: u16 = 0xA000;
const MMC5_PRG_WINDOW_C000_START: u16 = 0xC000;
const MMC5_PRG_WINDOW_E000_START: u16 = 0xE000;

/// MMC5 control/configuration registers in `$5100-$5107`.
/// - `$5100`: PRG mode (8/16/32 KiB windows).
/// - `$5101`: CHR mode (1/2/4/8 KiB pages).
/// - `$5102/$5103`: PRG-RAM write-protect keys.
/// - `$5104`: ExRAM mode (nametable/attribute/CPU RAM behaviour).
/// - `$5105`: per-nametable mapping control.
/// - `$5106/$5107`: fill tile and attribute for extended nametable modes.
const MMC5_REG_PRG_MODE: u16 = 0x5100;
const MMC5_REG_CHR_MODE: u16 = 0x5101;
const MMC5_REG_PRG_RAM_PROTECT1: u16 = 0x5102;
const MMC5_REG_PRG_RAM_PROTECT2: u16 = 0x5103;
const MMC5_REG_EXRAM_MODE: u16 = 0x5104;
const MMC5_REG_NAMETABLE_MAPPING: u16 = 0x5105;
const MMC5_REG_FILL_TILE: u16 = 0x5106;
const MMC5_REG_FILL_ATTR: u16 = 0x5107;

/// MMC5 PRG banking registers.
/// - `$5113`: PRG-RAM page for `$6000-$7FFF`.
/// - `$5114-$5117`: PRG-ROM/PRG-RAM bank registers for `$8000/$A000/$C000/$E000`.
const MMC5_REG_PRG_BANK_6000_7FFF: u16 = 0x5113;
const MMC5_REG_PRG_BANK_8000: u16 = 0x5114;
const MMC5_REG_PRG_BANK_A000: u16 = 0x5115;
const MMC5_REG_PRG_BANK_C000: u16 = 0x5116;
const MMC5_REG_PRG_BANK_E000: u16 = 0x5117;

/// MMC5 CHR banking registers `$5120-$512B` and upper CHR bank bits `$5130`.
const MMC5_REG_CHR_BANK_FIRST: u16 = 0x5120;
const MMC5_REG_CHR_BANK_LAST: u16 = 0x512B;
const MMC5_REG_CHR_BANK_A_LAST: u16 = 0x5127;
const MMC5_REG_CHR_UPPER_BITS: u16 = 0x5130;

/// MMC5 split-screen / IRQ / multiplier registers in `$5200-$5206`.
const MMC5_REG_SPLIT_CONTROL: u16 = 0x5200;
const MMC5_REG_SPLIT_SCROLL: u16 = 0x5201;
const MMC5_REG_SPLIT_CHR_BANK: u16 = 0x5202;
/// CPU `$5203`: scanline IRQ target.
const MMC5_REG_IRQ_SCANLINE: u16 = 0x5203;
/// CPU `$5204`: IRQ status (pending + in-frame bits).
const MMC5_REG_IRQ_STATUS: u16 = 0x5204;
/// CPU `$5205/$5206`: 8×8→16 multiplier result low/high bytes.
const MMC5_REG_MULTIPLIER_A: u16 = 0x5205;
const MMC5_REG_MULTIPLIER_B: u16 = 0x5206;

/// CPU `$5C00-$5FFF`: ExRAM CPU window.
const MMC5_EXRAM_CPU_START: u16 = 0x5C00;
const MMC5_EXRAM_CPU_END: u16 = 0x5FFF;

/// MMC5 expansion audio registers.
const MMC5_REG_AUDIO_SQ1_CTRL: u16 = 0x5000;
const MMC5_REG_AUDIO_SQ1_SWEEP: u16 = 0x5001;
const MMC5_REG_AUDIO_SQ1_TIMER_LO: u16 = 0x5002;
const MMC5_REG_AUDIO_SQ1_TIMER_HI: u16 = 0x5003;
const MMC5_REG_AUDIO_SQ2_CTRL: u16 = 0x5004;
const MMC5_REG_AUDIO_SQ2_SWEEP: u16 = 0x5005;
const MMC5_REG_AUDIO_SQ2_TIMER_LO: u16 = 0x5006;
const MMC5_REG_AUDIO_SQ2_TIMER_HI: u16 = 0x5007;
const MMC5_REG_AUDIO_PCM_CTRL: u16 = 0x5010;
const MMC5_REG_AUDIO_PCM_DATA: u16 = 0x5011;
const MMC5_REG_AUDIO_STATUS: u16 = 0x5015;

/// CPU-visible MMC5 register set.
///
/// MMC5 exposes a rich set of control registers across `$5100-$5206` as well
/// as CPU-mapped ExRAM and PRG-RAM/PRG-ROM windows. This enum groups the
/// major logical registers so that CPU-side logic can work with names instead
/// of raw addresses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Mmc5CpuRegister {
    AudioSquare1,
    AudioSquare2,
    AudioPcmControl,
    AudioPcmData,
    AudioStatus,
    PrgMode,
    ChrMode,
    PrgRamProtect1,
    PrgRamProtect2,
    ExRamMode,
    NametableMapping,
    FillTile,
    FillAttr,
    PrgBank6000,
    PrgBank8000,
    PrgBankA000,
    PrgBankC000,
    PrgBankE000,
    ChrBank,
    ChrUpperBits,
    SplitControl,
    SplitScroll,
    SplitChrBank,
    IrqScanline,
    IrqStatus,
    MultiplierA,
    MultiplierB,
    ExRamCpu,
    PrgRamWindow,
    PrgWindow,
}

impl Mmc5CpuRegister {
    fn from_addr(addr: u16) -> Option<Self> {
        use Mmc5CpuRegister::*;

        match addr {
            MMC5_REG_AUDIO_SQ1_CTRL
            | MMC5_REG_AUDIO_SQ1_SWEEP
            | MMC5_REG_AUDIO_SQ1_TIMER_LO
            | MMC5_REG_AUDIO_SQ1_TIMER_HI => Some(AudioSquare1),
            MMC5_REG_AUDIO_SQ2_CTRL
            | MMC5_REG_AUDIO_SQ2_SWEEP
            | MMC5_REG_AUDIO_SQ2_TIMER_LO
            | MMC5_REG_AUDIO_SQ2_TIMER_HI => Some(AudioSquare2),
            MMC5_REG_AUDIO_PCM_CTRL => Some(AudioPcmControl),
            MMC5_REG_AUDIO_PCM_DATA => Some(AudioPcmData),
            MMC5_REG_AUDIO_STATUS => Some(AudioStatus),
            MMC5_REG_PRG_MODE => Some(PrgMode),
            MMC5_REG_CHR_MODE => Some(ChrMode),
            MMC5_REG_PRG_RAM_PROTECT1 => Some(PrgRamProtect1),
            MMC5_REG_PRG_RAM_PROTECT2 => Some(PrgRamProtect2),
            MMC5_REG_EXRAM_MODE => Some(ExRamMode),
            MMC5_REG_NAMETABLE_MAPPING => Some(NametableMapping),
            MMC5_REG_FILL_TILE => Some(FillTile),
            MMC5_REG_FILL_ATTR => Some(FillAttr),
            MMC5_REG_PRG_BANK_6000_7FFF => Some(PrgBank6000),
            MMC5_REG_PRG_BANK_8000 => Some(PrgBank8000),
            MMC5_REG_PRG_BANK_A000 => Some(PrgBankA000),
            MMC5_REG_PRG_BANK_C000 => Some(PrgBankC000),
            MMC5_REG_PRG_BANK_E000 => Some(PrgBankE000),
            MMC5_REG_CHR_BANK_FIRST..=MMC5_REG_CHR_BANK_LAST => Some(ChrBank),
            MMC5_REG_CHR_UPPER_BITS => Some(ChrUpperBits),
            MMC5_REG_SPLIT_CONTROL => Some(SplitControl),
            MMC5_REG_SPLIT_SCROLL => Some(SplitScroll),
            MMC5_REG_SPLIT_CHR_BANK => Some(SplitChrBank),
            MMC5_REG_IRQ_SCANLINE => Some(IrqScanline),
            MMC5_REG_IRQ_STATUS => Some(IrqStatus),
            MMC5_REG_MULTIPLIER_A => Some(MultiplierA),
            MMC5_REG_MULTIPLIER_B => Some(MultiplierB),
            MMC5_EXRAM_CPU_START..=MMC5_EXRAM_CPU_END => Some(ExRamCpu),
            MMC5_PRG_RAM_WINDOW_START..=MMC5_PRG_RAM_WINDOW_END => Some(PrgRamWindow),
            MMC5_PRG_WINDOW_START..=MMC5_PRG_WINDOW_END => Some(PrgWindow),
            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum PrgWindowSize {
    /// 8 KiB CPU window.
    Size8K,
    /// 16 KiB CPU window.
    Size16K,
    /// 32 KiB CPU window.
    Size32K,
}

const MMC5_LENGTH_TABLE: [u8; 32] = [
    10, 254, 20, 2, 40, 4, 80, 6, 160, 8, 60, 10, 14, 12, 26, 14, 12, 16, 24, 18, 48, 20, 96, 22,
    192, 24, 72, 26, 16, 28, 32, 30,
];

const MMC5_PULSE_DUTY_TABLE: [[u8; 8]; 4] = [
    [0, 0, 0, 0, 0, 0, 0, 1],
    [0, 0, 0, 0, 0, 0, 1, 1],
    [0, 0, 0, 0, 1, 1, 1, 1],
    [1, 1, 1, 1, 1, 1, 0, 0],
];

const MMC5_FRAME_COUNTER_PERIOD_CPU: i32 = (CPU_CLOCK_NTSC as i32) / 240;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Mmc5LengthCounter {
    enabled: bool,
    halt: bool,
    new_halt: bool,
    counter: u8,
    reload_value: u8,
    previous_value: u8,
}

impl Mmc5LengthCounter {
    fn new() -> Self {
        Self {
            enabled: false,
            halt: false,
            new_halt: false,
            counter: 0,
            reload_value: 0,
            previous_value: 0,
        }
    }

    fn initialize(&mut self, halt_flag: bool) {
        self.new_halt = halt_flag;
    }

    fn load(&mut self, index: u8) {
        if self.enabled {
            self.reload_value = MMC5_LENGTH_TABLE[index as usize];
            self.previous_value = self.counter;
        }
    }

    fn tick(&mut self) {
        if self.counter > 0 && !self.halt {
            self.counter -= 1;
        }
    }

    fn reload_counter(&mut self) {
        if self.reload_value != 0 {
            if self.counter == self.previous_value {
                self.counter = self.reload_value;
            }
            self.reload_value = 0;
        }
        self.halt = self.new_halt;
    }

    fn set_enabled(&mut self, enabled: bool) {
        if !enabled {
            self.counter = 0;
        }
        self.enabled = enabled;
    }

    fn active(&self) -> bool {
        self.counter > 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Mmc5Envelope {
    constant_volume: bool,
    loop_flag: bool,
    volume: u8,
    start: bool,
    divider: i16,
    decay_level: u8,
}

impl Mmc5Envelope {
    fn new() -> Self {
        Self {
            constant_volume: false,
            loop_flag: false,
            volume: 0,
            start: false,
            divider: 0,
            decay_level: 0,
        }
    }

    fn initialize(&mut self, value: u8) {
        self.loop_flag = (value & 0x20) != 0;
        self.constant_volume = (value & 0x10) != 0;
        self.volume = value & 0x0F;
    }

    fn restart(&mut self) {
        self.start = true;
    }

    fn tick(&mut self) {
        if !self.start {
            self.divider -= 1;
            if self.divider < 0 {
                self.divider = self.volume as i16;
                if self.decay_level > 0 {
                    self.decay_level -= 1;
                } else if self.loop_flag {
                    self.decay_level = 15;
                }
            }
        } else {
            self.start = false;
            self.decay_level = 15;
            self.divider = self.volume as i16;
        }
    }

    fn output(&self, length_active: bool) -> u8 {
        if !length_active {
            0
        } else if self.constant_volume {
            self.volume
        } else {
            self.decay_level
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Mmc5PulseState {
    envelope: Mmc5Envelope,
    length_counter: Mmc5LengthCounter,
    duty: u8,
    duty_pos: u8,
    period: u16,
    timer: u16,
    current_output: i16,
}

impl Mmc5PulseState {
    fn new() -> Self {
        Self {
            envelope: Mmc5Envelope::new(),
            length_counter: Mmc5LengthCounter::new(),
            duty: 0,
            duty_pos: 0,
            period: 0,
            timer: 0,
            current_output: 0,
        }
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0x03 {
            0 => {
                self.envelope.initialize(value);
                self.length_counter.initialize((value & 0x20) != 0);
                self.duty = (value >> 6) & 0x03;
            }
            1 => {
                // MMC5 pulse channels do not implement sweep ($5001/$5005 ignored).
            }
            2 => {
                self.period = (self.period & 0x0700) | value as u16;
            }
            3 => {
                self.length_counter.load((value >> 3) & 0x1F);
                self.period = (self.period & 0x00FF) | (((value & 0x07) as u16) << 8);
                self.duty_pos = 0;
                self.envelope.restart();
            }
            _ => {}
        }
    }

    fn run_channel(&mut self) {
        if self.timer == 0 {
            self.duty_pos = self.duty_pos.wrapping_sub(1) & 0x07;
            let duty_bit = MMC5_PULSE_DUTY_TABLE[self.duty as usize][self.duty_pos as usize] as i16;
            self.current_output =
                duty_bit * self.envelope.output(self.length_counter.active()) as i16;
            self.timer = self.period;
        } else {
            self.timer -= 1;
        }
    }

    fn tick_length_counter(&mut self) {
        self.length_counter.tick();
    }

    fn tick_envelope(&mut self) {
        self.envelope.tick();
    }

    fn reload_length_counter(&mut self) {
        self.length_counter.reload_counter();
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.length_counter.set_enabled(enabled);
    }

    fn status(&self) -> bool {
        self.length_counter.active()
    }

    fn output(&self) -> i16 {
        self.current_output
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Mmc5AudioState {
    square1: Mmc5PulseState,
    square2: Mmc5PulseState,
    audio_counter: i32,
    pcm_read_mode: bool,
    pcm_irq_enabled: bool,
    pcm_output: u8,
}

impl Mmc5AudioState {
    fn new() -> Self {
        Self {
            square1: Mmc5PulseState::new(),
            square2: Mmc5PulseState::new(),
            audio_counter: 0,
            pcm_read_mode: false,
            pcm_irq_enabled: false,
            pcm_output: 0,
        }
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        match addr {
            MMC5_REG_AUDIO_SQ1_CTRL
            | MMC5_REG_AUDIO_SQ1_SWEEP
            | MMC5_REG_AUDIO_SQ1_TIMER_LO
            | MMC5_REG_AUDIO_SQ1_TIMER_HI => self.square1.write_register(addr, value),
            MMC5_REG_AUDIO_SQ2_CTRL
            | MMC5_REG_AUDIO_SQ2_SWEEP
            | MMC5_REG_AUDIO_SQ2_TIMER_LO
            | MMC5_REG_AUDIO_SQ2_TIMER_HI => self.square2.write_register(addr, value),
            MMC5_REG_AUDIO_PCM_CTRL => {
                self.pcm_read_mode = (value & 0x01) != 0;
                self.pcm_irq_enabled = (value & 0x80) != 0;
            }
            MMC5_REG_AUDIO_PCM_DATA => {
                if !self.pcm_read_mode && value != 0 {
                    self.pcm_output = value;
                }
            }
            MMC5_REG_AUDIO_STATUS => {
                self.square1.set_enabled((value & 0x01) != 0);
                self.square2.set_enabled((value & 0x02) != 0);
            }
            _ => {}
        }
    }

    fn read_register(&self, addr: u16) -> u8 {
        match addr {
            // PCM IRQ/read mode side effects are not implemented yet.
            MMC5_REG_AUDIO_PCM_CTRL => 0,
            MMC5_REG_AUDIO_STATUS => {
                let mut status = 0u8;
                if self.square1.status() {
                    status |= 0x01;
                }
                if self.square2.status() {
                    status |= 0x02;
                }
                status
            }
            _ => 0,
        }
    }

    fn clock_and_sample(&mut self) -> f32 {
        self.audio_counter -= 1;
        self.square1.run_channel();
        self.square2.run_channel();

        if self.audio_counter <= 0 {
            self.audio_counter = MMC5_FRAME_COUNTER_PERIOD_CPU;
            self.square1.tick_length_counter();
            self.square1.tick_envelope();
            self.square2.tick_length_counter();
            self.square2.tick_envelope();
        }

        let summed = -(self.square1.output() + self.square2.output() + self.pcm_output as i16);

        self.square1.reload_length_counter();
        self.square2.reload_length_counter();

        summed as f32
    }
}

#[derive(Debug, Clone)]
pub struct Mapper5 {
    prg_rom: PrgRom,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,
    exram: Box<[u8; EXRAM_SIZE]>,

    /// PRG ROM bank count in 8 KiB units.
    prg_bank_count: usize,

    // Configuration registers
    prg_mode: u8,         // $5100 (0-3)
    chr_mode: u8,         // $5101 (0-3)
    prg_ram_protect1: u8, // $5102
    prg_ram_protect2: u8, // $5103
    exram_mode: u8,       // $5104 (0-3)
    nt_mapping: u8,       // $5105
    fill_tile: u8,        // $5106
    fill_attr: u8,        // $5107 (low 2 bits used)

    // PRG banking registers ($5113-$5117).
    prg_bank_6000_7fff: u8, // $5113 (PRG-RAM / PRG-ROM, simplified)
    prg_bank_8000: u8,      // $5114
    prg_bank_a000: u8,      // $5115
    prg_bank_c000: u8,      // $5116
    prg_bank_e000: u8,      // $5117

    // CHR banking registers ($5120-$512B).
    chr_banks: [u16; 12],
    chr_upper_bits: u8, // $5130 (upper CHR bank bits)
    last_chr_reg: u16,
    ppu_ctrl: u8,

    // IRQ / scanline registers.
    irq_scanline: u8, // $5203
    irq_enabled: bool,
    irq_pending: Cell<bool>,

    // Vertical split registers ($5200-$5202). We currently only latch these;
    // proper split rendering requires richer PPU context (tile X/Y, BG vs
    // sprite, etc.) exposed from the PPU core.
    split_control: u8,  // $5200
    split_scroll: u8,   // $5201
    split_chr_bank: u8, // $5202
    split_tile: u16,
    sprite_fetch_window: bool,

    // Unsigned 8x8->16 multiplier ($5205/$5206).
    mul_a: u8,
    mul_b: u8,
    mul_result: u16,

    // Scanline IRQ / frame tracking state.
    scanline_counter: u8,
    in_frame: bool,
    need_in_frame: bool,
    ppu_idle_counter: u8,
    last_cpu_cycle: u64,
    last_ppu_read_addr: u16,
    nt_read_counter: u8,
    split_tile_number: u8,
    // ExGrafix state for ExRAM mode 1 read override sequencing.
    ex_attr_last_nametable_fetch: u16,
    ex_attr_fetch_remaining: u8,
    ex_attr_selected_chr_bank: u16,

    audio: Mmc5AudioState,
    audio_level: f32,
}

impl Mapper5 {
    pub fn new(header: Header, prg_rom: PrgRom, chr_rom: ChrRom, trainer: TrainerBytes) -> Self {
        let prg_ram = allocate_prg_ram_with_trainer(&header, trainer);

        // MMC5 boards often have large PRG-RAM; the allocate_prg_ram helper
        // already considers NES 2.0 hints. Games that rely on banking PRG-RAM
        // across CPU windows still work when we treat PRG-RAM as a flat superset.
        let exram = Box::new([0u8; EXRAM_SIZE]);

        let prg_bank_count = (prg_rom.len() / PRG_BANK_SIZE_8K).max(1);

        Self {
            prg_rom,
            prg_ram,
            chr: select_chr_storage(&header, chr_rom),
            exram,
            prg_bank_count,
            prg_mode: 3, // default to 4×8 KiB banking
            chr_mode: 3, // default to 1 KiB CHR pages
            prg_ram_protect1: 0,
            prg_ram_protect2: 0,
            exram_mode: 0,
            nt_mapping: 0,
            fill_tile: 0,
            fill_attr: 0,
            prg_bank_6000_7fff: 0,
            prg_bank_8000: 0,
            prg_bank_a000: 0,
            prg_bank_c000: 0,
            prg_bank_e000: 0,
            chr_banks: [0; 12],
            chr_upper_bits: 0,
            last_chr_reg: 0,
            ppu_ctrl: 0,
            irq_scanline: 0,
            irq_enabled: false,
            irq_pending: Cell::new(false),
            split_control: 0,
            split_scroll: 0,
            split_chr_bank: 0,
            split_tile: 0,
            sprite_fetch_window: false,
            mul_a: 0xFF,
            mul_b: 0xFF,
            // Power-on default $FE01 per MMC5A docs.
            mul_result: 0xFF * 0xFF,
            scanline_counter: 0,
            in_frame: false,
            need_in_frame: false,
            ppu_idle_counter: 0,
            last_cpu_cycle: 0,
            last_ppu_read_addr: 0,
            nt_read_counter: 0,
            split_tile_number: 0,
            ex_attr_last_nametable_fetch: 0,
            ex_attr_fetch_remaining: 0,
            ex_attr_selected_chr_bank: 0,
            audio: Mmc5AudioState::new(),
            audio_level: 0.0,
        }
    }

    fn prg_rom_bank(&self, bank: u8) -> usize {
        if self.prg_bank_count == 0 {
            0
        } else {
            (bank as usize) % self.prg_bank_count
        }
    }

    fn prg_ram_enabled(&self) -> bool {
        // Simple implementation: require $5102=0x02 and $5103=0x01 to enable
        // PRG-RAM writes. Reads are allowed whenever RAM is present.
        !self.prg_ram.is_empty()
            && self.prg_ram_protect1 & 0x03 == 0x02
            && self.prg_ram_protect2 & 0x03 == 0x01
    }

    /// Decode the effective 8 KiB PRG-ROM bank for a given register and
    /// window size, following the MMC5 bit layout described on Nesdev.
    fn prg_rom_bank_index(&self, reg: u8, size: PrgWindowSize, addr: u16) -> usize {
        if self.prg_bank_count == 0 {
            return 0;
        }

        // bit7 is RAM/ROM select, address decoding uses only bits 6..0.
        let reg7 = reg & 0x7F;
        let bank = match size {
            PrgWindowSize::Size8K => reg7 as usize,
            PrgWindowSize::Size16K => {
                // Bits 6..1 select A19..A14, CPU A13 selects the low bit.
                let a13 = ((addr >> 13) & 0x01) as usize;
                ((reg7 & 0x7E) as usize) | a13
            }
            PrgWindowSize::Size32K => {
                // Bits 6..2 select A19..A15, CPU A14..A13 provide the low bits.
                let a13 = ((addr >> 13) & 0x01) as usize;
                let a14 = ((addr >> 14) & 0x01) as usize;
                let high = (reg7 & 0x7C) as usize;
                let low = (a14 << 1) | a13;
                high | low
            }
        };

        bank % self.prg_bank_count
    }

    fn read_prg_rom_window(&self, addr: u16, reg: u8, size: PrgWindowSize) -> u8 {
        if self.prg_rom.is_empty() {
            return 0;
        }

        let bank = self.prg_rom_bank_index(reg, size, addr);
        let base = bank.saturating_mul(PRG_BANK_SIZE_8K);
        let offset = (addr as usize) & (PRG_BANK_SIZE_8K - 1);
        let idx = base.saturating_add(offset);
        self.prg_rom.get(idx).copied().unwrap_or(0)
    }

    /// Read PRG-RAM through a bankswitched window.
    fn read_prg_ram_page(&self, addr: u16, reg: u8) -> u8 {
        if self.prg_ram.is_empty() {
            return 0;
        }

        // Use low 3 bits as 8 KiB page index (superset mapping from Nesdev).
        let page = (reg & 0x07) as usize;
        let base = page * PRG_BANK_SIZE_8K;
        let offset = (addr as usize) & (PRG_BANK_SIZE_8K - 1);
        let idx = base + offset;
        let len = self.prg_ram.len();
        self.prg_ram[idx % len]
    }

    /// Write PRG-RAM through a bankswitched window, honoring write protection.
    fn write_prg_ram_page(&mut self, addr: u16, reg: u8, value: u8) {
        if self.prg_ram.is_empty() || !self.prg_ram_enabled() {
            return;
        }

        let page = (reg & 0x07) as usize;
        let base = page * PRG_BANK_SIZE_8K;
        let offset = (addr as usize) & (PRG_BANK_SIZE_8K - 1);
        let idx = base + offset;
        let len = self.prg_ram.len();
        if len != 0 {
            let wrapped = idx % len;
            self.prg_ram[wrapped] = value;
        }
    }

    /// Helper for PRG-ROM/PRG-RAM switchable windows (modes 1–3).
    fn read_prg_window_switchable(&self, addr: u16, reg: u8, size: PrgWindowSize) -> u8 {
        // $5114-$5116: bit7 = 0 => RAM, bit7 = 1 => ROM.
        let use_ram = (reg & 0x80) == 0 && !self.prg_ram.is_empty();
        if use_ram {
            self.read_prg_ram_page(addr, reg)
        } else {
            self.read_prg_rom_window(addr, reg, size)
        }
    }

    fn read_prg(&self, addr: u16) -> Option<u8> {
        match addr {
            MMC5_PRG_RAM_WINDOW_START..=MMC5_PRG_RAM_WINDOW_END => {
                if self.prg_ram.is_empty() {
                    return None;
                }
                // $6000-$7FFF always map PRG-RAM via $5113.
                Some(self.read_prg_ram_page(addr, self.prg_bank_6000_7fff))
            }
            MMC5_PRG_WINDOW_START..=MMC5_PRG_WINDOW_END => {
                let mode = self.prg_mode & 0x03;
                let value = match mode {
                    0 => {
                        // PRG mode 0: one 32 KiB ROM bank at $8000-$FFFF (ROM only).
                        self.read_prg_rom_window(addr, self.prg_bank_e000, PrgWindowSize::Size32K)
                    }
                    1 => {
                        // PRG mode 1:
                        // $8000-$BFFF: 16 KiB switchable ROM/RAM via $5115.
                        // $C000-$FFFF: 16 KiB ROM via $5117.
                        if addr < MMC5_PRG_WINDOW_C000_START {
                            self.read_prg_window_switchable(
                                addr,
                                self.prg_bank_a000,
                                PrgWindowSize::Size16K,
                            )
                        } else {
                            self.read_prg_rom_window(
                                addr,
                                self.prg_bank_e000,
                                PrgWindowSize::Size16K,
                            )
                        }
                    }
                    2 => {
                        // PRG mode 2:
                        // $8000-$BFFF: 16 KiB switchable ROM/RAM via $5115.
                        // $C000-$DFFF: 8 KiB switchable ROM/RAM via $5116.
                        // $E000-$FFFF: 8 KiB ROM via $5117.
                        if addr < MMC5_PRG_WINDOW_C000_START {
                            self.read_prg_window_switchable(
                                addr,
                                self.prg_bank_a000,
                                PrgWindowSize::Size16K,
                            )
                        } else if addr < MMC5_PRG_WINDOW_E000_START {
                            self.read_prg_window_switchable(
                                addr,
                                self.prg_bank_c000,
                                PrgWindowSize::Size8K,
                            )
                        } else {
                            self.read_prg_rom_window(
                                addr,
                                self.prg_bank_e000,
                                PrgWindowSize::Size8K,
                            )
                        }
                    }
                    _ => {
                        // PRG mode 3 (default for most games):
                        // $8000-$9FFF: 8 KiB ROM/RAM via $5114.
                        // $A000-$BFFF: 8 KiB ROM/RAM via $5115.
                        // $C000-$DFFF: 8 KiB ROM/RAM via $5116.
                        // $E000-$FFFF: 8 KiB ROM via $5117.
                        if addr < MMC5_PRG_WINDOW_A000_START {
                            self.read_prg_window_switchable(
                                addr,
                                self.prg_bank_8000,
                                PrgWindowSize::Size8K,
                            )
                        } else if addr < MMC5_PRG_WINDOW_C000_START {
                            self.read_prg_window_switchable(
                                addr,
                                self.prg_bank_a000,
                                PrgWindowSize::Size8K,
                            )
                        } else if addr < MMC5_PRG_WINDOW_E000_START {
                            self.read_prg_window_switchable(
                                addr,
                                self.prg_bank_c000,
                                PrgWindowSize::Size8K,
                            )
                        } else {
                            self.read_prg_rom_window(
                                addr,
                                self.prg_bank_e000,
                                PrgWindowSize::Size8K,
                            )
                        }
                    }
                };
                Some(value)
            }
            _ => None,
        }
    }

    fn write_prg(&mut self, addr: u16, data: u8) {
        match Mmc5CpuRegister::from_addr(addr) {
            // MMC5 control/config registers live in $5100-$51FF and $5200+.
            Some(Mmc5CpuRegister::AudioSquare1)
            | Some(Mmc5CpuRegister::AudioSquare2)
            | Some(Mmc5CpuRegister::AudioPcmControl)
            | Some(Mmc5CpuRegister::AudioPcmData)
            | Some(Mmc5CpuRegister::AudioStatus) => self.audio.write_register(addr, data),
            Some(Mmc5CpuRegister::PrgMode) => self.prg_mode = data & 0x03,
            Some(Mmc5CpuRegister::ChrMode) => self.chr_mode = data & 0x03,
            Some(Mmc5CpuRegister::PrgRamProtect1) => self.prg_ram_protect1 = data,
            Some(Mmc5CpuRegister::PrgRamProtect2) => self.prg_ram_protect2 = data,
            Some(Mmc5CpuRegister::ExRamMode) => self.exram_mode = data & 0x03,
            Some(Mmc5CpuRegister::NametableMapping) => self.nt_mapping = data,
            Some(Mmc5CpuRegister::FillTile) => self.fill_tile = data,
            Some(Mmc5CpuRegister::FillAttr) => self.fill_attr = data & 0x03,
            Some(Mmc5CpuRegister::PrgBank6000) => self.prg_bank_6000_7fff = data,
            Some(Mmc5CpuRegister::PrgBank8000) => self.prg_bank_8000 = data,
            Some(Mmc5CpuRegister::PrgBankA000) => self.prg_bank_a000 = data,
            Some(Mmc5CpuRegister::PrgBankC000) => self.prg_bank_c000 = data,
            Some(Mmc5CpuRegister::PrgBankE000) => self.prg_bank_e000 = data,
            Some(Mmc5CpuRegister::ChrBank) => {
                let idx = (addr - MMC5_REG_CHR_BANK_FIRST) as usize;
                self.chr_banks[idx] = ((self.chr_upper_bits as u16) << 8) | (data as u16);
                self.last_chr_reg = addr;
            }
            Some(Mmc5CpuRegister::ChrUpperBits) => self.chr_upper_bits = data & 0x03,
            Some(Mmc5CpuRegister::SplitControl) => {
                // Vertical split control. We only latch the value here; the
                // actual split behaviour is implemented in the PPU bus-address hook
                // and currently requires more detailed PPU context.
                self.split_control = data;
                // TODO: Use split_control in the PPU bus-address hook/map_nametable.
                // `PpuVramAccessContext` now includes `render_fetch` metadata
                // (target/fetch phase/tile coords); MMC5 split logic still needs
                // to be wired to those fields.
            }
            Some(Mmc5CpuRegister::SplitScroll) => {
                // Vertical split scroll value.
                self.split_scroll = data;
                // TODO: Honour split_scroll when emulating the split region.
            }
            Some(Mmc5CpuRegister::SplitChrBank) => {
                // Vertical split CHR bank.
                self.split_chr_bank = data;
                // TODO: Use split_chr_bank for BG CHR selection in split area.
            }
            Some(Mmc5CpuRegister::IrqScanline) => {
                self.irq_scanline = data;
                // Writes that modify the compare value also acknowledge a pending IRQ.
                self.irq_pending.set(false);
            }
            Some(Mmc5CpuRegister::IrqStatus) => {
                // Writing with bit7 set enables IRQ, clearing it disables.
                self.irq_enabled = data & 0x80 != 0;
            }
            Some(Mmc5CpuRegister::MultiplierA) => {
                // Unsigned 8-bit multiplicand.
                self.mul_a = data;
                self.mul_result = (self.mul_a as u16) * (self.mul_b as u16);
            }
            Some(Mmc5CpuRegister::MultiplierB) => {
                // Unsigned 8-bit multiplier.
                self.mul_b = data;
                self.mul_result = (self.mul_a as u16) * (self.mul_b as u16);
            }
            Some(Mmc5CpuRegister::ExRamCpu) => {
                // Internal ExRAM writes. $5104 controls CPU accessibility:
                // modes 0/1 are write-only, mode 2 is read/write, mode 3 is
                // read-only. In modes 0/1, writes outside rendering store 0.
                let idx = (addr - MMC5_EXRAM_CPU_START) as usize;
                if idx < EXRAM_SIZE {
                    match self.exram_mode & 0x03 {
                        0 | 1 => {
                            self.exram[idx] = if self.in_frame { data } else { 0 };
                        }
                        2 => {
                            self.exram[idx] = data;
                        }
                        _ => {
                            // Mode 3 is read-only.
                        }
                    }
                }
            }
            Some(Mmc5CpuRegister::PrgRamWindow) => {
                // $6000-$7FFF always map PRG-RAM via $5113.
                self.write_prg_ram_page(addr, self.prg_bank_6000_7fff, data);
            }
            Some(Mmc5CpuRegister::PrgWindow) => {
                // Some PRG windows in modes 1–3 can be mapped to PRG-RAM.
                if self.prg_ram.is_empty() || !self.prg_ram_enabled() {
                    return;
                }
                let mode = self.prg_mode & 0x03;
                match mode {
                    0 => {
                        // Mode 0 has PRG-ROM only at $8000-$FFFF.
                    }
                    1 => {
                        // $8000-$BFFF: 16 KiB ROM/RAM via $5115.
                        if addr < MMC5_PRG_WINDOW_C000_START && (self.prg_bank_a000 & 0x80) == 0 {
                            self.write_prg_ram_page(addr, self.prg_bank_a000, data);
                        }
                    }
                    2 => {
                        // $8000-$BFFF: 16 KiB ROM/RAM via $5115.
                        // $C000-$DFFF: 8 KiB ROM/RAM via $5116.
                        if addr < MMC5_PRG_WINDOW_C000_START {
                            if (self.prg_bank_a000 & 0x80) == 0 {
                                self.write_prg_ram_page(addr, self.prg_bank_a000, data);
                            }
                        } else if addr < MMC5_PRG_WINDOW_E000_START
                            && (self.prg_bank_c000 & 0x80) == 0
                        {
                            self.write_prg_ram_page(addr, self.prg_bank_c000, data);
                        }
                    }
                    _ => {
                        // Mode 3: three 8 KiB ROM/RAM windows.
                        if addr < MMC5_PRG_WINDOW_A000_START {
                            if (self.prg_bank_8000 & 0x80) == 0 {
                                self.write_prg_ram_page(addr, self.prg_bank_8000, data);
                            }
                        } else if addr < MMC5_PRG_WINDOW_C000_START {
                            if (self.prg_bank_a000 & 0x80) == 0 {
                                self.write_prg_ram_page(addr, self.prg_bank_a000, data);
                            }
                        } else if addr < MMC5_PRG_WINDOW_E000_START
                            && (self.prg_bank_c000 & 0x80) == 0
                        {
                            self.write_prg_ram_page(addr, self.prg_bank_c000, data);
                        }
                        // $E000-$FFFF is ROM-only.
                    }
                }
            }
            _ => {}
        }
    }

    fn chr_bank_for_addr(&self, addr: u16) -> (usize, usize) {
        // Decode CHR register + bank size based on CHR mode and CHR A/B set.
        let mode = self.chr_mode & 0x03;
        let chr_a = self.chr_a_selected();
        let (reg_index, bank_size) = match mode {
            0 => (if chr_a { 7usize } else { 11usize }, 0x2000usize),
            1 => {
                if addr < 0x1000 {
                    (if chr_a { 3usize } else { 11usize }, 0x1000usize)
                } else {
                    (if chr_a { 7usize } else { 11usize }, 0x1000usize)
                }
            }
            2 => match addr {
                0x0000..=0x07FF => (if chr_a { 1usize } else { 9usize }, 0x0800usize),
                0x0800..=0x0FFF => (if chr_a { 3usize } else { 11usize }, 0x0800usize),
                0x1000..=0x17FF => (if chr_a { 5usize } else { 9usize }, 0x0800usize),
                _ => (if chr_a { 7usize } else { 11usize }, 0x0800usize),
            },
            _ => {
                let index = ((addr as usize) >> 10) & 0x07;
                let reg_index = if chr_a {
                    index
                } else {
                    [8usize, 9, 10, 11, 8, 9, 10, 11][index]
                };
                (reg_index, 0x0400usize)
            }
        };

        let bank_index = self.chr_banks[reg_index] as usize;
        (bank_index, bank_size)
    }

    fn read_chr(&self, addr: u16) -> u8 {
        let (bank_index, bank_size) = self.chr_bank_for_addr(addr);
        let base = bank_index.saturating_mul(bank_size);
        let offset = (addr as usize) & (bank_size - 1);
        self.chr.read_indexed(base, offset)
    }

    fn write_chr(&mut self, addr: u16, value: u8) {
        let (bank_index, bank_size) = self.chr_bank_for_addr(addr);
        let base = bank_index.saturating_mul(bank_size);
        let offset = (addr as usize) & (bank_size - 1);
        self.chr.write_indexed(base, offset, value);
    }

    fn exram_index_for_nametable(&self, offset: u16) -> usize {
        // ExRAM is a single 1 KiB window mirrored for any nametable that maps
        // to it. Offset is already relative to the nametable (0-0x3FF).
        (offset as usize) & (EXRAM_SIZE - 1)
    }

    fn is_fill_offset(offset: u16) -> bool {
        offset & 0x1000 != 0
    }

    fn decode_fill_offset(offset: u16) -> u16 {
        offset & 0x03FF
    }

    fn read_chr_from_4k_bank(&self, bank_4k: u16, addr: u16) -> u8 {
        let base = (bank_4k as usize) << 12;
        let offset = (addr as usize) & 0x0FFF;
        self.chr.read_indexed(base, offset)
    }

    fn tick_cpu_idle_counter(&mut self, cpu_cycle: u64) {
        if self.last_cpu_cycle == 0 {
            self.last_cpu_cycle = cpu_cycle;
            return;
        }

        let elapsed = cpu_cycle.saturating_sub(self.last_cpu_cycle);
        self.last_cpu_cycle = cpu_cycle;
        if elapsed == 0 || self.ppu_idle_counter == 0 {
            return;
        }

        let dec = elapsed.min(self.ppu_idle_counter as u64) as u8;
        self.ppu_idle_counter = self.ppu_idle_counter.saturating_sub(dec);
        if self.ppu_idle_counter == 0 {
            self.in_frame = false;
        }
    }

    #[inline]
    fn is_nametable_tile_fetch(addr: u16) -> bool {
        (0x2000..=0x2FFF).contains(&addr) && (addr & 0x03FF) < 0x03C0
    }

    fn detect_scanline_start(&mut self, addr: u16) {
        if self.nt_read_counter >= 2 {
            // After 3 identical NT reads, the following attribute fetch marks a scanline step.
            if !self.in_frame && !self.need_in_frame {
                self.need_in_frame = true;
                self.scanline_counter = 0;
            } else {
                self.scanline_counter = self.scanline_counter.wrapping_add(1);
                if self.irq_scanline != 0 && self.scanline_counter == self.irq_scanline {
                    self.irq_pending.set(true);
                }
            }
        } else if (0x2000..=0x2FFF).contains(&addr) && self.last_ppu_read_addr == addr {
            self.nt_read_counter = self.nt_read_counter.saturating_add(1);
            if self.nt_read_counter >= 2 {
                self.split_tile_number = 0;
            }
        }

        if self.last_ppu_read_addr != addr {
            self.nt_read_counter = 0;
        }
    }

    fn chr_a_selected(&self) -> bool {
        let large_sprites = (self.ppu_ctrl & 0x20) != 0;
        if !large_sprites {
            return true;
        }

        if self.sprite_fetch_window {
            return true;
        }

        !self.in_frame && self.last_chr_reg <= MMC5_REG_CHR_BANK_A_LAST
    }
}

impl Mapper for Mapper5 {
    fn hook_mask(&self) -> MapperHookMask {
        MapperHookMask::CPU_BUS_ACCESS
            | MapperHookMask::PPU_BUS_ADDRESS
            | MapperHookMask::PPU_READ_OVERRIDE
    }

    fn on_mapper_event(&mut self, event: MapperEvent) {
        match event {
            MapperEvent::CpuClock { .. } => {}
            MapperEvent::CpuBusAccess {
                kind,
                addr,
                value,
                cpu_cycle,
                ..
            } => {
                self.tick_cpu_idle_counter(cpu_cycle);

                let is_write = matches!(
                    kind,
                    CpuBusAccessKind::Write
                        | CpuBusAccessKind::DmaWrite
                        | CpuBusAccessKind::DummyWrite
                );
                let is_read = matches!(
                    kind,
                    CpuBusAccessKind::Read
                        | CpuBusAccessKind::DmaRead
                        | CpuBusAccessKind::ExecOpcode
                        | CpuBusAccessKind::ExecOperand
                        | CpuBusAccessKind::DummyRead
                );

                if is_write && (0x2000..=0x3FFF).contains(&addr) && (addr & 0x2007) == 0x2000 {
                    self.ppu_ctrl = value;
                    if (self.ppu_ctrl & 0x20) == 0 {
                        self.last_chr_reg = 0;
                    }
                }

                if is_read && (addr == 0xFFFA || addr == 0xFFFB) {
                    self.in_frame = false;
                    self.ppu_idle_counter = 0;
                    self.last_cpu_cycle = 0;
                    self.last_ppu_read_addr = 0;
                    self.nt_read_counter = 0;
                    self.scanline_counter = 0;
                    self.sprite_fetch_window = false;
                    self.irq_pending.set(false);
                }
            }
            MapperEvent::PpuBusAddress { addr, ctx } => {
                if ctx.kind != PpuVramAccessKind::RenderingFetch {
                    return;
                }

                self.in_frame = true;
                self.ppu_idle_counter = 3;

                if let Some(fetch) = ctx.render_fetch {
                    self.sprite_fetch_window = fetch.target == PpuRenderFetchTarget::Sprite;
                    if fetch.target == PpuRenderFetchTarget::Background
                        && fetch.fetch == PpuRenderFetchType::Nametable
                        && (0..=239).contains(&ctx.ppu_scanline)
                        && self.irq_scanline != 0
                        && ctx.ppu_scanline as u8 == self.irq_scanline
                    {
                        self.irq_pending.set(true);
                    }
                }

                if Self::is_nametable_tile_fetch(addr) {
                    self.split_tile_number = self.split_tile_number.wrapping_add(1);
                    if !self.in_frame && self.need_in_frame {
                        self.need_in_frame = false;
                        self.in_frame = true;
                    }
                }

                self.detect_scanline_start(addr);
                self.last_ppu_read_addr = addr;
            }
        }
    }

    fn reset(&mut self, kind: ResetKind) {
        if !matches!(kind, ResetKind::PowerOn) {
            return;
        }

        // Mesen2-style defaults: PRG/CHR 8 KiB/1 KiB modes, ExRAM mode 0, all
        // banks pointing at the start of PRG/CHR.
        self.prg_mode = 3;
        self.chr_mode = 3;
        self.prg_ram_protect1 = 0;
        self.prg_ram_protect2 = 0;
        self.exram_mode = 0;
        self.nt_mapping = 0;
        self.fill_tile = 0;
        self.fill_attr = 0;
        self.prg_bank_6000_7fff = 0;
        self.prg_bank_8000 = 0;
        self.prg_bank_a000 = 1;
        self.prg_bank_c000 = 2;
        self.prg_bank_e000 = (self.prg_bank_count.saturating_sub(1)) as u8;
        self.chr_banks.fill(0);
        self.chr_upper_bits = 0;
        self.last_chr_reg = 0;
        self.ppu_ctrl = 0;
        self.irq_scanline = 0;
        self.irq_enabled = false;
        self.irq_pending.set(false);
        self.split_control = 0;
        self.split_scroll = 0;
        self.split_chr_bank = 0;
        self.split_tile = 0;
        self.sprite_fetch_window = false;
        self.mul_a = 0xFF;
        self.mul_b = 0xFF;
        self.mul_result = 0xFF * 0xFF; // $FE01
        self.scanline_counter = 0;
        self.in_frame = false;
        self.need_in_frame = false;
        self.ppu_idle_counter = 0;
        self.last_cpu_cycle = 0;
        self.last_ppu_read_addr = 0;
        self.nt_read_counter = 0;
        self.split_tile_number = 0;
        self.ex_attr_last_nametable_fetch = 0;
        self.ex_attr_fetch_remaining = 0;
        self.ex_attr_selected_chr_bank = 0;
        self.audio = Mmc5AudioState::new();
        self.audio_level = 0.0;
        self.exram.fill(0);
    }

    fn cpu_read(&self, addr: u16) -> Option<u8> {
        match Mmc5CpuRegister::from_addr(addr) {
            Some(Mmc5CpuRegister::AudioPcmControl) | Some(Mmc5CpuRegister::AudioStatus) => {
                Some(self.audio.read_register(addr))
            }
            Some(Mmc5CpuRegister::IrqStatus) => {
                // IRQ status ($5204). We expose the pending and "in frame" bits,
                // clearing the pending flag on read to match hardware ack
                // semantics. Bit 7 latches when the scanline IRQ triggers and
                // stays set until the CPU polls this register or rewrites the
                // IRQ counter.
                let mut value = 0u8;
                if self.irq_pending.get() {
                    value |= 0x80;
                }
                if self.in_frame {
                    value |= 0x40;
                }
                // Reading $5204 acknowledges a latched IRQ.
                // (Bit 6 remains as-is to reflect in-frame state.)
                // In-frame flag is not cleared here; it follows PPU fetch timing.
                // Source: observed emulator behaviour (Mesen2) and Nesdev docs.
                // Matches NES hardware by deasserting the IRQ level after the
                // CPU observes it.
                self.irq_pending.set(false);
                Some(value)
            }
            Some(Mmc5CpuRegister::MultiplierA) => Some(self.mul_result as u8),
            Some(Mmc5CpuRegister::MultiplierB) => Some((self.mul_result >> 8) as u8),
            Some(Mmc5CpuRegister::ExRamCpu) => {
                // Internal ExRAM CPU reads ($5C00-$5FFF).
                let idx = (addr - MMC5_EXRAM_CPU_START) as usize;
                if idx >= EXRAM_SIZE {
                    return Some(0);
                }
                let mode = self.exram_mode & 0x03;
                match mode {
                    0 | 1 => {
                        // Modes 0/1: CPU reads are open bus.
                        None
                    }
                    _ => {
                        // Modes 2/3: CPU can read ExRAM.
                        Some(self.exram[idx])
                    }
                }
            }
            _ => self.read_prg(addr),
        }
    }

    fn cpu_write(&mut self, addr: u16, data: u8, _cpu_cycle: u64) {
        self.write_prg(addr, data);
    }

    fn ppu_read(&self, addr: u16) -> Option<u8> {
        // MMC5 CHR banking only applies to pattern table space.
        if addr < 0x2000 {
            Some(self.read_chr(addr))
        } else {
            None
        }
    }

    fn ppu_write(&mut self, addr: u16, data: u8) {
        if addr < 0x2000 {
            self.write_chr(addr, data);
        }
    }

    fn ppu_read_override(&mut self, addr: u16, ctx: PpuVramAccessContext, value: u8) -> u8 {
        if ctx.kind != PpuVramAccessKind::RenderingFetch {
            return value;
        }

        // Vertical split mode ($5200-$5202): override background fetches in the
        // split region using ExRAM nametable/attribute data and $5202 CHR bank.
        if (self.exram_mode & 0x03) <= 0b01 && self.in_frame && (self.split_control & 0x80) != 0 {
            if let Some(fetch) = ctx.render_fetch {
                if fetch.target == PpuRenderFetchTarget::Background {
                    let tile_x = fetch.tile_x.unwrap_or(0);
                    let delimiter = self.split_control & 0x1F;
                    let right_side = (self.split_control & 0x40) != 0;
                    let in_region = if right_side {
                        tile_x >= delimiter
                    } else {
                        tile_x < delimiter
                    };

                    if in_region {
                        let scanline =
                            ((ctx.ppu_scanline.max(0) as u8).wrapping_add(self.split_scroll)) % 240;
                        match fetch.fetch {
                            PpuRenderFetchType::Nametable => {
                                let column = tile_x & 0x1F;
                                self.split_tile =
                                    (((scanline & 0xF8) as u16) << 2) | (column as u16);
                                let idx = self.exram_index_for_nametable(self.split_tile);
                                return self.exram[idx];
                            }
                            PpuRenderFetchType::Attribute => {
                                let shift =
                                    ((self.split_tile >> 4) & 0x04) | (self.split_tile & 0x02);
                                let attr_addr = 0x03C0
                                    | ((self.split_tile & 0x0380) >> 4)
                                    | ((self.split_tile & 0x001F) >> 2);
                                let attr = self.exram[self.exram_index_for_nametable(attr_addr)];
                                let palette = (attr >> shift) & 0x03;
                                return palette * 0x55;
                            }
                            PpuRenderFetchType::PatternLow | PpuRenderFetchType::PatternHigh => {
                                let row_addr = (((addr & !0x0007) | ((scanline as u16) & 0x0007))
                                    & 0x0FFF) as u16;
                                return self
                                    .read_chr_from_4k_bank(self.split_chr_bank as u16, row_addr);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // ExRAM mode 1 (ExGrafix): nametable fetches are not modified, but
        // the following attribute + tile low/high fetches are overridden
        // using the matching ExRAM entry.
        let ex_attr_active_window = !self.sprite_fetch_window;
        if (self.exram_mode & 0x03) != 0b01 || !self.in_frame || !ex_attr_active_window {
            self.ex_attr_fetch_remaining = 0;
            return value;
        }

        let is_nt_fetch = (0x2000..=0x2FFF).contains(&addr) && (addr & 0x03FF) < 0x03C0;
        if is_nt_fetch {
            self.ex_attr_last_nametable_fetch = addr & 0x03FF;
            self.ex_attr_fetch_remaining = 3;
            return value;
        }

        if self.ex_attr_fetch_remaining == 0 {
            return value;
        }

        self.ex_attr_fetch_remaining = self.ex_attr_fetch_remaining.saturating_sub(1);
        match self.ex_attr_fetch_remaining {
            2 => {
                let ex_idx = self.exram_index_for_nametable(self.ex_attr_last_nametable_fetch);
                let ex = self.exram[ex_idx];
                // Low 6 bits select a 4 KiB CHR bank, top two come from $5130.
                self.ex_attr_selected_chr_bank =
                    ((self.chr_upper_bits as u16) << 6) | ((ex as u16) & 0x3F);
                let palette = (ex >> 6) & 0x03;
                palette * 0x55
            }
            1 | 0 => self.read_chr_from_4k_bank(self.ex_attr_selected_chr_bank, addr),
            _ => value,
        }
    }

    fn map_nametable(&self, addr: u16) -> NametableTarget {
        // Mirror $3000-$3EFF to $2000-$2EFF before MMC5 nametable mapping.
        if !(0x2000..=0x3EFF).contains(&addr) {
            return NametableTarget::Ciram(addr & 0x07FF);
        }
        let mirrored = if addr >= 0x3000 { addr - 0x1000 } else { addr };
        let nt = ((mirrored - 0x2000) / 0x0400) as u8; // 0..3
        let offset = (mirrored - 0x2000) & 0x03FF;
        let sel_bits = (self.nt_mapping >> (nt * 2)) & 0x03;
        match sel_bits {
            0 => {
                // CIRAM page 0
                NametableTarget::Ciram(offset)
            }
            1 => {
                // CIRAM page 1
                NametableTarget::Ciram(0x0400 | offset)
            }
            2 => {
                // Internal ExRAM.
                NametableTarget::MapperVram(offset)
            }
            3 => {
                // Fill mode: encode using high bit so mapper_nametable_* can
                // distinguish it from ExRAM-backed nametables.
                NametableTarget::MapperVram(0x1000 | offset)
            }
            _ => NametableTarget::Ciram(offset),
        }
    }

    fn mapper_nametable_read(&self, offset: u16) -> u8 {
        if Self::is_fill_offset(offset) {
            let rel = Self::decode_fill_offset(offset);
            // Fill-mode tile vs attribute behaviour depends on the offset.
            if rel < 0x03C0 {
                // Nametable entries replaced by fill-tile byte.
                self.fill_tile
            } else {
                // Attribute bytes replaced by fill-color replicated into all 4 quads.
                let bits = self.fill_attr & 0x03;
                // Replicate two bits across the byte: b1b0 b1b0 b1b0 b1b0.
                bits * 0x55
            }
        } else {
            // ExRAM-backed nametable. When $5104 is %10 or %11, the
            // nametable reads back as all zeros instead of exposing the
            // underlying RAM (per Nesdev). We still allow CPU access to
            // ExRAM via $5C00-$5FFF regardless of this.
            if (self.exram_mode & 0x03) >= 0b10 {
                0
            } else {
                let idx = self.exram_index_for_nametable(offset);
                self.exram[idx]
            }
        }
    }

    fn mapper_nametable_write(&mut self, offset: u16, value: u8) {
        if Self::is_fill_offset(offset) {
            // Writes to fill-mode nametables are ignored; only $5106/$5107 matter.
            let _ = (offset, value);
        } else {
            let idx = self.exram_index_for_nametable(offset);
            self.exram[idx] = value;
        }
    }

    fn irq_pending(&self) -> bool {
        self.irq_pending.get() && self.irq_enabled
    }

    fn expansion_audio(&self) -> Option<&dyn ExpansionAudio> {
        Some(self)
    }

    fn expansion_audio_mut(&mut self) -> Option<&mut dyn ExpansionAudio> {
        Some(self)
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

    fn mapper_ram(&self) -> Option<&[u8]> {
        Some(self.exram.as_ref())
    }

    fn mapper_ram_mut(&mut self) -> Option<&mut [u8]> {
        Some(self.exram.as_mut())
    }

    fn mirroring(&self) -> Mirroring {
        // MMC5 nametable mapping is fully controlled by $5105; advertise
        // mapper-controlled mirroring to the rest of the system.
        Mirroring::MapperControlled
    }

    fn mapper_id(&self) -> u16 {
        5
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("MMC5")
    }
}

impl ExpansionAudio for Mapper5 {
    fn clock_cpu(&mut self, ctx: ExpansionAudioClockContext, sink: &mut dyn ExpansionAudioSink) {
        let level = self.audio.clock_and_sample();
        let delta = level - self.audio_level;
        if delta != 0.0 {
            sink.push_delta(AudioChannel::Mmc5, ctx.cpu_cycle, delta);
            self.audio_level = level;
        }
    }

    fn snapshot(&self) -> ExpansionAudioSnapshot {
        ExpansionAudioSnapshot {
            mmc5: self.audio_level,
            ..ExpansionAudioSnapshot::default()
        }
    }
}
