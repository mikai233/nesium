use bitflags::bitflags;

use crate::{memory::ppu as ppu_mem, ram::ppu::OamRam};

bitflags! {
    /// PPU control register (`$2000`).
    ///
    /// The register controls high level rendering settings and configures how the
    /// VRAM address auto-increments when the CPU accesses `$2007`.
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

bitflags! {
    /// PPU mask register (`$2001`).
    ///
    /// Controls color emphasis, grayscale mode, and which background/sprite
    /// layers are visible.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub(crate) struct Mask: u8 {
        /// Grayscale conversion enable (bit 0).
        const GRAYSCALE = 0b0000_0001;

        /// Show background in the leftmost eight pixels (bit 1).
        const SHOW_BACKGROUND_LEFT = 0b0000_0010;

        /// Show sprites in the leftmost eight pixels (bit 2).
        const SHOW_SPRITES_LEFT = 0b0000_0100;

        /// Enables background rendering (bit 3).
        const SHOW_BACKGROUND = 0b0000_1000;

        /// Enables sprite rendering (bit 4).
        const SHOW_SPRITES = 0b0001_0000;

        /// Emphasizes the red color channel (bit 5).
        const EMPHASIZE_RED = 0b0010_0000;

        /// Emphasizes the green color channel (bit 6).
        const EMPHASIZE_GREEN = 0b0100_0000;

        /// Emphasizes the blue color channel (bit 7).
        const EMPHASIZE_BLUE = 0b1000_0000;
    }
}

impl Default for Mask {
    fn default() -> Self {
        Self::empty()
    }
}

impl Mask {
    /// Returns `true` when either background or sprite rendering is enabled.
    pub(crate) fn rendering_enabled(self) -> bool {
        self.intersects(Mask::SHOW_BACKGROUND | Mask::SHOW_SPRITES)
    }
}

bitflags! {
    /// PPU status register (`$2002`).
    ///
    /// Bit layout:
    /// ```text
    /// 7 6 5 4 3 2 1 0
    /// V S O . . . . .
    /// ```
    /// - `V`: Vertical blank flag
    /// - `S`: Sprite zero hit
    /// - `O`: Sprite overflow flag
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub(crate) struct Status: u8 {
        /// Sprite overflow flag (bit 5).
        const SPRITE_OVERFLOW = 0b0010_0000;

        /// Sprite zero hit flag (bit 6).
        const SPRITE_ZERO_HIT = 0b0100_0000;

        /// Vertical blank flag (bit 7). Reading `$2002` clears this bit.
        const VERTICAL_BLANK = 0b1000_0000;
    }
}

impl Default for Status {
    fn default() -> Self {
        Self::empty()
    }
}

/// Internal latch that mirrors writes to `$2005` (scroll register).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) struct ScrollRegister {
    /// Raw horizontal scroll value written by the CPU.
    horizontal: u8,
    /// Raw vertical scroll value written by the CPU.
    vertical: u8,
    /// Tracks whether the next write targets the vertical component.
    latch: bool,
}

impl ScrollRegister {
    /// Writes a value to the scroll register, alternating between horizontal and vertical fields.
    pub(crate) fn write(&mut self, value: u8) {
        if !self.latch {
            self.horizontal = value;
        } else {
            self.vertical = value;
        }
        self.latch = !self.latch;
    }

    /// Returns the last horizontal scroll byte written via `$2005`.
    pub(crate) fn horizontal(&self) -> u8 {
        self.horizontal
    }

    /// Returns the last vertical scroll byte written via `$2005`.
    pub(crate) fn vertical(&self) -> u8 {
        self.vertical
    }

    /// Clears the write toggle; the next `$2005` write targets the horizontal byte.
    pub(crate) fn reset_latch(&mut self) {
        self.latch = false;
    }
}

/// Internal latch for the VRAM address register (`$2006`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub(crate) struct VramAddressRegister {
    /// Current 15-bit VRAM address.
    addr: u16,
    /// Indicates whether the high byte or low byte is expected on the next write.
    latch: bool,
}

impl VramAddressRegister {
    /// Writes either the high or low byte, depending on the internal latch state.
    pub(crate) fn write(&mut self, value: u8) {
        if !self.latch {
            self.addr = ((value as u16 & 0x3F) << 8) | (self.addr & 0x00FF);
        } else {
            self.addr = (self.addr & 0x7F00) | value as u16;
        }
        self.addr &= ppu_mem::VRAM_MIRROR_MASK;
        self.latch = !self.latch;
    }

    /// Reads the current VRAM address.
    pub(crate) fn addr(&self) -> u16 {
        self.addr
    }

    /// Increments the VRAM address by `step`, wrapping at `$3FFF`.
    pub(crate) fn increment(&mut self, step: u16) {
        self.addr = (self.addr + step) & ppu_mem::VRAM_MIRROR_MASK;
    }

    /// Resets the write latch so the next `$2006` write updates the high byte.
    pub(crate) fn reset_latch(&mut self) {
        self.latch = false;
    }
}

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
    /// Scroll register latch associated with `$2005`.
    pub(crate) scroll: ScrollRegister,
    /// VRAM address latch associated with `$2006`/`$2007`.
    pub(crate) addr: VramAddressRegister,
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
            scroll: ScrollRegister::default(),
            addr: VramAddressRegister::default(),
            vram_buffer: 0,
        }
    }

    /// Restores all register values to their reset defaults.
    pub(crate) fn reset(&mut self) {
        *self = Registers::new();
    }
}
