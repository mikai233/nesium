use crate::{
    apu::Apu,
    audio::NesSoundMixer,
    bus::{Bus, OpenBus},
    cartridge::Cartridge,
    controller::{Controller, SerialLogger},
    cpu::Cpu,
    mem_block::cpu as cpu_ram,
    memory::{apu as apu_mem, cpu as cpu_mem, ppu as ppu_mem},
    ppu::{Ppu, pattern_bus::PatternBus},
};

/// CPU-visible bus that bridges the core to RAM, the PPU, the APU, and the
/// cartridge mapper space. It borrows the hardware from the owning NES.
#[derive(Debug)]
pub struct CpuBus<'a> {
    ram: &'a mut cpu_ram::Ram,
    ppu: &'a mut Ppu,
    apu: &'a mut Apu,
    cartridge: Option<&'a mut Cartridge>,
    controllers: &'a mut [Controller; 2],
    serial_log: Option<&'a mut SerialLogger>,
    oam_dma_request: &'a mut Option<u8>,
    open_bus: &'a mut OpenBus,
    mixer: Option<&'a mut NesSoundMixer>,
    /// Approximate CPU bus cycle counter (increments per bus access).
    cycles: &'a mut u64,
    /// Master clock in master cycles (PPU = 4 mc, CPU = 12 mc).
    master_clock: &'a mut u64,
    /// CPU/PPU phase offset in master cycles.
    ppu_offset: u8,
    /// Master clock half-cycle lengths (start/end) in master cycles.
    clock_start_count: u8,
    clock_end_count: u8,
    /// Pending DMC stall cycles surfaced by the APU tick.
    pending_dmc_stall: Option<(u8, Option<u16>)>,
}

impl<'a> CpuBus<'a> {
    /// Creates a new bus by borrowing the attached hardware.
    pub(crate) fn new(
        ram: &'a mut cpu_ram::Ram,
        ppu: &'a mut Ppu,
        apu: &'a mut Apu,
        cartridge: Option<&'a mut Cartridge>,
        controllers: &'a mut [Controller; 2],
        serial_log: Option<&'a mut SerialLogger>,
        oam_dma_request: &'a mut Option<u8>,
        open_bus: &'a mut OpenBus,
        mixer: Option<&'a mut NesSoundMixer>,
        cycles: &'a mut u64,
        master_clock: &'a mut u64,
        ppu_offset: u8,
        clock_start_count: u8,
        clock_end_count: u8,
    ) -> Self {
        Self {
            ram,
            ppu,
            apu,
            cartridge,
            controllers,
            serial_log,
            oam_dma_request,
            open_bus,
            mixer,
            cycles,
            master_clock,
            ppu_offset,
            clock_start_count,
            clock_end_count,
            pending_dmc_stall: None,
        }
    }

    /// Returns `true` when a cartridge is loaded.
    pub fn has_cartridge(&self) -> bool {
        self.cartridge.is_some()
    }

    /// Returns the cartridge currently inserted on the bus, when present.
    pub fn cartridge(&self) -> Option<&Cartridge> {
        self.cartridge.as_deref()
    }

    /// Current CPU cycle counter (increments per bus access).
    pub fn cpu_cycles(&self) -> u64 {
        *self.cycles
    }

    /// CPU-visible read used by the DMC sample fetch path.
    ///
    /// Mesen2 models DMC DMA as a series of bus reads that can stall the CPU
    /// and interact with mappers. Nesium does not yet emulate those stalls, but
    /// for audio parity we still need to feed real PRG data into the DMC.
    /// This helper mirrors the cartridge space mapping without mutating any
    /// CPU-visible timing state.
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

    /// Immutable access to the PPU for visualization-heavy systems.
    pub fn ppu(&self) -> &Ppu {
        self.ppu
    }

    /// Mutable access to the PPU for DMA or rendering control.
    pub fn ppu_mut(&mut self) -> &mut Ppu {
        self.ppu
    }

    /// Immutable access to the audio subsystem.
    pub fn apu(&self) -> &Apu {
        self.apu
    }

    /// Mutable access to the audio subsystem.
    pub fn apu_mut(&mut self) -> &mut Apu {
        self.apu
    }

    /// Tick the PPU once, wiring CHR accesses through the currently inserted cartridge.
    pub fn clock_ppu(&mut self) {
        let mut pattern = PatternBus::new(self.cartridge.as_deref_mut(), *self.cycles);
        self.ppu.clock(&mut pattern);
    }

