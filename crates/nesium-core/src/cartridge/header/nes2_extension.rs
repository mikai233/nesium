use super::{
    ConsoleType, ExtendedConsoleType, Nes2ConsoleTypeData, Nes2ExpansionDevice, Nes2MiscRomCount,
    VsHardwareType, VsPpuType,
};

/// NES 2.0 extension bytes (header bytes 8..=15).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Nes2Extension {
    /// Byte 8 (submapper + mapper high bits).
    pub mapper_msb_submapper: u8,
    /// Byte 9 (PRG/CHR msb nibbles).
    pub prg_chr_msb: u8,
    /// Byte 10 (PRG RAM / PRG NVRAM shifts).
    pub prg_ram_shifts: u8,
    /// Byte 11 (CHR RAM / CHR NVRAM shifts).
    pub chr_ram_shifts: u8,
    /// Byte 12 (timing).
    pub timing: u8,
    /// Byte 13 (console-type dependent).
    pub console_type_data: u8,
    /// Byte 14 (misc ROMs).
    pub misc_roms: u8,
    /// Byte 15 (default expansion device).
    pub default_expansion_device: u8,
}

impl Nes2Extension {
    pub fn submapper(&self) -> u8 {
        self.mapper_msb_submapper >> 4
    }

    pub fn mapper_msb(&self) -> u8 {
        self.mapper_msb_submapper & 0x0F
    }

    pub fn prg_rom_msb(&self) -> u8 {
        self.prg_chr_msb & 0x0F
    }

    pub fn chr_rom_msb(&self) -> u8 {
        (self.prg_chr_msb >> 4) & 0x0F
    }

    pub fn prg_ram_shift(&self) -> u8 {
        self.prg_ram_shifts & 0x0F
    }

    pub fn prg_nvram_shift(&self) -> u8 {
        self.prg_ram_shifts >> 4
    }

    pub fn chr_ram_shift(&self) -> u8 {
        self.chr_ram_shifts & 0x0F
    }

    pub fn chr_nvram_shift(&self) -> u8 {
        self.chr_ram_shifts >> 4
    }

    pub fn console_type_data(&self, console_type: ConsoleType) -> Nes2ConsoleTypeData {
        match console_type {
            ConsoleType::NesFamicom => Nes2ConsoleTypeData::NesFamicom {
                raw: self.console_type_data,
            },
            ConsoleType::VsSystem => Nes2ConsoleTypeData::VsSystem {
                hardware_type: VsHardwareType::from_nibble((self.console_type_data >> 4) & 0x0F),
                ppu_type: VsPpuType::from_nibble(self.console_type_data & 0x0F),
            },
            ConsoleType::PlayChoice10 => Nes2ConsoleTypeData::PlayChoice10 {
                raw: self.console_type_data,
            },
            ConsoleType::Extended => Nes2ConsoleTypeData::Extended {
                console_type: ExtendedConsoleType::from_nibble(self.console_type_data & 0x0F),
            },
        }
    }

    pub fn misc_rom_count(&self) -> Nes2MiscRomCount {
        Nes2MiscRomCount(self.misc_roms & 0b11)
    }

    pub fn default_expansion_device(&self) -> Nes2ExpansionDevice {
        Nes2ExpansionDevice(self.default_expansion_device)
    }
}
