use std::fmt::Debug;

use crate::memory;

pub mod cpu;
#[cfg(test)]
pub mod mock;

/// Expose the CPU stack page start address for stack helpers.
pub(crate) const STACK_ADDR: u16 = memory::cpu::STACK_PAGE_START;

pub trait Bus: Debug {
    fn read(&mut self, addr: u16) -> u8;

    fn write(&mut self, addr: u16, data: u8);

    /// Returns `true` when any peripheral (cartridge/APU/...) asserts the IRQ line.
    fn irq_pending(&mut self) -> bool {
        false
    }

    /// Clears the IRQ sources that have been serviced.
    fn clear_irq(&mut self) {}
}
