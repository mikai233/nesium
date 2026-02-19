use crate::{
    apu::Apu,
    audio::NesSoundMixer,
    bus::{BusDevices, BusDevicesMut, DmcDmaEvent, OpenBus, PendingDma},
    cartridge::Cartridge,
    context::Context,
    controller::{ControllerPorts, SerialLogger},
    cpu::Cpu,
    mem_block::cpu as cpu_ram,
    memory::{apu as apu_mem, cpu as cpu_mem, ppu as ppu_mem, ppu::Register as PpuRegister},
    ppu::{Ppu, pattern_bus::PpuBus},
};

/// CPU-visible bus that bridges the core to RAM, the PPU, the APU, and the
/// cartridge mapper space. It borrows the hardware from the owning NES.
#[derive(Debug)]
pub struct CpuBus<'a> {
    pub(crate) ram: &'a mut cpu_ram::Ram,
    pub(crate) ppu: &'a mut Ppu,
    pub(crate) apu: &'a mut Apu,
    pub(crate) cartridge: Option<&'a mut Cartridge>,
    pub(crate) controllers: &'a mut ControllerPorts,
    pub(crate) serial_log: Option<&'a mut SerialLogger>,
    pub(crate) open_bus: &'a mut OpenBus,
    pub(crate) mixer: Option<&'a mut NesSoundMixer>,
    /// Approximate CPU bus cycle counter (increments per bus access).
    pub(crate) cycles: &'a mut u64,
    /// Master clock in master cycles (PPU = 4 mc, CPU = 12 mc).
    pub(crate) master_clock: &'a mut u64,
    /// CPU/PPU phase offset in master cycles.
    pub(crate) ppu_offset: u8,
    /// Master clock half-cycle lengths (start/end) in master cycles.
    pub(crate) clock_start_count: u8,
    pub(crate) clock_end_count: u8,
    pub(crate) pending_dma: &'a mut PendingDma,
}

impl<'a> CpuBus<'a> {
    /// Returns `true` when a cartridge is loaded.
    #[inline]
    pub fn has_cartridge(&self) -> bool {
        self.cartridge.is_some()
    }

    /// Returns the cartridge currently inserted on the bus, when present.
    #[inline]
    pub fn cartridge(&self) -> Option<&Cartridge> {
        self.cartridge.as_deref()
    }

    /// Current CPU cycle counter (increments per bus access).
    #[inline]
    pub fn cpu_cycles(&self) -> u64 {
        *self.cycles
    }

    /// Current CPU master clock in master cycles (12 per CPU cycle).
    #[inline]
    pub fn master_clock(&self) -> u64 {
        *self.master_clock
    }

    /// CPU-visible read used by the DMC sample fetch path.
    ///
    /// Mesen2 models DMC DMA as a series of bus reads that can stall the CPU
    /// and interact with mappers. Nesium does not yet emulate those stalls, but
    /// for audio parity we still need to feed real PRG data into the DMC.
    /// This helper mirrors the cartridge space mapping without mutating any
    /// CPU-visible timing state.
    #[inline]
    pub fn dmc_read(&self, addr: u16) -> u8 {
        use crate::memory::cpu as cpu_mem;

        // On the NES, the DMC sample address range is always in CPU cartridge
        // space ($8000-$FFFF). For any other range, return open bus (0) for
        // now. If no cartridge is present, there is nothing valid to read.
        if addr < cpu_mem::CARTRIDGE_SPACE_BASE {
            return 0;
        }

        self.read_cartridge(addr).unwrap_or(0)
    }

    /// Advances master clock and runs the PPU to catch up.
    #[inline]
    pub(crate) fn bump_master_clock(&mut self, delta: u8, cpu: &mut Cpu, ctx: &mut Context) {
        *self.master_clock = self.master_clock.wrapping_add(delta as u64);
        let ppu_target = self.master_clock.saturating_sub(self.ppu_offset as u64);
        Ppu::run_until(self, ppu_target, cpu, ctx);
    }

    /// PPU-facing mapper read for pattern table space.
    #[inline]
    pub fn ppu_pattern_read(&mut self, addr: u16) -> u8 {
        self.cartridge
            .as_ref()
            .and_then(|cart| cart.ppu_read(addr))
            .unwrap_or(0)
    }

    /// PPU-facing mapper write for pattern table space.
    #[inline]
    pub fn ppu_pattern_write(&mut self, addr: u16, value: u8) {
        if let Some(cart) = self.cartridge.as_deref_mut() {
            cart.ppu_write(addr, value);
        }
    }

