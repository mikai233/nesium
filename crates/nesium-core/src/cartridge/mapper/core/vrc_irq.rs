#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VrcIrq {
    reload: u8,
    counter: u8,
    prescaler: i32,
    enabled: bool,
    enabled_after_ack: bool,
    cycle_mode: bool,
    pending: bool,
}

impl VrcIrq {
    pub fn new() -> Self {
        Self {
            reload: 0,
            counter: 0,
            prescaler: 0,
            enabled: false,
            enabled_after_ack: false,
            cycle_mode: false,
            pending: false,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn reload(&self) -> u8 {
        self.reload
    }

    pub fn pending(&self) -> bool {
        self.pending
    }

    pub fn write_reload_low_nibble(&mut self, value: u8) {
        self.reload = (self.reload & 0xF0) | (value & 0x0F);
    }

    pub fn write_reload_high_nibble(&mut self, value: u8) {
        self.reload = (self.reload & 0x0F) | ((value & 0x0F) << 4);
    }

    pub fn write_reload(&mut self, value: u8) {
        self.reload = value;
    }

    pub fn write_control(&mut self, value: u8) {
        self.enabled_after_ack = (value & 0x01) != 0;
        self.enabled = (value & 0x02) != 0;
        self.cycle_mode = (value & 0x04) != 0;
        if self.enabled {
            self.counter = self.reload;
            self.prescaler = 341;
        }
        self.pending = false;
    }

    pub fn acknowledge(&mut self) {
        self.enabled = self.enabled_after_ack;
        self.pending = false;
    }

    pub fn clock(&mut self) {
        if !self.enabled {
            return;
        }

        if self.cycle_mode {
            self.clock_counter();
        } else {
            self.prescaler -= 3;
            if self.prescaler <= 0 {
                self.clock_counter();
                self.prescaler += 341;
            }
        }
    }

    fn clock_counter(&mut self) {
        if self.counter == 0xFF {
            self.counter = self.reload;
            self.pending = true;
        } else {
            self.counter = self.counter.wrapping_add(1);
        }
    }
}

impl Default for VrcIrq {
    fn default() -> Self {
        Self::new()
    }
}
