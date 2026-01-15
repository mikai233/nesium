//! Lockstep synchronization strategy.
//!
//! This strategy waits for all players' inputs before advancing each frame.
//! Best for low-latency networks where waiting is acceptable.

use std::collections::BTreeMap;

use super::{RollbackRequest, SyncMode, SyncStrategy};

/// Lockstep synchronization implementation.
///
/// Blocks frame advancement until all active ports have confirmed inputs.
#[derive(Debug)]
pub struct LockstepSync {
    /// Input queues per port (frame -> buttons).
    input_queues: [BTreeMap<u32, u16>; 4],
    /// Which ports are active (have players).
    active_ports: [bool; 4],
    /// Configured input delay in frames.
    input_delay: u32,
    /// Last consumed frame per port (for cleanup).
    last_consumed: [u32; 4],
}

impl Default for LockstepSync {
    fn default() -> Self {
        Self::new(2)
    }
}

impl LockstepSync {
    /// Create a new lockstep sync with the given input delay.
    pub fn new(input_delay: u32) -> Self {
        Self {
            input_queues: Default::default(),
            active_ports: [false; 4],
            input_delay,
            last_consumed: [0; 4],
        }
    }

    /// Set input delay.
    pub fn set_input_delay(&mut self, delay: u32) {
        self.input_delay = delay;
    }

    /// Prune old inputs to prevent unbounded memory growth.
    fn prune_old_inputs(&mut self, before_frame: u32) {
        for queue in &mut self.input_queues {
            queue.retain(|&f, _| f >= before_frame.saturating_sub(16));
        }
    }
}

impl SyncStrategy for LockstepSync {
    fn inputs_for_frame(&mut self, frame: u32) -> Option<[u16; 4]> {
        if !self.can_advance(frame) {
            return None;
        }

        let mut inputs = [0u16; 4];
        for (i, queue) in self.input_queues.iter_mut().enumerate() {
            if self.active_ports[i] {
                inputs[i] = queue.remove(&frame).unwrap_or(0);
                self.last_consumed[i] = frame;
            }
        }

        // Prune old inputs periodically
        if frame % 60 == 0 {
            self.prune_old_inputs(frame);
        }

        Some(inputs)
    }

    fn can_advance(&self, frame: u32) -> bool {
        for (i, &active) in self.active_ports.iter().enumerate() {
            if active && !self.input_queues[i].contains_key(&frame) {
                return false;
            }
        }
        true
    }

    fn on_local_input(&mut self, player: u8, frame: u32, buttons: u16) {
        if (player as usize) < 4 {
            self.input_queues[player as usize].insert(frame, buttons);
        }
    }

    fn on_remote_input(&mut self, player: u8, frame: u32, buttons: u16) {
        if (player as usize) < 4 {
            self.input_queues[player as usize].insert(frame, buttons);
        }
    }

    fn pending_rollback(&self) -> Option<RollbackRequest> {
        None // Lockstep never needs rollback
    }

    fn clear_rollback(&mut self) {
        // No-op for lockstep
    }

    fn should_fast_forward(&self, current_frame: u32) -> bool {
        // Fast forward if we have inputs buffered significantly ahead
        let threshold = self.input_delay + 2;
        for (i, &active) in self.active_ports.iter().enumerate() {
            if active {
                let max_frame = self.input_queues[i]
                    .keys()
                    .next_back()
                    .copied()
                    .unwrap_or(0);
                if max_frame > current_frame + threshold {
                    return true;
                }
            }
        }
        false
    }

    fn set_port_active(&mut self, port: usize, active: bool) {
        if port < 4 {
            self.active_ports[port] = active;
            if !active {
                self.input_queues[port].clear();
            }
        }
    }

    fn mode(&self) -> SyncMode {
        SyncMode::Lockstep
    }

    fn clear(&mut self) {
        for queue in &mut self.input_queues {
            queue.clear();
        }
        self.last_consumed = [0; 4];
    }

    fn last_confirmed_frame(&self) -> u32 {
        let mut min_confirmed = u32::MAX;
        let mut any_active = false;

        for (i, &active) in self.active_ports.iter().enumerate() {
            if active {
                any_active = true;
                // Find largest frame F such that all 0..=F exist or were consumed.
                // We check existing keys in BTreeMap.
                // Start from last_consumed[i] or 0.
                let mut current = self.last_consumed[i];
                while self.input_queues[i].contains_key(&current) {
                    current += 1;
                }
                let confirmed = current.saturating_sub(1);
                min_confirmed = min_confirmed.min(confirmed);
            }
        }

        if any_active && min_confirmed != u32::MAX {
            min_confirmed
        } else {
            0
        }
    }

