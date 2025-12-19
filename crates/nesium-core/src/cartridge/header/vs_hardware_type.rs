/// Vs. System hardware/protection type (NES 2.0 byte 13 high nibble, when console type is Vs. System).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VsHardwareType {
    /// $0: Vs. Unisystem (normal).
    UnisystemNormal,
    /// $1: Vs. Unisystem (RBI Baseball protection).
    UnisystemRbiBaseballProtection,
    /// $2: Vs. Unisystem (TKO Boxing protection).
    UnisystemTkoBoxingProtection,
    /// $3: Vs. Unisystem (Super Xevious protection).
    UnisystemSuperXeviousProtection,
    /// $4: Vs. Unisystem (Vs. Ice Climber Japan protection).
    UnisystemIceClimberJapanProtection,
    /// $5: Vs. Dual System (normal).
    DualSystemNormal,
    /// $6: Vs. Dual System (Raid on Bungeling Bay protection).
    DualSystemRaidOnBungelingBayProtection,
    /// Reserved/unknown values.
    Unknown(u8),
}

impl VsHardwareType {
    pub fn from_nibble(nibble: u8) -> Self {
        match nibble & 0x0F {
            0x0 => Self::UnisystemNormal,
            0x1 => Self::UnisystemRbiBaseballProtection,
            0x2 => Self::UnisystemTkoBoxingProtection,
            0x3 => Self::UnisystemSuperXeviousProtection,
            0x4 => Self::UnisystemIceClimberJapanProtection,
            0x5 => Self::DualSystemNormal,
            0x6 => Self::DualSystemRaidOnBungelingBayProtection,
            other => Self::Unknown(other),
        }
    }
}
