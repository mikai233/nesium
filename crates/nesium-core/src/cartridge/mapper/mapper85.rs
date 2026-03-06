//! Mapper 85 – Konami VRC7 (without expansion audio synthesis).
//!
//! This models the banking/IRQ behaviour of the VRC7 used by titles like Lagrange
//! Point. Expansion audio registers are accepted but muted; integrating full VRC7
//! audio would require an OPLL core wired through `ExpansionAudio`.
//!
//! - PRG: three switchable 8 KiB banks at `$8000/$A000/$C000`, fixed last bank
//!   at `$E000`.
//! - CHR: eight 1 KiB banks at `$0000-$1FFF`.
//! - Mirroring: control register `$E000` bits 0‑1 (H/V/screen A/B).
//! - PRG-RAM enable: control register bit 7 (when present via header sizing).
//! - IRQ: VRC-style counter with reload (`$E008`), control (`$F000`), ack
//!   (`$F008`), prescaler clocked by CPU cycles (divide by ~341).
//!
//! | Area | Address range       | Behaviour                                          | IRQ/Audio                          |
//! |------|---------------------|----------------------------------------------------|------------------------------------|
//! | CPU  | `$6000-$7FFF`       | Optional PRG-RAM with enable bit in control       | None                               |
//! | CPU  | `$8000/$A000/$C000` | Three switchable 8 KiB PRG-ROM banks             | None                               |
//! | CPU  | `$E000-$FFFF`       | Fixed 8 KiB PRG-ROM bank (last) + control/IRQ    | VRC7 IRQ (expansion audio muted)   |
//! | CPU  | `$8000-$D008`       | VRC7 PRG/CHR/mirroring/IRQ registers              | VRC7 IRQ (audio registers, muted)  |
//! | PPU  | `$0000-$1FFF`       | Eight 1 KiB CHR banks                             | None                               |
//! | PPU  | `$2000-$3EFF`       | Mirroring from VRC7 control register              | None                               |

use std::borrow::Cow;

use crate::cartridge::mapper::{MapperMemoryMut, MapperMemoryRef};

use crate::{
    apu::{ExpansionAudio, Vrc7Audio},
    cartridge::{
        ChrRom, Mapper, PrgRom, TrainerBytes,
        header::{Header, Mirroring},
        mapper::{
            ChrStorage, MapperEvent, MapperHookMask, allocate_prg_ram_with_trainer,
            core::{
                vrc_irq::VrcIrq,
                vrc7::{Vrc7Board, Vrc7Register},
            },
            select_chr_storage,
        },
    },
    memory::cpu as cpu_mem,
    reset_kind::ResetKind,
};

#[derive(Debug, Clone)]
pub struct Mapper85 {
    prg_rom: crate::cartridge::PrgRom,
    prg_ram: Box<[u8]>,
    chr: ChrStorage,
    board: Vrc7Board,
    irq: VrcIrq,
    audio: Vrc7Audio,
}

impl Mapper85 {
    pub fn new(header: Header, prg_rom: PrgRom, chr_rom: ChrRom, trainer: TrainerBytes) -> Self {
        let prg_ram = allocate_prg_ram_with_trainer(&header, trainer);

        let chr = select_chr_storage(&header, chr_rom);
        let board = Vrc7Board::new(&prg_rom, header.mirroring());

        Self {
            prg_rom,
            prg_ram,
            chr,
            board,
            irq: VrcIrq::new(),
            audio: Vrc7Audio::new(),
        }
    }

    #[inline]
    fn prg_ram_enabled(&self) -> bool {
        self.board.prg_ram_enabled(!self.prg_ram.is_empty())
    }

    fn update_control(&mut self, value: u8) {
        self.board.set_control(value);
        self.audio.set_muted((value & 0x40) != 0);
    }

    fn read_prg_rom(&self, addr: u16) -> u8 {
        self.board.read_prg_rom(&self.prg_rom, addr)
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

    fn read_chr(&self, addr: u16) -> u8 {
        self.board.read_chr(&self.chr, addr)
    }

    fn write_chr(&mut self, addr: u16, data: u8) {
        self.board.write_chr(&mut self.chr, addr, data);
    }

    fn write_register(&mut self, addr: u16, value: u8) {
        if let Some(reg) = Vrc7Register::from_addr(addr) {
            match reg {
                Vrc7Register::PrgBank8000 => self.board.write_prg_bank(0, value),
                Vrc7Register::PrgBankA000 => self.board.write_prg_bank(1, value),
                Vrc7Register::PrgBankC000 => self.board.write_prg_bank(2, value),
                Vrc7Register::AudioSelect => self.audio.write_register_select(value),
                Vrc7Register::AudioData => self.audio.write_register_data(value),
                Vrc7Register::ChrBank => self.board.write_chr_bank_by_addr(addr, value),
                Vrc7Register::Control => self.update_control(value),
                Vrc7Register::IrqReload => {
                    self.irq.write_reload(value);
                }
                Vrc7Register::IrqControl => {
                    self.irq.write_control(value);
                }
                Vrc7Register::IrqAck => {
                    self.irq.acknowledge();
                }
            }
        }
    }
}

impl Mapper for Mapper85 {
    fn hook_mask(&self) -> MapperHookMask {
        MapperHookMask::CPU_CLOCK
    }

    fn on_mapper_event(&mut self, event: MapperEvent) {
        if let MapperEvent::CpuClock { .. } = event {
            self.irq.clock();
        }
    }

    fn reset(&mut self, _kind: ResetKind) {
        self.board.reset();
        self.audio.reset();
        self.audio.set_muted((self.board.control() & 0x40) != 0);
        self.irq.reset();
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
            0x8000..=0xFFFF => {
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
        85
    }

    fn name(&self) -> Cow<'static, str> {
        Cow::Borrowed("Konami VRC7 (no audio)")
    }

    fn expansion_audio(&self) -> Option<&dyn ExpansionAudio> {
        Some(&self.audio)
    }

    fn expansion_audio_mut(&mut self) -> Option<&mut dyn ExpansionAudio> {
        Some(&mut self.audio)
    }
}
