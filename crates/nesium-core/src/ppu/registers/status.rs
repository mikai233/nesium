use bitflags::bitflags;

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
