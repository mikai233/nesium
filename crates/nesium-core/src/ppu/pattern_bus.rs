use crate::cartridge::{
    Cartridge,
    mapper::{NametableTarget, PpuVramAccessContext},
};

/// Temporary view that lets the PPU reach the cartridge CHR space without storing a raw pointer.
///
/// The bus creates one of these per PPU call, so lifetimes remain explicit and borrow-checked.
#[derive(Default)]
pub struct PpuBus<'a> {
    cartridge: Option<&'a mut Cartridge>,
    /// Snapshot of the current CPU bus cycle when this view was created.
    cpu_cycle: u64,
}

impl<'a> PpuBus<'a> {
    pub fn new(cartridge: Option<&'a mut Cartridge>, cpu_cycle: u64) -> Self {
        Self {
            cartridge,
            cpu_cycle,
        }
    }

    pub fn cpu_cycle(&self) -> u64 {
        self.cpu_cycle
    }

    pub fn read(&mut self, addr: u16, ctx: PpuVramAccessContext) -> Option<u8> {
        if let Some(cart) = self.cartridge.as_deref_mut() {
            cart.ppu_vram_access(addr, ctx);
            cart.ppu_read(addr)
        } else {
            None
        }
    }

    pub fn write(&mut self, addr: u16, value: u8, ctx: PpuVramAccessContext) -> bool {
        if let Some(cart) = self.cartridge.as_deref_mut() {
            cart.ppu_vram_access(addr, ctx);
            cart.ppu_write(addr, value);
            true
        } else {
            false
        }
    }

    /// CHR bus read convenience method that always returns a byte.
    pub fn chr_read(&mut self, addr: u16, ctx: PpuVramAccessContext) -> u8 {
        if let Some(cart) = self.cartridge.as_deref_mut() {
            cart.ppu_vram_access(addr, ctx);
            cart.chr_read(addr)
        } else {
            0
        }
    }

    /// CHR bus write convenience method for CHR RAM mappers.
    pub fn chr_write(&mut self, addr: u16, value: u8, ctx: PpuVramAccessContext) {
        if let Some(cart) = self.cartridge.as_deref_mut() {
            cart.ppu_vram_access(addr, ctx);
            cart.chr_write(addr, value);
        }
    }

    /// Notifies the mapper of a VRAM address bus access without performing a read/write.
    pub fn notify_vram_access(&mut self, addr: u16, ctx: PpuVramAccessContext) {
        if let Some(cart) = self.cartridge.as_deref_mut() {
            cart.ppu_vram_access(addr, ctx);
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
