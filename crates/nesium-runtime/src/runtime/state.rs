use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64};

use super::types::TileViewerConfig;

pub(crate) const TURBO_ON_FRAMES_DEFAULT: u8 = 2;
pub(crate) const TURBO_OFF_FRAMES_DEFAULT: u8 = 2;

pub(crate) struct RuntimeState {
    pub(crate) paused: AtomicBool,
    pub(crate) pad_masks: [AtomicU8; 4],
    pub(crate) turbo_masks: [AtomicU8; 4],
    pub(crate) turbo_on_frames: AtomicU8,
    pub(crate) turbo_off_frames: AtomicU8,
    pub(crate) frame_seq: AtomicU64,
    pub(crate) rom_hash: Mutex<Option<[u8; 32]>>,
    pub(crate) tile_viewer: Mutex<TileViewerConfig>,
    pub(crate) rewind_enabled: AtomicBool,
    pub(crate) rewind_capacity: AtomicU64,
    pub(crate) rewinding: AtomicBool,
    pub(crate) fast_forwarding: AtomicBool,
}

impl RuntimeState {
    pub(crate) fn new() -> Self {
        Self {
            paused: AtomicBool::new(false),
            pad_masks: std::array::from_fn(|_| AtomicU8::new(0)),
            turbo_masks: std::array::from_fn(|_| AtomicU8::new(0)),
            turbo_on_frames: AtomicU8::new(TURBO_ON_FRAMES_DEFAULT),
            turbo_off_frames: AtomicU8::new(TURBO_OFF_FRAMES_DEFAULT),
            frame_seq: AtomicU64::new(0),
            rom_hash: Mutex::new(None),
            tile_viewer: Mutex::new(TileViewerConfig::default()),
            rewind_enabled: AtomicBool::new(false),
            rewind_capacity: AtomicU64::new(600), // Default 10s @ 60fps
            rewinding: AtomicBool::new(false),
            fast_forwarding: AtomicBool::new(false),
        }
    }
}