    #[inline]
    fn read_internal_ram(&self, addr: u16) -> u8 {
        let idx = (addr & cpu_mem::INTERNAL_RAM_MASK) as usize;
        self.ram[idx]
    }

    #[inline]
    fn write_internal_ram(&mut self, addr: u16, value: u8) {
        let idx = (addr & cpu_mem::INTERNAL_RAM_MASK) as usize;
        self.ram[idx] = value;
    }

    #[inline]
    fn read_cartridge(&self, addr: u16) -> Option<u8> {
        self.cartridge.as_ref().and_then(|cart| cart.cpu_read(addr))
    }

    #[inline]
    fn write_cartridge(&mut self, addr: u16, value: u8) {
        if let Some(cart) = self.cartridge.as_deref_mut() {
            cart.cpu_write(addr, value, *self.cycles);
        }
    }

    /// Returns `true` when the inserted cartridge asserts IRQ.
    fn cartridge_irq_pending(&self) -> bool {
        self.cartridge
            .as_ref()
            .map(|cart| cart.irq_pending())
            .unwrap_or(false)
    }

    fn log_serial_bit(&mut self, data: u8) {
        if let Some(log) = self.serial_log.as_deref_mut() {
            log.push_bit((data & 0x01) != 0);
        }
    }

    #[inline]
    fn write_oam_dma(&mut self, page: u8) {
        self.pending_dma.oam_page = Some(page);
    }

    #[inline]
    fn trace_nmi_bus_event(&self, ev: &str, addr: u16, value: u8) {
        let ppu = &*self.ppu;
        crate::nmi_trace::log_line(&format!(
            "NMITRACE|src=nesium|ev={}|cpu_cycle={}|cpu_master={}|frame={}|scanline={}|dot={}|addr={:04X}|value={:02X}|vblank={}|nmi_enabled={}|nmi_level={}|prevent_vblank={}",
            ev,
            *self.cycles,
            *self.master_clock,
            ppu.frame,
            ppu.scanline,
            ppu.cycle,
            addr,
            value,
            crate::nmi_trace::flag(
                ppu.registers
                    .status
                    .contains(crate::ppu::Status::VERTICAL_BLANK),
            ),
            crate::nmi_trace::flag(ppu.registers.control.nmi_enabled()),
            crate::nmi_trace::flag(ppu.nmi_level),
            crate::nmi_trace::flag(ppu.prevent_vblank_flag)
        ));
    }

    #[inline]
    fn controller_open_bus_mask(_port: usize) -> u8 {
        // Match Mesen's default NES-001 open-bus mask for standard pads.
        0xE0
    }

    #[inline]
    fn read_controller_port(&mut self, port: usize) -> u8 {
        let mask = Self::controller_open_bus_mask(port);
        let open_bus = self.open_bus.sample() & mask;
        let data = self.controllers[port].read() & !mask;
        open_bus | data
    }

