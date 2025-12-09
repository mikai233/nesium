use crate::cartridge::{Cartridge, mapper::NametableTarget};

/// Temporary view that lets the PPU reach the cartridge CHR space without storing a raw pointer.
///
/// The bus creates one of these per PPU call, so lifetimes remain explicit and borrow-checked.
#[derive(Default)]
pub struct PatternBus<'a> {
    cartridge: Option<&'a mut Cartridge>,
    /// Snapshot of the current CPU bus cycle when this view was created.
    cpu_cycle: u64,
}

impl<'a> PatternBus<'a> {
    pub fn new(cartridge: Option<&'a mut Cartridge>, cpu_cycle: u64) -> Self {
        Self {
            cartridge,
            cpu_cycle,
        }
    }

    pub fn none() -> Self {
        Self {
            cartridge: None,
            cpu_cycle: 0,
        }
    }

    pub fn from_cartridge(cartridge: &'a mut Cartridge, cpu_cycle: u64) -> Self {
        Self {
            cartridge: Some(cartridge),
            cpu_cycle,
        }
    }

    pub fn cpu_cycle(&self) -> u64 {
        self.cpu_cycle
    }

    pub fn read(
        &mut self,
        addr: u16,
        ctx: crate::cartridge::mapper::PpuVramAccessContext,
    ) -> Option<u8> {
        if let Some(cart) = self.cartridge.as_deref_mut() {
            cart.ppu_vram_access(addr, ctx);
            cart.ppu_read(addr)
        } else {
            None
        }
    }

    pub fn write(
        &mut self,
        addr: u16,
        value: u8,
        ctx: crate::cartridge::mapper::PpuVramAccessContext,
    ) -> bool {
        if let Some(cart) = self.cartridge.as_deref_mut() {
            cart.ppu_vram_access(addr, ctx);
            cart.ppu_write(addr, value);
            true
        } else {
            false
        }
    }

    pub fn map_nametable(&self, addr: u16) -> NametableTarget {
        if let Some(cart) = self.cartridge.as_deref() {
            cart.map_nametable(addr)
        } else {
            // No cartridge: treat nametable area as a simple 2 KiB CIRAM window.
            let base = addr & 0x0FFF;
            let offset = base & 0x07FF;
            NametableTarget::Ciram(offset)
        }
    }

    pub fn mapper_nametable_read(&self, offset: u16) -> Option<u8> {
        self.cartridge
            .as_deref()
            .map(|cart| cart.mapper_nametable_read(offset))
    }

    pub fn mapper_nametable_write(&mut self, offset: u16, value: u8) -> bool {
        if let Some(cart) = self.cartridge.as_deref_mut() {
            cart.mapper_nametable_write(offset, value);
            true
        } else {
            false
        }
    }
}
