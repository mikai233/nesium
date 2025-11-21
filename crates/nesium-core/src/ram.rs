use core::ops::{Deref, DerefMut};

#[cfg(any(feature = "boxed-ram", target_arch = "wasm32", target_arch = "xtensa"))]
type RamStorage<const N: usize> = Box<[u8; N]>;

#[cfg(not(any(feature = "boxed-ram", target_arch = "wasm32", target_arch = "xtensa")))]
type RamStorage<const N: usize> = [u8; N];

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Ram<const N: usize>(RamStorage<N>);

pub mod cpu {
    use crate::memory::cpu as cpu_mem;

    pub type Ram = super::Ram<{ cpu_mem::INTERNAL_RAM_SIZE }>;
    pub type PrgRam = super::Ram<{ (cpu_mem::PRG_RAM_END - cpu_mem::PRG_RAM_START + 1) as usize }>;
    pub type AddressSpace = super::Ram<{ cpu_mem::CPU_ADDR_END as usize + 1 }>;
}

pub mod ppu {
    use crate::memory::ppu as ppu_mem;

    pub type Vram = super::Ram<{ ppu_mem::VRAM_SIZE }>;
    pub type PaletteRam = super::Ram<{ ppu_mem::PALETTE_RAM_SIZE }>;
    pub type OamRam = super::Ram<{ ppu_mem::OAM_RAM_SIZE }>;
    pub type SecondaryOamRam = super::Ram<{ ppu_mem::SECONDARY_OAM_RAM_SIZE }>;
    /// Generic 8-byte sprite line buffer (boxed on xtensa/wasm/with `boxed-ram`).
    pub type SpriteLineRam = super::Ram<8>;
}

pub mod apu {
    use crate::memory::apu as apu_mem;

    pub type RegisterRam = super::Ram<{ apu_mem::REGISTER_SPACE }>;
}

impl<const N: usize> Ram<N> {
    pub fn new() -> Self {
        Self(new_storage())
    }

    pub fn read(&self, addr: usize) -> u8 {
        self.as_slice()[addr]
    }

    pub fn write(&mut self, addr: usize, value: u8) {
        self.as_mut_slice()[addr] = value;
    }

    pub fn as_slice(&self) -> &[u8] {
        StorageView::<N>::view(&self.0)
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        StorageView::<N>::view_mut(&mut self.0)
    }
}

impl<const N: usize> Default for Ram<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize> Deref for Ram<N> {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<const N: usize> DerefMut for Ram<N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

#[cfg(not(any(feature = "boxed-ram", target_arch = "wasm32", target_arch = "xtensa")))]
impl<const N: usize> Copy for Ram<N> {}

trait StorageView<const N: usize> {
    fn view(&self) -> &[u8];
    fn view_mut(&mut self) -> &mut [u8];
}

impl<const N: usize> StorageView<N> for [u8; N] {
    fn view(&self) -> &[u8] {
        self.as_slice()
    }

    fn view_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}

impl<const N: usize> StorageView<N> for Box<[u8; N]> {
    fn view(&self) -> &[u8] {
        self.as_ref()
    }

    fn view_mut(&mut self) -> &mut [u8] {
        self.as_mut()
    }
}

#[cfg(any(feature = "boxed-ram", target_arch = "wasm32", target_arch = "xtensa"))]
fn new_storage<const N: usize>() -> RamStorage<N> {
    Box::new([0; N])
}

#[cfg(not(any(feature = "boxed-ram", target_arch = "wasm32", target_arch = "xtensa")))]
fn new_storage<const N: usize>() -> RamStorage<N> {
    [0; N]
}
