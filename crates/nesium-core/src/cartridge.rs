use std::{fs, path::Path};

use crate::{
    cartridge::header::{Header, Mirroring, NES_HEADER_LEN},
    error::Error,
};

use self::mapper::{
    Mapper0, Mapper1, Mapper2, Mapper3, Mapper4, Mapper5, Mapper6, Mapper7, Mapper8, Mapper9,
    Mapper10, Mapper11, Mapper13, Mapper19, Mapper21, Mapper23, Mapper25, Mapper26, Mapper34,
    NametableTarget,
};

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

    pub fn mirroring(&self) -> Mirroring {
        self.mapper.mirroring()
    }

    pub fn cpu_read(&self, addr: u16) -> Option<u8> {
        self.mapper.cpu_read(addr)
    }

    pub fn cpu_write(&mut self, addr: u16, data: u8, cpu_cycle: u64) {
        self.mapper.cpu_write(addr, data, cpu_cycle);
    }

    pub fn ppu_read(&self, addr: u16) -> Option<u8> {
        self.mapper.ppu_read(addr)
    }

    pub fn ppu_write(&mut self, addr: u16, data: u8) {
        self.mapper.ppu_write(addr, data);
    }

    /// Notify the mapper about a PPU VRAM access, including CPU bus timing.
    pub fn ppu_vram_access(
        &mut self,
        addr: u16,
        ctx: crate::cartridge::mapper::PpuVramAccessContext,
    ) {
        self.mapper.ppu_vram_access(addr, ctx);
    }

    /// Advance mapper-internal CPU-based timers and expansion audio by one bus cycle.
    pub fn cpu_clock(&mut self, cpu_cycle: u64) {
        self.mapper.cpu_clock(cpu_cycle);
        if let Some(expansion) = self.mapper.as_expansion_audio_mut() {
            expansion.clock_audio();
        }
    }

    /// Resolve a PPU nametable address to its backing storage.
    pub fn map_nametable(&self, addr: u16) -> NametableTarget {
        self.mapper.map_nametable(addr)
    }

    /// Mapper-controlled nametable read when [`map_nametable`] selects mapper VRAM/ROM.
    pub fn mapper_nametable_read(&self, offset: u16) -> u8 {
        self.mapper.mapper_nametable_read(offset)
    }

    /// Mapper-controlled nametable write when [`map_nametable`] selects mapper VRAM/ROM.
    pub fn mapper_nametable_write(&mut self, offset: u16, value: u8) {
        self.mapper.mapper_nametable_write(offset, value);
    }

    pub fn irq_pending(&self) -> bool {
        self.mapper.irq_pending()
    }

    pub fn clear_irq(&mut self) {
        self.mapper.clear_irq();
    }

    /// Applies a power-on reset sequence to the mapper.
    pub fn power_on(&mut self) {
        self.mapper.power_on();
    }

    /// Applies a console reset to the mapper.
    pub fn reset(&mut self) {
        self.mapper.reset();
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

    let mut mapper: Box<dyn Mapper> = match header.mapper {
        0 => Box::new(Mapper0::with_trainer(header, prg_rom, chr_rom, trainer)),
        1 => Box::new(Mapper1::with_trainer(header, prg_rom, chr_rom, trainer)),
        2 => Box::new(Mapper2::with_trainer(header, prg_rom, chr_rom, trainer)),
        3 => Box::new(Mapper3::with_trainer(header, prg_rom, chr_rom, trainer)),
        4 => Box::new(Mapper4::with_trainer(header, prg_rom, chr_rom, trainer)),
        5 => Box::new(Mapper5::with_trainer(header, prg_rom, chr_rom, trainer)),
        6 => Box::new(Mapper6::with_trainer(header, prg_rom, chr_rom, trainer)),
        7 => Box::new(Mapper7::with_trainer(header, prg_rom, chr_rom, trainer)),
        8 => Box::new(Mapper8::with_trainer(header, prg_rom, chr_rom, trainer)),
        9 => Box::new(Mapper9::with_trainer(header, prg_rom, chr_rom, trainer)),
        10 => Box::new(Mapper10::with_trainer(header, prg_rom, chr_rom, trainer)),
        11 => Box::new(Mapper11::with_trainer(header, prg_rom, chr_rom, trainer)),
        13 => Box::new(Mapper13::with_trainer(header, prg_rom, chr_rom, trainer)),
        19 => Box::new(Mapper19::with_trainer(header, prg_rom, chr_rom, trainer)),
        21 => Box::new(Mapper21::with_trainer(header, prg_rom, chr_rom, trainer)),
        23 => Box::new(Mapper23::with_trainer(header, prg_rom, chr_rom, trainer)),
        25 => Box::new(Mapper25::with_trainer(header, prg_rom, chr_rom, trainer)),
        26 => Box::new(Mapper26::with_trainer(header, prg_rom, chr_rom, trainer)),
        34 => Box::new(Mapper34::with_trainer(header, prg_rom, chr_rom, trainer)),
        other => return Err(Error::UnsupportedMapper(other)),
    };

    // Apply mapper-specific power-on defaults once after construction.
    mapper.power_on();

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
        assert_eq!(cartridge.cpu_read(cpu_mem::PRG_ROM_START), Some(0xAA));
        assert_eq!(cartridge.ppu_read(0x0000), Some(0x55));
    }

    #[test]
    fn loads_cartridge_with_trainer() {
        let mut rom = base_header(1, 0, 0b0000_0100).to_vec();
        rom.extend(vec![0xFE; TRAINER_SIZE]);
        rom.extend(vec![0xAA; 16 * 1024]);

        let cartridge = load_cartridge(&rom).expect("parse cartridge");

        assert!(cartridge.header().trainer_present);
        assert_eq!(cartridge.header().prg_rom_size, 16 * 1024);
        assert_eq!(cartridge.cpu_read(cpu_mem::PRG_ROM_START), Some(0xAA));
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

    #[test]
    fn errors_when_mapper_not_implemented() {
        // Choose a mapper number that is currently not implemented by this core.
        // With flags7/upper bytes zeroed, the high nibble of flags6 becomes
        // the mapper number. 0xC0 >> 4 = 12.
        let mut rom = base_header(1, 1, 0xC0).to_vec();
        rom.extend(vec![0xAA; 16 * 1024]); // PRG
        rom.extend(vec![0x55; 8 * 1024]); // CHR

        let err = load_cartridge(&rom).expect_err("unsupported mapper should fail");
        assert!(matches!(err, Error::UnsupportedMapper(12)));
    }
}
