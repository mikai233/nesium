#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioChannel {
    Pulse1 = 0,
    Pulse2 = 1,
    Triangle = 2,
    Noise = 3,
    Dmc = 4,
    Fds = 5,
    Mmc5 = 6,
    Namco163 = 7,
    Sunsoft5B = 8,
    Vrc6 = 9,
    Vrc7 = 10,
}

impl AudioChannel {
    pub const COUNT: usize = 11;

    pub fn idx(self) -> usize {
        self as usize
    }
}
