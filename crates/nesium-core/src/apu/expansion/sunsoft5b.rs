use crate::{
    apu::expansion::{
        ExpansionAudio, ExpansionAudioClockContext, ExpansionAudioSink, ExpansionAudioSnapshot,
    },
    audio::AudioChannel,
};

#[derive(Debug, Clone)]
pub struct Sunsoft5bAudio {
    volume_lut: [u8; 0x10],
    current_register: u8,
    registers: [u8; 0x10],
    current_output: f32,
    emitted_output: f32,
    timer: [i16; 3],
    tone_step: [u8; 3],
    process_tick: bool,
}

impl Sunsoft5bAudio {
    pub fn new() -> Self {
        let mut volume_lut = [0u8; 0x10];
        volume_lut[0] = 0;

        let mut output = 1.0f64;
        for item in volume_lut.iter_mut().skip(1) {
            output *= 1.188_502_227_437_018_4;
            output *= 1.188_502_227_437_018_4;
            *item = output as u8;
        }

        Self {
            volume_lut,
            current_register: 0,
            registers: [0; 0x10],
            current_output: 0.0,
            emitted_output: 0.0,
            timer: [0; 3],
            tone_step: [0; 3],
            process_tick: false,
        }
    }

    pub fn write_register(&mut self, addr: u16, value: u8) {
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
        self.current_output = summed as f32;
    }

    fn clock_inner(&mut self) {
        if self.process_tick {
            for channel in 0..3 {
                self.update_channel(channel);
            }
            self.update_output_level();
        }
        self.process_tick = !self.process_tick;
    }
}

impl Default for Sunsoft5bAudio {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpansionAudio for Sunsoft5bAudio {
    fn clock_cpu(&mut self, ctx: ExpansionAudioClockContext, sink: &mut dyn ExpansionAudioSink) {
        self.clock_inner();
        let delta = self.current_output - self.emitted_output;
        if delta != 0.0 {
            sink.push_delta(AudioChannel::Sunsoft5B, ctx.apu_cycle, delta);
            self.emitted_output = self.current_output;
        }
    }

    fn snapshot(&self) -> ExpansionAudioSnapshot {
        ExpansionAudioSnapshot {
            sunsoft5b: self.emitted_output,
            ..ExpansionAudioSnapshot::default()
        }
    }
}
