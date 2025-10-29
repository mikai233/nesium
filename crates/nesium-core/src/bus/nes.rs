use crate::bus::Bus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NesBus {}

impl Bus for NesBus {
    fn read(&mut self, addr: u16) -> u8 {
        unimplemented!()
    }

    fn write(&mut self, addr: u16, data: u8) {
        unimplemented!()
    }
}
