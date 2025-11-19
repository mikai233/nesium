use std::{fs, path::Path};

use crate::{
    cartridge::header::{Header, NES_HEADER_LEN},
    error::Error,
};

use self::mapper::{Mapper0, Mapper2, Mapper3, Mapper7};

pub const TRAINER_SIZE: usize = 512;

pub mod header;
pub mod mapper;
pub use mapper::{Mapper, mapper_downcast_mut, mapper_downcast_ref};

#[derive(Debug)]
pub struct Cartridge {
    header: Header,
    mapper: Box<dyn Mapper>,
}

impl Cartridge {
    pub fn new(header: Header, mapper: Box<dyn Mapper>) -> Self {
        Self { header, mapper }
    }

    pub fn header(&self) -> &Header {
        &self.header
    }

    pub fn mapper(&self) -> &dyn Mapper {
        self.mapper.as_ref()
    }

    pub fn mapper_mut(&mut self) -> &mut dyn Mapper {
        self.mapper.as_mut()
    }

    pub fn cpu_read(&self, addr: u16) -> u8 {
        self.mapper.cpu_read(addr)
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8) {
        self.mapper.cpu_write(addr, data);
    }

    pub fn ppu_read(&self, addr: u16) -> u8 {
        self.mapper.ppu_read(addr)
    }

    pub fn ppu_write(&mut self, addr: u16, data: u8) {
        self.mapper.ppu_write(addr, data);
    }

    pub fn irq_pending(&self) -> bool {
        self.mapper.irq_pending()
    }

    pub fn clear_irq(&mut self) {
        self.mapper.clear_irq();
    }
}

impl Clone for Cartridge {
    fn clone(&self) -> Self {
        Self {
            header: self.header,
            mapper: dyn_clone::clone_box(self.mapper()),
        }
    }
}

/// Load a cartridge from an in-memory byte slice.
pub fn load_cartridge(bytes: &[u8]) -> Result<Cartridge, Error> {
    let header_bytes = bytes.get(..NES_HEADER_LEN).ok_or(Error::TooShort {
        actual: bytes.len(),
    })?;
    let header = Header::parse(header_bytes)?;
    let (trainer, prg_rom, chr_rom) = slice_sections(bytes, &header)?;

    let mapper: Box<dyn Mapper> = match header.mapper {
        0 => Box::new(Mapper0::with_trainer(header, prg_rom, chr_rom, trainer)),
        2 => Box::new(Mapper2::with_trainer(header, prg_rom, chr_rom, trainer)),
        3 => Box::new(Mapper3::with_trainer(header, prg_rom, chr_rom, trainer)),
        7 => Box::new(Mapper7::with_trainer(header, prg_rom, chr_rom, trainer)),
        _ => unimplemented!("Mapper {} not implemented", header.mapper),
    };

    Ok(Cartridge::new(header, mapper))
}

/// Load a cartridge directly from disk.
pub fn load_cartridge_from_file<P>(path: P) -> Result<Cartridge, Error>
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
