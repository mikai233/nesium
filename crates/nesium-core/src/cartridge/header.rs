//! Cartridge loading primitives.
//!
//! The first 16 bytes of every `.nes` ROM are the *iNES header*. It stores how much
//! PRG/CHR data the cartridge exposes, which mapper is required, and a few
//! compatibility flags. Modern dumps may use the extended **NES 2.0** flavour of the
//! header, so the parser in this module understands both variants and presents the
//! data in a single beginner friendly [`Header`] structure.
//!
//! # Quick overview
//! - Read the first 16 bytes and pass them to [`Header::parse`].
//! - Inspect `header.mapper` to pick or construct a concrete [`mapper::Mapper`].
//! - Use `header.prg_rom_size` / `header.chr_rom_size` to slice the raw PRG/CHR
//!   sections out of the file.
//!
//! Unsupported or damaged headers turn into a descriptive [`HeaderParseError`].

use bitflags::bitflags;

use crate::error::Error;

const NES_MAGIC: &[u8; 4] = b"NES\x1A";

/// Size of the fixed iNES header in bytes.
pub const NES_HEADER_LEN: usize = 16;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Flags6: u8 {
        const MIRRORING        = 0b0000_0001;
        const BATTERY          = 0b0000_0010;
        const TRAINER          = 0b0000_0100;
        const FOUR_SCREEN      = 0b0000_1000;
        const MAPPER_LOW_MASK  = 0b1111_0000;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct Flags7: u8 {
        const VS_UNISYSTEM     = 0b0000_0001;
        const PLAYCHOICE_10    = 0b0000_0010;
        const NES2_DETECTION   = 0b0000_1100;
        const MAPPER_HIGH_MASK = 0b1111_0000;
    }
}

/// Layout mirroring type for the PPU nametables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mirroring {
    /// Two horizontal nametables that mirror vertically (common for NTSC games).
    Horizontal,
    /// Two vertical nametables that mirror horizontally.
    Vertical,
    /// Cartridge supplies its own four nametables.
    FourScreen,
}

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
    fn from_flags7(flags7: Flags7) -> Self {
        match (flags7.bits() >> 2) & 0b11 {
            0b10 => Self::Nes20,
            0b00 => Self::INes,
            _ => Self::Archaic,
        }
    }
}

/// Video timing hints embedded in the header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TvSystem {
    /// NTSC (60Hz) timing.
    Ntsc,
    /// PAL (50Hz) timing.
    Pal,
    /// Cartridge can run on either timing without modification.
    Dual, // region free: supports NTSC and PAL timing
    Dendy, // hybrid timing used by some Famiclones
    Unknown,
}

/// High level representation of an iNES / NES 2.0 cartridge header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Header {
    /// Detected header flavour.
    pub format: RomFormat,
    /// Mapper ID (0 == NROM, 1 == MMC1, ...).
    pub mapper: u16,
    /// NES 2.0 submapper value. Always 0 for legacy iNES files.
    pub submapper: u8,
    /// How the PPU nametables are mirrored.
    pub mirroring: Mirroring,
    /// Battery bit indicates the cartridge keeps RAM contents when powered off.
    pub battery_backed_ram: bool,
    /// Whether the optional 512 byte trainer block is present between the header and PRG data.
    pub trainer_present: bool,
    /// Amount of PRG ROM in bytes.
    pub prg_rom_size: usize,
    /// Amount of CHR ROM in bytes.
    pub chr_rom_size: usize,
    /// Volatile PRG RAM size (CPU accessible). Defaults to 8 KiB for legacy dumps that store 0.
    pub prg_ram_size: usize,
    /// Battery backed PRG RAM size.
    pub prg_nvram_size: usize,
    /// Volatile CHR RAM size located on the PPU side.
    pub chr_ram_size: usize,
    /// Battery backed CHR RAM size.
    pub chr_nvram_size: usize,
    /// Set when the game targets the Vs. UniSystem arcade hardware.
    pub vs_unisystem: bool,
    /// Set when the cartridge contains PlayChoice-10 data.
    pub playchoice_10: bool,
    /// Region / timing hints described in the header.
    pub tv_system: TvSystem,
}

