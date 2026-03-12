//! Mapper 23 – Konami VRC2b / VRC4e implementation.
//!
//! This mapper family shares most behaviour with VRC4: 8 KiB PRG banking,
//! eight 1 KiB CHR banks, mapper-controlled mirroring, and (for VRC4e)
//! an IRQ counter. Address line permutations differ between VRC2b and
//! VRC4e; submapper 0 enables a heuristic that ORs both layouts to keep
//! ambiguous dumps playable, mirroring Mesen2.
//!
//! | Area | Address range     | Behaviour                                          | IRQ/Audio             |
//! |------|-------------------|----------------------------------------------------|-----------------------|
//! | CPU  | `$6000-$7FFF`     | Optional PRG-RAM                                   | None                  |
//! | CPU  | `$8000-$DFFF`     | Two switchable 8 KiB PRG banks + fixed window      | None                  |
//! | CPU  | `$8000-$FFFF`     | PRG/CHR/mirroring/IRQ registers (after translation)| VRC4 IRQ (VRC4e only) |
//! | PPU  | `$0000-$1FFF`     | Eight 1 KiB CHR banks with split low/high nibbles  | None                  |
//! | PPU  | `$2000-$3EFF`     | Mirroring from VRC2b/VRC4e register                | None                  |

use std::borrow::Cow;

use crate::cartridge::mapper::{MapperMemoryMut, MapperMemoryRef};

use crate::{
    cartridge::{
        ChrRom, Mapper, PrgRom, TrainerBytes,
        header::{Header, Mirroring},
        mapper::{
            ChrStorage, MapperEvent, MapperHookMask, allocate_prg_ram_with_trainer,
            core::{
                vrc_irq::VrcIrq,
                vrc2_4::{
                    Vrc2_4AddressConfig, Vrc2_4Banking, Vrc2_4Register, VrcAddressBits,
                    read_prg_ram_window, translate_vrc2_4_address, write_prg_ram_window,
                    write_vrc2_4_register,
                },
            },
            select_chr_storage,
        },
    },
    memory::cpu as cpu_mem,
    reset_kind::ResetKind,
};

/// CPU `$8000-$FFFF`: VRC2b/VRC4e register I/O window. Writes in this range,
/// after address line translation, target PRG/CHR/mirroring/IRQ registers.
const VRC23_IO_WINDOW_START: u16 = 0x8000;
const VRC23_IO_WINDOW_END: u16 = 0xFFFF;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Variant {
    Vrc2b,
    Vrc4e,
}

#[derive(Debug, Clone)]
pub struct Mapper23 {
    prg_rom: PrgRom,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,
    banking: Vrc2_4Banking,
    irq: Option<VrcIrq>,
    variant: Variant,
    use_heuristics: bool,
}

impl Mapper23 {
    pub fn new(header: Header, prg_rom: PrgRom, chr_rom: ChrRom, trainer: TrainerBytes) -> Self {
        let prg_ram = allocate_prg_ram_with_trainer(&header, trainer);
        let chr = select_chr_storage(&header, chr_rom);
        let banking = Vrc2_4Banking::new(&prg_rom, header.mirroring());

        let variant = match header.submapper() {
            2 => Variant::Vrc4e,
            _ => Variant::Vrc2b,
        };
        let use_heuristics = header.submapper() == 0;
        let irq = matches!(variant, Variant::Vrc4e).then(VrcIrq::new);

        Self {
            prg_rom,
            prg_ram,
            chr,
            banking,
            irq,
            variant,
            use_heuristics,
        }
    }

    fn has_irq(&self) -> bool {
        self.irq.is_some()
    }

    fn translate_address(&self, addr: u16) -> u16 {
        let config = match self.variant {
            Variant::Vrc2b => Vrc2_4AddressConfig {
                primary: VrcAddressBits::new(0, 1),
                heuristic_alt: Some(VrcAddressBits::new(2, 3)),
            },
            Variant::Vrc4e => Vrc2_4AddressConfig {
                primary: VrcAddressBits::new(2, 3),
                heuristic_alt: Some(VrcAddressBits::new(0, 1)),
            },
        };
        translate_vrc2_4_address(addr, config, self.use_heuristics)
    }

    fn read_prg_ram(&self, addr: u16) -> Option<u8> {
        read_prg_ram_window(&self.prg_ram, addr)
    }

    fn write_prg_ram(&mut self, addr: u16, data: u8) {
        write_prg_ram_window(&mut self.prg_ram, addr, data);
    }

    fn mirroring_mask(&self) -> u8 {
        if matches!(self.variant, Variant::Vrc2b) && !self.use_heuristics {
            0x01
        } else {
            0x03
        }
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        if let Some(reg) = Vrc2_4Register::from_addr(addr, true) {
            let mode_controls_prg_swap = self.has_irq();
            let mirroring_mask = self.mirroring_mask();
            write_vrc2_4_register(
                &mut self.banking,
                self.irq.as_mut(),
                reg,
                addr,
                value,
                mirroring_mask,
                mode_controls_prg_swap,
            );
        }
    }
}

impl Mapper for Mapper23 {
    fn hook_mask(&self) -> MapperHookMask {
        MapperHookMask::CPU_BUS_ACCESS
    }

    fn on_mapper_event(&mut self, event: MapperEvent) {
        if let MapperEvent::CpuBusAccess { .. } = event {
            if let Some(irq) = &mut self.irq {
                irq.clock();
            }
        }
    }

    fn reset(&mut self, _kind: ResetKind) {
        self.banking.reset();
        if let Some(irq) = &mut self.irq {
            irq.reset();
        }
    }

    fn cpu_read(&self, addr: u16, _open_bus: u8) -> Option<u8> {
        match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.read_prg_ram(addr),
            cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END => Some(self.banking.read_prg_rom(
                &self.prg_rom,
                addr,
                self.has_irq(),
            )),
            _ => None,
        }
    }

    fn cpu_write(&mut self, addr: u16, data: u8, _cpu_cycle: u64) {
        match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.write_prg_ram(addr, data),
            VRC23_IO_WINDOW_START..=VRC23_IO_WINDOW_END => {
                let translated = self.translate_address(addr) & 0xF00F;
                self.write_register(translated, data);
            }
            _ => {}
        }
    }

    fn ppu_read(&self, addr: u16) -> Option<u8> {
        Some(self.banking.read_chr(&self.chr, addr))
    }

    fn ppu_write(&mut self, addr: u16, data: u8) {
        self.banking.write_chr(&mut self.chr, addr, data);
    }

    fn irq_pending(&self) -> bool {
        self.irq.as_ref().is_some_and(VrcIrq::pending)
    }

    fn memory_ref(&self) -> MapperMemoryRef<'_> {
        MapperMemoryRef {
            prg_rom: Some(self.prg_rom.as_ref()),
            prg_ram: (!self.prg_ram.is_empty()).then_some(self.prg_ram.as_ref()),
            prg_work_ram: None,
            mapper_ram: None,
            chr_rom: self.chr.as_rom(),
            chr_ram: self.chr.as_ram(),
            chr_battery_ram: None,
        }
    }

    fn memory_mut(&mut self) -> MapperMemoryMut<'_> {
        MapperMemoryMut {
            prg_ram: (!self.prg_ram.is_empty()).then_some(self.prg_ram.as_mut()),
            prg_work_ram: None,
            mapper_ram: None,
            chr_ram: self.chr.as_ram_mut(),
            chr_battery_ram: None,
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.banking.mirroring()
    }

    fn mapper_id(&self) -> u16 {
        23
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Konami VRC2b / VRC4e")
    }
}
