#[cfg(feature = "savestate-serde")]
use serde::{Deserialize, Serialize};

/// Serializable state for the background shifters.
#[cfg_attr(feature = "savestate-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BgPipelineState {
    pub pattern: [u16; 2],
    pub palette: [u16; 2],
}

/// Serializable state for the PPU open-bus latch.
#[cfg_attr(feature = "savestate-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PpuOpenBusState {
    pub value: u8,
    pub decay_stamp: [u32; 8],
}

#[cfg_attr(feature = "savestate-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpriteSlotState {
    pub pattern_low: u8,
    pub pattern_high: u8,
    pub attributes: u8,
    pub x_counter: u8,
    pub sprite0: bool,
}

#[cfg_attr(feature = "savestate-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpritePipelineState {
    pub active_count: u8,
    pub slots: [SpriteSlotState; 8],
}

#[cfg_attr(feature = "savestate-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpriteEvalState {
    pub sprite_addr_h: u8,
    pub sprite_addr_l: u8,
    pub secondary_oam_addr: u8,
    pub sprite_in_range: bool,
    pub oam_copy_done: bool,
    pub overflow_bug_counter: u8,
    pub sprite0_in_range_next: bool,
    pub count: u8,
}

#[cfg_attr(feature = "savestate-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpriteFetchState {
    pub i: u8,
    pub sub: u8,
}

#[cfg_attr(feature = "savestate-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SpriteLineBuffersState {
    pub y: [u8; 8],
    pub tile: [u8; 8],
    pub attr: [u8; 8],
    pub x: [u8; 8],
    pub pattern_low: [u8; 8],
    pub pattern_high: [u8; 8],
}

#[cfg_attr(feature = "savestate-serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PendingVramIncrementState(pub u8);

impl PendingVramIncrementState {
    pub fn none() -> Self {
        Self(0)
    }
    pub fn by1() -> Self {
        Self(1)
    }
    pub fn by32() -> Self {
        Self(32)
    }
}
