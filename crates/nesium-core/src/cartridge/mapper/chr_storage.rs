//! CHR storage helpers for simple mappers.
//!
//! Many NES boards expose either CHR ROM *or* CHR RAM to the PPU. This module
//! wraps those cases in a tiny enum so that mappers can share the common
//! address decoding and mirroring logic instead of re‑implementing it.
//!
//! # Quick overview
//! - [`select_chr_storage`] inspects the cartridge [`Header`] and chooses
//!   between ROM, RAM, or no CHR storage.
//! - [`ChrStorage::read`] / [`ChrStorage::write`] operate in the PPU address
//!   range `0x0000..=0x1FFF`, applying the usual 8 KiB mirroring.
//! - Introspection helpers (`as_rom` / `as_ram` / `as_ram_mut`) let tests or
//!   tools peek at the underlying CHR contents.

use crate::cartridge::header::Header;

/// High‑level description of PPU‑side CHR storage.
#[derive(Debug, Clone)]
pub(crate) enum ChrStorage {
    /// No CHR memory is present; reads return `0` and writes are ignored.
    None,
    /// CHR is backed by read‑only ROM data from the cartridge image.
    Rom(Box<[u8]>),
    /// CHR is backed by writable RAM located on the cartridge.
    Ram(Box<[u8]>),
}

impl ChrStorage {
    /// Read a byte from CHR space, applying 8 KiB mirroring.
    pub(crate) fn read(&self, addr: u16) -> u8 {
        let offset = (addr as usize) & 0x1FFF;
        match self {
            ChrStorage::Rom(rom) => {
                if rom.is_empty() {
                    0
                } else {
                    let len = rom.len();
                    rom[offset % len]
                }
            }
            ChrStorage::Ram(ram) => {
                if ram.is_empty() {
                    0
                } else {
                    let len = ram.len();
                    ram[offset % len]
                }
            }
            ChrStorage::None => 0,
        }
    }

    /// Write a byte to CHR RAM, if present.
    ///
    /// Writes are ignored when the cartridge only provides CHR ROM or no CHR at all.
    pub(crate) fn write(&mut self, addr: u16, data: u8) {
        let offset = (addr as usize) & 0x1FFF;
        if let ChrStorage::Ram(ram) = self {
            let len = ram.len();
            if len != 0 {
                ram[offset % len] = data;
            }
        }
    }

    /// Returns a view of the underlying CHR ROM, when present.
    pub(crate) fn as_rom(&self) -> Option<&[u8]> {
        if let ChrStorage::Rom(rom) = self {
            Some(rom.as_ref())
        } else {
            None
        }
    }

    /// Returns a view of the underlying CHR RAM, when present.
    pub(crate) fn as_ram(&self) -> Option<&[u8]> {
        if let ChrStorage::Ram(ram) = self {
            Some(ram.as_ref())
        } else {
            None
        }
    }

    /// Returns a mutable view of the underlying CHR RAM, when present.
    pub(crate) fn as_ram_mut(&mut self) -> Option<&mut [u8]> {
        if let ChrStorage::Ram(ram) = self {
            Some(ram.as_mut())
        } else {
            None
        }
    }
}

/// Construct an appropriate [`ChrStorage`] instance based on the header.
///
/// - When `chr_rom_size > 0`, CHR is treated as ROM only.
/// - Otherwise, a CHR RAM slice is allocated using the larger of the volatile
///   and battery‑backed CHR sizes, if any.
/// - When neither ROM nor RAM is present, [`ChrStorage::None`] is used.
pub(crate) fn select_chr_storage(header: &Header, chr_rom: Box<[u8]>) -> ChrStorage {
    if header.chr_rom_size > 0 {
        ChrStorage::Rom(chr_rom)
    } else {
        let chr_ram_size = header.chr_ram_size.max(header.chr_nvram_size);
        if chr_ram_size == 0 {
            ChrStorage::None
        } else {
            let chr_ram = vec![0; chr_ram_size].into_boxed_slice();
            ChrStorage::Ram(chr_ram)
        }
    }
}
