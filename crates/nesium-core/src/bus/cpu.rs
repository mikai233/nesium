use crate::{
    apu::Apu,
    bus::Bus,
    cartridge::Cartridge,
    controller::{Controller, SerialLogger},
    memory::{
        apu as apu_mem,
        cpu as cpu_mem,
        ppu::{self as ppu_mem},
    },
        ppu::{PatternBus, Ppu},
        ram::cpu as cpu_ram,
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
    oam_dma_page: Option<u8>,
    serial_log: Option<&'a mut SerialLogger>,
}

impl<'a> CpuBus<'a> {
    /// Creates a new bus by borrowing the attached hardware.
    pub fn new(
        ram: &'a mut cpu_ram::Ram,
        ppu: &'a mut Ppu,
        apu: &'a mut Apu,
        cartridge: Option<&'a mut Cartridge>,
        controllers: &'a mut [Controller; 2],
        serial_log: Option<&'a mut SerialLogger>,
    ) -> Self {
        Self {
            ram,
            ppu,
            apu,
            cartridge,
            controllers,
            oam_dma_page: None,
            serial_log,
        }
    }

    /// Returns `true` when a cartridge is loaded.
    pub fn has_cartridge(&self) -> bool {
        self.cartridge.is_some()
    }

    /// Returns the cartridge currently inserted on the bus, when present.
    pub fn cartridge(&self) -> Option<&Cartridge> {
        self.cartridge.as_ref().map(|c| &**c)
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
        let mut pattern = PatternBus::new(self.cartridge.as_deref_mut());
        self.ppu.clock(&mut pattern);
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
            .map(|cart| cart.ppu_read(addr))
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

    fn read_cartridge(&self, addr: u16) -> u8 {
        self.cartridge
            .as_ref()
            .map(|cart| cart.cpu_read(addr))
            .unwrap_or(0)
    }

    fn write_cartridge(&mut self, addr: u16, value: u8) {
        if let Some(cart) = self.cartridge.as_deref_mut() {
            cart.cpu_write(addr, value);
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

    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            cpu_mem::INTERNAL_RAM_START..=cpu_mem::INTERNAL_RAM_MIRROR_END => {
                self.read_internal_ram(addr)
            }
            cpu_mem::PPU_REGISTER_BASE..=cpu_mem::PPU_REGISTER_END => {
                let mut pattern = PatternBus::new(self.cartridge.as_deref_mut());
                self.ppu.cpu_read(addr, &mut pattern)
            }
            cpu_mem::APU_REGISTER_BASE..=cpu_mem::APU_REGISTER_END => self.apu.cpu_read(addr),
            ppu_mem::OAM_DMA => 0,
            cpu_mem::APU_STATUS => self.apu.cpu_read(addr),
            cpu_mem::CONTROLLER_PORT_1 => self.controllers[0].read(),
            cpu_mem::CONTROLLER_PORT_2 => self.controllers[1].read(),
            cpu_mem::TEST_MODE_BASE..=cpu_mem::TEST_MODE_END => 0,
            cpu_mem::CARTRIDGE_SPACE_BASE..=cpu_mem::CPU_ADDR_END => self.read_cartridge(addr),
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            cpu_mem::INTERNAL_RAM_START..=cpu_mem::INTERNAL_RAM_MIRROR_END => {
                self.write_internal_ram(addr, data)
            }
            cpu_mem::PPU_REGISTER_BASE..=cpu_mem::PPU_REGISTER_END => {
                let mut pattern = PatternBus::new(self.cartridge.as_deref_mut());
                self.ppu.cpu_write(addr, data, &mut pattern)
            }
            cpu_mem::APU_REGISTER_BASE..=cpu_mem::APU_REGISTER_END => {
                self.apu.cpu_write(addr, data)
            }
            ppu_mem::OAM_DMA => self.oam_dma_page = Some(data),
            cpu_mem::APU_STATUS => self.apu.cpu_write(addr, data),
            apu_mem::FRAME_COUNTER => {
                // $4017 doubles as both controller port 2 and the APU frame counter.
                self.apu.cpu_write(addr, data);
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
    }

    fn irq_pending(&mut self) -> bool {
        let apu_irq = self.apu.irq_pending();
        let cartridge_irq = self.cartridge_irq_pending();
        apu_irq || cartridge_irq
    }

    fn take_oam_dma_request(&mut self) -> Option<u8> {
        self.oam_dma_page.take()
    }

    fn clear_irq(&mut self) {
        self.apu.clear_irq();
        self.clear_cartridge_irq();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cartridge::{
        header::{Header, Mirroring, RomFormat, TvSystem},
        mapper::mapper0::Mapper0,
    };

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
            .into_boxed_slice();
        let chr_rom = vec![0; header.chr_rom_size].into_boxed_slice();
        let mapper = Mapper0::new(header, prg_rom, chr_rom);
        Cartridge::new(header, Box::new(mapper))
    }

    #[test]
    fn mirrors_internal_ram() {
        let mut ppu = Ppu::new();
        let mut apu = Apu::new();
        let mut ram = cpu_ram::Ram::new();
        let mut controllers = [Controller::new(), Controller::new()];
        let mut bus =
            CpuBus::new(&mut ram, &mut ppu, &mut apu, None, &mut controllers, None);
        bus.write(cpu_mem::INTERNAL_RAM_START + 0x0002, 0xDE);
        assert_eq!(bus.read(cpu_mem::INTERNAL_RAM_START + 0x0002), 0xDE);
        assert_eq!(bus.read(0x0802), 0xDE);
        assert_eq!(bus.read(0x1002), 0xDE);
        assert_eq!(bus.read(0x1802), 0xDE);
    }

    #[test]
    fn reads_from_prg_rom_with_mirroring() {
        let mut ppu = Ppu::new();
        let mut apu = Apu::new();
        let mut ram = cpu_ram::Ram::new();
        let mut cartridge = cartridge_with_pattern(0x4000, 0x2000);
        let mut controllers = [Controller::new(), Controller::new()];
        let mut bus = CpuBus::new(
            &mut ram,
            &mut ppu,
            &mut apu,
            Some(&mut cartridge),
            &mut controllers,
            None,
        );

        let first_bank = bus.read(cpu_mem::PRG_ROM_START);
        let mirrored_bank = bus.read(cpu_mem::PRG_ROM_START + 0x4000);
        assert_eq!(first_bank, mirrored_bank);
    }

    #[test]
    fn reads_and_writes_prg_ram() {
        let mut ppu = Ppu::new();
        let mut apu = Apu::new();
        let mut ram = cpu_ram::Ram::new();
        let mut cartridge = cartridge_with_pattern(0x4000, 0x2000);
        let mut controllers = [Controller::new(), Controller::new()];
        let mut bus = CpuBus::new(
            &mut ram,
            &mut ppu,
            &mut apu,
            Some(&mut cartridge),
            &mut controllers,
            None,
        );

        bus.write(cpu_mem::PRG_RAM_START, 0x42);
        assert_eq!(bus.read(cpu_mem::PRG_RAM_START), 0x42);
    }
}
