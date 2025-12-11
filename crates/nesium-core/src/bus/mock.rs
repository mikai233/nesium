use crate::{
    bus::{Bus, BusDevices, BusDevicesMut},
    cpu::Cpu,
    mem_block::cpu::AddressSpace,
};

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
    fn devices(&self) -> BusDevices<'_> {
        panic!("MockBus does not expose attached devices");
    }

    fn devices_mut(&mut self) -> BusDevicesMut<'_> {
        panic!("MockBus does not expose attached devices");
    }

    fn peek(&mut self, _: &mut Cpu, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    fn mem_read(&mut self, _: &mut Cpu, addr: u16) -> u8 {
        self.mem[addr as usize] // Safe: addr is 0x0000..=0xFFFF, array is size 0x10000
    }

    fn mem_write(&mut self, _: &mut Cpu, addr: u16, data: u8) {
        self.mem[addr as usize] = data; // Safe: same reason
    }

    fn internal_cycle(&mut self, _: &mut Cpu) {}

    fn cycles(&self) -> u64 {
        0
    }

    fn take_oam_dma_request(&mut self) -> Option<u8> {
        None
    }

    fn ppu_read(&mut self, addr: u16) -> u8 {
        let _ = addr;
        0
    }

    fn ppu_write(&mut self, addr: u16, value: u8) {
        let _ = (addr, value);
    }

    fn nmi_line(&mut self) -> bool {
        false
    }

    fn irq_pending(&mut self) -> bool {
        false
    }

    fn clear_irq(&mut self) {}
}
