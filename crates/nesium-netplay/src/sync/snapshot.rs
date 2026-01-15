//! Snapshot buffer for rollback synchronization.
//!
//! Manages a ring buffer of NES state snapshots to support rollback resimulation.

use std::collections::VecDeque;

/// A timestamped snapshot (frame, serialized state).
#[derive(Debug, Clone)]
pub struct FrameSnapshot {
    /// Frame number this snapshot was taken at.
    pub frame: u32,
    /// Serialized NES state.
    pub data: Vec<u8>,
}

/// Ring buffer of snapshots for rollback support.
///
/// Maintains the most recent N frames of state for efficient rollback.
#[derive(Debug)]
pub struct SnapshotBuffer {
    /// Stored snapshots (oldest first).
    snapshots: VecDeque<FrameSnapshot>,
    /// Maximum number of snapshots to keep.
    capacity: usize,
    /// Save frequency: save every Nth frame.
    save_interval: u32,
}

impl Default for SnapshotBuffer {
    fn default() -> Self {
        Self::new(120, 1) // ~2 seconds of snapshots at 60fps, every frame
    }
}

impl SnapshotBuffer {
    /// Create a new snapshot buffer.
    ///
    /// # Arguments
    /// - `capacity`: Maximum number of snapshots to store
    /// - `save_interval`: Save a snapshot every N frames (1 = every frame)
    pub fn new(capacity: usize, save_interval: u32) -> Self {
        Self {
            snapshots: VecDeque::with_capacity(capacity),
            capacity,
            save_interval: save_interval.max(1),
        }
    }

    /// Check if we should save a snapshot for this frame.
    pub fn should_save(&self, frame: u32) -> bool {
        frame % self.save_interval == 0
    }

    /// Push a new snapshot. Old snapshots are evicted if capacity is exceeded.
    pub fn push(&mut self, frame: u32, data: Vec<u8>) {
        // Evict oldest if at capacity
        while self.snapshots.len() >= self.capacity {
            self.snapshots.pop_front();
        }
        self.snapshots.push_back(FrameSnapshot { frame, data });
    }

    /// Find the snapshot closest to but not after the target frame.
    ///
    /// Returns `None` if no suitable snapshot exists.
    pub fn find_before(&self, target_frame: u32) -> Option<&FrameSnapshot> {
        self.snapshots
            .iter()
            .rev()
            .find(|s| s.frame <= target_frame)
    }

    /// Get the most recent snapshot.
    pub fn latest(&self) -> Option<&FrameSnapshot> {
        self.snapshots.back()
    }

    /// Clear all snapshots (e.g., on game reset).
    pub fn clear(&mut self) {
        self.snapshots.clear();
    }

    /// Get the number of stored snapshots.
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    /// Check if the buffer is empty.
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    /// Get the oldest frame in the buffer.
    pub fn oldest_frame(&self) -> Option<u32> {
        self.snapshots.front().map(|s| s.frame)
    }

    /// Get the newest frame in the buffer.
    pub fn newest_frame(&self) -> Option<u32> {
        self.snapshots.back().map(|s| s.frame)
    }

    /// Discard snapshots older than the given frame.
    pub fn prune_before(&mut self, frame: u32) {
        while let Some(oldest) = self.snapshots.front() {
            if oldest.frame < frame {
                self.snapshots.pop_front();
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_buffer_basic() {
        let mut buf = SnapshotBuffer::new(5, 1);
        assert!(buf.is_empty());

        // Push 5 snapshots
        for f in 0..5 {
            buf.push(f, vec![f as u8]);
        }
        assert_eq!(buf.len(), 5);

        // Push 6th, should evict oldest
        buf.push(5, vec![5]);
        assert_eq!(buf.len(), 5);
        assert_eq!(buf.oldest_frame(), Some(1));
        assert_eq!(buf.newest_frame(), Some(5));
    }

    #[test]
    fn snapshot_find_before() {
        let mut buf = SnapshotBuffer::new(10, 1);
        for f in [0, 2, 4, 6, 8] {
            buf.push(f, vec![f as u8]);
        }

        // Find snapshot for frame 5 -> should return frame 4
        let snap = buf.find_before(5);
        assert!(snap.is_some());
        assert_eq!(snap.unwrap().frame, 4);

        // Find snapshot for frame 4 -> should return frame 4
        let snap = buf.find_before(4);
        assert_eq!(snap.unwrap().frame, 4);

        // Find snapshot for frame 1 -> should return frame 0
        let snap = buf.find_before(1);
        assert_eq!(snap.unwrap().frame, 0);
    }

    #[test]
    fn snapshot_save_interval() {
        let buf = SnapshotBuffer::new(10, 3);
        assert!(buf.should_save(0));
        assert!(!buf.should_save(1));
        assert!(!buf.should_save(2));
        assert!(buf.should_save(3));
        assert!(buf.should_save(6));
    }
}
