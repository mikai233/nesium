use super::{Flags6, Flags7, Mirroring};

/// iNES-defined fields shared by both iNES 1.0 and NES 2.0.
///
/// NES 2.0 is explicitly designed to reuse the original iNES header layout for
/// the first 8 bytes (PRG/CHR LSB sizing + flags 6/7). The remaining bytes are
/// interpreted differently depending on the detected format, so they live in
/// per-format extension structures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct INesHeader {
    /// PRG ROM size least-significant byte (units of 16 KiB).
    pub prg_rom_lsb: u8,
    /// CHR ROM size least-significant byte (units of 8 KiB).
    pub chr_rom_lsb: u8,
    /// iNES flags 6.
    pub flags6: Flags6,
    /// iNES flags 7.
    pub flags7: Flags7,
}

impl INesHeader {
    pub(super) fn from_bytes(bytes: &[u8]) -> Self {
        Self {
            prg_rom_lsb: bytes[4],
            chr_rom_lsb: bytes[5],
            flags6: Flags6::from_bits_truncate(bytes[6]),
            flags7: Flags7::from_bits_truncate(bytes[7]),
        }
    }

    /// How the PPU nametables are mirrored.
    pub fn mirroring(&self) -> Mirroring {
        super::resolve_mirroring(self.flags6)
    }

    /// Whether the optional 512 byte trainer block is present between the header and PRG data.
    pub fn trainer_present(&self) -> bool {
        self.flags6.contains(Flags6::TRAINER)
    }
}
