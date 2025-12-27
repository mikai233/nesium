use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64};

pub(crate) const TURBO_FRAMES_PER_TOGGLE_DEFAULT: u8 = 2;

pub(crate) struct RuntimeState {
    pub(crate) paused: AtomicBool,
    pub(crate) pad_masks: [AtomicU8; 4],
    pub(crate) turbo_masks: [AtomicU8; 4],
    pub(crate) turbo_frames_per_toggle: AtomicU8,
    pub(crate) frame_seq: AtomicU64,
}

impl RuntimeState {
    pub(crate) fn new() -> Self {
        Self {
            paused: AtomicBool::new(false),
            pad_masks: std::array::from_fn(|_| AtomicU8::new(0)),
            turbo_masks: std::array::from_fn(|_| AtomicU8::new(0)),
            turbo_frames_per_toggle: AtomicU8::new(TURBO_FRAMES_PER_TOGGLE_DEFAULT),
            frame_seq: AtomicU64::new(0),
        }
    }
}
