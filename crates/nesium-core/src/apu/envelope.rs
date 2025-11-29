//! Envelope unit shared by pulse and noise channels.

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct Envelope {
    loop_flag: bool,
    constant_volume: bool,
    volume: u8,
    start: bool,
    divider: u8,
    decay_level: u8,
}

impl Envelope {
    pub(super) fn configure(&mut self, value: u8) {
        self.loop_flag = value & 0b0010_0000 != 0;
        self.constant_volume = value & 0b0001_0000 != 0;
        self.volume = value & 0b0000_1111;
    }

    pub(super) fn restart(&mut self) {
        self.start = true;
    }

    pub(super) fn clock(&mut self) {
        if self.start {
            self.start = false;
            self.decay_level = 15;
            self.divider = self.volume;
            return;
        }

        if self.divider == 0 {
            self.divider = self.volume;
            if self.decay_level > 0 {
                self.decay_level -= 1;
            } else if self.loop_flag {
                self.decay_level = 15;
            }
        } else {
            self.divider -= 1;
        }
    }

    pub(super) fn output(&self) -> u8 {
        if self.constant_volume {
            self.volume
        } else {
            self.decay_level
        }
    }

    pub(super) fn halt_length(&self) -> bool {
        self.loop_flag
    }
}
