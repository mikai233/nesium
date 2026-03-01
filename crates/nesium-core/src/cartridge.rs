use std::{borrow::Cow, fs, path::Path};

use crate::{
    cartridge::header::{Header, Mirroring, NES_HEADER_LEN},
    error::Error,
    reset_kind::ResetKind,
};

use self::mapper::{
    Mapper0, Mapper1, Mapper2, Mapper3, Mapper4, Mapper5, Mapper6, Mapper7, Mapper8, Mapper9,
    Mapper10, Mapper11, Mapper13, Mapper19, Mapper21, Mapper23, Mapper25, Mapper26, Mapper34,
    Mapper66, Mapper71, Mapper78, Mapper85, Mapper90, Mapper119, Mapper228, NametableTarget,
};

pub const TRAINER_SIZE: usize = 512;

/// Borrowed view of the optional 512-byte trainer section.
///
/// The trainer is always exactly [`TRAINER_SIZE`] bytes when present, and
/// is only needed during mapper construction to initialize any RAM that
/// should be preloaded with trainer data. By borrowing from the original
/// ROM image instead of allocating a boxed array, we avoid an extra copy.
pub type TrainerBytes<'a> = Option<&'a [u8; TRAINER_SIZE]>;

/// Backing type for PRG ROM data.
///
/// - On desktop/host platforms, this is typically `Cow::Owned(Vec<u8>)`.
/// - On embedded targets (e.g. ESP32), this can borrow from a static
///   `include_bytes!` blob via `Cow::Borrowed(&'static [u8])` so that
///   loading a cartridge does not require heap allocation for PRG ROM.
pub type PrgRom = Cow<'static, [u8]>;

/// Backing type for CHR ROM data (PPU pattern tables).
///
/// Follows the same ownership model as [`PrgRom`], allowing CHR data to be
/// borrowed from static blobs on platforms where heap usage is constrained.
pub type ChrRom = Cow<'static, [u8]>;

/// Source image used to construct a [`Cartridge`].
///
/// This allows a single loading API to support both heap-owned ROM
/// bytes (e.g. loaded from disk or network) and statically embedded
/// ROM blobs (e.g. via `include_bytes!` on embedded targets).
pub enum CartridgeImage {
    /// ROM bytes owned in heap memory.
    Owned(Vec<u8>),
    /// ROM bytes embedded in the binary or other static storage.
    Static(&'static [u8]),
}

impl From<Vec<u8>> for CartridgeImage {
    fn from(v: Vec<u8>) -> Self {
        CartridgeImage::Owned(v)
    }
}

impl<const N: usize> From<&'static [u8; N]> for CartridgeImage {
    fn from(bytes: &'static [u8; N]) -> Self {
        CartridgeImage::Static(&bytes[..])
    }
}

impl<'a> From<&'a [u8]> for CartridgeImage {
    fn from(bytes: &'a [u8]) -> Self {
        // Fall back to owning a copy when given a non-'static slice.
        CartridgeImage::Owned(bytes.to_vec())
    }
}

