use std::{fs, path::Path};

use crate::{
    cartridge::header::{Header, NES_HEADER_LEN},
    error::Error,
};

pub mod header;
pub mod mapper;

/// Parsed NES cartridge, including header metadata and raw ROM sections.
#[derive(Debug, Clone)]
pub struct Cartridge {
    pub header: Header,
    pub trainer: Option<[u8; 512]>,
    pub prg_rom: Vec<u8>,
    pub chr_rom: Vec<u8>,
}

impl Cartridge {
    /// Parse a cartridge from an in-memory byte slice.
    pub fn new(bytes: &[u8]) -> Result<Self, Error> {
        let header_bytes = bytes.get(..NES_HEADER_LEN).ok_or(Error::TooShort {
            actual: bytes.len(),
        })?;
        let header = Header::parse(header_bytes)?;

        let mut cursor = NES_HEADER_LEN;
        let trainer = if header.trainer_present {
            let trainer_slice = section(bytes, &mut cursor, 512, "trainer")?;
            let mut trainer = [0u8; 512];
            trainer.copy_from_slice(&trainer_slice);
            Some(trainer)
        } else {
            None
        };

        let prg_rom = section(bytes, &mut cursor, header.prg_rom_size, "PRG ROM")?;
        let chr_rom = section(bytes, &mut cursor, header.chr_rom_size, "CHR ROM")?;

        Ok(Self {
            header,
            trainer,
            prg_rom,
            chr_rom,
        })
    }

    /// Load and parse a cartridge directly from disk.
    pub fn from_file<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let bytes = fs::read(path)?;
        Self::new(&bytes)
    }
}

fn section(
    bytes: &[u8],
    cursor: &mut usize,
    len: usize,
    name: &'static str,
) -> Result<Vec<u8>, Error> {
    if len == 0 {
        return Ok(Vec::new());
    }

    let end = cursor.checked_add(len).ok_or(Error::SectionTooShort {
        section: name,
        expected: len,
        actual: bytes.len().saturating_sub(*cursor),
    })?;

    let slice = bytes.get(*cursor..end).ok_or(Error::SectionTooShort {
        section: name,
        expected: len,
        actual: bytes.len().saturating_sub(*cursor),
    })?;

    *cursor = end;
    Ok(slice.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_header(prg_banks: u8, chr_banks: u8, flags6: u8) -> [u8; NES_HEADER_LEN] {
        [
            b'N', b'E', b'S', 0x1A, prg_banks, chr_banks, flags6, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]
    }

    #[test]
    fn parses_basic_cartridge() {
        let mut rom = base_header(1, 1, 0).to_vec();
        rom.extend(vec![0xAA; 16 * 1024]);
        rom.extend(vec![0x55; 8 * 1024]);

        let cartridge = Cartridge::new(&rom).expect("parse cartridge");

        assert_eq!(cartridge.prg_rom.len(), 16 * 1024);
        assert_eq!(cartridge.chr_rom.len(), 8 * 1024);
        assert!(cartridge.trainer.is_none());
    }

    #[test]
    fn parses_trainer_when_present() {
        let mut rom = base_header(1, 0, 0b0000_0100).to_vec();
        rom.extend(vec![0xFE; 512]);
        rom.extend(vec![0xAA; 16 * 1024]);

        let cartridge = Cartridge::new(&rom).expect("parse cartridge");

        let trainer = cartridge.trainer.expect("trainer present");
        assert!(trainer.iter().all(|&byte| byte == 0xFE));
        assert_eq!(cartridge.prg_rom.len(), 16 * 1024);
        assert!(cartridge.chr_rom.is_empty());
    }

    #[test]
    fn errors_when_prg_section_missing() {
        let mut rom = base_header(1, 0, 0).to_vec();
        rom.extend(vec![0xAA; 1024]); // insufficient PRG data

        let err = Cartridge::new(&rom).expect_err("should fail");
        assert!(matches!(
            err,
            Error::SectionTooShort {
                section: "PRG ROM",
                ..
            }
        ));
    }
}
