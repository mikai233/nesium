use bitflags::bitflags;

use crate::memory::ppu as ppu_mem;

bitflags! {
    /// PPU control register (`$2000`).
    ///
    /// The register controls high level rendering settings and configures how the
    /// VRAM address auto-increments when the CPU accesses `$2007`.
    ///
    /// Bit layout:
    /// ```text
    /// 7 6 5 4 3 2 1 0
    /// N M S B s I n n
    /// ```
    /// - `n n`: base nametable select
    /// - `I`: VRAM increment (0=+1, 1=+32)
    /// - `s`: sprite pattern table (8x8)
    /// - `B`: background pattern table
    /// - `S`: sprite size (0=8x8, 1=8x16)
    /// - `M`: master/slave select
    /// - `N`: generate NMI at VBlank start
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub(crate) struct Control: u8 {
        /// Selects the base nametable address (bits 0 and 1).
        ///
        /// The two bits encode the four nametable pages:
        /// - `00`: `$2000`
        /// - `01`: `$2400`
        /// - `10`: `$2800`
        /// - `11`: `$2C00`
        const NAMETABLE = 0b0000_0011;

        /// Controls the VRAM address increment unit (bit 2).
        /// `0` increments by 1 (horizontal), `1` increments by 32 (vertical).
        const INCREMENT_32 = 0b0000_0100;

        /// Selects the sprite pattern table for 8x8 sprites (bit 3).
        /// `0` uses `$0000`, `1` uses `$1000`.
        const SPRITE_TABLE = 0b0000_1000;

        /// Selects the background pattern table (bit 4).
        /// `0` uses `$0000`, `1` uses `$1000`.
        const BACKGROUND_TABLE = 0b0001_0000;

        /// Chooses the sprite size (bit 5).
        /// `0` renders 8x8 sprites, `1` renders 8x16 sprites.
        const SPRITE_SIZE_16 = 0b0010_0000;

        /// Master/slave select (bit 6).
        /// Only meaningful when the PPU is configured for external video.
        const MASTER_SLAVE = 0b0100_0000;

        /// Enables NMI generation at the start of VBlank (bit 7).
        const GENERATE_NMI = 0b1000_0000;
    }
}

impl Default for Control {
    fn default() -> Self {
        Self::empty()
    }
}

impl Control {
    /// Computes the base nametable address (`$2000`, `$2400`, `$2800`, `$2C00`).
    pub(crate) fn base_nametable_addr(self) -> u16 {
        ppu_mem::NAMETABLE_BASE + ((self.bits() as u16 & 0b11) * ppu_mem::NAMETABLE_SIZE)
    }

    /// Returns the nametable select bits (0..3).
    pub(crate) fn nametable_index(self) -> u8 {
        self.bits() & 0b11
    }

    /// Returns the VRAM increment amount (1 or 32) based on bit 2.
    pub(crate) fn vram_increment(self) -> u16 {
        if self.contains(Control::INCREMENT_32) {
            32
        } else {
            1
        }
    }

    /// Returns the sprite pattern table base address.
    pub(crate) fn sprite_pattern_table(self) -> u16 {
        if self.contains(Control::SPRITE_TABLE) {
            ppu_mem::PATTERN_TABLE_1
        } else {
            ppu_mem::PATTERN_TABLE_0
        }
    }

    /// Returns the background pattern table base address.
    pub(crate) fn background_pattern_table(self) -> u16 {
        if self.contains(Control::BACKGROUND_TABLE) {
            ppu_mem::PATTERN_TABLE_1
        } else {
            ppu_mem::PATTERN_TABLE_0
        }
    }

    /// Indicates whether sprites use the 8x16 mode.
    pub(crate) fn use_8x16_sprites(self) -> bool {
        self.contains(Control::SPRITE_SIZE_16)
    }

    /// Indicates whether the PPU should fire an NMI at the start of VBlank.
    pub(crate) fn nmi_enabled(self) -> bool {
        self.contains(Control::GENERATE_NMI)
    }
}
