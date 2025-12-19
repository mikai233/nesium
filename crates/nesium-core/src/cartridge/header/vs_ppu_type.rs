/// Vs. System PPU model id (NES 2.0 byte 13 low nibble, when console type is Vs. System).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VsPpuType {
    /// $0: Any RP2C03/RC2C03 variant.
    AnyRp2c03OrRc2c03,
    /// $2: RP2C04-0001.
    Rp2c04_0001,
    /// $3: RP2C04-0002.
    Rp2c04_0002,
    /// $4: RP2C04-0003.
    Rp2c04_0003,
    /// $5: RP2C04-0004.
    Rp2c04_0004,
    /// $8: RC2C05-01.
    Rc2c05_01,
    /// $9: RC2C05-02.
    Rc2c05_02,
    /// $A: RC2C05-03.
    Rc2c05_03,
    /// $B: RC2C05-04.
    Rc2c05_04,
    /// Reserved/unknown values.
    Unknown(u8),
}

impl VsPpuType {
    pub fn from_nibble(nibble: u8) -> Self {
        match nibble & 0x0F {
            0x0 => Self::AnyRp2c03OrRc2c03,
            0x2 => Self::Rp2c04_0001,
            0x3 => Self::Rp2c04_0002,
            0x4 => Self::Rp2c04_0003,
            0x5 => Self::Rp2c04_0004,
            0x8 => Self::Rc2c05_01,
            0x9 => Self::Rc2c05_02,
            0xA => Self::Rc2c05_03,
            0xB => Self::Rc2c05_04,
            other => Self::Unknown(other),
        }
    }
}
