use std::fmt::Debug;

use crate::cartridge::{ChrRom, Mapper, PrgRom, TrainerBytes, header::Header};

/// Source of user-provided mappers when the core does not implement a board.
///
/// `Provider` is consulted only when the requested mapper ID is unknown to the
/// built-in registry. Returning `None` defers to the core's default
/// `UnsupportedMapper` error.
///
/// # Example
/// ```ignore
/// use nesium_core::{
///     cartridge::{Mapper, Provider, TrainerBytes, header::Header, load_cartridge_with_provider},
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
///     fn supports_mapper(&self, mapper_id: u16) -> bool {
///         // Take over mapper ID 1234 even if the core implements it.
///         mapper_id == 1234
///     }
///
///     fn get_mapper(
///         &self,
///         header: Header,
///         prg_rom: PrgRom,
///         chr_rom: ChrRom,
///         trainer: TrainerBytes<'_>,
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
/// // Mapper 1234 (and any other IDs for which `supports_mapper` returns true)
/// // will now use `CustomProvider`'s implementation. Unknown mapper IDs that
/// // are not supported by the core will still fall back to `CustomProvider`.
/// ```
pub trait Provider: Debug + Send {
    /// Returns `true` when this provider wants to supply the mapper
    /// implementation for the given mapper ID.
    ///
    /// When this returns `true`, the core will prefer the provider's
    /// implementation even if it has a built‑in mapper for that ID.
    /// The default implementation returns `false`, meaning the provider
    /// only participates as a fallback for mapper IDs that the core
    /// does not implement.
    fn supports_mapper(&self, mapper_id: u16) -> bool;

    fn get_mapper(
        &self,
        header: Header,
        prg_rom: PrgRom,
        chr_rom: ChrRom,
        trainer: TrainerBytes,
    ) -> Option<Box<dyn Mapper>>;
}
