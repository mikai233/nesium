//! CPU-visible PPU register state and internal VRAM address latches.
//!
//! This module mirrors the `$2000-$2007` register set and the internal
//! `v/t/x/w` VRAM latches described on NESDev. The concrete bit layouts live
//! in submodules for clarity.

mod control;
mod mask;
mod status;
mod vram_addr;
mod vram_registers;

pub(crate) use control::Control;
pub(crate) use mask::Mask;
pub(crate) use status::Status;
pub(crate) use vram_addr::VramAddr;
pub(crate) use vram_registers::VramRegisters;

use crate::mem_block::ppu::OamRam;

/// Aggregates the state of all CPU visible PPU registers.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Registers {
    /// Mirror of the control register (`$2000`).
    pub(crate) control: Control,
    /// Mirror of the mask register (`$2001`).
    pub(crate) mask: Mask,
    /// Status register (`$2002`).
    pub(crate) status: Status,
    /// Current OAM pointer driven by `$2003`/`$2004`.
    pub(crate) oam_addr: u8,
    /// Primary sprite memory accessible through `$2004`.
    pub(crate) oam: OamRam,
    /// Internal VRAM registers (`v`/`t`/`x`/`w`).
    pub(crate) vram: VramRegisters,
    /// Internal buffer implementing the delayed `$2007` read behavior.
    pub(crate) vram_buffer: u8,
}

impl Default for Registers {
    fn default() -> Self {
        Self::new()
    }
}

impl Registers {
    /// Creates a new register block with the power-on reset state.
    pub(crate) fn new() -> Self {
        Self {
            control: Control::default(),
            mask: Mask::default(),
            status: Status::default(),
            oam_addr: 0,
            oam: OamRam::new(),
            vram: VramRegisters::default(),
            vram_buffer: 0,
        }
    }

    /// Restores all register values to their reset defaults.
    pub(crate) fn reset(&mut self) {
        *self = Registers::new();
    }

    /// Updates control, also syncing the nametable bits into `t` per NES spec.
    pub(crate) fn write_control(&mut self, value: u8) {
        self.control = Control::from_bits_retain(value);
        self.vram.t.set_nametable(self.control.nametable_index());
    }
}
