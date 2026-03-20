//! Mapper 26 – Konami VRC6b.
//!
//! This implementation mirrors the PRG/CHR banking and IRQ behaviour of VRC6,
//! following Mesen2's layout, including VRC6 expansion audio.
//!
//! | Area | Address range       | Behaviour                                          | IRQ/Audio                         |
//! |------|---------------------|----------------------------------------------------|-----------------------------------|
//! | CPU  | `$6000-$7FFF`       | Optional PRG-RAM (enabled via banking_mode bit 7)  | None                              |
//! | CPU  | `$8000-$BFFF`       | 16 KiB switchable PRG-ROM window (2×8 KiB)         | None                              |
//! | CPU  | `$C000-$DFFF`       | 8 KiB switchable PRG-ROM window                    | None                              |
//! | CPU  | `$E000-$FFFF`       | 8 KiB fixed PRG-ROM window (last)                  | None                              |
//! | CPU  | `$B003/$F000-$F002` | Banking/mirroring/IRQ control registers            | VRC6 IRQ                          |
//! | PPU  | `$0000-$1FFF`       | Eight 1 KiB CHR banks with mode-dependent mapping  | None                              |
//! | PPU  | `$2000-$3EFF`       | Mirroring from VRC6 control (`banking_mode`)       | None                              |

use std::borrow::Cow;

use crate::cartridge::mapper::{MapperMemoryMut, MapperMemoryRef};

use crate::{
    apu::{ExpansionAudio, Vrc6Audio},
    cartridge::{
        ChrRom, Mapper, PrgRom, TrainerBytes,
        header::{Header, Mirroring},
        mapper::{
            ChrStorage, MapperEvent, MapperHookMask, NametableTarget,
            allocate_prg_ram_with_trainer,
            core::{
                vrc_irq::VrcIrq,
                vrc6::{Vrc6Board, Vrc6Register},
            },
            select_chr_storage,
        },
    },
    memory::cpu as cpu_mem,
    reset_kind::ResetKind,
};

/// CPU `$8000-$FFFF`: VRC6b register I/O and PRG banking window.
const VRC6_IO_WINDOW_START: u16 = 0x8000;
const VRC6_IO_WINDOW_END: u16 = 0xFFFF;

#[derive(Debug, Clone)]
pub struct Mapper26 {
    prg_rom: PrgRom,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,
    board: Vrc6Board,
    irq: VrcIrq,
    audio: Vrc6Audio,
}

impl Mapper26 {
    pub fn new(header: Header, prg_rom: PrgRom, chr_rom: ChrRom, trainer: TrainerBytes) -> Self {
        let prg_ram = allocate_prg_ram_with_trainer(&header, trainer);
        let chr = select_chr_storage(&header, chr_rom);
        let board = Vrc6Board::new(&prg_rom, header.mirroring());

        Self {
            prg_rom,
            prg_ram,
            chr,
            board,
            irq: VrcIrq::new(),
            audio: Vrc6Audio::new(),
        }
    }

    #[inline]
    fn prg_ram_enabled(&self) -> bool {
        self.board.prg_ram_enabled(!self.prg_ram.is_empty())
    }

    fn read_prg_ram(&self, addr: u16) -> Option<u8> {
        if !self.prg_ram_enabled() {
            return None;
        }
        let idx = (addr - cpu_mem::PRG_RAM_START) as usize % self.prg_ram.len();
        Some(self.prg_ram[idx])
    }

    fn write_prg_ram(&mut self, addr: u16, data: u8) {
        if !self.prg_ram_enabled() {
            return;
        }
        let idx = (addr - cpu_mem::PRG_RAM_START) as usize % self.prg_ram.len();
        self.prg_ram[idx] = data;
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        self.board.read_prg_rom(&self.prg_rom, addr)
    }

    fn read_chr(&self, addr: u16) -> u8 {
        self.board.read_chr(&self.chr, addr)
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        self.board.write_chr(&mut self.chr, addr, data);
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        if let Some(reg) = Vrc6Register::from_addr(addr) {
            use Vrc6Register::*;

            match reg {
                PrgBank8000_2x => self.board.write_prg_bank_8000(value),
                ExpansionAudio => self.audio.write_register(addr, value),
                Control => self.board.write_control(value),
                PrgBankC000 => self.board.write_prg_bank_c000(value),
                ChrBankLow => self.board.write_chr_low((addr & 0x0003) as usize, value),
                ChrBankHigh => self.board.write_chr_high((addr & 0x0003) as usize, value),
                IrqReload => self.irq.write_reload(value),
                IrqControl => self.irq.write_control(value),
                IrqAck => self.irq.acknowledge(),
            }
        }
    }
}

impl Mapper for Mapper26 {
    fn hook_mask(&self) -> MapperHookMask {
        MapperHookMask::CPU_CLOCK
    }

    fn on_mapper_event(&mut self, event: MapperEvent) {
        if !matches!(event, MapperEvent::CpuClock { .. }) {
            return;
        }

        self.irq.clock();
    }

    fn reset(&mut self, _kind: ResetKind) {
        self.board.reset();
        self.irq.reset();
        self.audio.reset();
    }

    fn cpu_read(&self, addr: u16, _open_bus: u8) -> Option<u8> {
        match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.read_prg_ram(addr),
            cpu_mem::PRG_ROM_START..=cpu_mem::CPU_ADDR_END => Some(self.read_prg_rom(addr)),
            _ => None,
        }
    }

    fn cpu_write(&mut self, addr: u16, data: u8, _cpu_cycle: u64) {
        match addr {
            cpu_mem::PRG_RAM_START..=cpu_mem::PRG_RAM_END => self.write_prg_ram(addr, data),
            VRC6_IO_WINDOW_START..=VRC6_IO_WINDOW_END => {
                let translated = self.board.translate_address(addr);
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

    fn map_nametable(&self, addr: u16) -> NametableTarget {
        self.board.map_nametable(addr)
    }

    fn mapper_nametable_read(&self, offset: u32) -> u8 {
        self.board.mapper_nametable_read(&self.chr, offset)
    }

    fn mapper_nametable_write(&mut self, offset: u32, value: u8) {
        self.board
            .mapper_nametable_write(&mut self.chr, offset, value);
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
        self.board.mirroring()
    }

    fn mapper_id(&self) -> u16 {
        26
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Konami VRC6b")
    }

    fn expansion_audio(&self) -> Option<&dyn ExpansionAudio> {
        Some(&self.audio)
    }

    fn expansion_audio_mut(&mut self) -> Option<&mut dyn ExpansionAudio> {
        Some(&mut self.audio)
    }
}