pub mod a12_watcher;
pub mod header;
pub mod mapper;
pub use mapper::{
    CpuBusAccessKind, Mapper, MapperEvent, MapperHookMask, MapperMemoryOperation,
    PpuRenderFetchInfo, PpuRenderFetchTarget, PpuRenderFetchType, PpuVramAccessContext,
    PpuVramAccessSource, Provider, mapper_downcast_mut, mapper_downcast_ref,
};

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

    /// Convenience CHR read (`$0000-$1FFF`) that always returns a byte.
    pub fn chr_read(&self, addr: u16) -> u8 {
        self.mapper.chr_read(addr)
    }

    /// Convenience CHR write (`$0000-$1FFF`) for CHR RAM mappers.
    pub fn chr_write(&mut self, addr: u16, data: u8) {
        self.mapper.chr_write(addr, data);
    }

    /// Notify the mapper about a PPU VRAM access, including CPU bus timing.
    pub fn ppu_vram_access(&mut self, addr: u16, ctx: PpuVramAccessContext) {
        if self
            .mapper
            .hook_mask()
            .contains(MapperHookMask::PPU_BUS_ADDRESS)
        {
            self.mapper
                .on_mapper_event(MapperEvent::PpuBusAddress { addr, ctx });
        }
    }

    /// Allows mappers to post-process the final value returned for a PPU VRAM read.
    pub fn ppu_read_override(&mut self, addr: u16, ctx: PpuVramAccessContext, value: u8) -> u8 {
        if self
            .mapper
            .hook_mask()
            .contains(MapperHookMask::PPU_READ_OVERRIDE)
        {
            self.mapper.ppu_read_override(addr, ctx, value)
        } else {
            value
        }
    }

    /// Advance expansion-audio channels by one CPU bus cycle.
    pub fn clock_expansion_audio(&mut self) {
        if let Some(expansion) = self.mapper.as_expansion_audio_mut() {
            expansion.clock_audio();
        }
    }

    /// Notify mapper of a CPU bus access.
    pub fn cpu_bus_access(
        &mut self,
        kind: CpuBusAccessKind,
        addr: u16,
        value: u8,
        cpu_cycle: u64,
        master_clock: u64,
    ) {
        if self
            .mapper
            .hook_mask()
            .contains(MapperHookMask::CPU_BUS_ACCESS)
        {
            self.mapper.on_mapper_event(MapperEvent::CpuBusAccess {
                kind,
                addr,
                value,
                cpu_cycle,
                master_clock,
            });
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

    /// Applies a mapper reset using the requested reset kind.
    pub fn reset(&mut self, kind: ResetKind) {
        self.mapper.reset(kind);
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

/// Load a cartridge from a ROM image.
///
/// The image can be provided as a heap-owned `Vec<u8>` (e.g. from disk
/// or network) or as a statically embedded blob (e.g. `include_bytes!`).
pub fn load_cartridge(image: impl Into<CartridgeImage>) -> Result<Cartridge, Error> {
    load_cartridge_with_provider(image, None)
}

/// Load a cartridge from a ROM image with an optional mapper provider.
pub fn load_cartridge_with_provider(
    image: impl Into<CartridgeImage>,
    provider: Option<&dyn Provider>,
) -> Result<Cartridge, Error> {
    match image.into() {
        CartridgeImage::Owned(bytes) => load_cartridge_from_bytes(&bytes, provider),
        CartridgeImage::Static(bytes) => load_cartridge_from_static_bytes(bytes, provider),
    }
}

fn build_cartridge_from_sections<'a>(
    header: Header,
    trainer: TrainerBytes<'a>,
    prg_rom: PrgRom,
    chr_rom: ChrRom,
    provider: Option<&dyn Provider>,
) -> Result<Cartridge, Error> {
    // 1) Give the external provider first chance when it explicitly
    //    declares support for this mapper ID.
    if let Some(provider) = provider
        && provider.supports_mapper(header.mapper())
    {
        let mut mapper = provider
            .get_mapper(header, prg_rom, chr_rom, trainer)
            .ok_or(Error::UnsupportedMapper(header.mapper()))?;
        mapper.reset(ResetKind::PowerOn);
        return Ok(Cartridge::new(header, mapper));
    }

    // 2) Fall back to the built-in mapper registry for known IDs.
    let mut mapper: Box<dyn Mapper> = match header.mapper() {
        0 => Box::new(Mapper0::new(header, prg_rom, chr_rom, trainer)),
        1 => Box::new(Mapper1::new(header, prg_rom, chr_rom, trainer)),
        2 => Box::new(Mapper2::new(header, prg_rom, chr_rom, trainer)),
        3 => Box::new(Mapper3::new(header, prg_rom, chr_rom, trainer)),
        4 => Box::new(Mapper4::new(header, prg_rom, chr_rom, trainer)),
        5 => Box::new(Mapper5::new(header, prg_rom, chr_rom, trainer)),
        6 => Box::new(Mapper6::new(header, prg_rom, chr_rom, trainer)),
        7 => Box::new(Mapper7::new(header, prg_rom, chr_rom, trainer)),
        8 => Box::new(Mapper8::new(header, prg_rom, chr_rom, trainer)),
        9 => Box::new(Mapper9::new(header, prg_rom, chr_rom, trainer)),
        10 => Box::new(Mapper10::new(header, prg_rom, chr_rom, trainer)),
        11 => Box::new(Mapper11::new(header, prg_rom, chr_rom, trainer)),
        13 => Box::new(Mapper13::new(header, prg_rom, chr_rom, trainer)),
        19 => Box::new(Mapper19::new(header, prg_rom, chr_rom, trainer)),
        21 => Box::new(Mapper21::new(header, prg_rom, chr_rom, trainer)),
        23 => Box::new(Mapper23::new(header, prg_rom, chr_rom, trainer)),
        25 => Box::new(Mapper25::new(header, prg_rom, chr_rom, trainer)),
        26 => Box::new(Mapper26::new(header, prg_rom, chr_rom, trainer)),
        34 => Box::new(Mapper34::new(header, prg_rom, chr_rom, trainer)),
        66 => Box::new(Mapper66::new(header, prg_rom, chr_rom, trainer)),
        71 => Box::new(Mapper71::new(header, prg_rom, chr_rom, trainer)),
        78 => Box::new(Mapper78::new(header, prg_rom, chr_rom, trainer)),
        85 => Box::new(Mapper85::new(header, prg_rom, chr_rom, trainer)),
        90 => Box::new(Mapper90::new(header, prg_rom, chr_rom, trainer)),
        119 => Box::new(Mapper119::new(header, prg_rom, chr_rom, trainer)),
        228 => Box::new(Mapper228::new(header, prg_rom, chr_rom, trainer)),
        // 3) Unknown to the core: let the provider try to supply a mapper
        //    implementation as a final fallback.
        other => provider
            .and_then(|provider| provider.get_mapper(header, prg_rom, chr_rom, trainer))
            .ok_or(Error::UnsupportedMapper(other))?,
    };

    // Apply mapper-specific power-on defaults once after construction.
    mapper.reset(ResetKind::PowerOn);
    Ok(Cartridge::new(header, mapper))
}

fn load_cartridge_from_bytes(
    bytes: &[u8],
    provider: Option<&dyn Provider>,
) -> Result<Cartridge, Error> {
    let header_bytes = bytes.get(..NES_HEADER_LEN).ok_or(Error::TooShort {
        actual: bytes.len(),
    })?;
    let header = Header::parse(header_bytes)?;
    let (trainer, prg_rom, chr_rom) = slice_sections(bytes, &header)?;

    build_cartridge_from_sections(header, trainer, prg_rom, chr_rom, provider)
}

fn load_cartridge_from_static_bytes(
    bytes: &'static [u8],
    provider: Option<&dyn Provider>,
) -> Result<Cartridge, Error> {
    let header_bytes = bytes.get(..NES_HEADER_LEN).ok_or(Error::TooShort {
        actual: bytes.len(),
    })?;
    let header = Header::parse(header_bytes)?;
    let (trainer, prg_rom, chr_rom) = slice_sections_static(bytes, &header)?;

    build_cartridge_from_sections(header, trainer, prg_rom, chr_rom, provider)
}

/// Load a cartridge directly from disk.
pub fn load_cartridge_from_file<P>(path: P) -> Result<Cartridge, Error>
where
    P: AsRef<Path>,
{
    load_cartridge_from_file_with_provider(path, None)
}

/// Load a cartridge directly from disk with an optional mapper provider.
pub fn load_cartridge_from_file_with_provider<P>(
    path: P,
    provider: Option<&dyn Provider>,
) -> Result<Cartridge, Error>
where
    P: AsRef<Path>,
{
    let bytes = fs::read(path)?;
    load_cartridge_with_provider(bytes, provider)
}

fn slice_trainer<'a>(
    bytes: &'a [u8],
    cursor: &mut usize,
    header: &Header,
) -> Result<TrainerBytes<'a>, Error> {
    if !header.trainer_present() {
        return Ok(None);
    }

    let end = cursor
        .checked_add(TRAINER_SIZE)
        .ok_or(Error::SectionTooShort {
            section: "trainer",
            expected: TRAINER_SIZE,
            actual: bytes.len().saturating_sub(*cursor),
        })?;
    let slice = bytes.get(*cursor..end).ok_or(Error::SectionTooShort {
        section: "trainer",
        expected: TRAINER_SIZE,
        actual: bytes.len().saturating_sub(*cursor),
    })?;
    *cursor = end;
    let array_ref: &[u8; TRAINER_SIZE] = slice.try_into().expect("trainer length mismatch");
    Ok(Some(array_ref))
}

