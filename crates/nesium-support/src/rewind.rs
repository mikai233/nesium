//! Frame-by-frame rewind history with lightweight, in-memory compression.
//!
//! This module optimizes for *stepping backwards one frame at a time*.
//! It intentionally does **not** support random access or seeking.
//!
//! # Design
//! - Each frame stores the machine snapshot as postcard bytes.
//! - Each frame stores the framebuffer as palette indices (1 byte per pixel).
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
//! # Index plane patch
//! The framebuffer index plane has a fixed length (one byte per pixel).
//! For two index buffers `prev` and `cur` of equal length, we store:
//! - `xor[i] = prev[i] ^ cur[i]`
//!
//! Given `cur`, we can recover `prev` by XORing `cur` with `xor`.
//! The `xor` bytes are compressed with LZ4.
//!
//! # Failure handling
//! Any decompression/decoding failure is treated as corruption. The history is
//! cleared to avoid propagating inconsistent state.

use std::collections::VecDeque;

use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use nesium_core::state::nes::NesSnapshot;

/// Number of frames between keyframes for efficient random access.
const KEYFRAME_INTERVAL: usize = 64;

struct RewindFrame {
    /// LZ4-compressed reverse patch for stepping back one frame.
    ///
    /// The first frame in a chain has no patch, because there is no previous
    /// snapshot to reconstruct.
    snapshot_delta: Option<Vec<u8>>,

    /// LZ4-compressed XOR patch for the framebuffer index plane.
    ///
    /// The first frame in a chain has no patch, because there is no previous
    /// index plane to reconstruct.
    index_delta: Option<Vec<u8>>,

    /// Full uncompressed index plane bytes for keyframes.
    /// Only populated every KEYFRAME_INTERVAL frames for efficient seeking.
    index_full: Option<Vec<u8>>,
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

    /// Bytes of the *currently selected* framebuffer index plane.
    ///
    /// This is updated in-place when rewinding.
    current_index_bytes: Vec<u8>,

    /// Absolute sequence number of the first (oldest) frame in the buffer.
    /// Used to display absolute frame numbers in the History Viewer.
    first_frame_seq: u64,
}

impl Default for RewindState {
    fn default() -> Self {
        Self::new()
    }
}

impl RewindState {
    /// Creates an empty rewind history.
    pub fn new() -> Self {
        Self {
            frames: VecDeque::new(),
            current_full_bytes: Vec::new(),
            current_index_bytes: Vec::new(),
            first_frame_seq: 0,
        }
    }

    /// Returns `true` if the history contains at least two frames.
    ///
    /// With fewer than two frames, a single-step rewind is not possible.
    pub fn can_rewind(&self) -> bool {
        self.frames.len() >= 2
            && !self.current_full_bytes.is_empty()
            && !self.current_index_bytes.is_empty()
    }

