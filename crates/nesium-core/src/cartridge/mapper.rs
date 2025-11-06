use std::fmt::Debug;

use dyn_clone::DynClone;

use crate::cartridge::header::Header;

pub trait Mapper: DynClone + Debug {
    fn new(header: &Header) -> Self
    where
        Self: Sized;

    fn read(&self, addr: u16) -> u8;

    fn write(&mut self, addr: u16, data: u8);
}

dyn_clone::clone_trait_object!(Mapper);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DummyMapper;

impl Mapper for DummyMapper {
    fn new(header: &Header) -> Self
    where
        Self: Sized,
    {
        todo!()
    }

    fn read(&self, addr: u16) -> u8 {
        todo!()
    }

    fn write(&mut self, addr: u16, data: u8) {
        todo!()
    }
}

pub fn get_mapper(id: u16, header: &Header) -> Box<dyn Mapper> {
    match id {
        _ => Box::new(DummyMapper::new(header)),
    }
}