fn slice_section<'a>(
    bytes: &'a [u8],
    cursor: &mut usize,
    len: usize,
    name: &'static str,
) -> Result<&'a [u8], Error> {
    if len == 0 {
        return Ok(&bytes[0..0]);
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
    Ok(slice)
}

fn slice_sections_static(
    bytes: &'static [u8],
    header: &Header,
) -> Result<(TrainerBytes<'static>, PrgRom, ChrRom), Error> {
    let mut cursor = NES_HEADER_LEN;
    let trainer = slice_trainer(bytes, &mut cursor, header)?;

    let prg_slice = slice_section(bytes, &mut cursor, header.prg_rom_size(), "PRG ROM")?;
    let chr_slice = slice_section(bytes, &mut cursor, header.chr_rom_size(), "CHR ROM")?;

    Ok((
        trainer,
        PrgRom::Borrowed(prg_slice),
        ChrRom::Borrowed(chr_slice),
    ))
}

fn slice_sections<'a>(
    bytes: &'a [u8],
    header: &Header,
) -> Result<(TrainerBytes<'a>, PrgRom, ChrRom), Error> {
    let mut cursor = NES_HEADER_LEN;
    let trainer = slice_trainer(bytes, &mut cursor, header)?;

    let prg_rom = slice_section(bytes, &mut cursor, header.prg_rom_size(), "PRG ROM")?;
    let chr_rom = slice_section(bytes, &mut cursor, header.chr_rom_size(), "CHR ROM")?;

    Ok((
        trainer,
        PrgRom::Owned(prg_rom.to_vec()),
        ChrRom::Owned(chr_rom.to_vec()),
    ))
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

        let cartridge = load_cartridge(rom).expect("parse cartridge");

        assert_eq!(cartridge.header().prg_rom_size(), 16 * 1024);
        assert_eq!(cartridge.header().chr_rom_size(), 8 * 1024);
        assert_eq!(cartridge.cpu_read(cpu_mem::PRG_ROM_START), Some(0xAA));
        assert_eq!(cartridge.ppu_read(0x0000), Some(0x55));
    }

    #[test]
    fn loads_cartridge_with_trainer() {
        let mut rom = base_header(1, 0, 0b0000_0100).to_vec();
        rom.extend(vec![0xFE; TRAINER_SIZE]);
        rom.extend(vec![0xAA; 16 * 1024]);

        let cartridge = load_cartridge(rom).expect("parse cartridge");

        assert!(cartridge.header().trainer_present());
        assert_eq!(cartridge.header().prg_rom_size(), 16 * 1024);
        assert_eq!(cartridge.cpu_read(cpu_mem::PRG_ROM_START), Some(0xAA));
    }

    #[test]
    fn errors_when_prg_section_missing() {
        let mut rom = base_header(1, 0, 0).to_vec();
        rom.extend(vec![0xAA; 1024]); // insufficient PRG data

        let err = load_cartridge(rom).expect_err("should fail");
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

        let err = load_cartridge(rom).expect_err("unsupported mapper should fail");
        assert!(matches!(err, Error::UnsupportedMapper(12)));
    }

    #[derive(Debug, Clone)]
    struct DummyMapper;

    impl Mapper for DummyMapper {
        fn cpu_read(&self, _addr: u16) -> Option<u8> {
            Some(0xFF)
        }

        fn cpu_write(&mut self, _addr: u16, _data: u8, _cpu_cycle: u64) {}

        fn ppu_read(&self, _addr: u16) -> Option<u8> {
            Some(0)
        }

        fn ppu_write(&mut self, _addr: u16, _data: u8) {}

        fn mirroring(&self) -> Mirroring {
            Mirroring::Horizontal
        }

        fn mapper_id(&self) -> u16 {
            999
        }
    }

    #[derive(Debug)]
    struct DummyProvider;

    impl Provider for DummyProvider {
        fn get_mapper(
            &self,
            _header: Header,
            _prg_rom: PrgRom,
            _chr_rom: ChrRom,
            _trainer: TrainerBytes<'_>,
        ) -> Option<Box<dyn Mapper>> {
            Some(Box::new(DummyMapper))
        }

        fn supports_mapper(&self, mapper_id: u16) -> bool {
            mapper_id == 999
        }
    }

    #[test]
    fn uses_provider_for_unknown_mapper() {
        let mut rom = base_header(1, 1, 0xC0).to_vec();
        rom.extend(vec![0xAA; 16 * 1024]); // PRG
        rom.extend(vec![0x55; 8 * 1024]); // CHR

        let provider = DummyProvider;
        let cartridge =
            load_cartridge_with_provider(rom, Some(&provider)).expect("provider supplies mapper");

        assert_eq!(cartridge.mapper().mapper_id(), 999);
    }
}