    /// Pushes a new frame into the rewind history.
    ///
    /// `indices` must be the framebuffer index plane (one byte per pixel).
    /// `frame_seq` is the NES emulator's absolute frame number.
    /// The buffer is delta-compressed against the previously pushed frame.
    pub fn push_frame(
        &mut self,
        snapshot: &NesSnapshot,
        indices: Vec<u8>,
        capacity: usize,
        frame_seq: u64,
    ) {
        let Ok(full_bytes) = snapshot.to_postcard_bytes() else {
            // If serialization fails, skip storing this frame.
            return;
        };

        if indices.is_empty() {
            // A missing index plane would break reconstruction.
            return;
        }

        // Start a new chain if this is the first frame.
        if self.current_full_bytes.is_empty() {
            self.first_frame_seq = frame_seq;
            self.current_full_bytes = full_bytes;
            self.current_index_bytes = indices.clone();
            self.frames.push_back(RewindFrame {
                snapshot_delta: None,
                index_delta: None,
                index_full: Some(indices), // First frame is always a keyframe
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

        let snapshot_delta = compress_prepend_size(&patch);

        // Build an XOR patch for the fixed-size index plane.
        if self.current_index_bytes.len() != indices.len() {
            // The index plane is expected to be a stable, fixed-size buffer.
            // Treat a size mismatch as a chain break and restart the history.
            self.clear();
            self.first_frame_seq = frame_seq;
            self.current_full_bytes = full_bytes;
            self.current_index_bytes = indices.clone();
            self.frames.push_back(RewindFrame {
                snapshot_delta: None,
                index_delta: None,
                index_full: Some(indices), // Chain restart is always a keyframe
            });
            self.trim_to_capacity(capacity);
            return;
        }

        let mut xor = vec![0u8; indices.len()];
        for (i, (p, c)) in self
            .current_index_bytes
            .iter()
            .zip(indices.iter())
            .enumerate()
        {
            xor[i] = p ^ c;
        }
        let index_delta = compress_prepend_size(&xor);

        self.current_full_bytes = full_bytes;
        self.current_index_bytes = indices.clone();

        // Store keyframe every KEYFRAME_INTERVAL frames for efficient seeking
        let is_keyframe = self.frames.len() % KEYFRAME_INTERVAL == 0;
        let index_full = if is_keyframe { Some(indices) } else { None };

        self.frames.push_back(RewindFrame {
            snapshot_delta: Some(snapshot_delta),
            index_delta: Some(index_delta),
            index_full,
        });

        self.trim_to_capacity(capacity);
    }

    /// Rewinds by one frame and returns the reconstructed snapshot and index plane bytes.
    ///
    /// This performs an LZ4 decompression and a byte-wise XOR. On any decoding
    /// error, the rewind history is cleared and `None` is returned.
    pub fn rewind_frame(&mut self) -> Option<(NesSnapshot, Vec<u8>)> {
        if !self.can_rewind() {
            return None;
        }

        // Remove the current/latest frame. Its patches describe how to go from
        // the previous frame to the current frame.
        let current = self.frames.pop_back()?;
        let snapshot_compressed = current.snapshot_delta?;
        let index_compressed = current.index_delta?;

        let patch = decompress_size_prepended(&snapshot_compressed).ok()?;
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

        let xor = decompress_size_prepended(&index_compressed).ok()?;
        if xor.len() != self.current_index_bytes.len() {
            self.clear();
            return None;
        }
        for (b, x) in self.current_index_bytes.iter_mut().zip(xor.iter()) {
            *b ^= *x;
        }

        let snapshot = NesSnapshot::from_postcard_bytes(&self.current_full_bytes).ok()?;
        Some((snapshot, self.current_index_bytes.clone()))
    }

    /// Returns the number of frames currently stored in the history.
    #[inline]
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    /// Returns `true` if the history is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    /// Returns a reference to the current (latest) frame's index bytes.
    ///
    /// This is the framebuffer index plane of the most recent frame,
    /// useful for rendering the current frame without seeking.
    #[inline]
    pub fn current_index_bytes(&self) -> &[u8] {
        &self.current_index_bytes
    }

    /// Peeks at a frame at the given index (0 = oldest, len-1 = newest).
    ///
    /// This is a **read-only** operation that does NOT modify internal state.
    /// Uses keyframes for efficient O(KEYFRAME_INTERVAL) seeking.
    ///
    /// Returns the index plane bytes for the target frame, or `None` on error.
    pub fn peek_frame_at(&self, target_index: usize) -> Option<Vec<u8>> {
        if target_index >= self.frames.len() || self.current_index_bytes.is_empty() {
            return None;
        }

        // If target is the latest frame, just return a copy of current bytes.
        let latest_index = self.frames.len() - 1;
        if target_index == latest_index {
            return Some(self.current_index_bytes.clone());
        }

        // Find the nearest keyframe at or after target_index.
        // We search from target_index upwards to find a keyframe to start from.
        let (start_index, mut working_index) = self.find_nearest_keyframe(target_index)?;

        // Walk backwards from start_index to target_index (exclusive of target)
        // applying reverse patches.
        for frame_idx in (target_index + 1..=start_index).rev() {
            let frame = &self.frames[frame_idx];

            // Apply the reverse patch for index plane.
            let index_compressed = frame.index_delta.as_ref()?;
            let xor = decompress_size_prepended(index_compressed).ok()?;

            if xor.len() != working_index.len() {
                return None;
            }

            for (b, x) in working_index.iter_mut().zip(xor.iter()) {
                *b ^= *x;
            }
        }

        Some(working_index)
    }

    /// Finds the nearest keyframe at or after `target_index`.
    /// Returns (keyframe_index, keyframe_bytes) or None if not found.
    fn find_nearest_keyframe(&self, target_index: usize) -> Option<(usize, Vec<u8>)> {
        let latest_index = self.frames.len() - 1;

        // Search upwards from target_index to find the nearest keyframe
        for idx in target_index..=latest_index {
            if let Some(ref full_bytes) = self.frames[idx].index_full {
                return Some((idx, full_bytes.clone()));
            }
        }

        // If no keyframe found, fall back to using current_index_bytes (latest frame).
        // This should rarely happen since we store keyframes periodically.
        Some((latest_index, self.current_index_bytes.clone()))
    }

    /// Steps forward one frame from the given base buffer.
    ///
    /// This is O(1) - just one LZ4 decompress + XOR.
    /// Returns the new frame's index buffer, or None on error.
    ///
    /// `base_bytes`: The index buffer of frame at `base_index`
    /// `base_index`: Current frame index (0 = oldest)
    ///
    /// Returns the index buffer of frame `base_index + 1`.
    pub fn step_forward(&self, base_bytes: &[u8], base_index: usize) -> Option<Vec<u8>> {
        let next_index = base_index + 1;
        if next_index >= self.frames.len() {
            return None;
        }

        // To go forward: apply the delta of the NEXT frame.
        // The delta at frame[i] reconstructs frame[i-1] from frame[i].
        // So to go from frame[i] to frame[i+1], we need to REVERSE the operation:
        // frame[i+1] = frame[i] XOR delta[i+1]
        let frame = &self.frames[next_index];
        let index_compressed = frame.index_delta.as_ref()?;
        let xor = decompress_size_prepended(index_compressed).ok()?;

        if xor.len() != base_bytes.len() {
            return None;
        }

        let mut result = base_bytes.to_vec();
        for (b, x) in result.iter_mut().zip(xor.iter()) {
            *b ^= *x;
        }

        Some(result)
    }

    /// Steps backward one frame from the given base buffer.
    ///
    /// This is O(1) - just one LZ4 decompress + XOR.
    /// Returns the new frame's index buffer, or None on error.
    ///
    /// `base_bytes`: The index buffer of frame at `base_index`
    /// `base_index`: Current frame index (0 = oldest)
    ///
    /// Returns the index buffer of frame `base_index - 1`.
    pub fn step_backward(&self, base_bytes: &[u8], base_index: usize) -> Option<Vec<u8>> {
        if base_index == 0 {
            return None;
        }

        // To go backward: apply the delta of the CURRENT frame.
        // The delta at frame[i] reconstructs frame[i-1] from frame[i].
        // So: frame[i-1] = frame[i] XOR delta[i]
        let frame = &self.frames[base_index];
        let index_compressed = frame.index_delta.as_ref()?;
        let xor = decompress_size_prepended(index_compressed).ok()?;

        if xor.len() != base_bytes.len() {
            return None;
        }

        let mut result = base_bytes.to_vec();
        for (b, x) in result.iter_mut().zip(xor.iter()) {
            *b ^= *x;
        }

        Some(result)
    }

    /// Peeks at a snapshot at the given index (0 = oldest, len-1 = newest).
    ///
    /// This is a **read-only** operation that does NOT modify internal state.
    /// It reconstructs the target snapshot by walking backwards from the current frame.
    ///
    /// Returns the reconstructed machine snapshot, or `None` on error.
    /// Note: This is O(n) due to delta reconstruction.
    pub fn peek_snapshot_at(&self, target_index: usize) -> Option<NesSnapshot> {
        if target_index >= self.frames.len() || self.current_full_bytes.is_empty() {
            return None;
        }

        // If target is the latest frame, just return a copy of current bytes.
        let latest_index = self.frames.len() - 1;
        if target_index == latest_index {
            return NesSnapshot::from_postcard_bytes(&self.current_full_bytes).ok();
        }

        // We need to walk backwards from latest to target.
        let mut working_snapshot_bytes = self.current_full_bytes.clone();

        // Walk backwards from latest to target+1 (inclusive of frames we need to unapply)
        for frame_idx in (target_index + 1..=latest_index).rev() {
            let frame = &self.frames[frame_idx];

            let snapshot_compressed = frame.snapshot_delta.as_ref()?;
            let patch = decompress_size_prepended(snapshot_compressed).ok()?;
            if patch.len() < 8 {
                return None;
            }

            let prev_len = u32::from_le_bytes(patch[0..4].try_into().ok()?) as usize;
            let cur_len = u32::from_le_bytes(patch[4..8].try_into().ok()?) as usize;

            // The patch must match the working bytes length.
            if working_snapshot_bytes.len() != cur_len {
                return None;
            }

            let min_len = prev_len.min(cur_len);
            let xor_prefix_off = 8;
            let xor_prefix_end = xor_prefix_off + min_len;
            if patch.len() < xor_prefix_end {
                return None;
            }

            let prev_tail_len = prev_len.saturating_sub(min_len);
            let prev_tail_off = xor_prefix_end;
            let prev_tail_end = prev_tail_off + prev_tail_len;
            if patch.len() != prev_tail_end {
                return None;
            }

            // `prev_prefix[i] = cur[i] ^ xor_prefix[i]`
            for i in 0..min_len {
                working_snapshot_bytes[i] ^= patch[xor_prefix_off + i];
            }

            // Adjust the length to `prev_len`.
            if prev_len < cur_len {
                working_snapshot_bytes.truncate(prev_len);
            } else if prev_len > cur_len {
                working_snapshot_bytes.extend_from_slice(&patch[prev_tail_off..prev_tail_end]);
            }
        }

        NesSnapshot::from_postcard_bytes(&working_snapshot_bytes).ok()
    }

    /// Drops all rewind history and resets internal state.
    pub fn clear(&mut self) {
        self.frames.clear();
        self.current_full_bytes.clear();
        self.current_index_bytes.clear();
        self.first_frame_seq = 0;
    }

    /// Returns the absolute sequence number of the first frame in the buffer.
    #[inline]
    pub fn first_frame_seq(&self) -> u64 {
        self.first_frame_seq
    }

    fn trim_to_capacity(&mut self, capacity: usize) {
        while self.frames.len() > capacity {
            self.frames.pop_front();
            self.first_frame_seq += 1;
        }

        // If we dropped everything, also reset the current bytes.
        if self.frames.is_empty() {
            self.current_full_bytes.clear();
            self.current_index_bytes.clear();
            self.first_frame_seq = 0;
        }
    }
}
