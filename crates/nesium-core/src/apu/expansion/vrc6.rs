use crate::{
    apu::expansion::{
        ExpansionAudio, ExpansionAudioClockContext, ExpansionAudioSink, ExpansionAudioSnapshot,
    },
    audio::AudioChannel,
};

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

    fn write_reg(&mut self, addr: u16, value: u8) {
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
        if self.timer == 0 {
            self.step = (self.step + 1) & 0x0F;
            self.timer = ((self.frequency >> self.frequency_shift) + 1) as i32;
        }
    }

    fn volume(&self) -> u8 {
        if !self.enabled {
            0
        } else if self.ignore_duty || self.step <= self.duty_cycle {
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

    fn write_reg(&mut self, addr: u16, value: u8) {
        match addr & 0x03 {
            0 => {
                self.accumulator_rate = value & 0x3F;
            }
            1 => {
                self.frequency = (self.frequency & 0x0F00) | value as u16;
            }
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
        if self.timer == 0 {
            self.step = (self.step + 1) % 14;
            self.timer = ((self.frequency >> self.frequency_shift) + 1) as i32;

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
pub struct Vrc6Audio {
    pulse1: Vrc6PulseState,
    pulse2: Vrc6PulseState,
    saw: Vrc6SawState,
    halt_audio: bool,
    last_output: i32,
}

impl Vrc6Audio {
    pub fn new() -> Self {
        Self {
            pulse1: Vrc6PulseState::new(),
            pulse2: Vrc6PulseState::new(),
            saw: Vrc6SawState::new(),
            halt_audio: false,
            last_output: 0,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn write_register(&mut self, addr: u16, value: u8) {
        match addr & 0xF003 {
            0x9000 | 0x9001 | 0x9002 => self.pulse1.write_reg(addr, value),
            0xA000 | 0xA001 | 0xA002 => self.pulse2.write_reg(addr, value),
            0xB000 | 0xB001 | 0xB002 => self.saw.write_reg(addr, value),
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
            _ => {}
        }
    }

    fn clock_delta(&mut self) -> f32 {
        if !self.halt_audio {
            self.pulse1.clock();
            self.pulse2.clock();
            self.saw.clock();
        }

        let output_level =
            self.pulse1.volume() as i32 + self.pulse2.volume() as i32 + self.saw.volume() as i32;
        let delta = ((output_level - self.last_output) * 15) as f32;
        self.last_output = output_level;
        delta
    }

    fn current_level(&self) -> f32 {
        self.last_output as f32
    }
}

impl Default for Vrc6Audio {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpansionAudio for Vrc6Audio {
    fn clock_cpu(&mut self, ctx: ExpansionAudioClockContext, sink: &mut dyn ExpansionAudioSink) {
        let delta = self.clock_delta();
        if delta != 0.0 {
            sink.push_delta(AudioChannel::Vrc6, ctx.apu_cycle, delta);
        }
    }

    fn snapshot(&self) -> ExpansionAudioSnapshot {
        ExpansionAudioSnapshot {
            vrc6: self.current_level(),
            ..ExpansionAudioSnapshot::default()
        }
    }
}
