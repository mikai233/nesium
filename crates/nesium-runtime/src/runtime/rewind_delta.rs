//! Frame-by-frame rewind history with lightweight, in-memory compression.
//!
//! This module optimizes for *stepping backwards one frame at a time*.
//! It intentionally does **not** support random access or seeking.
//!
//! # Design
//! - Each frame stores the machine snapshot as postcard bytes.
//! - The latest snapshot bytes are kept uncompressed in `current_full_bytes`.
//! - For each new frame after the first, we store a *reverse patch* that can
//!   reconstruct the previous frame from the current frame.
//!
//! # Reverse patch format
//! For two byte strings `prev` and `cur`, define:
//! - `min_len = min(prev.len(), cur.len())`
//! - `xor_prefix[i] = prev[i] ^ cur[i]` for `i < min_len`
//! - `prev_tail = prev[min_len..]` (only present when `prev.len() > min_len`)
//!
//! Given `cur`, we can recover `prev` as:
//! - `prev_prefix[i] = cur[i] ^ xor_prefix[i]` for `i < min_len`
//! - If `prev.len() < cur.len()`, truncate to `prev.len()`.
//! - If `prev.len() > cur.len()`, append `prev_tail`.
//!
//! The patch is stored as:
//! ```text
//! [prev_len: u32 LE][cur_len: u32 LE][xor_prefix: min_len bytes][prev_tail: prev_len-min_len bytes]
//! ```
//! and compressed with LZ4.
//!
//! This format intentionally allows *variable-length* snapshots, which avoids
//! requiring fixed-width integer encoding in the snapshot serializer.
//!
//! # Failure handling
//! Any decompression/decoding failure is treated as corruption. The history is
//! cleared to avoid propagating inconsistent state.

use std::collections::VecDeque;

use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use nesium_core::state::nes::NesSnapshot;

struct RewindFrame {
    /// LZ4-compressed reverse patch for stepping back one frame.
    ///
    /// The first frame in a chain has no patch, because there is no previous
    /// snapshot to reconstruct.
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

        // Build a reverse patch that reconstructs `prev` (the current bytes)
        // from `cur` (the new bytes).
        let prev = &self.current_full_bytes;
        let cur = &full_bytes;

        let prev_len = prev.len();
        let cur_len = cur.len();
        let min_len = prev_len.min(cur_len);

        // Patch layout:
        // [prev_len u32][cur_len u32][xor_prefix min_len][prev_tail prev_len-min_len]
        let mut patch = Vec::with_capacity(8 + min_len + (prev_len - min_len));
        patch.extend_from_slice(&(prev_len as u32).to_le_bytes());
        patch.extend_from_slice(&(cur_len as u32).to_le_bytes());

        // xor_prefix
        for i in 0..min_len {
            patch.push(prev[i] ^ cur[i]);
        }

        // prev_tail (only needed when previous snapshot is longer)
        if prev_len > min_len {
            patch.extend_from_slice(&prev[min_len..]);
        }

        let compressed = compress_prepend_size(&patch);

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

        // Remove the current/latest frame. Its patch describes how to go from
        // the previous frame to the current frame.
        let current = self.frames.pop_back()?;
        let compressed = current.delta?;

        let patch = decompress_size_prepended(&compressed).ok()?;
        if patch.len() < 8 {
            self.clear();
            return None;
        }

        let prev_len = u32::from_le_bytes(patch[0..4].try_into().ok()?) as usize;
        let cur_len = u32::from_le_bytes(patch[4..8].try_into().ok()?) as usize;

        // The patch must match the currently selected bytes.
        if self.current_full_bytes.len() != cur_len {
            self.clear();
            return None;
        }

        let min_len = prev_len.min(cur_len);
        let xor_prefix_off = 8;
        let xor_prefix_end = xor_prefix_off + min_len;
        if patch.len() < xor_prefix_end {
            self.clear();
            return None;
        }

        let prev_tail_len = prev_len.saturating_sub(min_len);
        let prev_tail_off = xor_prefix_end;
        let prev_tail_end = prev_tail_off + prev_tail_len;
        if patch.len() != prev_tail_end {
            // The patch must contain exactly the declared tail bytes.
            self.clear();
            return None;
        }

        // `prev_prefix[i] = cur[i] ^ xor_prefix[i]`
        for i in 0..min_len {
            self.current_full_bytes[i] ^= patch[xor_prefix_off + i];
        }

        // Adjust the length to `prev_len`.
        if prev_len < cur_len {
            self.current_full_bytes.truncate(prev_len);
        } else if prev_len > cur_len {
            self.current_full_bytes
                .extend_from_slice(&patch[prev_tail_off..prev_tail_end]);
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
