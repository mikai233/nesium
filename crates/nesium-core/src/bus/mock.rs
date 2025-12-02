use crate::{bus::Bus, mem_block::cpu::AddressSpace};

#[derive(Debug)]
pub(crate) struct MockBus {
    pub(crate) mem: AddressSpace, // 65536 bytes (0x0000 to 0xFFFF)
}

impl Default for MockBus {
    fn default() -> Self {
        Self {
            mem: AddressSpace::new(),
        } // Initialize all 65536 bytes to 0
    }
}

impl Bus for MockBus {
    fn read(&mut self, addr: u16) -> u8 {
        self.mem[addr as usize] // Safe: addr is 0x0000..=0xFFFF, array is size 0x10000
    }

    fn write(&mut self, addr: u16, data: u8) {
        self.mem[addr as usize] = data; // Safe: same reason
    }
}
