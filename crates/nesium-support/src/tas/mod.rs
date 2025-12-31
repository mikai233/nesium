pub mod fm2;

use bitflags::bitflags;

/// Unified internal Movie IR used to drive the emulator.
/// It is decoupled from specific file formats (FM2, BK2, etc.).
#[derive(Debug, Clone, Default)]
pub struct Movie {
    /// Original TAS data and format-specific metadata.
    pub data: TasData,

    /// Expected ROM hash (MD5/SHA1).
    pub rom_hash: Option<Vec<u8>>,

    /// Whether it is PAL format.
    pub is_pal: bool,

    /// Input data for each frame.
    pub frames: Vec<InputFrame>,

    /// Initial savestate (if any).
    pub savestate: Option<Vec<u8>>,
}

/// Represents different TAS format data.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TasData {
    Fm2(fm2::Fm2Header),
    // Future: Bk2(bk2::Bk2Header),
    Unknown,
}

impl Default for TasData {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InputFrame {
    /// Command flags for the frame (Reset, Power, FDS, etc.)
    pub commands: FrameFlags,
    /// Controller bitmasks for up to 4 ports.
    /// Bit mapping (matching nesium-core/standard NES):
    /// 0: A, 1: B, 2: Select, 3: Start, 4: Up, 5: Down, 6: Left, 7: Right
    pub ports: [u8; 4],
}

bitflags! {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct FrameFlags: u8 {
        const NONE = 0;
        const RESET = 1 << 0;
        const POWER = 1 << 1;
        const FDS_INSERT = 1 << 2;
        const FDS_SELECT = 1 << 3;
        const VS_INSERT_COIN = 1 << 4;
        const VS_INSERT_COIN2 = 1 << 5;
        const VS_SERVICE = 1 << 6;
    }
}
