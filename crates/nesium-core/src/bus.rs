use std::fmt::Debug;

pub mod mock;
pub mod nes;

pub(crate) const STACK_ADDR: u16 = 0x0100;

pub trait Bus: Debug {
    fn read(&mut self, addr: u16) -> u8;

    fn write(&mut self, addr: u16, data: u8);
}