    pub fn devices(&self) -> BusDevices<'_> {
        BusDevices {
            ram: &*self.ram,
            ppu: &*self.ppu,
            apu: &*self.apu,
            cartridge: self.cartridge.as_deref(),
            controllers: &*self.controllers,
        }
    }

    pub fn devices_mut(&mut self) -> BusDevicesMut<'_> {
        BusDevicesMut {
            ram: &mut *self.ram,
            ppu: &mut *self.ppu,
            apu: &mut *self.apu,
            cartridge: self.cartridge.as_deref_mut(),
            controllers: &mut *self.controllers,
        }
    }

    #[inline]
    pub fn nmi_level(&self) -> bool {
        self.ppu.nmi_level
    }

    #[inline]
    pub fn ppu_read(&mut self, addr: u16) -> u8 {
        self.ppu_pattern_read(addr)
    }

    #[inline]
    pub fn ppu_write(&mut self, addr: u16, value: u8) {
        self.ppu_pattern_write(addr, value);
    }

    pub fn peek(&mut self, addr: u16, _cpu: &mut Cpu, _context: &mut Context) -> u8 {
        match addr {
            cpu_mem::INTERNAL_RAM_START..=cpu_mem::INTERNAL_RAM_MIRROR_END => {
                self.read_internal_ram(addr)
            }
            cpu_mem::PPU_REGISTER_BASE..=cpu_mem::PPU_REGISTER_END => {
                let reg = PpuRegister::from_cpu_addr(addr);
                let mut pattern = PpuBus::new(self.cartridge.as_deref_mut(), *self.cycles);
                let value = self.ppu.cpu_read(addr, &mut pattern);
                if matches!(reg, PpuRegister::Status) {
                    self.trace_nmi_bus_event("read", PpuRegister::Status.addr(), value);
                }
                value
            }
            cpu_mem::APU_REGISTER_BASE..=cpu_mem::APU_REGISTER_END => OpenBus::peek(addr),
            ppu_mem::OAM_DMA => OpenBus::peek(addr),
            cpu_mem::APU_STATUS => {
                let internal = self.open_bus.internal_sample();
                let status = self.apu.cpu_read(addr);
                status | (internal & 0x20)
            }
            cpu_mem::CONTROLLER_PORT_1 => self.read_controller_port(0),
            cpu_mem::CONTROLLER_PORT_2 => self.read_controller_port(1),
            cpu_mem::TEST_MODE_BASE..=cpu_mem::TEST_MODE_END => OpenBus::peek(addr),
            cpu_mem::CARTRIDGE_SPACE_BASE..=cpu_mem::CPU_ADDR_END => {
                match self.read_cartridge(addr) {
                    Some(value) => value,
                    None => OpenBus::peek(addr),
                }
            }
        }
    }

    pub fn read(&mut self, addr: u16, _cpu: &mut Cpu, _ctx: &mut Context) -> u8 {
        let mut driven = true;
        let value = match addr {
            cpu_mem::INTERNAL_RAM_START..=cpu_mem::INTERNAL_RAM_MIRROR_END => {
                self.read_internal_ram(addr)
            }
            cpu_mem::PPU_REGISTER_BASE..=cpu_mem::PPU_REGISTER_END => {
                let mut pattern = PpuBus::new(self.cartridge.as_deref_mut(), *self.cycles);
                self.ppu.cpu_read(addr, &mut pattern)
            }
            cpu_mem::APU_REGISTER_BASE..=cpu_mem::APU_REGISTER_END => {
                driven = false;
                self.open_bus.sample()
            }
            ppu_mem::OAM_DMA => {
                driven = false;
                self.open_bus.sample()
            }
            // $4015 (APU status) updates only the CPU's *internal* data bus on
            // Mesen2; the external bus latch is left unchanged. We mirror that
            // by reading the current internal bus bit-5, mixing it into the
            // returned status, and then updating only the internal latch.
            cpu_mem::APU_STATUS => {
                driven = false;
                let internal = self.open_bus.internal_sample();
                let status = self.apu.cpu_read(addr);
                let value = status | (internal & 0x20);
                self.open_bus.set_internal_only(value);
                value
            }
            cpu_mem::CONTROLLER_PORT_1 => self.read_controller_port(0),
            cpu_mem::CONTROLLER_PORT_2 => self.read_controller_port(1),
            cpu_mem::TEST_MODE_BASE..=cpu_mem::TEST_MODE_END => {
                driven = false;
                self.open_bus.sample()
            }
            cpu_mem::CARTRIDGE_SPACE_BASE..=cpu_mem::CPU_ADDR_END => {
                match self.read_cartridge(addr) {
                    Some(value) => value,
                    None => {
                        driven = false;
                        self.open_bus.sample()
                    }
                }
            }
        };

        if driven {
            self.open_bus.latch(value);
        }
        self.apu.trace_mem_read(addr, value);
        value
    }

    pub fn write(&mut self, addr: u16, data: u8, _cpu: &mut Cpu, _ctx: &mut Context) {
        self.open_bus.latch(data);
        match addr {
            cpu_mem::INTERNAL_RAM_START..=cpu_mem::INTERNAL_RAM_MIRROR_END => {
                self.write_internal_ram(addr, data)
            }
            cpu_mem::PPU_REGISTER_BASE..=cpu_mem::PPU_REGISTER_END => {
                let reg = PpuRegister::from_cpu_addr(addr);
                let mut pattern = PpuBus::new(self.cartridge.as_deref_mut(), *self.cycles);
                self.ppu.cpu_write(addr, data, &mut pattern);
                if matches!(
                    reg,
                    PpuRegister::Control
                        | PpuRegister::Mask
                        | PpuRegister::OamAddr
                        | PpuRegister::OamData
                        | PpuRegister::Scroll
                        | PpuRegister::Addr
                        | PpuRegister::Data
                ) {
                    self.trace_nmi_bus_event("write", reg.addr(), data);
                }
            }
            cpu_mem::APU_REGISTER_BASE..=cpu_mem::APU_REGISTER_END => {
                self.apu.cpu_write(addr, data, *self.cycles)
            }
            ppu_mem::OAM_DMA => {
                self.write_oam_dma(data);
                self.trace_nmi_bus_event("write", ppu_mem::OAM_DMA, data);
            }
            cpu_mem::APU_STATUS => self.apu.cpu_write(addr, data, *self.cycles),
            apu_mem::FRAME_COUNTER => self.apu.cpu_write(addr, data, *self.cycles),
            cpu_mem::CONTROLLER_PORT_1 => {
                self.log_serial_bit(data);
                for ctrl in self.controllers.iter_mut() {
                    ctrl.write_strobe(data);
                }
            }
            cpu_mem::TEST_MODE_BASE..=cpu_mem::TEST_MODE_END => {}
            cpu_mem::CARTRIDGE_SPACE_BASE..=cpu_mem::CPU_ADDR_END => {
                self.write_cartridge(addr, data)
            }
        }
    }

    #[inline]
    pub fn mem_read(&mut self, addr: u16, cpu: &mut Cpu, ctx: &mut Context) -> u8 {
        cpu.handle_dma(addr, self, ctx);
        cpu.begin_cycle(true, self, ctx);
        let value = self.read(addr, cpu, ctx);
        cpu.end_cycle(true, self, ctx);
        value
    }

    #[inline]
    pub fn mem_write(&mut self, addr: u16, data: u8, cpu: &mut Cpu, ctx: &mut Context) {
        cpu.begin_cycle(false, self, ctx);
        self.write(addr, data, cpu, ctx);
        cpu.end_cycle(false, self, ctx);
    }

    #[inline]
    pub fn dma_read(&mut self, addr: u16, cpu: &mut Cpu, ctx: &mut Context) -> u8 {
        cpu.begin_cycle(true, self, ctx);
        let v = self.read(addr, cpu, ctx);
        cpu.end_cycle(true, self, ctx);
        v
    }

    #[inline]
    pub fn dma_write(&mut self, addr: u16, data: u8, cpu: &mut Cpu, ctx: &mut Context) {
        cpu.begin_cycle(true, self, ctx);
        self.write(addr, data, cpu, ctx);
        cpu.end_cycle(true, self, ctx);
    }

    #[inline]
    pub fn irq_level(&mut self) -> bool {
        let apu_irq = self.apu.irq_pending();
        let cartridge_irq = self.cartridge_irq_pending();
        apu_irq || cartridge_irq
    }

    #[inline]
    pub fn cycles(&self) -> u64 {
        *self.cycles
    }

    /// Drain a pending OAM DMA request (page number) if present.
    #[inline]
    pub fn take_oam_dma(&mut self) -> Option<u8> {
        self.pending_dma.oam_page.take()
    }

    /// Drain a pending DMC DMA event (request/abort) if present.
    #[inline]
    pub fn take_dmc_dma_event(&mut self) -> Option<DmcDmaEvent> {
        self.pending_dma.dmc.take()
    }

    /// Queue a DMC DMA request.
    #[inline]
    pub(crate) fn request_dmc_dma(&mut self, addr: u16) {
        self.pending_dma.dmc = Some(DmcDmaEvent::Request { addr });
    }

    /// Queue a DMC DMA abort.
    #[inline]
    pub(crate) fn abort_dmc_dma(&mut self) {
        self.pending_dma.dmc = Some(DmcDmaEvent::Abort);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cartridge::{header::Header, mapper::mapper0::Mapper0},
        context::Context,
        cpu::Cpu,
    };

    fn test_header(prg_rom_size: usize, prg_ram_size: usize) -> Header {
        let prg_rom_units = (prg_rom_size / (16 * 1024)) as u8;
        let prg_ram_units = if prg_ram_size == 0 {
            0
        } else {
            (prg_ram_size / (8 * 1024)) as u8
        };

        let header_bytes = [
            b'N',
            b'E',
            b'S',
            0x1A,
            prg_rom_units,
            0,
            0,
            0,
            prg_ram_units,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ];

        Header::parse(&header_bytes).expect("header parses")
    }

    fn cartridge_with_pattern(prg_rom_size: usize, prg_ram_size: usize) -> Cartridge {
        let header = test_header(prg_rom_size, prg_ram_size);
        let prg_rom = (0..prg_rom_size)
            .map(|value| (value & 0xFF) as u8)
            .collect::<Vec<_>>()
            .into();
        let chr_rom = vec![0; header.chr_rom_size()].into();
        let mapper = Mapper0::new(header, prg_rom, chr_rom, None);
        Cartridge::new(header, Box::new(mapper))
    }

    #[test]
    fn mirrors_internal_ram() {
        let mut cpu = Cpu::new();
        let mut ppu = Ppu::default();
        let mut apu = Apu::new();
        let mut ram = cpu_ram::Ram::new();
        let mut controllers = ControllerPorts::new();
        let mut pending_dma = PendingDma::default();
        let mut open_bus = OpenBus::new();
        let mut cpu_bus_cycle = 0;
        let mut master_clock = 0;
        let mut bus = CpuBus {
            ram: &mut ram,
            ppu: &mut ppu,
            apu: &mut apu,
            cartridge: None,
            controllers: &mut controllers,
            serial_log: None,
            open_bus: &mut open_bus,
            mixer: None,
            cycles: &mut cpu_bus_cycle,
            master_clock: &mut master_clock,
            ppu_offset: 1,
            clock_start_count: 6,
            clock_end_count: 6,
            pending_dma: &mut pending_dma,
        };
        bus.mem_write(
            cpu_mem::INTERNAL_RAM_START + 0x0002,
            0xDE,
            &mut cpu,
            &mut Context::None,
        );
        assert_eq!(
            bus.mem_read(
                cpu_mem::INTERNAL_RAM_START + 0x0002,
                &mut cpu,
                &mut Context::None
            ),
            0xDE
        );
        assert_eq!(bus.mem_read(0x0802, &mut cpu, &mut Context::None), 0xDE);
        assert_eq!(bus.mem_read(0x1002, &mut cpu, &mut Context::None), 0xDE);
        assert_eq!(bus.mem_read(0x1802, &mut cpu, &mut Context::None), 0xDE);
    }

    #[test]
    fn reads_from_prg_rom_with_mirroring() {
        let mut cpu = Cpu::new();
        let mut ppu = Ppu::default();
        let mut apu = Apu::new();
        let mut ram = cpu_ram::Ram::new();
        let mut cartridge = cartridge_with_pattern(0x4000, 0x2000);
        let mut controllers = ControllerPorts::new();
        let mut pending_dma = PendingDma::default();
        let mut open_bus = OpenBus::new();
        let mut cpu_bus_cycle = 0;
        let mut master_clock = 0;
        let mut bus = CpuBus {
            ram: &mut ram,
            ppu: &mut ppu,
            apu: &mut apu,
            cartridge: Some(&mut cartridge),
            controllers: &mut controllers,
            serial_log: None,
            open_bus: &mut open_bus,
            mixer: None,
            cycles: &mut cpu_bus_cycle,
            master_clock: &mut master_clock,
            ppu_offset: 1,
            clock_start_count: 6,
            clock_end_count: 6,
            pending_dma: &mut pending_dma,
        };
        let first_bank = bus.mem_read(cpu_mem::PRG_ROM_START, &mut cpu, &mut Context::None);
        let mirrored_bank = bus.mem_read(
            cpu_mem::PRG_ROM_START + 0x4000,
            &mut cpu,
            &mut Context::None,
        );
        assert_eq!(first_bank, mirrored_bank);
    }

    #[test]
    fn reads_and_writes_prg_ram() {
        let mut cpu = Cpu::new();
        let mut ppu = Ppu::default();
        let mut apu = Apu::new();
        let mut ram = cpu_ram::Ram::new();
        let mut cartridge = cartridge_with_pattern(0x4000, 0x2000);
        let mut controllers = ControllerPorts::new();
        let mut pending_dma = PendingDma::default();
        let mut open_bus = OpenBus::new();
        let mut cpu_bus_cycle = 0;
        let mut master_clock = 0;
        let mut bus = CpuBus {
            ram: &mut ram,
            ppu: &mut ppu,
            apu: &mut apu,
            cartridge: Some(&mut cartridge),
            controllers: &mut controllers,
            serial_log: None,
            open_bus: &mut open_bus,
            mixer: None,
            cycles: &mut cpu_bus_cycle,
            master_clock: &mut master_clock,
            ppu_offset: 1,
            clock_start_count: 6,
            clock_end_count: 6,
            pending_dma: &mut pending_dma,
        };
        bus.mem_write(cpu_mem::PRG_RAM_START, 0x42, &mut cpu, &mut Context::None);
        assert_eq!(
            bus.mem_read(cpu_mem::PRG_RAM_START, &mut cpu, &mut Context::None),
            0x42
        );
    }
}
