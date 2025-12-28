use crate::{
    apu::Apu, cartridge::Cartridge, controller::ControllerPorts, mem_block::cpu as cpu_ram, memory,
    ppu::Ppu,
};

pub mod cpu;
pub(crate) mod open_bus;
pub(crate) mod savestate;

pub use cpu::CpuBus;
pub(crate) use open_bus::OpenBus;

/// Expose the CPU stack page start address for stack helpers.
pub(crate) const STACK_ADDR: u16 = memory::cpu::STACK_PAGE_START;

/// Immutable view of the hardware attached to the CPU bus.
pub struct BusDevices<'a> {
    pub ram: &'a cpu_ram::Ram,
    pub ppu: &'a Ppu,
    pub apu: &'a Apu,
    pub cartridge: Option<&'a Cartridge>,
    pub controllers: &'a ControllerPorts,
}

/// Mutable view of the hardware attached to the CPU bus.
pub struct BusDevicesMut<'a> {
    pub ram: &'a mut cpu_ram::Ram,
    pub ppu: &'a mut Ppu,
    pub apu: &'a mut Apu,
    pub cartridge: Option<&'a mut Cartridge>,
    pub controllers: &'a mut ControllerPorts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(
    feature = "savestate-serde",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum DmcDmaEvent {
    Request { addr: u16 },
    Abort,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(
    feature = "savestate-serde",
    derive(serde::Serialize, serde::Deserialize)
)]
pub struct PendingDma {
    pub oam_page: Option<u8>,
    pub dmc: Option<DmcDmaEvent>,
}
