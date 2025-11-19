//! Cartridge mapper registry, traits, and shared helpers.
//!
//! This module wires together the concrete mapper implementations, defines the
//! core [`Mapper`] trait they implement, and exposes a few small helpers for
//! PRG RAM allocation and trainer placement that are reused across mappers.

use std::{any::Any, borrow::Cow, fmt::Debug};

use dyn_clone::DynClone;

pub mod chr_storage;
pub mod mapper0;
pub mod mapper2;
pub mod mapper3;
pub mod mapper7;

pub(crate) use chr_storage::{ChrStorage, select_chr_storage};
pub use mapper0::Mapper0;
pub use mapper2::Mapper2;
pub use mapper3::Mapper3;
pub use mapper7::Mapper7;

use crate::{
    cartridge::{header::Header, TRAINER_SIZE},
    memory::cpu as cpu_mem,
};

/// CPU address at which the optional 512 byte trainer is mapped into PRG RAM.
const TRAINER_BASE_ADDR: u16 = 0x7000;
/// Offset of the trainer region within the PRG RAM window.
const TRAINER_RAM_OFFSET: usize = (TRAINER_BASE_ADDR - cpu_mem::PRG_RAM_START) as usize;

/// Core mapper interface implemented by all cartridge boards.
pub trait Mapper: Debug + DynClone + Any + 'static {
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

    /// Optional introspection hook for PRG ROM contents.
    fn prg_rom(&self) -> Option<&[u8]> {
        None
    }

    /// Optional introspection hook for PRG RAM contents.
    fn prg_ram(&self) -> Option<&[u8]> {
        None
    }

    /// Optional mutable access to PRG RAM contents.
    fn prg_ram_mut(&mut self) -> Option<&mut [u8]> {
        None
    }

    /// Optional introspection hook for CHR ROM contents.
    fn chr_rom(&self) -> Option<&[u8]> {
        None
    }

    /// Optional introspection hook for CHR RAM contents.
    fn chr_ram(&self) -> Option<&[u8]> {
        None
    }

    /// Optional mutable access to CHR RAM contents.
    fn chr_ram_mut(&mut self) -> Option<&mut [u8]> {
        None
    }

    /// Mapper identifier as used in the iNES header.
    fn mapper_id(&self) -> u16;

    /// Human readable mapper name.
    fn name(&self) -> Cow<'static, str> {
        Cow::Owned(format!("Mapper {}", self.mapper_id()))
    }
}

dyn_clone::clone_trait_object!(Mapper);

/// Downcasts a mapper reference to a concrete implementation.
pub fn mapper_downcast_ref<T: Mapper + 'static>(mapper: &dyn Mapper) -> Option<&T> {
    (mapper as &dyn Any).downcast_ref::<T>()
}

/// Downcasts a mutable mapper reference to a concrete implementation.
pub fn mapper_downcast_mut<T: Mapper + 'static>(mapper: &mut dyn Mapper) -> Option<&mut T> {
    (mapper as &mut dyn Any).downcast_mut::<T>()
}

/// Allocate CPU‑visible PRG RAM according to the header hints.
///
/// For NES 2.0 headers this picks the larger of volatile and battery‑backed
/// PRG RAM sizes. Legacy iNES headers with `0` fall back to an empty slice.
pub(crate) fn allocate_prg_ram(header: &Header) -> Box<[u8]> {
    let size = header.prg_ram_size.max(header.prg_nvram_size);
    if size == 0 {
        Vec::new().into_boxed_slice()
    } else {
        vec![0; size].into_boxed_slice()
    }
}

/// Returns the region of PRG RAM where the optional trainer should be copied.
///
/// When the PRG RAM region is too small to host the trainer, `None` is
/// returned and the trainer contents are silently ignored.
pub(crate) fn trainer_destination(prg_ram: &mut [u8]) -> Option<&mut [u8]> {
    if prg_ram.len() < TRAINER_RAM_OFFSET + TRAINER_SIZE {
        return None;
    }
    Some(&mut prg_ram[TRAINER_RAM_OFFSET..TRAINER_RAM_OFFSET + TRAINER_SIZE])
}
