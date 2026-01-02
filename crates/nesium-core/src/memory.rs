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
    /// NMI vector low byte address (`$FFFA`).
    pub const NMI_VECTOR_LO: u16 = 0xFFFA;
    /// NMI vector high byte address (`$FFFB`).
    pub const NMI_VECTOR_HI: u16 = 0xFFFB;
    /// IRQ/BRK vector low byte address (`$FFFE`).
    pub const IRQ_VECTOR_LO: u16 = 0xFFFE;
    /// IRQ/BRK vector high byte address (`$FFFF`).
    pub const IRQ_VECTOR_HI: u16 = 0xFFFF;

    /// First byte of CPU internal RAM.
    pub const INTERNAL_RAM_START: u16 = 0x0000;
    /// Last mirrored internal RAM address visible to the CPU (`$1FFF`).
    pub const INTERNAL_RAM_MIRROR_END: u16 = 0x1FFF;
    /// Size of the CPU internal RAM block (2 KiB mirrored through `$1FFF`).
    pub const INTERNAL_RAM_SIZE: usize = 0x0800;
    /// Mask applied to mirror CPU RAM accesses within `$0000-$1FFF`.
    pub const INTERNAL_RAM_MASK: u16 = (INTERNAL_RAM_SIZE as u16) - 1;

    /// First CPU address mapped to the PPU register mirror.
    pub const PPU_REGISTER_BASE: u16 = 0x2000;
    /// Last CPU address mirrored to the PPU register set.
    pub const PPU_REGISTER_END: u16 = 0x3FFF;

    /// First CPU-visible APU register.
    pub const APU_REGISTER_BASE: u16 = 0x4000;
    /// Final APU register before the status / frame counter region.
    pub const APU_REGISTER_END: u16 = 0x4013;
    /// APU status register (`$4015`).
    pub const APU_STATUS: u16 = 0x4015;
    /// Controller port 1 strobe/read address (`$4016`).
    pub const CONTROLLER_PORT_1: u16 = 0x4016;
    /// Controller port 2 strobe/read address (`$4017`).
    pub const CONTROLLER_PORT_2: u16 = 0x4017;

    /// Experimental I/O range reserved by Nintendo's diagnostics hardware.
    pub const TEST_MODE_BASE: u16 = 0x4018;
    /// End of the test mode I/O window.
    pub const TEST_MODE_END: u16 = 0x401F;

    /// First address handled by the cartridge expansion / PRG window.
    pub const CARTRIDGE_SPACE_BASE: u16 = 0x4020;
    /// PRG RAM window start address (`$6000`).
    pub const PRG_RAM_START: u16 = 0x6000;
    /// PRG RAM window end address (inclusive).
    pub const PRG_RAM_END: u16 = 0x7FFF;
    /// PRG ROM window start address (`$8000`).
    pub const PRG_ROM_START: u16 = 0x8000;
    /// Final CPU-visible address (`$FFFF`).
    pub const CPU_ADDR_END: u16 = 0xFFFF;
}

/// PPU register layout and VRAM mirror rules.
pub mod ppu {
    /// First CPU-visible PPU register address.
    pub const REGISTER_BASE: u16 = 0x2000;
    /// Last CPU-visible PPU register address (before mirroring repeats).
    pub const REGISTER_END: u16 = 0x2007;
    /// Mask for decoding register mirrors (`addr & 0x0007`).
    pub const REGISTER_SELECT_MASK: u16 = 0x0007;

    /// Size of the internal Character Internal RAM (CIRAM) used for nametables.
    /// The NES has 2 KiB of CIRAM, which is mapped to the nametable address space
    /// ($2000-$2FFF) with mirroring controlled by the cartridge.
    /// Pattern table space ($0000-$1FFF) is provided by the cartridge CHR ROM/RAM.
    pub const CIRAM_SIZE: usize = 0x0800; // 2 KiB

    /// Address mask applied after each PPU VRAM access to wrap to the 16 KiB space.
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
    /// Size of a single pattern table (4 KiB).
    pub const PATTERN_TABLE_SIZE: usize = 0x1000;
    /// Total size of both pattern tables ($0000-$1FFF = 8 KiB).
    pub const CHR_SIZE: usize = 0x2000;

    /// Primary Object Attribute Memory (OAM) byte count.
    pub const OAM_RAM_SIZE: usize = 0x100;
    /// Secondary OAM byte count used during sprite evaluation.
    pub const SECONDARY_OAM_RAM_SIZE: usize = 0x20;

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

