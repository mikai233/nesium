#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Bus {}

impl Bus {
    pub fn read(&mut self, addr: u16) -> u8 {
        unimplemented!()
    }

    pub fn write(&mut self, addr: u16, data: u8) {
        unimplemented!()
    }
}
