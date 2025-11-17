use crate::{
    apu::Apu,
    bus::Bus,
    cartridge::Cartridge,
    memory::{
        cpu as cpu_mem,
        ppu::{self as ppu_mem, Register as PpuRegister},
    },
    ppu::Ppu,
    ram::cpu as cpu_ram,
};

const DMA_TRANSFER_BYTES: usize = 256;

/// Shared hardware context that the CPU bus needs to communicate with.
///
/// The context owns the Picture Processing Unit, Audio Processing Unit, and the
/// currently inserted cartridge (if any). `CpuBus` only keeps mutable
/// references to these components so that ownership can stay at the system
/// level without resorting to reference counting.
#[derive(Debug)]
pub struct Context {
    ppu: Ppu,
    apu: Apu,
    cartridge: Option<CartridgeSlot>,
}

impl Context {
    /// Creates a new context with powered-on peripherals and no cartridge
    /// inserted.
    pub fn new() -> Self {
        Self {
            ppu: Ppu::new(),
            apu: Apu::new(),
            cartridge: None,
        }
    }

    /// Resets the PPU and APU back to their power-on state.
    pub fn reset(&mut self) {
        self.ppu.reset();
        self.apu.reset();
    }

    /// Inserts a cartridge into the context, replacing any previous one.
    pub fn insert_cartridge(&mut self, cartridge: Cartridge) {
        self.cartridge = Some(CartridgeSlot::new(cartridge));
    }

    /// Returns `true` when a cartridge is loaded.
    pub fn has_cartridge(&self) -> bool {
        self.cartridge.is_some()
    }

    /// Provides read-only access to the currently inserted cartridge, if any.
    pub fn cartridge(&self) -> Option<&Cartridge> {
        self.cartridge.as_ref().map(|slot| slot.cartridge())
    }
}

impl Default for Context {
    fn default() -> Self {
        Self::new()
    }
}

/// CPU-visible bus that bridges the core to RAM, the PPU, the APU, and the
/// cartridge mapper space.
#[derive(Debug)]
pub struct CpuBus<'a> {
    ram: cpu_ram::Ram,
    context: &'a mut Context,
}

impl<'a> CpuBus<'a> {
    /// Creates a new bus backed by the provided context.
    pub fn new(context: &'a mut Context) -> Self {
        Self {
            ram: cpu_ram::Ram::new(),
            context,
        }
    }

    /// Resets the CPU-visible RAM along with the attached peripherals.
    pub fn reset(&mut self) {
        self.ram.fill(0);
        self.context.reset();
    }

    /// Convenience helper that forwards cartridge insertion to the context.
    pub fn insert_cartridge(&mut self, cartridge: Cartridge) {
        self.context.insert_cartridge(cartridge);
    }

    /// Returns `true` when the context currently holds a cartridge.
    pub fn has_cartridge(&self) -> bool {
        self.context.has_cartridge()
    }

    /// Returns the cartridge currently inserted in the context, when present.
    pub fn cartridge(&self) -> Option<&Cartridge> {
        self.context.cartridge()
    }

    /// Immutable access to the PPU for visualization-heavy systems.
    pub fn ppu(&self) -> &Ppu {
        &self.context.ppu
    }

    /// Mutable access to the PPU for DMA or rendering control.
    pub fn ppu_mut(&mut self) -> &mut Ppu {
        &mut self.context.ppu
    }

    /// Immutable access to the audio subsystem.
    pub fn apu(&self) -> &Apu {
        &self.context.apu
    }

    /// Mutable access to the audio subsystem.
    pub fn apu_mut(&mut self) -> &mut Apu {
        &mut self.context.apu
    }

    /// Returns a read-only view of CPU RAM.
    pub fn ram(&self) -> &[u8] {
        self.ram.as_slice()
    }

    /// Returns a mutable view of CPU RAM.
    pub fn ram_mut(&mut self) -> &mut [u8] {
        self.ram.as_mut_slice()
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
        self.context
            .cartridge
            .as_ref()
            .map(|slot| slot.read(addr))
            .unwrap_or(0)
    }

    fn write_cartridge(&mut self, addr: u16, value: u8) {
        if let Some(slot) = self.context.cartridge.as_mut() {
            slot.write(addr, value);
        }
    }

    fn write_oam_dma(&mut self, page: u8) {
        let base = (page as u16) << 8;
        for offset in 0..DMA_TRANSFER_BYTES {
            let addr = base.wrapping_add(offset as u16);
            let value = self.read(addr);
            self.context
                .ppu
                .cpu_write(PpuRegister::OamData.addr(), value);
        }
    }
}

