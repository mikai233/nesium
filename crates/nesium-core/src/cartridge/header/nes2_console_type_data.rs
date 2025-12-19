use super::{ExtendedConsoleType, VsHardwareType, VsPpuType};

/// NES 2.0: interpretation of header byte 13 depends on [`super::ConsoleType`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Nes2ConsoleTypeData {
    /// For standard NES/Famicom cartridges this byte is currently unused/reserved.
    NesFamicom { raw: u8 },
    /// Vs. System: upper nibble = hardware type, lower nibble = PPU type.
    VsSystem {
        hardware_type: VsHardwareType,
        ppu_type: VsPpuType,
    },
    /// PlayChoice-10: this byte is not specified by the NES 2.0 table; keep raw.
    PlayChoice10 { raw: u8 },
    /// Extended console type.
    Extended { console_type: ExtendedConsoleType },
}
