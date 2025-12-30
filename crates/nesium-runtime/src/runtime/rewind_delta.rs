//! Frame-by-frame rewind history with lightweight, in-memory compression.
//!
//! This module optimizes for *stepping backwards one frame at a time*.
//! It intentionally does **not** support random access or seeking.
//!
//! # Design
//! - Each frame stores the machine snapshot as postcard bytes.
//! - The latest snapshot bytes are kept uncompressed in `current_full_bytes`.
//! - For each new frame after the first, we store an XOR diff against the
//!   previous frame (`diff = cur ^ prev`), compressed with LZ4.
//!
//! # Why XOR deltas?
//! If `diff = cur ^ prev`, then `prev = cur ^ diff`. Rewinding one frame is
//! therefore an LZ4 decompression plus a byte-wise XOR.
//!
//! # Invariants
//! - `current_full_bytes` always represents the snapshot bytes of the *latest*
//!   frame currently selected at the end of the history.
//! - All snapshots in a chain must serialize to the same byte length.
//!   If the length changes unexpectedly, the history is cleared.
//!
//! # Failure handling
//! Any decompression or decoding failure is treated as corruption. The history
//! is cleared to avoid propagating inconsistent state.

use std::collections::VecDeque;

use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use nesium_core::state::nes::NesSnapshot;

struct RewindFrame {
    /// LZ4-compressed XOR diff (`cur ^ prev`).
    ///
    /// The first frame in a chain has no diff, because there is no previous
    /// snapshot to diff against.
    delta: Option<Vec<u8>>,

    /// Frame pixels for display. This is kept uncompressed for now.
    pixels: Vec<u8>,
}

/// Stores rewind history for frame-by-frame rewind.
///
/// This type optimizes for repeated `rewind_one_frame()` calls.
/// Random access (jumping to an arbitrary frame) is intentionally out of scope.
pub struct RewindState {
    frames: VecDeque<RewindFrame>,

    /// Postcard bytes of the *currently selected* frame at the end of the history.
    ///
    /// When rewinding, this is updated in-place to the previous frame's bytes.
    current_full_bytes: Vec<u8>,
}

impl RewindState {
    /// Creates an empty rewind history.
    pub fn new() -> Self {
        Self {
            frames: VecDeque::new(),
            current_full_bytes: Vec::new(),
        }
    }

    /// Returns `true` if the history contains at least two frames.
    ///
    /// With fewer than two frames, a single-step rewind is not possible.
    pub fn can_rewind(&self) -> bool {
        self.frames.len() >= 2 && !self.current_full_bytes.is_empty()
    }

    /// Pushes a new frame into the rewind history.
    ///
    /// `capacity` is the maximum number of frames to keep. When exceeded, the
    /// oldest frames are dropped.
    pub fn push_frame(&mut self, snapshot: &NesSnapshot, pixels: Vec<u8>, capacity: usize) {
        let Ok(full_bytes) = snapshot.to_postcard_bytes() else {
            // If serialization fails, skip storing this frame.
            return;
        };

        // Start a new chain if this is the first frame.
        if self.current_full_bytes.is_empty() {
            self.current_full_bytes = full_bytes;
            self.frames.push_back(RewindFrame {
                delta: None,
                pixels,
            });
            self.trim_to_capacity(capacity);
            return;
        }

        // If the serialized length changed, XOR diffs are no longer valid.
        if self.current_full_bytes.len() != full_bytes.len() {
            self.clear();

            self.current_full_bytes = full_bytes;
            self.frames.push_back(RewindFrame {
                delta: None,
                pixels,
            });
            self.trim_to_capacity(capacity);
            return;
        }

        // Delta frame: XOR diff against the previous frame's state.
        let mut diff = vec![0u8; full_bytes.len()];
        for (i, b) in full_bytes.iter().enumerate() {
            diff[i] = *b ^ self.current_full_bytes[i];
        }

        let compressed = compress_prepend_size(&diff);

        self.current_full_bytes = full_bytes;
        self.frames.push_back(RewindFrame {
            delta: Some(compressed),
            pixels,
        });

        self.trim_to_capacity(capacity);
    }

    /// Rewinds by one frame and returns the reconstructed snapshot and pixels.
    ///
    /// This performs an LZ4 decompression and a byte-wise XOR. On any decoding
    /// error, the rewind history is cleared and `None` is returned.
    pub fn rewind_one_frame(&mut self) -> Option<(NesSnapshot, Vec<u8>)> {
        if !self.can_rewind() {
            return None;
        }

        // Remove the current/latest frame. Its delta describes how to go from
        // the previous frame to the current frame.
        let current = self.frames.pop_back()?;
        let compressed = current.delta?;

        let diff_bytes = decompress_size_prepended(&compressed).ok()?;
        if diff_bytes.len() != self.current_full_bytes.len() {
            self.clear();
            return None;
        }

        // `prev = cur ^ diff`
        for (i, d) in diff_bytes.iter().enumerate() {
            self.current_full_bytes[i] ^= *d;
        }

        // The new "current" is now the last frame in the deque.
        let prev_pixels = self.frames.back()?.pixels.clone();
        let snapshot = NesSnapshot::from_postcard_bytes(&self.current_full_bytes).ok()?;
        Some((snapshot, prev_pixels))
    }

    /// Drops all rewind history and resets internal state.
    pub fn clear(&mut self) {
        self.frames.clear();
        self.current_full_bytes.clear();
    }

    fn trim_to_capacity(&mut self, capacity: usize) {
        while self.frames.len() > capacity {
            self.frames.pop_front();
        }

        // If we dropped everything, also reset the current bytes.
        if self.frames.is_empty() {
            self.current_full_bytes.clear();
        }
    }
}
