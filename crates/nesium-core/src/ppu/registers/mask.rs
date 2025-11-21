use bitflags::bitflags;

bitflags! {
    /// PPU mask register (`$2001`).
    ///
    /// Controls color emphasis, grayscale mode, and which background/sprite
    /// layers are visible.
    ///
    /// Bit layout:
    /// ```text
    /// 7 6 5 4 3 2 1 0
    /// B G R S B s b g
    /// ```
    /// - `g`: grayscale
    /// - `b`: show background in leftmost 8 pixels
    /// - `s`: show sprites in leftmost 8 pixels
    /// - `B`: background enable
    /// - `S`: sprite enable
    /// - `R/G/B`: color emphasis bits
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
