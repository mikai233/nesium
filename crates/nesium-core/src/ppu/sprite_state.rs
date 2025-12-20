use crate::mem_block::ppu::SpriteLineRam;

/// Cycle-accurate sprite evaluation state, modeled after Mesen2.
///
/// This state tracks the internal address counters (high/low), secondary OAM
/// write address, and the sprite overflow bug state machine.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct SpriteEvalState {
    /// High 6 bits of the internal OAM address (`_spriteAddrH`, 0..=63).
    pub(crate) sprite_addr_h: u8,
    /// Low 2 bits of the internal OAM address (`_spriteAddrL`, 0..=3).
    pub(crate) sprite_addr_l: u8,
    /// Secondary OAM address (`_secondaryOamAddr`, counts bytes copied).
    pub(crate) secondary_oam_addr: u8,
    /// Latched: currently copying a sprite that matched the range check.
    pub(crate) sprite_in_range: bool,
    /// Latched: OAM copy has wrapped/realigned and further matching is suppressed.
    pub(crate) oam_copy_done: bool,
    /// Countdown used for the sprite overflow address realignment glitch.
    pub(crate) overflow_bug_counter: u8,
    /// Latched: sprite 0 is considered visible for the next scanline.
    pub(crate) sprite0_in_range_next: bool,
    /// Number of sprites selected for the next scanline (0..=8).
    pub(crate) count: u8,
}

impl SpriteEvalState {
    #[inline]
    pub(crate) fn start(&mut self, oam_addr: u8) {
        // Mirrors `ProcessSpriteEvaluationStart`.
        self.sprite0_in_range_next = false;
        self.sprite_in_range = false;
        self.secondary_oam_addr = 0;
        self.overflow_bug_counter = 0;
        self.oam_copy_done = false;
        self.sprite_addr_h = (oam_addr >> 2) & 0x3F;
        self.sprite_addr_l = oam_addr & 0x03;
        self.count = 0;
    }

    #[inline]
    pub(crate) fn latch_end_of_evaluation(
        &mut self,
        _scanline: i16,
        _last_oam_byte: u8,
        _sprite_height: u8,
    ) {
        // Mirrors `ProcessSpriteEvaluationEnd` (for the default "bug enabled" path).
        let bytes = self.secondary_oam_addr;
        self.count = ((bytes.saturating_add(3)) >> 2).min(8);

        // Early 2C02 behavior: if sprite eval wrapped and the last copied byte,
        // interpreted as a Y coordinate, is in range, count it as an extra sprite.
        // This is known as the "Phantom Sprite Bug".
        //
        // TODO: Make this configurable (Mesen: EnablePpuSpriteEvalBug).
        // It is currently disabled by default to prevent visual artifacts (dots at X=255)
        // in games like Shadow of the Ninja, effectively emulating a later PPU revision.
        // self.apply_phantom_sprite_bug(scanline, last_oam_byte, sprite_height);
    }

    /// Simulates the "Phantom Sprite" hardware bug found in early PPU revisions (e.g., RP2C02B).
    ///
    /// If the sprite evaluation logic is halted abruptly at cycle 256 while in a misaligned state,
    /// the last byte read from OAM can be erroneously interpreted as the Y-coordinate of a new sprite.
    /// If this "Y-coordinate" falls within the current scanline range, a "phantom" sprite is added.
    /// Since the remaining sprite data (tile, attr, x) is not fetched, this typically results in
    /// a sprite drawn with tile $FF at X=255.
    #[allow(dead_code)]
    fn apply_phantom_sprite_bug(&mut self, scanline: i16, last_oam_byte: u8, sprite_height: u8) {
        let y = last_oam_byte as i16;
        let end = y + sprite_height as i16;
        let in_range = scanline >= y && scanline < end;
        if in_range && self.count < 8 {
            self.count += 1;
        }
    }

    #[inline]
    pub(crate) fn primary_oam_addr(&self) -> u8 {
        (self.sprite_addr_l & 0x03) | (self.sprite_addr_h << 2)
    }
}

/// Cycle-accurate sprite pattern fetch state (dots 257..=320).
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct SpriteFetchState {
    /// Sprite index within secondary OAM being fetched (0..=7).
    pub(crate) i: u8,
    /// Sub-dot within the 8-dot sprite fetch slot (0..=7).
    pub(crate) sub: u8,
}

/// Buffered sprite data for the upcoming scanline.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct SpriteLineBuffers {
    pub(crate) y: SpriteLineRam,
    pub(crate) tile: SpriteLineRam,
    pub(crate) attr: SpriteLineRam,
    pub(crate) x: SpriteLineRam,
    pub(crate) pattern_low: SpriteLineRam,
    pub(crate) pattern_high: SpriteLineRam,
}

impl Default for SpriteLineBuffers {
    fn default() -> Self {
        Self::new()
    }
}

impl SpriteLineBuffers {
    pub(crate) fn new() -> Self {
        Self {
            y: SpriteLineRam::new(),
            tile: SpriteLineRam::new(),
            attr: SpriteLineRam::new(),
            x: SpriteLineRam::new(),
            pattern_low: SpriteLineRam::new(),
            pattern_high: SpriteLineRam::new(),
        }
    }

    pub(crate) fn clear(&mut self) {
        *self = Self::new();
    }

    pub(crate) fn set_meta(&mut self, idx: usize, y: u8, tile: u8, attr: u8, x: u8) {
        if idx < 8 {
            self.y[idx] = y;
            self.tile[idx] = tile;
            self.attr[idx] = attr;
            self.x[idx] = x;
        }
    }

    pub(crate) fn set_pattern_low(&mut self, idx: usize, value: u8) {
        if idx < 8 {
            self.pattern_low[idx] = value;
        }
    }

    pub(crate) fn set_pattern_high(&mut self, idx: usize, value: u8) {
        if idx < 8 {
            self.pattern_high[idx] = value;
        }
    }

    pub(crate) fn attr_slice(&self) -> &[u8] {
        self.attr.as_slice()
    }

    pub(crate) fn x_slice(&self) -> &[u8] {
        self.x.as_slice()
    }

    pub(crate) fn pattern_low_slice(&self) -> &[u8] {
        self.pattern_low.as_slice()
    }

    pub(crate) fn pattern_high_slice(&self) -> &[u8] {
        self.pattern_high.as_slice()
    }
}