    /// CPU-visible APU register identifiers.
    ///
    /// The NES APU exposes a small set of CPU-mapped registers in the
    /// `$4000-$4017` range. Most of these configure the individual audio
    /// channels; a handful of control registers manage global status and the
    /// frame counter. Keeping the mapping in one place avoids magic numbers
    /// in the APU implementation and mirrors the layout described on Nesdev.
    #[repr(u16)]
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum Register {
        /// `$4000` - Pulse channel 1: duty, envelope, length counter halt.
        Pulse1Control = 0x4000,
        /// `$4001` - Pulse channel 1: sweep unit configuration.
        Pulse1Sweep = 0x4001,
        /// `$4002` - Pulse channel 1: timer low byte.
        Pulse1TimerLow = 0x4002,
        /// `$4003` - Pulse channel 1: timer high 3 bits + length counter load.
        Pulse1TimerHigh = 0x4003,

        /// `$4004` - Pulse channel 2: duty, envelope, length counter halt.
        Pulse2Control = 0x4004,
        /// `$4005` - Pulse channel 2: sweep unit configuration.
        Pulse2Sweep = 0x4005,
        /// `$4006` - Pulse channel 2: timer low byte.
        Pulse2TimerLow = 0x4006,
        /// `$4007` - Pulse channel 2: timer high 3 bits + length counter load.
        Pulse2TimerHigh = 0x4007,

        /// `$4008` - Triangle channel: length counter halt + linear counter.
        TriangleControl = 0x4008,
        /// `$400A` - Triangle channel: timer low byte.
        TriangleTimerLow = 0x400A,
        /// `$400B` - Triangle channel: timer high 3 bits + length counter load.
        TriangleTimerHigh = 0x400B,

        /// `$400C` - Noise channel: envelope and length counter halt.
        NoiseControl = 0x400C,
        /// `$400E` - Noise channel: mode flag and period index.
        NoiseModeAndPeriod = 0x400E,
        /// `$400F` - Noise channel: length counter load.
        NoiseLength = 0x400F,

        /// `$4010` - DMC: IRQ enable, loop flag, and rate index.
        DmcControl = 0x4010,
        /// `$4011` - DMC: direct load value for the sample DAC.
        DmcDirectLoad = 0x4011,
        /// `$4012` - DMC: sample address (high bits of the CPU address).
        DmcSampleAddress = 0x4012,
        /// `$4013` - DMC: sample length (number of bytes to play).
        DmcSampleLength = 0x4013,

        /// `$4015` - APU status: channel enables and IRQ flags.
        Status = 0x4015,
        /// `$4017` - Frame counter: mode select and IRQ inhibit.
        FrameCounter = 0x4017,
    }

    impl Register {
        /// Raw CPU address for this APU register.
        pub const fn addr(self) -> u16 {
            self as u16
        }

        /// Resolves a CPU address to an APU register, if the address is one of
        /// the documented APU-visible locations.
        ///
        /// Returns `None` for unused holes in the `$4000-$4017` range (for
        /// example `$4009`, `$400D`, `$4014`, `$4016` which are handled by
        /// other subsystems such as the PPU or controllers).
        pub const fn from_cpu_addr(addr: u16) -> Option<Self> {
            match addr {
                0x4000 => Some(Self::Pulse1Control),
                0x4001 => Some(Self::Pulse1Sweep),
                0x4002 => Some(Self::Pulse1TimerLow),
                0x4003 => Some(Self::Pulse1TimerHigh),
                0x4004 => Some(Self::Pulse2Control),
                0x4005 => Some(Self::Pulse2Sweep),
                0x4006 => Some(Self::Pulse2TimerLow),
                0x4007 => Some(Self::Pulse2TimerHigh),
                0x4008 => Some(Self::TriangleControl),
                0x400A => Some(Self::TriangleTimerLow),
                0x400B => Some(Self::TriangleTimerHigh),
                0x400C => Some(Self::NoiseControl),
                0x400E => Some(Self::NoiseModeAndPeriod),
                0x400F => Some(Self::NoiseLength),
                0x4010 => Some(Self::DmcControl),
                0x4011 => Some(Self::DmcDirectLoad),
                0x4012 => Some(Self::DmcSampleAddress),
                0x4013 => Some(Self::DmcSampleLength),
                0x4015 => Some(Self::Status),
                0x4017 => Some(Self::FrameCounter),
                _ => None,
            }
        }

        /// Returns `true` if this register is a channel parameter register
        /// that is mirrored into the internal APU register RAM window
        /// (`$4000-$4013`).
        pub const fn is_channel_register(self) -> bool {
            matches!(
                self,
                Self::Pulse1Control
                    | Self::Pulse1Sweep
                    | Self::Pulse1TimerLow
                    | Self::Pulse1TimerHigh
                    | Self::Pulse2Control
                    | Self::Pulse2Sweep
                    | Self::Pulse2TimerLow
                    | Self::Pulse2TimerHigh
                    | Self::TriangleControl
                    | Self::TriangleTimerLow
                    | Self::TriangleTimerHigh
                    | Self::NoiseControl
                    | Self::NoiseModeAndPeriod
                    | Self::NoiseLength
                    | Self::DmcControl
                    | Self::DmcDirectLoad
                    | Self::DmcSampleAddress
                    | Self::DmcSampleLength
            )
        }

        /// Index into the APU's internal register RAM for channel parameter
        /// registers (`$4000-$4013`).
        ///
        /// Returns `None` for registers that are not mirrored into the APU
        /// register RAM (such as `$4015` and `$4017`).
        pub const fn channel_ram_index(self) -> Option<usize> {
            if self.is_channel_register() {
                Some((self.addr() - REGISTER_BASE) as usize)
            } else {
                None
            }
        }
    }

    /// Address of the status register (`$4015`).
    pub const STATUS: u16 = Register::Status as u16;
    /// Address of the frame counter configuration register (`$4017`).
    pub const FRAME_COUNTER: u16 = Register::FrameCounter as u16;
}