    fn set_input_delay(&mut self, delay: u32) {
        self.input_delay = delay;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== can_advance 测试 ====================

    #[test]
    fn lockstep_blocks_without_inputs() {
        let mut sync = LockstepSync::new(2);
        sync.set_port_active(0, true);
        sync.set_port_active(1, true);

        assert!(!sync.can_advance(0));
    }

    #[test]
    fn lockstep_advances_with_all_inputs() {
        let mut sync = LockstepSync::new(2);
        sync.set_port_active(0, true);
        sync.set_port_active(1, true);

        sync.on_local_input(0, 0, 0x01);
        assert!(!sync.can_advance(0)); // Still missing player 1

        sync.on_remote_input(1, 0, 0x02);
        assert!(sync.can_advance(0)); // Now ready

        let inputs = sync.inputs_for_frame(0);
        assert_eq!(inputs, Some([0x01, 0x02, 0, 0]));
    }

    #[test]
    fn inactive_ports_ignored() {
        let mut sync = LockstepSync::new(2);
        sync.set_port_active(0, true);
        // Port 1, 2, 3 are inactive

        sync.on_local_input(0, 0, 0xFF);
        assert!(sync.can_advance(0));
    }

    #[test]
    fn can_advance_with_no_active_ports() {
        let sync = LockstepSync::new(2);
        // No ports active - should always be able to advance
        assert!(sync.can_advance(0));
        assert!(sync.can_advance(100));
    }

    #[test]
    fn can_advance_checks_correct_frame() {
        let mut sync = LockstepSync::new(2);
        sync.set_port_active(0, true);

        // Input for frame 5
        sync.on_local_input(0, 5, 0x01);

        // Can't advance frame 0-4
        assert!(!sync.can_advance(0));
        assert!(!sync.can_advance(4));
        // Can advance frame 5
        assert!(sync.can_advance(5));
        // Can't advance frame 6
        assert!(!sync.can_advance(6));
    }

    // ==================== inputs_for_frame 测试 ====================

    #[test]
    fn inputs_for_frame_returns_none_when_not_ready() {
        let mut sync = LockstepSync::new(2);
        sync.set_port_active(0, true);
        sync.set_port_active(1, true);

        sync.on_local_input(0, 0, 0x01);
        // Missing input for port 1
        assert!(sync.inputs_for_frame(0).is_none());
    }

    #[test]
    fn inputs_for_frame_consumes_inputs() {
        let mut sync = LockstepSync::new(2);
        sync.set_port_active(0, true);

        sync.on_local_input(0, 0, 0x01);
        assert!(sync.can_advance(0));

        let inputs = sync.inputs_for_frame(0);
        assert_eq!(inputs, Some([0x01, 0, 0, 0]));

        // After consuming, should not be able to advance same frame again
        assert!(!sync.can_advance(0));
    }

    #[test]
    fn inputs_for_frame_handles_multiple_frames() {
        let mut sync = LockstepSync::new(2);
        sync.set_port_active(0, true);
        sync.set_port_active(1, true);

        // Queue up inputs for frames 0, 1, 2
        for f in 0..3 {
            sync.on_local_input(0, f, (f + 1) as u16);
            sync.on_remote_input(1, f, ((f + 1) * 10) as u16);
        }

        // Consume in order
        assert_eq!(sync.inputs_for_frame(0), Some([1, 10, 0, 0]));
        assert_eq!(sync.inputs_for_frame(1), Some([2, 20, 0, 0]));
        assert_eq!(sync.inputs_for_frame(2), Some([3, 30, 0, 0]));
    }

    // ==================== on_local_input / on_remote_input 测试 ====================

    #[test]
    fn input_stored_correctly() {
        let mut sync = LockstepSync::new(2);
        sync.set_port_active(0, true);
        sync.set_port_active(1, true);
        sync.set_port_active(2, true);
        sync.set_port_active(3, true);

        sync.on_local_input(0, 10, 0xAA);
        sync.on_remote_input(1, 10, 0xBB);
        sync.on_local_input(2, 10, 0xCC);
        sync.on_remote_input(3, 10, 0xDD);

        let inputs = sync.inputs_for_frame(10);
        assert_eq!(inputs, Some([0xAA, 0xBB, 0xCC, 0xDD]));
    }

    #[test]
    fn input_overwrite_same_frame() {
        let mut sync = LockstepSync::new(2);
        sync.set_port_active(0, true);

        sync.on_local_input(0, 5, 0x01);
        sync.on_local_input(0, 5, 0x02); // Overwrite

        let inputs = sync.inputs_for_frame(5);
        assert_eq!(inputs, Some([0x02, 0, 0, 0]));
    }

    #[test]
    fn input_invalid_port_ignored() {
        let mut sync = LockstepSync::new(2);
        sync.set_port_active(0, true);

        sync.on_local_input(0, 0, 0x01);
        sync.on_local_input(4, 0, 0xFF); // Invalid port 4
        sync.on_remote_input(10, 0, 0xFF); // Invalid port 10

        assert!(sync.can_advance(0));
        let inputs = sync.inputs_for_frame(0);
        assert_eq!(inputs, Some([0x01, 0, 0, 0]));
    }

    // ==================== set_port_active 测试 ====================

    #[test]
    fn set_port_active_clears_queue_on_deactivation() {
        let mut sync = LockstepSync::new(2);
        sync.set_port_active(0, true);

        sync.on_local_input(0, 0, 0x01);
        sync.on_local_input(0, 1, 0x02);
        sync.on_local_input(0, 2, 0x03);

        // Deactivate clears the queue
        sync.set_port_active(0, false);

        // Re-activate
        sync.set_port_active(0, true);

        // Queue should be empty now
        assert!(!sync.can_advance(0));
    }

    #[test]
    fn set_port_active_dynamic_change() {
        let mut sync = LockstepSync::new(2);
        sync.set_port_active(0, true);
        sync.set_port_active(1, true);

        sync.on_local_input(0, 0, 0x01);
        // Can't advance - missing port 1
        assert!(!sync.can_advance(0));

        // Player 2 disconnects
        sync.set_port_active(1, false);

        // Now we can advance with just port 0
        assert!(sync.can_advance(0));
    }

    // ==================== should_fast_forward 测试 ====================

    #[test]
    fn fast_forward_when_inputs_ahead() {
        let mut sync = LockstepSync::new(2);
        sync.set_port_active(0, true);

        // Buffer 10 frames of input
        for f in 0..10 {
            sync.on_local_input(0, f, 0x01);
        }

        // At frame 0 with delay=2, threshold=4, we have inputs up to frame 9
        // 9 > 0 + 4 = 4, so should fast forward
        assert!(sync.should_fast_forward(0));
    }

    #[test]
    fn no_fast_forward_when_caught_up() {
        let mut sync = LockstepSync::new(2);
        sync.set_port_active(0, true);

        sync.on_local_input(0, 0, 0x01);
        sync.on_local_input(0, 1, 0x02);

        // At frame 0 with only inputs up to frame 1
        // 1 is not > 0 + 4 = 4, so should not fast forward
        assert!(!sync.should_fast_forward(0));
    }

    #[test]
    fn fast_forward_considers_all_active_ports() {
        let mut sync = LockstepSync::new(2);
        sync.set_port_active(0, true);
        sync.set_port_active(1, true);

        // Port 0 has many inputs
        for f in 0..20 {
            sync.on_local_input(0, f, 0x01);
        }
        // Port 1 only has a few
        for f in 0..3 {
            sync.on_remote_input(1, f, 0x02);
        }

        // Even though port 0 could fast-forward, port 1 triggers the check
        // max_frame for port 0 = 19, 19 > 0 + 4 = 4 → true
        assert!(sync.should_fast_forward(0));
    }

    // ==================== clear 测试 ====================

    #[test]
    fn clear_resets_all_queues() {
        let mut sync = LockstepSync::new(2);
        sync.set_port_active(0, true);
        sync.set_port_active(1, true);

        for f in 0..10 {
            sync.on_local_input(0, f, 0x01);
            sync.on_remote_input(1, f, 0x02);
        }

        sync.clear();

        // All queues should be empty
        assert!(!sync.can_advance(0));
        assert!(!sync.can_advance(5));
    }

    // ==================== mode 测试 ====================

    #[test]
    fn mode_returns_lockstep() {
        let sync = LockstepSync::new(2);
        assert_eq!(sync.mode(), SyncMode::Lockstep);
    }

    // ==================== pending_rollback 测试 ====================

    #[test]
    fn lockstep_never_requests_rollback() {
        let mut sync = LockstepSync::new(2);
        sync.set_port_active(0, true);
        sync.set_port_active(1, true);

        // Simulate some frames
        for f in 0..10 {
            sync.on_local_input(0, f, 0x01);
            sync.on_remote_input(1, f, 0x02);
            let _ = sync.inputs_for_frame(f);
        }

        assert!(sync.pending_rollback().is_none());
    }

    // ==================== prune_old_inputs 测试 ====================

    #[test]
    fn prune_old_inputs_on_frame_boundary() {
        let mut sync = LockstepSync::new(2);
        sync.set_port_active(0, true);

        // Add inputs for frames 0-100
        for f in 0..100 {
            sync.on_local_input(0, f, 0x01);
        }

        // Consume up to frame 59
        for f in 0..60 {
            let _ = sync.inputs_for_frame(f);
        }

        // Frame 60 triggers prune (frame % 60 == 0)
        let _ = sync.inputs_for_frame(60);

        // Old frames should be pruned (keeping only from frame 60-16=44 onwards)
        // But since we consumed them, they're already gone
        // Frame 61 should still exist
        assert!(sync.can_advance(61));
    }
}
