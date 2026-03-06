//! Mapper 21 – Konami VRC4 (VRC4a/VRC4c) implementation.
//!
//! This mapper provides:
//! - Two switchable 8 KiB PRG banks and two fixed banks (second‑last and last).
//! - Eight 1 KiB CHR banks with split low/high nibble registers.
//! - Mapper‑controlled nametable mirroring.
//! - An IRQ counter modelled after Mesen2's `VrcIrq` (prescaler + reloadable 8‑bit
//!   counter with optional CPU‑cycle mode).
//!
//! | Area | Address range     | Behaviour                                          | IRQ/Audio   |
//! |------|-------------------|----------------------------------------------------|-------------|
//! | CPU  | `$6000-$7FFF`     | Optional PRG-RAM                                   | None        |
//! | CPU  | `$8000-$DFFF`     | Two switchable 8 KiB PRG banks + one fixed window  | None        |
//! | CPU  | `$8000-$E003`     | PRG/CHR/mirroring/IRQ registers (after translation)| VRC4 IRQ    |
//! | CPU  | `$E000-$FFFF`     | Fixed 8 KiB PRG (last) + IRQ control/ack          | VRC4 IRQ    |
//! | PPU  | `$0000-$1FFF`     | Eight 1 KiB CHR banks with split low/high nibbles  | None        |
//! | PPU  | `$2000-$3EFF`     | Mirroring controlled by VRC4 register              | None        |

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
                vrc2_4::{Vrc2_4Banking, Vrc2_4Register, write_vrc2_4_register},
            },
            select_chr_storage,
        },
    },
    memory::cpu as cpu_mem,
    reset_kind::ResetKind,
};

/// CPU `$8000-$FFFF`: VRC4 register I/O window.
const VRC4_IO_WINDOW_START: u16 = 0x8000;
const VRC4_IO_WINDOW_END: u16 = 0xFFFF;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Vrc4Variant {
    /// Standard VRC4a address wiring (mapper 21 submapper 0/1).
    Vrc4a,
    /// VRC4c address wiring (mapper 21 submapper 2).
    Vrc4c,
}

#[derive(Debug, Clone)]
pub struct Mapper21 {
    prg_rom: PrgRom,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,
    banking: Vrc2_4Banking,

    // IRQ state ------------------------------------------------------------
    irq: VrcIrq,

    /// Address decoding variant and optional heuristic mode that ORs both
    /// VRC4a/VRC4c address line layouts (Mesen2 behaviour when submapper == 0).
    variant: Vrc4Variant,
    use_heuristics: bool,
}

impl Mapper21 {
    pub fn new(header: Header, prg_rom: PrgRom, chr_rom: ChrRom, trainer: TrainerBytes) -> Self {
        let prg_ram = allocate_prg_ram_with_trainer(&header, trainer);

        let chr = select_chr_storage(&header, chr_rom);
        let banking = Vrc2_4Banking::new(&prg_rom, header.mirroring());

        let variant = match header.submapper() {
            2 => Vrc4Variant::Vrc4c,
            _ => Vrc4Variant::Vrc4a,
        };
        let use_heuristics = header.submapper() == 0;

        Self {
            prg_rom,
            prg_ram,
            chr,
            banking,
            irq: VrcIrq::new(),
            variant,
            use_heuristics,
        }
    }

    /// Translate the CPU address into the VRC4 register layout, emulating the
    /// A0/A1 pin permutations documented on Nesdev and mirrored in Mesen2.
    fn translate_address(&self, addr: u16) -> u16 {
        let (mut a0, mut a1) = if self.use_heuristics {
            // Heuristic mode ORs both possible wirings to maximise compatibility
            // for submapper 0 ROMs.
            let base_a0 = (addr >> 1) & 0x01;
            let base_a1 = (addr >> 2) & 0x01;
            match self.variant {
                Vrc4Variant::Vrc4a | Vrc4Variant::Vrc4c => {
                    let alt_a0 = (addr >> 6) & 0x01;
                    let alt_a1 = (addr >> 7) & 0x01;
                    (base_a0 | alt_a0, base_a1 | alt_a1)
                }
            }
        } else {
            match self.variant {
                Vrc4Variant::Vrc4a => ((addr >> 1) & 0x01, (addr >> 2) & 0x01),
                Vrc4Variant::Vrc4c => ((addr >> 6) & 0x01, (addr >> 7) & 0x01),
            }
        };

        a0 &= 0x01;
        a1 &= 0x01;
        (addr & 0xFF00) | (a1 << 1) | a0
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        self.banking.read_prg_rom(&self.prg_rom, addr, true)
    }

    fn read_prg_ram(&self, addr: u16) -> Option<u8> {
        if self.prg_ram.is_empty() {
            return None;
        }
        let idx = (addr - cpu_mem::PRG_RAM_START) as usize % self.prg_ram.len();
        Some(self.prg_ram[idx])
    }

    fn write_prg_ram(&mut self, addr: u16, data: u8) {
        if self.prg_ram.is_empty() {
            return;
        }
        let idx = (addr - cpu_mem::PRG_RAM_START) as usize % self.prg_ram.len();
        self.prg_ram[idx] = data;
    }

    fn read_chr(&self, addr: u16) -> u8 {
        self.banking.read_chr(&self.chr, addr)
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        self.banking.write_chr(&mut self.chr, addr, data);
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        if let Some(reg) = Vrc2_4Register::from_addr(addr, false) {
            write_vrc2_4_register(
                &mut self.banking,
                Some(&mut self.irq),
                reg,
                addr,
                value,
                0x03,
                true,
            );
        }
    }
}

impl Mapper for Mapper21 {
    fn hook_mask(&self) -> MapperHookMask {
        MapperHookMask::CPU_BUS_ACCESS
    }

    fn on_mapper_event(&mut self, event: MapperEvent) {
        if let MapperEvent::CpuBusAccess { .. } = event {
            self.irq.clock();
        }
    }

    fn reset(&mut self, _kind: ResetKind) {
        // Basic VRC4 defaults: mirroring from header, PRG mode 0, IRQ disabled.
        self.banking.reset();
        self.irq.reset();

        // Keep the header-provided mirroring until the game selects otherwise.
        // (Some dumps ship with single-screen headers even when they later
        // configure mirroring via $9000.)
        // Mirroring is left unchanged here.
    }

    fn cpu_read(&self, addr: u16) -> Option<u8> {
        match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.read_prg_ram(addr),
            cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END => Some(self.read_prg_rom(addr)),
            _ => None,
        }
    }

    fn cpu_write(&mut self, addr: u16, data: u8, _cpu_cycle: u64) {
        match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.write_prg_ram(addr, data),
            VRC4_IO_WINDOW_START..=VRC4_IO_WINDOW_END => {
                let translated = self.translate_address(addr) & 0xF00F;
                self.write_register(translated, data);
            }
            _ => {}
        }
    }

    fn ppu_read(&self, addr: u16) -> Option<u8> {
        Some(self.read_chr(addr))
    }

    fn ppu_write(&mut self, addr: u16, data: u8) {
        self.write_chr(addr, data);
    }

    fn irq_pending(&self) -> bool {
        self.irq.pending()
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
        21
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Konami VRC4")
    }
}
