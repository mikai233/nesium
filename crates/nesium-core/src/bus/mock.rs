use crate::bus::Bus;

#[derive(Debug)]
pub(crate) struct MockBus {
    pub(crate) mem: [u8; 0xFFFF],
}

impl Default for MockBus {
    fn default() -> Self {
        Self { mem: [0; 0xFFFF] }
    }
}

impl Bus for MockBus {
    fn read(&mut self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    fn write(&mut self, addr: u16, data: u8) {
        self.mem[addr as usize] = data;
    }
}
