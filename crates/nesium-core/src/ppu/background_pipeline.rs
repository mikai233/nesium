use super::savestate::BgPipelineState;
/// 16-bit left-shifting register used by the NES PPU background pipeline.
///
/// Layout:
///   [ high 8 bits | low 8 bits ]
///
/// The high byte holds pixels that are currently "in flight" toward the screen,
/// the low byte is used to load the next 8 pixels (or repeated palette bits).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Shift16(u16);

impl Shift16 {
    /// Clears the low 8 bits, preserving the high 8 bits.
    ///
    /// Background shifters are reloaded every 8 pixels: the low byte is replaced
    /// with freshly fetched tile data while the high byte continues shifting out.
    #[inline]
    fn clear_low_byte(&mut self) {
        self.0 &= 0xFF00;
    }

    /// Loads a new low byte into the register.
    ///
    /// The argument is typically:
    /// - a pattern byte for one bitplane (8 pixels), or
    /// - `0x00`/`0xFF` for repeated palette bits across 8 pixels.
    #[inline]
    fn load_low_byte(&mut self, byte: u8) {
        self.0 |= byte as u16;
    }

    /// Returns the most significant bit.
    ///
    /// The NES PPU uses the MSB of each shifter on the current dot to
    /// assemble the background pixel.
    #[inline]
    fn msb(&self) -> u8 {
        ((self.0 >> 15) & 1) as u8
    }

    /// Returns the bit at position `15 - fine_x`.
    ///
    /// The PPU samples shifters using the fine X scroll as an offset rather
    /// than delaying shifts. This helper keeps the physical shift timing the
    /// same while exposing the scrolled bit.
    #[inline]
    fn bit_with_fine_x(&self, fine_x: u8) -> u8 {
        let shift = 15 - (fine_x & 0b111);
        ((self.0 >> shift) & 1) as u8
    }

    /// Shifts the register one bit to the left.
    ///
    /// This models the per-pixel shift that happens once for each PPU dot.
    #[inline]
    fn shift(&mut self) {
        self.0 <<= 1;
    }
}

/// Background pixel pipeline emulating the NES PPU background shifters.
///
/// The real PPU uses four 16-bit shift registers for background rendering:
/// - 2 pattern shifters (bitplane 0 and bitplane 1),
/// - 2 attribute/palette shifters (palette bit 0 and bit 1).
///
/// On each visible pixel:
/// - the MSB of each shifter is sampled,
/// - two bits form the pattern index within the tile (0..3),
/// - two bits form the palette index from the attribute table (0..3),
/// - all four shifters are advanced by one bit.
///
/// Every 8 pixels (at a tile boundary), the low bytes of these shifters
/// are reloaded with:
/// - the next tile row from the pattern table, and
/// - the palette bits derived from the attribute table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct BgPipeline {
    /// Background pattern bitplanes: [bitplane0, bitplane1].
    pattern: [Shift16; 2],
    /// Background palette bits: [palette_bit0, palette_bit1].
    palette: [Shift16; 2],
}

impl BgPipeline {
    /// Creates a new, empty background pipeline.
    pub fn new() -> Self {
        Self::default()
    }

    /// Clears all background shifters.
    ///
    /// This is useful when resetting the PPU or entering a state where
    /// background rendering is disabled.
    pub fn clear(&mut self) {
        self.pattern = [Shift16(0); 2];
        self.palette = [Shift16(0); 2];
    }

    /// Reloads the background shifters at a tile boundary.
    ///
    /// `tile_pattern` must contain the two pattern bytes for the current
    /// tile row:
    /// - `tile_pattern[0]`: bitplane 0 (low pattern bits),
    /// - `tile_pattern[1]`: bitplane 1 (high pattern bits).
    ///
    /// `palette_index` is the 2-bit palette number (0..3) selected
    /// from the attribute table for this tile.
    ///
    /// This matches the NES PPU behavior at dots 8, 16, 24, ... where
    /// new tile and attribute data are loaded into the shifters.
    pub fn reload(&mut self, tile_pattern: [u8; 2], palette_index: u8) {
        // Reload pattern shifters: keep the high byte, replace the low byte
        // with the newly fetched tile row for each bitplane.
        for (i, pattern) in tile_pattern.iter().enumerate() {
            self.pattern[i].clear_low_byte();
            self.pattern[i].load_low_byte(*pattern);
        }

        // Reload palette shifters: each palette bit is replicated across
        // 8 pixels in the low byte (0x00 or 0xFF), so that shifting the
        // register yields a constant palette bit for the entire tile row.
        for i in 0..=1 {
            let bit = (palette_index >> i) & 1;
            let repeated = if bit != 0 { 0xFF } else { 0x00 };

            self.palette[i].clear_low_byte();
            self.palette[i].load_low_byte(repeated);
        }
    }

    /// Samples the current background pixel (respecting fine X scroll).
    ///
    /// Returns `(palette_bits, pattern_bits)`:
    /// - `palette_bits` (`0..=3`): 2-bit palette index from attribute data,
    /// - `pattern_bits` (`0..=3`): 2-bit pattern index within the tile.
    ///
    /// Both values can then be used to look up a final color in the PPU
    /// palette RAM (e.g. at $3F00 + palette * 4 + color_index).
    ///
    pub fn sample(&self, fine_x: u8) -> (u8, u8) {
        // Sample the relevant bit of each shifter; fine X offsets which bit is
        // visible without altering the shift cadence.
        let pattern_bit0 = self.pattern[0].bit_with_fine_x(fine_x);
        let pattern_bit1 = self.pattern[1].bit_with_fine_x(fine_x);
        let palette_bit0 = self.palette[0].bit_with_fine_x(fine_x);
        let palette_bit1 = self.palette[1].bit_with_fine_x(fine_x);

        (
            (palette_bit1 << 1) | palette_bit0,
            (pattern_bit1 << 1) | pattern_bit0,
        )
    }

    /// Advances all background shifters by one bit (one PPU dot).
    pub fn shift(&mut self) {
        for i in 0..=1 {
            self.pattern[i].shift();
            self.palette[i].shift();
        }
    }

    pub(crate) fn save_state(&self) -> BgPipelineState {
        BgPipelineState {
            pattern: [self.pattern[0].0, self.pattern[1].0],
            palette: [self.palette[0].0, self.palette[1].0],
        }
    }

    pub(crate) fn load_state(&mut self, state: BgPipelineState) {
        self.pattern = [Shift16(state.pattern[0]), Shift16(state.pattern[1])];
        self.palette = [Shift16(state.palette[0]), Shift16(state.palette[1])];
    }
}
