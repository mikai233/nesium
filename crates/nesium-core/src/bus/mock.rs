use crate::{bus::Bus, cpu::Cpu, mem_block::cpu::AddressSpace};

#[derive(Debug, Default)]
pub(crate) struct MockBus {
    pub(crate) mem: AddressSpace, // 65536 bytes (0x0000 to 0xFFFF)
}

impl MockBus {
    /// Direct memory read without needing a CPU reference (test helpers).
    pub(crate) fn mem_read(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    /// Direct memory write without needing a CPU reference (test helpers).
    pub(crate) fn mem_write(&mut self, addr: u16, data: u8) {
        self.mem[addr as usize] = data;
    }
}

impl Bus for MockBus {
    fn peek(&mut self, _cpu: &mut Cpu, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    fn mem_read(&mut self, _cpu: &mut Cpu, addr: u16) -> u8 {
        self.mem[addr as usize] // Safe: addr is 0x0000..=0xFFFF, array is size 0x10000
    }

    fn mem_write(&mut self, _cpu: &mut Cpu, addr: u16, data: u8) {
        self.mem[addr as usize] = data; // Safe: same reason
    }

    fn internal_cycle(&mut self) {}

    fn cycles(&self) -> u64 {
        0
    }
}
