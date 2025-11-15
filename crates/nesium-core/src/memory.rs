//! Shared definitions for the NES memory map.
//!
//! Centralizing address-related constants keeps the hardware layout in one
//! location, prevents magic numbers from sneaking into other modules, and makes
//! it easier to reference the original console documentation while reading the
//! code base.

/// CPU memory map details.
pub mod cpu {
    /// First address of the hardware stack page.
    pub const STACK_PAGE_START: u16 = 0x0100;
    /// Last address (inclusive) of the hardware stack page.
    pub const STACK_PAGE_END: u16 = 0x01FF;

    /// Reset vector low byte address (`$FFFC`).
    pub const RESET_VECTOR_LO: u16 = 0xFFFC;
    /// Reset vector high byte address (`$FFFD`).
    pub const RESET_VECTOR_HI: u16 = 0xFFFD;
}

/// PPU register layout and VRAM mirror rules.
pub mod ppu {
    /// First CPU-visible PPU register address.
    pub const REGISTER_BASE: u16 = 0x2000;
    /// Last CPU-visible PPU register address (before mirroring repeats).
    pub const REGISTER_END: u16 = 0x2007;
    /// Mask for decoding register mirrors (`addr & 0x0007`).
    pub const REGISTER_SELECT_MASK: u16 = 0x0007;

    /// Total VRAM space that the PPU maps through `$2007` (16 KiB mirrored).
    pub const VRAM_SIZE: usize = 0x4000;
    /// Address mask applied after each VRAM access.
    pub const VRAM_MIRROR_MASK: u16 = 0x3FFF;

    /// Palette RAM base address (`$3F00`).
    pub const PALETTE_BASE: u16 = 0x3F00;
    /// Palette RAM byte count (32 bytes mirrored every 32 bytes).
    pub const PALETTE_RAM_SIZE: usize = 0x20;
    /// Palette mirroring period.
    pub const PALETTE_STRIDE: u16 = 0x20;

    /// Base address of nametable 0.
    pub const NAMETABLE_BASE: u16 = 0x2000;
    /// Size of a single nametable in bytes.
    pub const NAMETABLE_SIZE: u16 = 0x0400;

    /// Pattern table base address for table 0.
    pub const PATTERN_TABLE_0: u16 = 0x0000;
    /// Pattern table base address for table 1.
    pub const PATTERN_TABLE_1: u16 = 0x1000;

    /// DMA register used for transferring OAM data (`$4014`).
    pub const OAM_DMA: u16 = 0x4014;

    /// CPU-visible PPU register identifiers.
    #[repr(u16)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Register {
        /// `$2000` - PPUCTRL
        Control = 0x2000,
        /// `$2001` - PPUMASK
        Mask = 0x2001,
        /// `$2002` - PPUSTATUS
        Status = 0x2002,
        /// `$2003` - OAMADDR
        OamAddr = 0x2003,
        /// `$2004` - OAMDATA
        OamData = 0x2004,
        /// `$2005` - PPUSCROLL
        Scroll = 0x2005,
        /// `$2006` - PPUADDR
        Addr = 0x2006,
        /// `$2007` - PPUDATA
        Data = 0x2007,
    }

    impl Register {
        /// Raw address backing the register.
        pub const fn addr(self) -> u16 {
            self as u16
        }

        /// Resolves the canonical register for a CPU address in `$2000-$3FFF`.
        pub const fn from_cpu_addr(addr: u16) -> Self {
            match addr & REGISTER_SELECT_MASK {
                0 => Self::Control,
                1 => Self::Mask,
                2 => Self::Status,
                3 => Self::OamAddr,
                4 => Self::OamData,
                5 => Self::Scroll,
                6 => Self::Addr,
                7 => Self::Data,
                _ => unreachable!(),
            }
        }
    }
}

/// Audio Processing Unit (APU) register layout.
pub mod apu {
    /// Start of the CPU-mapped APU register range.
    pub const REGISTER_BASE: u16 = 0x4000;
    /// End of the CPU-mapped APU register range.
    pub const REGISTER_END: u16 = 0x4017;
    /// Total number of addresses exposed by the APU.
    pub const REGISTER_SPACE: usize = (REGISTER_END - REGISTER_BASE + 1) as usize;

    /// Final channel register before the status and DMA/OAM bridges.
    pub const CHANNEL_REGISTER_END: u16 = 0x4013;

    /// Address of the status register (`$4015`).
    pub const STATUS: u16 = 0x4015;
    /// Address of the frame counter configuration register (`$4017`).
    pub const FRAME_COUNTER: u16 = 0x4017;
}