impl<'a> Bus for CpuBus<'a> {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            cpu_mem::INTERNAL_RAM_START..=cpu_mem::INTERNAL_RAM_MIRROR_END => {
                self.read_internal_ram(addr)
            }
            cpu_mem::PPU_REGISTER_BASE..=cpu_mem::PPU_REGISTER_END => {
                self.context.ppu.cpu_read(addr)
            }
            cpu_mem::APU_REGISTER_BASE..=cpu_mem::APU_REGISTER_END => {
                self.context.apu.cpu_read(addr)
            }
            ppu_mem::OAM_DMA => 0,
            cpu_mem::APU_STATUS => self.context.apu.cpu_read(addr),
            cpu_mem::CONTROLLER_PORT_1 | cpu_mem::CONTROLLER_PORT_2 => 0,
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
                self.context.ppu.cpu_write(addr, data)
            }
            cpu_mem::APU_REGISTER_BASE..=cpu_mem::APU_REGISTER_END => {
                self.context.apu.cpu_write(addr, data)
            }
            ppu_mem::OAM_DMA => self.write_oam_dma(data),
            cpu_mem::APU_STATUS => self.context.apu.cpu_write(addr, data),
            cpu_mem::CONTROLLER_PORT_1 | cpu_mem::CONTROLLER_PORT_2 => {}
            cpu_mem::TEST_MODE_BASE..=cpu_mem::TEST_MODE_END => {}
            cpu_mem::CARTRIDGE_SPACE_BASE..=cpu_mem::CPU_ADDR_END => {
                self.write_cartridge(addr, data)
            }
        }
    }
}

/// Wrapper around the inserted cartridge that also tracks CPU-visible PRG RAM.
#[derive(Debug, Clone)]
struct CartridgeSlot {
    cartridge: Cartridge,
    prg_ram: Vec<u8>,
}

impl CartridgeSlot {
    fn new(cartridge: Cartridge) -> Self {
        let prg_ram = if cartridge.header.prg_ram_size > 0 {
            vec![0; cartridge.header.prg_ram_size]
        } else {
            Vec::new()
        };

        Self { cartridge, prg_ram }
    }

    fn cartridge(&self) -> &Cartridge {
        &self.cartridge
    }

    fn read(&self, addr: u16) -> u8 {
        match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.read_prg_ram(addr),
            cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END => self.read_prg_rom(addr),
            _ => 0,
        }
    }

    fn write(&mut self, addr: u16, value: u8) {
        match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.write_prg_ram(addr, value),
            cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END => self.handle_mapper_write(addr, value),
            _ => {}
        }
    }

    fn read_prg_ram(&self, addr: u16) -> u8 {
        if self.prg_ram.is_empty() {
            return 0;
        }
        let idx = (addr - cpu_mem::PRG_RAM_START) as usize % self.prg_ram.len();
        self.prg_ram[idx]
    }

    fn write_prg_ram(&mut self, addr: u16, value: u8) {
        if self.prg_ram.is_empty() {
            return;
        }
        let idx = (addr - cpu_mem::PRG_RAM_START) as usize % self.prg_ram.len();
        self.prg_ram[idx] = value;
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        if self.cartridge.prg_rom.is_empty() {
            return 0;
        }
        let idx = (addr - cpu_mem::PRG_ROM_START) as usize % self.cartridge.prg_rom.len();
        self.cartridge.prg_rom[idx]
    }

    fn handle_mapper_write(&mut self, _addr: u16, _value: u8) {
        // Placeholder until concrete mappers are wired up.
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cartridge::header::{Header, Mirroring, RomFormat, TvSystem};

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
            .collect();
        Cartridge {
            header,
            trainer: None,
            prg_rom,
            chr_rom: vec![0; header.chr_rom_size],
        }
    }

    #[test]
    fn mirrors_internal_ram() {
        let mut context = Context::new();
        let mut bus = CpuBus::new(&mut context);
        bus.write(cpu_mem::INTERNAL_RAM_START + 0x0002, 0xDE);
        assert_eq!(bus.read(cpu_mem::INTERNAL_RAM_START + 0x0002), 0xDE);
        assert_eq!(bus.read(0x0802), 0xDE);
        assert_eq!(bus.read(0x1002), 0xDE);
        assert_eq!(bus.read(0x1802), 0xDE);
    }

    #[test]
    fn reads_from_prg_rom_with_mirroring() {
        let mut context = Context::new();
        let mut bus = CpuBus::new(&mut context);
        let cartridge = cartridge_with_pattern(0x4000, 0x2000);
        bus.insert_cartridge(cartridge);

        let first_bank = bus.read(cpu_mem::PRG_ROM_START);
        let mirrored_bank = bus.read(cpu_mem::PRG_ROM_START + 0x4000);
        assert_eq!(first_bank, mirrored_bank);
    }

    #[test]
    fn reads_and_writes_prg_ram() {
        let mut context = Context::new();
        let mut bus = CpuBus::new(&mut context);
        let cartridge = cartridge_with_pattern(0x4000, 0x2000);
        bus.insert_cartridge(cartridge);

        bus.write(cpu_mem::PRG_RAM_START, 0x42);
        assert_eq!(bus.read(cpu_mem::PRG_RAM_START), 0x42);
    }
}
