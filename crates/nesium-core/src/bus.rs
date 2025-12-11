use std::fmt::Debug;

use crate::{
    apu::Apu, cartridge::Cartridge, controller::Controller, cpu::Cpu, mem_block::cpu as cpu_ram,
    memory, ppu::Ppu,
};

pub mod cpu;
#[cfg(test)]
pub mod mock;
pub(crate) mod open_bus;

pub(crate) use open_bus::OpenBus;

/// Expose the CPU stack page start address for stack helpers.
pub(crate) const STACK_ADDR: u16 = memory::cpu::STACK_PAGE_START;

/// Immutable view of the hardware attached to the CPU bus.
pub struct BusDevices<'a> {
    pub ram: &'a cpu_ram::Ram,
    pub ppu: &'a Ppu,
    pub apu: &'a Apu,
    pub cartridge: Option<&'a Cartridge>,
    pub controllers: &'a [Controller; 2],
}

/// Mutable view of the hardware attached to the CPU bus.
pub struct BusDevicesMut<'a> {
    pub ram: &'a mut cpu_ram::Ram,
    pub ppu: &'a mut Ppu,
    pub apu: &'a mut Apu,
    pub cartridge: Option<&'a mut Cartridge>,
    pub controllers: &'a mut [Controller; 2],
}

/// Core CPU/PPU bus abstraction.
///
/// Implementations are expected to honour open-bus behaviour (see
/// [`open_bus::OpenBus`]), returning the last driven value for write-only or
/// unmapped addresses when no device actively drives the data lines.
pub trait Bus: Debug {
    fn mem_read(&mut self, cpu: &mut Cpu, addr: u16) -> u8;

    fn mem_write(&mut self, cpu: &mut Cpu, addr: u16, data: u8);

    /// Side-effect-free read used for reset vector fetches. Defaults to a
    /// regular timed read so existing implementations remain valid.
    fn peek(&mut self, cpu: &mut Cpu, addr: u16) -> u8 {
        self.mem_read(cpu, addr)
    }

    /// Internal CPU cycle that does not perform a bus access but must advance
    /// timing (master clock, PPU/APU, open-bus decay, mapper clocks).
    fn internal_cycle(&mut self, cpu: &mut Cpu);

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

    fn cycles(&self) -> u64;

    /// Snapshot of the attached devices for read-only access.
    fn devices(&self) -> BusDevices<'_>;

    /// Mutable view of all devices on the bus.
    fn devices_mut(&mut self) -> BusDevicesMut<'_>;
}

// Allow mutable references to Bus implementors (including trait objects) to be used
// wherever a Bus is expected, enabling easier static dispatch in CPU helpers.
impl<T: Bus + ?Sized> Bus for &mut T {
    fn mem_read(&mut self, cpu: &mut Cpu, addr: u16) -> u8 {
        (**self).mem_read(cpu, addr)
    }

    fn mem_write(&mut self, cpu: &mut Cpu, addr: u16, data: u8) {
        (**self).mem_write(cpu, addr, data)
    }

    fn peek(&mut self, cpu: &mut Cpu, addr: u16) -> u8 {
        (**self).peek(cpu, addr)
    }

    fn internal_cycle(&mut self, cpu: &mut Cpu) {
        (**self).internal_cycle(cpu)
    }

    fn take_oam_dma_request(&mut self) -> Option<u8> {
        (**self).take_oam_dma_request()
    }

    fn devices(&self) -> BusDevices<'_> {
        (**self).devices()
    }

    fn devices_mut(&mut self) -> BusDevicesMut<'_> {
        (**self).devices_mut()
    }

    fn ppu_read(&mut self, addr: u16) -> u8 {
        (**self).ppu_read(addr)
    }

    fn ppu_write(&mut self, addr: u16, value: u8) {
        (**self).ppu_write(addr, value)
    }

    fn nmi_line(&mut self) -> bool {
        (**self).nmi_line()
    }

    fn irq_pending(&mut self) -> bool {
        (**self).irq_pending()
    }

    fn clear_irq(&mut self) {
        (**self).clear_irq()
    }

    fn cycles(&self) -> u64 {
        (**self).cycles()
    }
}
