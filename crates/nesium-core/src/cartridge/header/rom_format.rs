use super::Flags7;

/// Identifies the header flavour encountered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RomFormat {
    /// The original iNES 1.0 specification.
    INes,
    /// NES 2.0 with extended sizing and metadata fields.
    Nes20,
    /// Rare prototypes that pre-date the iNES standard.
    Archaic,
}

impl RomFormat {
    pub(super) fn from_flags7(flags7: Flags7) -> Self {
        match (flags7.bits() >> 2) & 0b11 {
            0b10 => Self::Nes20,
            0b00 => Self::INes,
            _ => Self::Archaic,
        }
    }
}