    /// Advances master clock and runs the PPU to catch up.
    fn bump_master_clock(&mut self, delta: u8) {
        *self.master_clock = self.master_clock.wrapping_add(delta as u64);
        // crate::ppu::CPU_MASTER.set(*self.master_clock);
        // crate::ppu::CPU_CYCLE.set(*self.cycles);
        // crate::ppu::MEM0F.set(self.peek(0x000f));
        let mut pattern = PatternBus::new(self.cartridge.as_deref_mut(), *self.cycles);
        // Apply CPU/PPU phase offset before running the PPU.
        let ppu_target = self.master_clock.saturating_sub(self.ppu_offset as u64);
        self.ppu.run_until(ppu_target, &mut pattern);
    }

    fn begin_cycle(&mut self, for_read: bool) {
        let start_delta = if for_read {
            self.clock_start_count.saturating_sub(1)
        } else {
            self.clock_start_count.saturating_add(1)
        };
        *self.cycles = self.cycles.wrapping_add(1);
        self.bump_master_clock(start_delta);

        if let Some(cart) = self.cartridge.as_mut() {
            cart.cpu_clock(*self.cycles);
        }
        self.open_bus.step();

        // Run one APU CPU-cycle tick; stash any pending DMC DMA stall.
        let (stall_cycles, dma_addr) = match &mut self.mixer {
            Some(mixer) => self.apu.clock_with_mixer(mixer),
            None => self.apu.clock(),
        };
        self.pending_dmc_stall = if stall_cycles > 0 {
            Some((stall_cycles, dma_addr))
        } else {
            None
        };
    }

    fn end_cycle(&mut self, for_read: bool) {
        let end_delta = if for_read {
            self.clock_end_count.saturating_add(1)
        } else {
            self.clock_end_count.saturating_sub(1)
        };
        self.bump_master_clock(end_delta);
    }

    pub fn internal_cycle(&mut self) {
        self.begin_cycle(true);
        self.end_cycle(true);
    }

    /// Drains and returns any pending DMC DMA stall produced by the last APU tick.
    pub fn take_pending_dmc_stall(&mut self) -> Option<(u8, Option<u16>)> {
        self.pending_dmc_stall.take()
    }

    /// Returns a read-only view of CPU RAM.
    pub fn ram(&self) -> &[u8] {
        self.ram.as_slice()
    }

    /// Returns a mutable view of CPU RAM.
    pub fn ram_mut(&mut self) -> &mut [u8] {
        self.ram.as_mut_slice()
    }

    /// PPU-facing mapper read for pattern table space.
    pub fn ppu_pattern_read(&mut self, addr: u16) -> u8 {
        self.cartridge
            .as_ref()
            .and_then(|cart| cart.ppu_read(addr))
            .unwrap_or(0)
    }

    /// PPU-facing mapper write for pattern table space.
    pub fn ppu_pattern_write(&mut self, addr: u16, value: u8) {
        if let Some(cart) = self.cartridge.as_deref_mut() {
            cart.ppu_write(addr, value);
        }
    }

    fn read_internal_ram(&self, addr: u16) -> u8 {
        let idx = (addr & cpu_mem::INTERNAL_RAM_MASK) as usize;
        self.ram[idx]
    }

    fn write_internal_ram(&mut self, addr: u16, value: u8) {
        let idx = (addr & cpu_mem::INTERNAL_RAM_MASK) as usize;
        self.ram[idx] = value;
    }

    fn read_cartridge(&self, addr: u16) -> Option<u8> {
        self.cartridge.as_ref().and_then(|cart| cart.cpu_read(addr))
    }

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

    /// Clears any mapper IRQ sources.
    fn clear_cartridge_irq(&mut self) {
        if let Some(cart) = self.cartridge.as_deref_mut() {
            cart.clear_irq();
        }
    }

    fn log_serial_bit(&mut self, data: u8) {
        if let Some(log) = self.serial_log.as_deref_mut() {
            log.push_bit((data & 0x01) != 0);
        }
    }

    fn write_oam_dma(&mut self, page: u8) {
        *self.oam_dma_request = Some(page);
    }
}