impl Header {
    /// Parse an iNES header from the given byte slice.
    pub fn parse(bytes: &[u8]) -> Result<Self, Error> {
        if bytes.len() < NES_HEADER_LEN {
            return Err(Error::TooShort {
                actual: bytes.len(),
            });
        }

        if &bytes[0..4] != NES_MAGIC {
            return Err(Error::InvalidMagic);
        }

        let flags6 = Flags6::from_bits_truncate(bytes[6]);
        let flags7 = Flags7::from_bits_truncate(bytes[7]);

        let format = RomFormat::from_flags7(flags7);
        match format {
            RomFormat::INes => Self::parse_ines(bytes, flags6, flags7),
            RomFormat::Nes20 => Self::parse_nes20(bytes, flags6, flags7),
            RomFormat::Archaic => Err(Error::UnsupportedFormat(format)),
        }
    }

    fn parse_ines(bytes: &[u8], flags6: Flags6, flags7: Flags7) -> Result<Self, Error> {
        let prg_rom_units = bytes[4] as usize;
        let chr_rom_units = bytes[5] as usize;
        let prg_ram_units = bytes[8].max(1) as usize; // Header stores 0 for "assume 8 KiB".
        let tv_system = if bytes[9] & 0b1 == 0 {
            TvSystem::Ntsc
        } else {
            TvSystem::Pal
        };

        Ok(Self {
            format: RomFormat::INes,
            mapper: combine_mapper(flags6, flags7, 0),
            submapper: 0,
            mirroring: resolve_mirroring(flags6),
            battery_backed_ram: flags6.contains(Flags6::BATTERY),
            trainer_present: flags6.contains(Flags6::TRAINER),
            prg_rom_size: prg_rom_units * 16 * 1024,
            chr_rom_size: chr_rom_units * 8 * 1024,
            prg_ram_size: prg_ram_units * 8 * 1024,
            prg_nvram_size: if flags6.contains(Flags6::BATTERY) {
                prg_ram_units * 8 * 1024
            } else {
                0
            },
            chr_ram_size: if chr_rom_units == 0 { 8 * 1024 } else { 0 },
            chr_nvram_size: 0,
            vs_unisystem: flags7.contains(Flags7::VS_UNISYSTEM),
            playchoice_10: flags7.contains(Flags7::PLAYCHOICE_10),
            tv_system,
        })
    }

    fn parse_nes20(bytes: &[u8], flags6: Flags6, flags7: Flags7) -> Result<Self, Error> {
        let prg_msb = bytes[9] & 0x0F;
        let chr_msb = (bytes[9] >> 4) & 0x0F;
        let prg_rom_size = decode_nes2_rom_size(bytes[4], prg_msb, 16 * 1024);
        let chr_rom_size = decode_nes2_rom_size(bytes[5], chr_msb, 8 * 1024);

        let mapper = combine_mapper(flags6, flags7, bytes[8] & 0x0F);
        let submapper = bytes[8] >> 4;

        let prg_ram_size = decode_nes2_ram_size(bytes[10] & 0x0F);
        let prg_nvram_size = decode_nes2_ram_size(bytes[10] >> 4);
        let chr_ram_size = decode_nes2_ram_size(bytes[11] & 0x0F);
        let chr_nvram_size = decode_nes2_ram_size(bytes[11] >> 4);

        let console_type = flags7.bits() & 0b11;
        let tv_system: TvSystem = match bytes[12] & 0b11 {
            0b00 => TvSystem::Ntsc,
            0b01 => TvSystem::Pal,
            0b10 => TvSystem::Dual,
            0b11 => TvSystem::Dendy,
            _ => TvSystem::Unknown,
        };

        Ok(Self {
            format: RomFormat::Nes20,
            mapper,
            submapper,
            mirroring: resolve_mirroring(flags6),
            battery_backed_ram: prg_nvram_size != 0
                || chr_nvram_size != 0
                || flags6.contains(Flags6::BATTERY),
            trainer_present: flags6.contains(Flags6::TRAINER),
            prg_rom_size,
            chr_rom_size,
            prg_ram_size,
            prg_nvram_size,
            chr_ram_size,
            chr_nvram_size,
            vs_unisystem: console_type == 1 || flags7.contains(Flags7::VS_UNISYSTEM),
            playchoice_10: console_type == 2 || flags7.contains(Flags7::PLAYCHOICE_10),
            tv_system,
        })
    }
}

fn resolve_mirroring(flags6: Flags6) -> Mirroring {
    if flags6.contains(Flags6::FOUR_SCREEN) {
        Mirroring::FourScreen
    } else if flags6.contains(Flags6::MIRRORING) {
        Mirroring::Vertical
    } else {
        Mirroring::Horizontal
    }
}

fn combine_mapper(flags6: Flags6, flags7: Flags7, upper: u8) -> u16 {
    let lower = (flags6.bits() >> 4) as u16;
    let middle = (flags7.bits() & 0xF0) as u16;
    let upper = (upper as u16) << 8;
    lower | middle | upper
}

