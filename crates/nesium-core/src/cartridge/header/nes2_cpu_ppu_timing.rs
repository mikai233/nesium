/// NES 2.0 CPU/PPU timing mode (header byte 12 bits 0..=1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Nes2CpuPpuTiming {
    /// 0: RP2C02 ("NTSC NES").
    Rp2c02,
    /// 1: RP2C07 ("Licensed PAL NES").
    Rp2c07,
    /// 2: Multiple-region.
    MultipleRegion,
    /// 3: UA6538 ("Dendy").
    Ua6538,
    /// Reserved/unknown values.
    Unknown(u8),
}

impl Nes2CpuPpuTiming {
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0b11 {
            0 => Self::Rp2c02,
            1 => Self::Rp2c07,
            2 => Self::MultipleRegion,
            3 => Self::Ua6538,
            _ => Self::Unknown(bits),
        }
    }
}