impl Bus for CpuBus<'_> {
    fn nmi_line(&mut self) -> bool {
        // PPU NMI output is a level: VBLANK && CTRL.NMI_ENABLE.
        self.ppu.nmi_output()
    }

    fn ppu_read(&mut self, addr: u16) -> u8 {
        self.ppu_pattern_read(addr)
    }

    fn ppu_write(&mut self, addr: u16, value: u8) {
        self.ppu_pattern_write(addr, value);
    }

    fn peek(&mut self, _cpu: &mut Cpu, addr: u16) -> u8 {
        let mut driven = true;
        let value = match addr {
            cpu_mem::INTERNAL_RAM_START..=cpu_mem::INTERNAL_RAM_MIRROR_END => {
                self.read_internal_ram(addr)
            }
            cpu_mem::PPU_REGISTER_BASE..=cpu_mem::PPU_REGISTER_END => {
                let mut pattern = PatternBus::new(self.cartridge.as_deref_mut(), *self.cycles);
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
            cpu_mem::APU_STATUS => {
                driven = false;
                let internal = self.open_bus.internal_sample();
                let status = self.apu.cpu_read(addr);
                let value = status | (internal & 0x20);
                self.open_bus.set_internal_only(value);
                value
            }
            cpu_mem::CONTROLLER_PORT_1 => self.controllers[0].read(),
            cpu_mem::CONTROLLER_PORT_2 => self.controllers[1].read(),
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

        value
    }

    fn mem_read(&mut self, _cpu: &mut Cpu, addr: u16) -> u8 {
        self.begin_cycle(true);
        let mut driven = true;
        let value = match addr {
            cpu_mem::INTERNAL_RAM_START..=cpu_mem::INTERNAL_RAM_MIRROR_END => {
                self.read_internal_ram(addr)
            }
            cpu_mem::PPU_REGISTER_BASE..=cpu_mem::PPU_REGISTER_END => {
                let mut pattern = PatternBus::new(self.cartridge.as_deref_mut(), *self.cycles);
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
            cpu_mem::CONTROLLER_PORT_1 => self.controllers[0].read(),
            cpu_mem::CONTROLLER_PORT_2 => self.controllers[1].read(),
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

        self.end_cycle(true);
        value
    }

    fn mem_write(&mut self, _cpu: &mut Cpu, addr: u16, data: u8) {
        self.begin_cycle(false);
        self.open_bus.latch(data);

        match addr {
            cpu_mem::INTERNAL_RAM_START..=cpu_mem::INTERNAL_RAM_MIRROR_END => {
                self.write_internal_ram(addr, data)
            }
            cpu_mem::PPU_REGISTER_BASE..=cpu_mem::PPU_REGISTER_END => {
                let mut pattern = PatternBus::new(self.cartridge.as_deref_mut(), *self.cycles);
                self.ppu.cpu_write(addr, data, &mut pattern)
            }
            cpu_mem::APU_REGISTER_BASE..=cpu_mem::APU_REGISTER_END => {
                self.apu.cpu_write(addr, data, *self.cycles)
            }
            ppu_mem::OAM_DMA => self.write_oam_dma(data),
            cpu_mem::APU_STATUS => self.apu.cpu_write(addr, data, *self.cycles),
            apu_mem::FRAME_COUNTER => {
                // $4017 doubles as both controller port 2 and the APU frame counter.
                self.apu.cpu_write(addr, data, *self.cycles);
                self.log_serial_bit(data);
                for ctrl in self.controllers.iter_mut() {
                    ctrl.write_strobe(data);
                }
            }
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

        self.end_cycle(false);
    }

    fn internal_cycle(&mut self) {
        self.begin_cycle(true);
        self.end_cycle(true);
    }

    fn irq_pending(&mut self) -> bool {
        let apu_irq = self.apu.irq_pending();
        let cartridge_irq = self.cartridge_irq_pending();
        apu_irq || cartridge_irq
    }

    fn take_oam_dma_request(&mut self) -> Option<u8> {
        self.oam_dma_request.take()
    }

    fn clear_irq(&mut self) {
        self.apu.clear_irq();
        self.clear_cartridge_irq();
    }

    fn cycles(&self) -> u64 {
        *self.cycles
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cartridge::{
        header::{Header, Mirroring, RomFormat, TvSystem},
        mapper::mapper0::Mapper0,
    };
    use crate::cpu::Cpu;

    fn test_header(prg_rom_size: usize, prg_ram_size: usize) -> Header {
        Header {
            format: RomFormat::INes,
            mapper: 0,
            submapper: 0,
            mirroring: Mirroring::Horizontal,
            battery_backed_ram: false,
            trainer_present: false,
            prg_rom_size,
            chr_rom_size: 0,
            prg_ram_size,
            prg_nvram_size: 0,
            chr_ram_size: 0,
            chr_nvram_size: 0,
            vs_unisystem: false,
            playchoice_10: false,
            tv_system: TvSystem::Ntsc,
        }
    }

    fn cartridge_with_pattern(prg_rom_size: usize, prg_ram_size: usize) -> Cartridge {
        let header = test_header(prg_rom_size, prg_ram_size);
        let prg_rom = (0..prg_rom_size)
            .map(|value| (value & 0xFF) as u8)
            .collect::<Vec<_>>()
            .into();
        let chr_rom = vec![0; header.chr_rom_size].into();
        let mapper = Mapper0::new(header, prg_rom, chr_rom, None);
        Cartridge::new(header, Box::new(mapper))
    }

    #[test]
    fn mirrors_internal_ram() {
        let mut cpu = Cpu::new();
        let mut ppu = Ppu::default();
        let mut apu = Apu::new();
        let mut ram = cpu_ram::Ram::new();
        let mut controllers = [Controller::new(), Controller::new()];
        let mut oam_dma_request = None;
        let mut open_bus = OpenBus::new();
        let mut cpu_bus_cycle = 0;
        let mut master_clock = 0;
        let mut bus = CpuBus::new(
            &mut ram,
            &mut ppu,
            &mut apu,
            None,
            &mut controllers,
            None,
            &mut oam_dma_request,
            &mut open_bus,
            None,
            &mut cpu_bus_cycle,
            &mut master_clock,
            1,
            6,
            6,
        );
        bus.mem_write(&mut cpu, cpu_mem::INTERNAL_RAM_START + 0x0002, 0xDE);
        assert_eq!(
            bus.mem_read(&mut cpu, cpu_mem::INTERNAL_RAM_START + 0x0002),
            0xDE
        );
        assert_eq!(bus.mem_read(&mut cpu, 0x0802), 0xDE);
        assert_eq!(bus.mem_read(&mut cpu, 0x1002), 0xDE);
        assert_eq!(bus.mem_read(&mut cpu, 0x1802), 0xDE);
    }

    #[test]
    fn reads_from_prg_rom_with_mirroring() {
        let mut cpu = Cpu::new();
        let mut ppu = Ppu::default();
        let mut apu = Apu::new();
        let mut ram = cpu_ram::Ram::new();
        let mut cartridge = cartridge_with_pattern(0x4000, 0x2000);
        let mut controllers = [Controller::new(), Controller::new()];
        let mut oam_dma_request = None;
        let mut open_bus = OpenBus::new();
        let mut cpu_bus_cycle = 0;
        let mut master_clock = 0;
        let mut bus = CpuBus::new(
            &mut ram,
            &mut ppu,
            &mut apu,
            Some(&mut cartridge),
            &mut controllers,
            None,
            &mut oam_dma_request,
            &mut open_bus,
            None,
            &mut cpu_bus_cycle,
            &mut master_clock,
            1,
            6,
            6,
        );

        let first_bank = bus.mem_read(&mut cpu, cpu_mem::PRG_ROM_START);
        let mirrored_bank = bus.mem_read(&mut cpu, cpu_mem::PRG_ROM_START + 0x4000);
        assert_eq!(first_bank, mirrored_bank);
    }

    #[test]
    fn reads_and_writes_prg_ram() {
        let mut cpu = Cpu::new();
        let mut ppu = Ppu::default();
        let mut apu = Apu::new();
        let mut ram = cpu_ram::Ram::new();
        let mut cartridge = cartridge_with_pattern(0x4000, 0x2000);
        let mut controllers = [Controller::new(), Controller::new()];
        let mut oam_dma_request = None;
        let mut open_bus = OpenBus::new();
        let mut cpu_bus_cycle = 0;
        let mut master_clock = 0;
        let mut bus = CpuBus::new(
            &mut ram,
            &mut ppu,
            &mut apu,
            Some(&mut cartridge),
            &mut controllers,
            None,
            &mut oam_dma_request,
            &mut open_bus,
            None,
            &mut cpu_bus_cycle,
            &mut master_clock,
            1,
            6,
            6,
        );

        bus.mem_write(&mut cpu, cpu_mem::PRG_RAM_START, 0x42);
        assert_eq!(bus.mem_read(&mut cpu, cpu_mem::PRG_RAM_START), 0x42);
    }
}
