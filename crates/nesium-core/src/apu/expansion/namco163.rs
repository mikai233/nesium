use std::cell::Cell;

use crate::{
    apu::expansion::{
        ExpansionAudio, ExpansionAudioClockContext, ExpansionAudioSink, ExpansionAudioSnapshot,
    },
    audio::AudioChannel,
    mem_block::{ByteBlock, MemBlock},
};

const NAMCO163_AUDIO_ADDR_MASK: u16 = 0xF800;
const NAMCO163_AUDIO_RAM_PORT_BASE: u16 = 0x4800;
const NAMCO163_AUDIO_CTRL_PORT_BASE: u16 = 0xE000;
const NAMCO163_AUDIO_ADDR_PORT_BASE: u16 = 0xF800;

type Namco163InternalRam = ByteBlock<0x80>;
type Namco163ChannelOutput = MemBlock<i16, 8>;

#[derive(Debug, Clone)]
pub struct Namco163Audio {
    internal_ram: Namco163InternalRam,
    channel_output: Namco163ChannelOutput,
    ram_position: Cell<u8>,
    auto_increment: Cell<bool>,
    update_counter: u8,
    current_channel: i8,
    current_output: f32,
    emitted_output: f32,
    disabled: bool,
    active: bool,
}

impl Namco163Audio {
    pub fn new() -> Self {
        Self {
            internal_ram: Namco163InternalRam::new(),
            channel_output: Namco163ChannelOutput::new(),
            ram_position: Cell::new(0),
            auto_increment: Cell::new(false),
            update_counter: 0,
            current_channel: 7,
            current_output: 0.0,
            emitted_output: 0.0,
            disabled: false,
            active: true,
        }
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
    }

    pub fn write_register(&mut self, addr: u16, value: u8) {
        match addr & NAMCO163_AUDIO_ADDR_MASK {
            NAMCO163_AUDIO_RAM_PORT_BASE => {
                let pos = self.ram_position.get();
                self.internal_ram[pos as usize] = value;
                if self.auto_increment.get() {
                    self.ram_position.set((pos + 1) & 0x7F);
                }
            }
            NAMCO163_AUDIO_CTRL_PORT_BASE => {
                self.disabled = (value & 0x40) != 0;
            }
            NAMCO163_AUDIO_ADDR_PORT_BASE => {
                self.ram_position.set(value & 0x7F);
                self.auto_increment.set((value & 0x80) != 0);
            }
            _ => {}
        }
    }

    pub fn read_register(&self, addr: u16) -> u8 {
        match addr & NAMCO163_AUDIO_ADDR_MASK {
            NAMCO163_AUDIO_RAM_PORT_BASE => {
                let pos = self.ram_position.get();
                let value = self.internal_ram[pos as usize];
                if self.auto_increment.get() {
                    self.ram_position.set((pos + 1) & 0x7F);
                }
                value
            }
            _ => 0,
        }
    }

    fn num_channels(&self) -> u8 {
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
        self.update_output_level();
        self.set_phase(channel, phase);
    }

    fn update_output_level(&mut self) {
        let channels = self.num_channels();
        let active = channels as i16 + 1;
        let mut sum = 0i16;
        for i in (7 - channels as i8)..=7 {
            sum += self.channel_output[i as usize];
        }
        self.current_output = (sum / active) as f32;
    }

    fn clock_inner(&mut self) {
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
}

impl Default for Namco163Audio {
    fn default() -> Self {
        Self::new()
    }
}

impl ExpansionAudio for Namco163Audio {
    fn clock_cpu(&mut self, ctx: ExpansionAudioClockContext, sink: &mut dyn ExpansionAudioSink) {
        if self.active {
            self.clock_inner();
            let delta = self.current_output - self.emitted_output;
            if delta != 0.0 {
                sink.push_delta(AudioChannel::Namco163, ctx.apu_cycle, delta);
                self.emitted_output = self.current_output;
            }
        } else if self.emitted_output != 0.0 {
            sink.push_delta(AudioChannel::Namco163, ctx.apu_cycle, -self.emitted_output);
            self.emitted_output = 0.0;
        }
    }

    fn snapshot(&self) -> ExpansionAudioSnapshot {
        ExpansionAudioSnapshot {
            namco163: self.emitted_output,
            ..ExpansionAudioSnapshot::default()
        }
    }
}
