//! Cartridge loading primitives.
//!
//! The first 16 bytes of every `.nes` ROM are the *iNES header*. It stores how much
//! PRG/CHR data the cartridge exposes, which mapper is required, and a few
//! compatibility flags. Modern dumps may use the extended **NES 2.0** flavour of the
//! header, so the parser in this module understands both variants and presents the
//! data as a beginner friendly [`Header`] enum.
//!
//! # Quick overview
//! - Read the first 16 bytes and pass them to [`Header::parse`].
//! - Inspect [`Header::mapper`] to pick or construct a concrete [`crate::cartridge::Mapper`]
//!   implementation and wrap it in a [`crate::cartridge::Cartridge`].
//! - Use [`Header::prg_rom_size`] / [`Header::chr_rom_size`] to slice the raw PRG/CHR
//!   sections out of the file.
//!
//! Unsupported or damaged headers turn into a descriptive [`Error`].
//! Submodules live in `cartridge/header/`.

mod console_type;
mod extended_console_type;
mod flags6;
mod flags7;
mod ines10_extension;
mod ines10_header;
mod ines_header;
mod mirroring;
mod nes2_console_type_data;
mod nes2_cpu_ppu_timing;
mod nes2_default_expansion_device;
mod nes2_expansion_device;
mod nes2_extension;
mod nes2_header;
mod nes2_misc_rom_count;
mod rom_format;
mod tv_system;
mod vs_hardware_type;
mod vs_ppu_type;

pub use console_type::ConsoleType;
pub use extended_console_type::ExtendedConsoleType;
pub use flags6::Flags6;
pub use flags7::Flags7;
pub use ines_header::INesHeader;
pub use ines10_extension::INes10Extension;
pub use ines10_header::INes10Header;
pub use mirroring::Mirroring;
pub use nes2_console_type_data::Nes2ConsoleTypeData;
pub use nes2_cpu_ppu_timing::Nes2CpuPpuTiming;
pub use nes2_default_expansion_device::Nes2DefaultExpansionDevice;
pub use nes2_expansion_device::Nes2ExpansionDevice;
pub use nes2_extension::Nes2Extension;
pub use nes2_header::Nes2Header;
pub use nes2_misc_rom_count::Nes2MiscRomCount;
pub use rom_format::RomFormat;
pub use tv_system::TvSystem;
pub use vs_hardware_type::VsHardwareType;
pub use vs_ppu_type::VsPpuType;

use crate::error::Error;

const NES_MAGIC: &[u8; 4] = b"NES\x1A";

/// Size of the fixed iNES header in bytes.
pub const NES_HEADER_LEN: usize = 16;

