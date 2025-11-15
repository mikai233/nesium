use std::fmt::Debug;

use crate::memory;

pub mod mock;
pub mod nes;

/// Expose the CPU stack page start address for stack helpers.
pub(crate) const STACK_ADDR: u16 = memory::cpu::STACK_PAGE_START;

pub trait Bus: Debug {
    fn read(&mut self, addr: u16) -> u8;

    fn write(&mut self, addr: u16, data: u8);
}
