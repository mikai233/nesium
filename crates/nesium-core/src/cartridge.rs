use std::{fmt::Debug, fs, path::Path};

use dyn_clone::DynClone;

use crate::{
    cartridge::header::{Header, NES_HEADER_LEN},
    error::Error,
};

use self::{axrom::Axrom, cnrom::Cnrom, nrom::Nrom, uxrom::Uxrom};

pub const TRAINER_SIZE: usize = 512;

pub mod axrom;
pub mod cnrom;
pub mod header;
pub mod nrom;
pub mod uxrom;

pub trait Cartridge: Debug + DynClone {
    fn header(&self) -> &Header;

    fn cpu_read(&self, addr: u16) -> u8;

    fn cpu_write(&mut self, addr: u16, data: u8);

    fn ppu_read(&self, addr: u16) -> u8;

    fn ppu_write(&mut self, addr: u16, data: u8);

    /// Returns `true` when the mapper asserts the CPU IRQ line.
    fn irq_pending(&self) -> bool {
        false
    }

    /// Clears any IRQ sources latched by the mapper.
    fn clear_irq(&mut self) {}
}

dyn_clone::clone_trait_object!(Cartridge);

/// Load a cartridge from an in-memory byte slice.
pub fn load_cartridge(bytes: &[u8]) -> Result<Box<dyn Cartridge>, Error> {
    let header_bytes = bytes.get(..NES_HEADER_LEN).ok_or(Error::TooShort {
        actual: bytes.len(),
    })?;
    let header = Header::parse(header_bytes)?;
    let (trainer, prg_rom, chr_rom) = slice_sections(bytes, &header)?;

    let cartridge: Box<dyn Cartridge> = match header.mapper {
        0 => Box::new(Nrom::with_trainer(header, prg_rom, chr_rom, trainer)),
        2 => Box::new(Uxrom::with_trainer(header, prg_rom, chr_rom, trainer)),
        3 => Box::new(Cnrom::with_trainer(header, prg_rom, chr_rom, trainer)),
        7 => Box::new(Axrom::with_trainer(header, prg_rom, chr_rom, trainer)),
        _ => unimplemented!("Mapper {} not implemented", header.mapper),
    };

    Ok(cartridge)
}

/// Load a cartridge directly from disk.
pub fn load_cartridge_from_file<P>(path: P) -> Result<Box<dyn Cartridge>, Error>
where
    P: AsRef<Path>,
{
    let bytes = fs::read(path)?;
    load_cartridge(&bytes)
}

fn slice_sections(
    bytes: &[u8],
    header: &Header,
) -> Result<(Option<Box<[u8; TRAINER_SIZE]>>, Box<[u8]>, Box<[u8]>), Error> {
    let mut cursor = NES_HEADER_LEN;
    let trainer = if header.trainer_present {
        let trainer_slice = section(bytes, &mut cursor, TRAINER_SIZE, "trainer")?;
        Some(
            trainer_slice
                .into_boxed_slice()
                .try_into()
                .expect("trainer length mismatch"),
        )
    } else {
        None
    };

    let prg_rom = section(bytes, &mut cursor, header.prg_rom_size, "PRG ROM")?;
    let chr_rom = section(bytes, &mut cursor, header.chr_rom_size, "CHR ROM")?;

    Ok((
        trainer,
        prg_rom.into_boxed_slice(),
        chr_rom.into_boxed_slice(),
    ))
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
    use crate::memory::cpu as cpu_mem;

    fn base_header(prg_banks: u8, chr_banks: u8, flags6: u8) -> [u8; NES_HEADER_LEN] {
        [
            b'N', b'E', b'S', 0x1A, prg_banks, chr_banks, flags6, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]
    }

    #[test]
    fn loads_basic_nrom_cartridge() {
        let mut rom = base_header(1, 1, 0).to_vec();
        rom.extend(vec![0xAA; 16 * 1024]);
        rom.extend(vec![0x55; 8 * 1024]);

        let cartridge = load_cartridge(&rom).expect("parse cartridge");

        assert_eq!(cartridge.header().prg_rom_size, 16 * 1024);
        assert_eq!(cartridge.header().chr_rom_size, 8 * 1024);
        assert_eq!(cartridge.cpu_read(cpu_mem::PRG_ROM_START), 0xAA);
        assert_eq!(cartridge.ppu_read(0x0000), 0x55);
    }

    #[test]
    fn loads_cartridge_with_trainer() {
        let mut rom = base_header(1, 0, 0b0000_0100).to_vec();
        rom.extend(vec![0xFE; TRAINER_SIZE]);
        rom.extend(vec![0xAA; 16 * 1024]);

        let cartridge = load_cartridge(&rom).expect("parse cartridge");

        assert!(cartridge.header().trainer_present);
        assert_eq!(cartridge.header().prg_rom_size, 16 * 1024);
        assert_eq!(cartridge.cpu_read(cpu_mem::PRG_ROM_START), 0xAA);
    }

    #[test]
    fn errors_when_prg_section_missing() {
        let mut rom = base_header(1, 0, 0).to_vec();
        rom.extend(vec![0xAA; 1024]); // insufficient PRG data

        let err = load_cartridge(&rom).expect_err("should fail");
        assert!(matches!(
            err,
            Error::SectionTooShort {
                section: "PRG ROM",
                ..
            }
        ));
    }
}
