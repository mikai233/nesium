use crate::{bus::Bus, mem_block::cpu::AddressSpace};

#[derive(Debug, Default)]
pub(crate) struct MockBus {
    pub(crate) mem: AddressSpace, // 65536 bytes (0x0000 to 0xFFFF)
}

impl Bus for MockBus {
    fn peek(&mut self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    fn mem_read(&mut self, addr: u16) -> u8 {
        self.mem[addr as usize] // Safe: addr is 0x0000..=0xFFFF, array is size 0x10000
    }

    fn mem_write(&mut self, addr: u16, data: u8) {
        self.mem[addr as usize] = data; // Safe: same reason
    }

    fn internal_cycle(&mut self) {}
}
