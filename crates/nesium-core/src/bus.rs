use std::fmt::Debug;

use crate::memory;

pub mod cpu;
#[cfg(test)]
pub mod mock;
pub(crate) mod open_bus;

pub(crate) use open_bus::OpenBus;

/// Expose the CPU stack page start address for stack helpers.
pub(crate) const STACK_ADDR: u16 = memory::cpu::STACK_PAGE_START;

/// Core CPU/PPU bus abstraction.
///
/// Implementations are expected to honour open-bus behaviour (see
/// [`open_bus::OpenBus`]), returning the last driven value for write-only or
/// unmapped addresses when no device actively drives the data lines.
pub trait Bus: Debug {
    fn read(&mut self, addr: u16) -> u8;

    fn write(&mut self, addr: u16, data: u8);

    /// Returns a pending OAM DMA page value (written via `$4014`), if any.
    /// Default implementations have no DMA bridge, so they always return `None`.
    fn take_oam_dma_request(&mut self) -> Option<u8> {
        None
    }

    /// PPU-side read for pattern table accesses (`$0000-$1FFF`).
    fn ppu_read(&mut self, addr: u16) -> u8 {
        let _ = addr;
        0
    }

    /// PPU-side write for pattern table accesses (`$0000-$1FFF`).
    fn ppu_write(&mut self, addr: u16, value: u8) {
        let _ = (addr, value);
    }

    /// Returns current NMI line level (PPU NMI output). Non-destructive.
    /// CPU is responsible for edge-detecting and latching.
    fn nmi_line(&mut self) -> bool {
        false
    }

    /// Returns `true` when any peripheral (cartridge/APU/...) asserts the IRQ line.
    fn irq_pending(&mut self) -> bool {
        false
    }

    /// Clears the IRQ sources that have been serviced.
    fn clear_irq(&mut self) {}
}
