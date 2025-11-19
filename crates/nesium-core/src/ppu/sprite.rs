use bitflags::bitflags;
use core::slice::GetDisjointMutError;

bitflags! {
    /// Attribute bits stored in sprite byte 2.
    ///
    /// Bit layout:
    /// ```text
    /// 7 6 5 4 3 2 1 0
    /// V H P . . . p p
    /// ```
    /// - `V`: Vertical flip
    /// - `H`: Horizontal flip
    /// - `P`: Priority (behind background when set)
    /// - `p`: Sprite palette select (0..=3)
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub(crate) struct SpriteAttributes: u8 {
        /// Use sprite palette 0 (default).
        const PALETTE_0 = 0b0000_0000;
        /// Use sprite palette 1.
        const PALETTE_1 = 0b0000_0001;
        /// Use sprite palette 2.
        const PALETTE_2 = 0b0000_0010;
        /// Use sprite palette 3.
        const PALETTE_3 = 0b0000_0011;

        /// When set, sprite is drawn behind the background.
        const PRIORITY_BEHIND_BACKGROUND = 0b0010_0000;

        /// Horizontal flip.
        const FLIP_HORIZONTAL = 0b0100_0000;

        /// Vertical flip.
        const FLIP_VERTICAL = 0b1000_0000;
    }
}

/// Mutable view over a single sprite entry in primary or secondary OAM.
///
/// The NES encodes each sprite as four consecutive bytes:
/// - byte 0: Y position
/// - byte 1: tile index
/// - byte 2: attribute bits (see [`SpriteAttributes`])
/// - byte 3: X position
///
/// This helper provides typed accessors on top of the raw OAM memory while
/// only borrowing the four bytes that belong to this sprite.
pub(crate) struct SpriteView<'a> {
    bytes: &'a mut [u8],
}

impl<'a> SpriteView<'a> {
    const BYTES_PER_SPRITE: usize = 4;

    /// Wraps a single sprite worth of bytes (4 bytes) in a view.
    ///
    /// Callers are expected to pass exactly one sprite's data; in debug builds
    /// the length is asserted.
    pub(crate) fn new(bytes: &'a mut [u8]) -> Self {
        debug_assert_eq!(bytes.len(), Self::BYTES_PER_SPRITE);
        Self { bytes }
    }

    /// Returns the raw four-byte view backing this sprite.
    pub(crate) fn as_bytes(&self) -> &[u8] {
        self.bytes
    }

    /// Returns a mutable raw four-byte view backing this sprite.
    pub(crate) fn as_bytes_mut(&mut self) -> &mut [u8] {
        self.bytes
    }

    /// Sprite Y position.
    pub(crate) fn y(&self) -> u8 {
        self.bytes[0]
    }

    pub(crate) fn set_y(&mut self, y: u8) {
        self.bytes[0] = y;
    }

    /// Sprite tile index.
    pub(crate) fn tile(&self) -> u8 {
        self.bytes[1]
    }

    pub(crate) fn set_tile(&mut self, tile: u8) {
        self.bytes[1] = tile;
    }

    /// Decoded attribute flags for this sprite.
    pub(crate) fn attributes(&self) -> SpriteAttributes {
        SpriteAttributes::from_bits_retain(self.bytes[2])
    }

    /// Replaces the attribute flags for this sprite.
    pub(crate) fn set_attributes(&mut self, attributes: SpriteAttributes) {
        self.bytes[2] = attributes.bits();
    }

    /// Sprite X position.
    pub(crate) fn x(&self) -> u8 {
        self.bytes[3]
    }

    pub(crate) fn set_x(&mut self, x: u8) {
        self.bytes[3] = x;
    }

    /// Builds a fixed-size list of sprite views for the given sprite indices.
    ///
    /// The `sprite_indices` are sprite slots (0-based); each slot occupies four
    /// bytes inside `oam`. This helper uses `get_disjoint_mut` to safely create
    /// multiple mutable views at once.
    pub(crate) fn views_for_indices<const N: usize>(
        oam: &'a mut [u8],
        sprite_indices: [usize; N],
    ) -> Result<[SpriteView<'a>; N], GetDisjointMutError> {
        let ranges = sprite_indices.map(|i| {
            let start = i * Self::BYTES_PER_SPRITE;
            let end = start + Self::BYTES_PER_SPRITE;
            start..end
        });

        let slices = oam.get_disjoint_mut(ranges)?;
        Ok(slices.map(SpriteView::new))
    }
}
