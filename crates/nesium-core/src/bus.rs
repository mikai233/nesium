use std::fmt::Debug;

use crate::bus::nes::NesBus;

pub mod nes;

pub trait Bus: Debug {
    fn read(&mut self, addr: u16) -> u8;

    fn write(&mut self, addr: u16, data: u8);
}

#[derive(Debug)]
pub(crate) enum BusImpl {
    Nes(NesBus),
    Dynamic(Box<dyn Bus>),
}

impl Bus for BusImpl {
    fn read(&mut self, addr: u16) -> u8 {
        match self {
            BusImpl::Nes(nes_bus) => nes_bus.read(addr),
            BusImpl::Dynamic(dynamic) => dynamic.read(addr),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match self {
            BusImpl::Nes(nes_bus) => nes_bus.write(addr, data),
            BusImpl::Dynamic(dynamic) => dynamic.write(addr, data),
        }
    }
}