fn decode_nes2_rom_size(lower: u8, upper_nibble: u8, unit: usize) -> usize {
    if upper_nibble != 0x0F {
        (((upper_nibble as usize) << 8) | lower as usize).saturating_mul(unit)
    } else {
        let exponent = ((lower & 0x3F) as u32).saturating_add(8);
        let base = 1usize.checked_shl(exponent).unwrap_or(usize::MAX);
        let multiplier = ((lower >> 6) as usize) + 1;
        base.saturating_mul(multiplier)
    }
}

fn decode_nes2_ram_size(nibble: u8) -> usize {
    if nibble == 0 {
        0
    } else {
        64usize << nibble.min(0x0F)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_header() {
        let header_bytes = [
            b'N',
            b'E',
            b'S',
            0x1A,        // magic
            2,           // 2 * 16 KiB PRG ROM
            1,           // 1 * 8 KiB CHR ROM
            0b0000_0001, // vertical mirroring
            0b0000_0000, // mapper 0
            0,           // prg ram
            0,           // tv system NTSC
            0,
            0,
            0,
            0,
            0,
            0, // padding
        ];

        let header = Header::parse(&header_bytes).expect("header parses");

        assert!(matches!(header.format, RomFormat::INes));
        assert_eq!(header.prg_rom_size, 2 * 16 * 1024);
        assert_eq!(header.chr_rom_size, 1 * 8 * 1024);
        assert_eq!(header.mirroring, Mirroring::Vertical);
        assert!(!header.trainer_present);
        assert_eq!(header.mapper, 0);
        assert!(matches!(header.tv_system, TvSystem::Ntsc));
        assert_eq!(header.prg_ram_size, 8 * 1024);
        assert_eq!(header.prg_nvram_size, 0);
        assert_eq!(header.submapper, 0);
    }

    #[test]
    fn rejects_invalid_magic() {
        let mut header_bytes = [0u8; NES_HEADER_LEN];
        header_bytes[..4].copy_from_slice(b"NOPE");

        let err = Header::parse(&header_bytes).unwrap_err();
        assert!(matches!(err, Error::InvalidMagic));
    }

    #[test]
    fn parses_nes2_header() {
        let header_bytes = [
            b'N',
            b'E',
            b'S',
            0x1A,        // magic
            2,           // PRG LSB (2 * 16 KiB = 32 KiB)
            1,           // CHR LSB (1 * 8 KiB = 8 KiB)
            0b0000_0010, // horizontal mirroring
            0b0000_1000, // NES 2.0 format bits
            0b0011_0000, // mapper upper nibble = 0, submapper = 3
            0b0001_0000, // PRG MSB = 0x0, CHR MSB = 0x1 (adds 256 × 8 KiB)
            0b0010_0010, // PRG RAM = 256 B, PRG NVRAM = 256 B
            0b0100_0011, // CHR RAM = 512 B, CHR NVRAM = 1 KiB
            0b0000_0010, // timing: dual region
            0,
            0,
            0, // remaining padding
        ];

        let header = Header::parse(&header_bytes).expect("header parses");

        assert!(matches!(header.format, RomFormat::Nes20));
        assert_eq!(header.mapper, 0);
        assert_eq!(header.submapper, 3);
        assert_eq!(header.prg_rom_size, 2 * 16 * 1024);
        assert_eq!(header.chr_rom_size, (1 + (1 << 8)) * 8 * 1024);
        assert_eq!(header.prg_ram_size, 256);
        assert_eq!(header.prg_nvram_size, 256);
        assert_eq!(header.chr_ram_size, 512);
        assert_eq!(header.chr_nvram_size, 1024);
        assert_eq!(header.mirroring, Mirroring::Horizontal);
        assert!(matches!(header.tv_system, TvSystem::Dual));
    }

    #[test]
    fn parses_nes2_exponent_encoded_rom_size() {
        let header_bytes = [
            b'N',
            b'E',
            b'S',
            0x1A,
            0b0000_0001, // exponent = 1, multiplier = 1 (see formula)
            0,
            0,
            0b0000_1000, // NES 2.0
            0,
            0b0000_1111, // PRG MSB = 0xF triggers exponent encoding
            0,
            0,
            0,
            0,
            0,
            0,
        ];

        let header = Header::parse(&header_bytes).expect("header parses");

        assert!(matches!(header.format, RomFormat::Nes20));
        assert_eq!(header.prg_rom_size, 512);
        assert_eq!(header.chr_rom_size, 0);
    }
}
