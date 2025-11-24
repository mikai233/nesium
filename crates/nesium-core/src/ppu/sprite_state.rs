use crate::ram::ppu::SpriteLineRam;

/// Phases of the hardware sprite evaluation state machine.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) enum SpriteEvalPhase {
    /// Scanning primary OAM for an in-range sprite (reading byte 0 / Y first).
    #[default]
    ScanY,
    /// Copying the remaining 3 bytes of an in-range sprite into secondary OAM.
    CopyRest,
    /// Overflow scan phase after 8 sprites are found.
    /// Hardware continues scanning with a buggy n/m increment pattern.
    OverflowScan,
}

/// Cycle-accurate sprite evaluation state for secondary OAM selection.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct SpriteEvalState {
    /// Current evaluation phase.
    pub(crate) phase: SpriteEvalPhase,
    /// Primary OAM sprite index being scanned (0..=63).
    pub(crate) n: u8,
    /// Byte index within the current sprite (0..=3).
    pub(crate) m: u8,
    /// Next write position in secondary OAM (0..=32 bytes).
    pub(crate) sec_idx: u8,
    /// Number of sprites selected for the next scanline (0..=8).
    pub(crate) count: u8,
    /// Whether we are currently copying this sprite into secondary OAM.
    pub(crate) copying: bool,
    /// Latched during overflow scan: a byte has matched the scanline range.
    pub(crate) overflow_in_range: bool,
    /// Countdown used to emulate the overflow address realignment glitch.
    pub(crate) overflow_bug_counter: u8,
    /// Latched: sprite 0 will be in range on the *next* scanline.
    pub(crate) sprite0_in_range_next: bool,
    /// Latched: sprite overflow has been observed for the next scanline.
    pub(crate) overflow_next: bool,
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