/// Parsed cartridge header, naturally distinguishing iNES 1.0 from NES 2.0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Header {
    INes(INes10Header),
    Nes20(Nes2Header),
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

        let base = INesHeader::from_bytes(bytes);
        let format = RomFormat::from_flags7(base.flags7);
        match format {
            RomFormat::INes => Ok(Self::INes(INes10Header {
                base,
                ext: INes10Extension {
                    prg_ram_units: bytes[8],
                    flags9: bytes[9],
                    flags10: bytes[10],
                    padding: bytes[11..16]
                        .try_into()
                        .expect("iNES padding length mismatch"),
                },
            })),
            RomFormat::Nes20 => Ok(Self::Nes20(Nes2Header {
                base,
                ext: Nes2Extension {
                    mapper_msb_submapper: bytes[8],
                    prg_chr_msb: bytes[9],
                    prg_ram_shifts: bytes[10],
                    chr_ram_shifts: bytes[11],
                    timing: bytes[12],
                    console_type_data: bytes[13],
                    misc_roms: bytes[14],
                    default_expansion_device: bytes[15],
                },
            })),
            RomFormat::Archaic => Err(Error::UnsupportedFormat(format)),
        }
    }

    /// Detected header flavour.
    pub fn format(&self) -> RomFormat {
        match self {
            Header::INes(_) => RomFormat::INes,
            Header::Nes20(_) => RomFormat::Nes20,
        }
    }

    /// Shared iNES-defined fields (bytes 4..=7).
    pub fn base(&self) -> &INesHeader {
        match self {
            Header::INes(header) => &header.base,
            Header::Nes20(header) => &header.base,
        }
    }

    /// Raw iNES flags 6.
    pub fn flags6(&self) -> Flags6 {
        self.base().flags6
    }

    /// Raw iNES flags 7.
    pub fn flags7(&self) -> Flags7 {
        self.base().flags7
    }

    /// Console type as advertised by flags 7 bits 0..=1.
    pub fn console_type(&self) -> ConsoleType {
        ConsoleType::from_bits(self.base().flags7.bits() & 0b11)
    }

    /// NES 2.0: console-type dependent byte 13 information.
    pub fn nes2_console_type_data(&self) -> Option<Nes2ConsoleTypeData> {
        match self {
            Header::Nes20(header) => Some(header.ext.console_type_data(self.console_type())),
            _ => None,
        }
    }

    /// NES 2.0: number of miscellaneous ROM regions (0..=3).
    pub fn nes2_misc_rom_count(&self) -> Option<Nes2MiscRomCount> {
        match self {
            Header::Nes20(header) => Some(header.ext.misc_rom_count()),
            _ => None,
        }
    }

    /// NES 2.0: default expansion device id.
    pub fn nes2_default_expansion_device(&self) -> Option<Nes2ExpansionDevice> {
        match self {
            Header::Nes20(header) => Some(header.ext.default_expansion_device()),
            _ => None,
        }
    }

    /// NES 2.0: interpreted default expansion device.
    pub fn nes2_default_expansion_device_kind(&self) -> Option<Nes2DefaultExpansionDevice> {
        self.nes2_default_expansion_device()
            .map(Nes2ExpansionDevice::kind)
    }

    /// NES 2.0 CPU/PPU timing mode (byte 12 bits 0..=1).
    pub fn nes2_cpu_ppu_timing(&self) -> Option<Nes2CpuPpuTiming> {
        match self {
            Header::Nes20(header) => Some(Nes2CpuPpuTiming::from_bits(header.ext.timing)),
            _ => None,
        }
    }

    /// iNES 1.0: exposes the raw extension bytes 8..=15 (for diagnostics).
    pub fn ines_extension(&self) -> Option<INes10Extension> {
        match self {
            Header::INes(header) => Some(header.ext),
            _ => None,
        }
    }

    /// Mapper ID (0 == NROM, 1 == MMC1, ...).
    pub fn mapper(&self) -> u16 {
        match self {
            Header::INes(header) => combine_mapper(header.base.flags6, header.base.flags7, 0),
            Header::Nes20(header) => combine_mapper(
                header.base.flags6,
                header.base.flags7,
                header.ext.mapper_msb(),
            ),
        }
    }

    /// NES 2.0 submapper value. Always 0 for legacy iNES files.
    pub fn submapper(&self) -> u8 {
        match self {
            Header::INes(_) => 0,
            Header::Nes20(header) => header.ext.submapper(),
        }
    }

    /// How the PPU nametables are mirrored.
    pub fn mirroring(&self) -> Mirroring {
        self.base().mirroring()
    }

    /// Battery bit indicates the cartridge keeps RAM contents when powered off.
    pub fn battery_backed_ram(&self) -> bool {
        match self {
            Header::INes(header) => header.base.flags6.contains(Flags6::BATTERY),
            Header::Nes20(header) => {
                self.prg_nvram_size() != 0
                    || self.chr_nvram_size() != 0
                    || header.base.flags6.contains(Flags6::BATTERY)
            }
        }
    }

    /// Whether the optional 512 byte trainer block is present between the header and PRG data.
    pub fn trainer_present(&self) -> bool {
        self.base().trainer_present()
    }

    /// Amount of PRG ROM in bytes.
    pub fn prg_rom_size(&self) -> usize {
        match self {
            Header::INes(header) => (header.base.prg_rom_lsb as usize) * 16 * 1024,
            Header::Nes20(header) => {
                decode_nes2_rom_size(header.base.prg_rom_lsb, header.ext.prg_rom_msb(), 16 * 1024)
            }
        }
    }

    /// Amount of CHR ROM in bytes.
    pub fn chr_rom_size(&self) -> usize {
        match self {
            Header::INes(header) => (header.base.chr_rom_lsb as usize) * 8 * 1024,
            Header::Nes20(header) => {
                decode_nes2_rom_size(header.base.chr_rom_lsb, header.ext.chr_rom_msb(), 8 * 1024)
            }
        }
    }

    /// Volatile PRG RAM size (CPU accessible). Defaults to 8 KiB for legacy dumps that store 0.
    pub fn prg_ram_size(&self) -> usize {
        match self {
            Header::INes(header) => (header.ext.prg_ram_units.max(1) as usize) * 8 * 1024,
            Header::Nes20(header) => decode_nes2_ram_size(header.ext.prg_ram_shift()),
        }
    }

    /// Battery-backed PRG RAM size.
    pub fn prg_nvram_size(&self) -> usize {
        match self {
            Header::INes(header) => {
                if header.base.flags6.contains(Flags6::BATTERY) {
                    (header.ext.prg_ram_units.max(1) as usize) * 8 * 1024
                } else {
                    0
                }
            }
            Header::Nes20(header) => decode_nes2_ram_size(header.ext.prg_nvram_shift()),
        }
    }

    /// Volatile CHR RAM size located on the PPU side.
    pub fn chr_ram_size(&self) -> usize {
        match self {
            Header::INes(header) => {
                if header.base.chr_rom_lsb == 0 {
                    8 * 1024
                } else {
                    0
                }
            }
            Header::Nes20(header) => decode_nes2_ram_size(header.ext.chr_ram_shift()),
        }
    }

    /// Battery-backed CHR RAM size.
    pub fn chr_nvram_size(&self) -> usize {
        match self {
            Header::INes(_) => 0,
            Header::Nes20(header) => decode_nes2_ram_size(header.ext.chr_nvram_shift()),
        }
    }

    /// Set when the game targets the Vs. UniSystem arcade hardware.
    pub fn vs_unisystem(&self) -> bool {
        match self {
            Header::INes(header) => header.base.flags7.contains(Flags7::VS_UNISYSTEM),
            Header::Nes20(header) => {
                let console_type = header.base.flags7.bits() & 0b11;
                console_type == 1 || header.base.flags7.contains(Flags7::VS_UNISYSTEM)
            }
        }
    }

    /// Set when the cartridge contains PlayChoice-10 data.
    pub fn playchoice_10(&self) -> bool {
        match self {
            Header::INes(header) => header.base.flags7.contains(Flags7::PLAYCHOICE_10),
            Header::Nes20(header) => {
                let console_type = header.base.flags7.bits() & 0b11;
                console_type == 2 || header.base.flags7.contains(Flags7::PLAYCHOICE_10)
            }
        }
    }

    /// Region / timing hints described in the header.
    pub fn tv_system(&self) -> TvSystem {
        match self {
            Header::INes(header) => {
                let tv_bits = header.ext.flags10 & 0b11;
                match tv_bits {
                    0b00 => {
                        if header.ext.flags9 & 0b1 == 0 {
                            TvSystem::Ntsc
                        } else {
                            TvSystem::Pal
                        }
                    }
                    0b10 => TvSystem::Pal,
                    0b01 | 0b11 => TvSystem::Dual,
                    _ => TvSystem::Unknown,
                }
            }
            Header::Nes20(header) => match header.ext.timing & 0b11 {
                0b00 => TvSystem::Ntsc,
                0b01 => TvSystem::Pal,
                0b10 => TvSystem::Dual,
                0b11 => TvSystem::Dendy,
                _ => TvSystem::Unknown,
            },
        }
    }

    /// iNES 1.0 flags 10: hint whether the board has bus conflicts.
    pub fn ines_bus_conflicts(&self) -> Option<bool> {
        match self {
            Header::INes(header) => Some((header.ext.flags10 & 0x80) != 0),
            _ => None,
        }
    }

    /// iNES 1.0 flags 10: hint whether PRG RAM is present ($6000-$7FFF).
    ///
    /// Note: this is not part of the official iNES specification and is rarely used.
    pub fn ines_prg_ram_present_hint(&self) -> Option<bool> {
        match self {
            Header::INes(header) => Some((header.ext.flags10 & 0x10) == 0),
            _ => None,
        }
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

        assert!(matches!(header.format(), RomFormat::INes));
        assert_eq!(header.prg_rom_size(), 2 * 16 * 1024);
        assert_eq!(header.chr_rom_size(), 8 * 1024);
        assert_eq!(header.mirroring(), Mirroring::Vertical);
        assert!(!header.trainer_present());
        assert_eq!(header.mapper(), 0);
        assert!(matches!(header.tv_system(), TvSystem::Ntsc));
        assert_eq!(header.prg_ram_size(), 8 * 1024);
        assert_eq!(header.prg_nvram_size(), 0);
        assert_eq!(header.submapper(), 0);
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

        assert!(matches!(header.format(), RomFormat::Nes20));
        assert_eq!(header.mapper(), 0);
        assert_eq!(header.submapper(), 3);
        assert_eq!(header.prg_rom_size(), 2 * 16 * 1024);
        assert_eq!(header.chr_rom_size(), (1 + (1 << 8)) * 8 * 1024);
        assert_eq!(header.prg_ram_size(), 256);
        assert_eq!(header.prg_nvram_size(), 256);
        assert_eq!(header.chr_ram_size(), 512);
        assert_eq!(header.chr_nvram_size(), 1024);
        assert_eq!(header.mirroring(), Mirroring::Horizontal);
        assert!(matches!(header.tv_system(), TvSystem::Dual));
        assert_eq!(
            header.nes2_cpu_ppu_timing(),
            Some(Nes2CpuPpuTiming::MultipleRegion)
        );
    }

    #[test]
    fn parses_nes2_console_type_and_misc_fields() {
        let header_bytes = [
            b'N',
            b'E',
            b'S',
            0x1A,        // magic
            1,           // PRG LSB
            0,           // CHR LSB (CHR RAM)
            0,           // flags6
            0b0000_1001, // NES 2.0 + console type = Vs System
            0b0001_0000, // submapper 1
            0,           // PRG/CHR msb
            0,
            0,
            0,           // timing
            0xA3,        // hw type 0xA (unknown), ppu type 0x3 (RP2C04-0002)
            0b0000_0010, // misc ROM count = 2
            0x2A,        // expansion device id = 0x2A (Multicart)
        ];

        let header = Header::parse(&header_bytes).expect("header parses");

        assert_eq!(header.console_type(), ConsoleType::VsSystem);
        assert!(matches!(
            header.nes2_console_type_data(),
            Some(Nes2ConsoleTypeData::VsSystem {
                hardware_type: VsHardwareType::Unknown(0xA),
                ppu_type: VsPpuType::Rp2c04_0002
            })
        ));

        assert_eq!(header.nes2_misc_rom_count(), Some(Nes2MiscRomCount(2)));
        assert_eq!(
            header.nes2_default_expansion_device(),
            Some(Nes2ExpansionDevice(0x2A))
        );
        assert_eq!(
            header.nes2_default_expansion_device_kind(),
            Some(Nes2DefaultExpansionDevice::Multicart)
        );
        assert_eq!(header.nes2_cpu_ppu_timing(), Some(Nes2CpuPpuTiming::Rp2c02));
    }

    #[test]
    fn preserves_ines_padding_bytes() {
        let header_bytes = [
            b'N', b'E', b'S', 0x1A, // magic
            1, 0, 0, 0, // sizes + flags6/7
            0, 0, 0, // bytes 8..=10
            1, 2, 3, 4, 5, // bytes 11..=15 padding
        ];

        let header = Header::parse(&header_bytes).expect("header parses");
        let ext = header.ines_extension().expect("ines header");
        assert_eq!(ext.padding, [1, 2, 3, 4, 5]);
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

        assert!(matches!(header.format(), RomFormat::Nes20));
        assert_eq!(header.prg_rom_size(), 512);
        assert_eq!(header.chr_rom_size(), 0);
    }
}
