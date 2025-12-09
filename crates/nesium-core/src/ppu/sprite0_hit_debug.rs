/// Captures the position of the first sprite-0 hit in the current frame (debug).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sprite0HitPos {
    pub scanline: i16,
    pub cycle: u16,
}

/// Debug info captured on the first sprite-0 hit of a frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Sprite0HitDebug {
    pub pos: Sprite0HitPos,
    pub oam: [u8; 4],
}
