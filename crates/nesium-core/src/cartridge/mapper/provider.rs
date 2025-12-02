use std::fmt::Debug;

use crate::cartridge::{ChrRom, Mapper, PrgRom, TrainerBytes, header::Header};

/// Source of user-provided mappers when the core does not implement a board.
///
/// `Provider` is consulted only when the requested mapper ID is unknown to the
/// built-in registry. Returning `None` defers to the core's default
/// `UnsupportedMapper` error.
///
/// # Example
/// ```no_run
/// use nesium_core::{
///     cartridge::{Mapper, Provider, TRAINER_SIZE, header::Header, load_cartridge_with_provider},
///     ppu::palette::PaletteKind,
///     Nes,
/// };
///
/// #[derive(Debug)]
/// struct CustomMapper;
///
/// impl Mapper for CustomMapper {
///     fn cpu_read(&self, _addr: u16) -> Option<u8> { Some(0) }
///     fn cpu_write(&mut self, _addr: u16, _data: u8, _cpu_cycle: u64) {}
///     fn ppu_read(&self, _addr: u16) -> Option<u8> { Some(0) }
///     fn ppu_write(&mut self, _addr: u16, _data: u8) {}
///     fn mirroring(&self) -> crate::cartridge::header::Mirroring {
///         crate::cartridge::header::Mirroring::Horizontal
///     }
///     fn mapper_id(&self) -> u16 { 1234 }
/// }
///
/// #[derive(Debug)]
/// struct CustomProvider;
///
/// impl Provider for CustomProvider {
///     fn get_mapper(
///         &self,
///         header: Header,
///         prg_rom: PrgRom,
///         chr_rom: ChrRom,
///         trainer: Option<Box<[u8; TRAINER_SIZE]>>,
///     ) -> Option<Box<dyn Mapper>> {
///         let _ = (prg_rom, chr_rom, trainer);
///         (header.mapper == 1234).then(|| Box::new(CustomMapper) as Box<dyn Mapper>)
///     }
/// }
///
/// let provider = CustomProvider;
/// let rom_bytes: Vec<u8> = std::fs::read("custom_board.nes")?;
/// let _cart = load_cartridge_with_provider(&rom_bytes, Some(&provider))?;
///
/// let mut nes = Nes::new(nesium_core::ppu::buffer::ColorFormat::Rgb555);
/// nes.set_palette(PaletteKind::NesdevNtsc.palette());
/// nes.set_mapper_provider(Some(Box::new(provider)));
/// // Unknown mapper IDs will now ask `CustomProvider` for an implementation.
/// ```
pub trait Provider: Debug + Send {
    fn get_mapper(
        &self,
        header: Header,
        prg_rom: PrgRom,
        chr_rom: ChrRom,
        trainer: TrainerBytes,
    ) -> Option<Box<dyn Mapper>>;
}
