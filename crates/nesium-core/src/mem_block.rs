use core::ops::{Deref, DerefMut};

#[cfg(any(
    feature = "boxed-memblock",
    target_arch = "wasm32",
    target_arch = "xtensa"
))]
type MemBlockStorage<T, const N: usize> = Box<[T; N]>;

#[cfg(not(any(
    feature = "boxed-memblock",
    target_arch = "wasm32",
    target_arch = "xtensa"
)))]
type MemBlockStorage<T, const N: usize> = [T; N];

#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MemBlock<T, const N: usize>(MemBlockStorage<T, N>);

/// Convenience alias for a `MemBlock` of bytes.
pub type ByteBlock<const N: usize> = MemBlock<u8, N>;

pub mod cpu {
    use crate::memory::cpu as cpu_mem;

    pub type Ram = super::MemBlock<u8, { cpu_mem::INTERNAL_RAM_SIZE }>;
    pub type PrgRam =
        super::MemBlock<u8, { (cpu_mem::PRG_RAM_END - cpu_mem::PRG_RAM_START + 1) as usize }>;
    pub type AddressSpace = super::MemBlock<u8, { cpu_mem::CPU_ADDR_END as usize + 1 }>;
}

pub mod ppu {
    use crate::memory::ppu as ppu_mem;

    pub type Vram = super::MemBlock<u8, { ppu_mem::VRAM_SIZE }>;
    pub type PaletteRam = super::MemBlock<u8, { ppu_mem::PALETTE_RAM_SIZE }>;
    pub type OamRam = super::MemBlock<u8, { ppu_mem::OAM_RAM_SIZE }>;
    pub type SecondaryOamRam = super::MemBlock<u8, { ppu_mem::SECONDARY_OAM_RAM_SIZE }>;
    /// Generic 8-byte sprite line buffer (boxed on xtensa/wasm/with `boxed-memblock`).
    pub type SpriteLineRam = super::MemBlock<u8, 8>;
}

pub mod apu {
    use crate::memory::apu as apu_mem;

    pub type RegisterRam = super::MemBlock<u8, { apu_mem::REGISTER_SPACE }>;
}

impl<T, const N: usize> MemBlock<T, N> {
    #[inline]
    pub fn as_slice(&self) -> &[T] {
        #[cfg(any(
            feature = "boxed-memblock",
            target_arch = "wasm32",
            target_arch = "xtensa"
        ))]
        {
            &*self.0
        }
        #[cfg(not(any(
            feature = "boxed-memblock",
            target_arch = "wasm32",
            target_arch = "xtensa"
        )))]
        {
            &self.0
        }
    }

    #[inline]
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        #[cfg(any(
            feature = "boxed-memblock",
            target_arch = "wasm32",
            target_arch = "xtensa"
        ))]
        {
            &mut *self.0
        }
        #[cfg(not(any(
            feature = "boxed-memblock",
            target_arch = "wasm32",
            target_arch = "xtensa"
        )))]
        {
            &mut self.0
        }
    }
}

impl<T: Copy + Default, const N: usize> MemBlock<T, N> {
    pub fn new() -> Self {
        Self(new_storage())
    }
}

impl<T: Copy + Default, const N: usize> Default for MemBlock<T, N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> Deref for MemBlock<T, N> {
    type Target = [T];

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<T, const N: usize> DerefMut for MemBlock<T, N> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}

#[cfg(not(any(
    feature = "boxed-memblock",
    target_arch = "wasm32",
    target_arch = "xtensa"
)))]
impl<T: Copy, const N: usize> Copy for MemBlock<T, N> {}

#[cfg(any(
    feature = "boxed-memblock",
    target_arch = "wasm32",
    target_arch = "xtensa"
))]
fn new_storage<T: Copy + Default, const N: usize>() -> MemBlockStorage<T, N> {
    Box::new([T::default(); N])
}

#[cfg(not(any(
    feature = "boxed-memblock",
    target_arch = "wasm32",
    target_arch = "xtensa"
)))]
fn new_storage<T: Copy + Default, const N: usize>() -> MemBlockStorage<T, N> {
    [T::default(); N]
}
