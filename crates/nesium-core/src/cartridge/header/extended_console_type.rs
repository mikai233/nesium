/// NES 2.0 extended console type (NES 2.0 byte 13 low nibble, when console type is Extended).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtendedConsoleType {
    /// $0: Regular NES/Famicom/Dendy.
    Regular,
    /// $1: Nintendo Vs. System.
    VsSystem,
    /// $2: PlayChoice-10.
    PlayChoice10,
    /// $3: Regular Famiclone, but with CPU that supports Decimal Mode.
    FamicloneWithDecimalMode,
    /// $4: Regular NES/Famicom with EPSM module or plug-through cartridge.
    NesFamicomWithEpsm,
    /// $5: V.R. Technology VT01 with red/cyan STN palette.
    Vt01RedCyanStnPalette,
    /// $6: V.R. Technology VT02.
    Vt02,
    /// $7: V.R. Technology VT03.
    Vt03,
    /// $8: V.R. Technology VT09.
    Vt09,
    /// $9: V.R. Technology VT32.
    Vt32,
    /// $A: V.R. Technology VT369.
    Vt369,
    /// $B: UMC UM6578.
    UmcUm6578,
    /// $C: Famicom Network System.
    FamicomNetworkSystem,
    /// Reserved/unknown values.
    Unknown(u8),
}

impl ExtendedConsoleType {
    pub fn from_nibble(nibble: u8) -> Self {
        match nibble & 0x0F {
            0x0 => Self::Regular,
            0x1 => Self::VsSystem,
            0x2 => Self::PlayChoice10,
            0x3 => Self::FamicloneWithDecimalMode,
            0x4 => Self::NesFamicomWithEpsm,
            0x5 => Self::Vt01RedCyanStnPalette,
            0x6 => Self::Vt02,
            0x7 => Self::Vt03,
            0x8 => Self::Vt09,
            0x9 => Self::Vt32,
            0xA => Self::Vt369,
            0xB => Self::UmcUm6578,
            0xC => Self::FamicomNetworkSystem,
            other => Self::Unknown(other),
        }
    }
}
