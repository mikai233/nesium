//! Rollback synchronization strategy.
//!
//! This strategy predicts remote inputs and rolls back on misprediction.
//! Best for high-latency networks where waiting would cause noticeable lag.

use std::collections::HashMap;

use super::{RollbackRequest, SyncMode, SyncStrategy};

/// Rollback synchronization implementation.
///
/// Predicts remote inputs (repeats last known input) and triggers rollback
/// when confirmed inputs differ from predictions.
#[derive(Debug)]
pub struct RollbackSync {
    /// Confirmed inputs per port (frame -> buttons).
    confirmed_inputs: [HashMap<u32, u16>; 4],
    /// Last known input per port (for prediction).
    last_inputs: [u16; 4],
    /// Which ports are active.
    active_ports: [bool; 4],
    /// Last confirmed frame (all ports have confirmed up to this frame).
    last_confirmed_frame: u32,
    /// Current predicted frame.
    current_frame: u32,
    /// Pending rollback request if prediction was wrong.
    pending_rollback: Option<RollbackRequest>,
    /// Local player index.
    local_player: Option<u8>,
    /// Input delay frames.
    input_delay: u32,
}

impl Default for RollbackSync {
    fn default() -> Self {
        Self::new(2)
    }
}

impl RollbackSync {
    /// Create a new rollback sync with the given input delay.
    pub fn new(input_delay: u32) -> Self {
        Self {
            confirmed_inputs: Default::default(),
            last_inputs: [0; 4],
            active_ports: [false; 4],
            last_confirmed_frame: 0,
            current_frame: 0,
            pending_rollback: None,
            local_player: None,
            input_delay,
        }
    }

    /// Set the local player index.
    pub fn set_local_player(&mut self, player: Option<u8>) {
        self.local_player = player;
    }

    /// Set input delay.
    pub fn set_input_delay(&mut self, delay: u32) {
        self.input_delay = delay;
    }

    /// Get predicted input for a remote port at a specific frame.
    fn predict_input(&self, port: usize, frame: u32) -> u16 {
        // Use confirmed input if available
        if let Some(&buttons) = self.confirmed_inputs[port].get(&frame) {
            return buttons;
        }
        // Otherwise predict using last known input
        self.last_inputs[port]
    }

    /// Update the last confirmed frame.
    fn update_confirmed_frame(&mut self) {
        let mut min_confirmed = u32::MAX;
        let mut any_active = false;

        for (i, &active) in self.active_ports.iter().enumerate() {
            if active {
                any_active = true;
                // Find largest frame F such that all 0..=F exist for this port.
                // Since confirmed_inputs is a HashMap, we check sequentially.
                // Optimization: start from last_confirmed_frame.
                let mut current = self.last_confirmed_frame;
                while self.confirmed_inputs[i].contains_key(&current) {
                    current += 1;
                }
                // 'current' is the first MISSING frame, so 'current - 1' is confirmed.
                let confirmed = current.saturating_sub(1);
                min_confirmed = min_confirmed.min(confirmed);
            }
        }

        if any_active && min_confirmed != u32::MAX {
            self.last_confirmed_frame = min_confirmed;
        }
    }

    /// Prune old confirmed inputs.
    fn prune_old_inputs(&mut self, keep_from: u32) {
        for inputs in &mut self.confirmed_inputs {
            inputs.retain(|&f, _| f >= keep_from.saturating_sub(120));
        }
    }
}

impl SyncStrategy for RollbackSync {
    fn inputs_for_frame(&mut self, frame: u32) -> Option<[u16; 4]> {
        self.current_frame = frame;

        let mut inputs = [0u16; 4];
        for (i, &active) in self.active_ports.iter().enumerate() {
            if active {
                inputs[i] = self.predict_input(i, frame);
            }
        }

        // Prune periodically
        if frame % 60 == 0 {
            self.prune_old_inputs(self.last_confirmed_frame);
        }

        Some(inputs)
    }

    fn can_advance(&self, _frame: u32) -> bool {
        // Rollback mode can always advance using predictions
        true
    }

    fn on_local_input(&mut self, player: u8, frame: u32, buttons: u16) {
        let port = player as usize;
        if port >= 4 {
            return;
        }

        // Ignore inputs for frames that are already finalized and pruned
        if frame < self.last_confirmed_frame.saturating_sub(120) {
            return;
        }

        self.confirmed_inputs[port].insert(frame, buttons);
        self.last_inputs[port] = buttons;
        self.update_confirmed_frame();
    }

    fn on_remote_input(&mut self, player: u8, frame: u32, buttons: u16) {
        let port = player as usize;
        if port >= 4 {
            return;
        }

        // Ignore inputs for frames that are already finalized and pruned
        if frame < self.last_confirmed_frame.saturating_sub(120) {
            return;
        }

        // Check if this differs from our prediction (if we already simulated this frame)
        if frame < self.current_frame {
            let predicted = self.predict_input(port, frame);
            if predicted != buttons {
                // Prediction was wrong, need to rollback
                let target = match &self.pending_rollback {
                    Some(rb) => rb.target_frame.min(frame),
                    None => frame,
                };
                self.pending_rollback = Some(RollbackRequest {
                    target_frame: target,
                    current_frame: self.current_frame,
                });
            }
        }

        // Store confirmed input
        self.confirmed_inputs[port].insert(frame, buttons);
        self.last_inputs[port] = buttons;
        self.update_confirmed_frame();
    }

    fn pending_rollback(&self) -> Option<RollbackRequest> {
        self.pending_rollback.clone()
    }

    fn clear_rollback(&mut self) {
        self.pending_rollback = None;
    }

    fn should_fast_forward(&self, current_frame: u32) -> bool {
        // Fast forward if we're behind confirmed frame significantly
        self.last_confirmed_frame > current_frame + self.input_delay + 2
    }

    fn set_port_active(&mut self, port: usize, active: bool) {
        if port < 4 {
            self.active_ports[port] = active;
            if !active {
                self.confirmed_inputs[port].clear();
                self.last_inputs[port] = 0;
            }
        }
    }

    fn mode(&self) -> SyncMode {
        SyncMode::Rollback
    }

    fn clear(&mut self) {
        self.confirmed_inputs = Default::default();
        self.last_inputs = [0; 4];
        self.last_confirmed_frame = 0;
        self.current_frame = 0;
        self.pending_rollback = None;
    }

    fn last_confirmed_frame(&self) -> u32 {
        self.last_confirmed_frame
    }

    fn set_input_delay(&mut self, delay: u32) {
        self.input_delay = delay;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rollback_properties() {
        let sync = RollbackSync::new(2);
        assert_eq!(sync.mode(), SyncMode::Rollback);
        assert!(sync.can_advance(0));
        assert!(sync.can_advance(100));
    }

    #[test]
    fn rollback_uses_prediction() {
        let mut sync = RollbackSync::new(2);
        sync.set_port_active(0, true);
        sync.set_port_active(1, true);

        // Only local input available for frame 0
        sync.on_local_input(0, 0, 0x01);

        // Frame 0: port 0 confirmed, port 1 predicted (0)
        let inputs = sync.inputs_for_frame(0).unwrap();
        assert_eq!(inputs[0], 0x01);
        assert_eq!(inputs[1], 0);

        // Simulate frame 1: port 0 uses new local input, port 1 still predicts (0)
        sync.on_local_input(0, 1, 0x02);
        let inputs = sync.inputs_for_frame(1).unwrap();
        assert_eq!(inputs[0], 0x02);
        assert_eq!(inputs[1], 0);

        // Now remote input arrives for frame 0
        sync.on_remote_input(1, 0, 0xAA);
        // last_inputs for port 1 should now be 0xAA

        // Frame 2: port 1 should predict 0xAA
        let inputs = sync.inputs_for_frame(2).unwrap();
        assert_eq!(inputs[1], 0xAA);
    }

    #[test]
    fn rollback_triggers_on_misprediction() {
        let mut sync = RollbackSync::new(2);
        sync.set_port_active(0, true);
        sync.set_port_active(1, true);

        // Step 1: Advance frames with predictions
        for f in 0..5 {
            sync.on_local_input(0, f, 0x01);
            let _ = sync.inputs_for_frame(f);
        }
        // current_frame is now 4 (it was set in inputs_for_frame(4))

        // Step 2: Receive remote input that matches prediction -> No rollback
        sync.on_remote_input(1, 0, 0);
        assert!(sync.pending_rollback().is_none());

        // Step 3: Receive remote input that DIFFERS from prediction in the past
        sync.on_remote_input(1, 2, 0xFF);
        let rollback = sync.pending_rollback();
        assert!(rollback.is_some());
        assert_eq!(rollback.unwrap().target_frame, 2);

        // Step 4: Clear rollback
        sync.clear_rollback();
        assert!(sync.pending_rollback().is_none());
    }

    #[test]
    fn rollback_not_triggered_for_future_frames() {
        let mut sync = RollbackSync::new(2);
        sync.set_port_active(0, true);
        sync.set_port_active(1, true);

        // At frame 0
        sync.inputs_for_frame(0);

        // Receive input for frame 10 (future)
        // Even if it differs from current last_inputs, it shouldn't trigger rollback
        // because we haven't simulated frame 10 yet.
        sync.on_remote_input(1, 10, 0xBB);
        assert!(sync.pending_rollback().is_none());
    }

    #[test]
    fn last_confirmed_frame_calculation() {
        let mut sync = RollbackSync::new(0);
        sync.set_port_active(0, true);
        sync.set_port_active(1, true);

        // Provide inputs for 0..10 and 0..5
        for f in 0..=10 {
            sync.on_local_input(0, f, 0x01);
        }
        for f in 0..=5 {
            sync.on_remote_input(1, f, 0x02);
        }

        // Last confirmed should be min(10, 5) = 5
        assert_eq!(sync.last_confirmed_frame, 5);

        for f in 6..=12 {
            sync.on_remote_input(1, f, 0x02);
        }
        // Last confirmed should be min(10, 12) = 10
        assert_eq!(sync.last_confirmed_frame, 10);
    }

    #[test]
    fn fast_forward_threshold() {
        let mut sync = RollbackSync::new(2);
        sync.set_port_active(0, true);
        sync.set_port_active(1, true);

        // current_frame = 0, input_delay = 2, threshold = 0 + 2 + 2 = 4
        sync.inputs_for_frame(0);

        // current_frame = 0, input_delay = 2, threshold = 0 + 2 + 2 = 4
        sync.inputs_for_frame(0);

        // Both players confirm up to frame 3
        for f in 0..=3 {
            sync.on_local_input(0, f, 1);
            sync.on_remote_input(1, f, 1);
        }
        // 3 is not > 0 + 2 + 2 = 4
        assert!(!sync.should_fast_forward(0));

        // Both players confirm up to frame 5
        for f in 4..=5 {
            sync.on_local_input(0, f, 1);
            sync.on_remote_input(1, f, 1);
        }
        // 5 is > 4
        assert!(sync.should_fast_forward(0));
    }

    #[test]
    fn set_port_active_clears_data() {
        let mut sync = RollbackSync::new(2);
        sync.set_port_active(0, true);
        sync.on_local_input(0, 10, 0xFF);
        assert_eq!(sync.last_inputs[0], 0xFF);
        assert!(!sync.confirmed_inputs[0].is_empty());

        sync.set_port_active(0, false);
        assert_eq!(sync.last_inputs[0], 0);
        assert!(sync.confirmed_inputs[0].is_empty());
    }

    #[test]
    fn clear_resets_everything() {
        let mut sync = RollbackSync::new(2);
        sync.set_port_active(0, true);
        sync.on_local_input(0, 5, 0x01);
        sync.inputs_for_frame(5);
        sync.on_remote_input(1, 2, 0x02); // Trigger rollback

        sync.clear();
        assert_eq!(sync.last_inputs, [0; 4]);
        assert_eq!(sync.last_confirmed_frame, 0);
        assert_eq!(sync.current_frame, 0);
        assert!(sync.pending_rollback().is_none());
        for i in 0..4 {
            assert!(sync.confirmed_inputs[i].is_empty());
        }
    }

    #[test]
    fn prune_old_inputs_logic() {
        let mut sync = RollbackSync::new(0);
        sync.set_port_active(0, true);

        // Fill some inputs
        for f in 0..200 {
            sync.on_local_input(0, f, 0x01);
        }

        // Keep from 150 (prunes before 150 - 120 = 30)
        sync.prune_old_inputs(150);

        assert!(!sync.confirmed_inputs[0].contains_key(&20));
        assert!(sync.confirmed_inputs[0].contains_key(&40));
        assert!(sync.confirmed_inputs[0].contains_key(&199));
    }

    #[test]
    fn invalid_port_handling() {
        let mut sync = RollbackSync::new(0);
        sync.on_local_input(4, 0, 1); // Should be ignored
        sync.on_remote_input(4, 0, 1); // Should be ignored

        // Check that it didn't crash or modify anything
        for i in 0..4 {
            assert!(sync.confirmed_inputs[i].is_empty());
            assert_eq!(sync.last_inputs[i], 0);
        }
    }

    #[test]
    fn input_holes_confirmed_frame() {
        let mut sync = RollbackSync::new(0);
        sync.set_port_active(0, true);
        sync.set_port_active(1, true);

        // P1 confirms 0, 1, 2
        sync.on_local_input(0, 0, 1);
        sync.on_local_input(0, 1, 1);
        sync.on_local_input(0, 2, 1);

        // P2 confirms 0, 1, then MISSES 2, but has 3
        sync.on_remote_input(1, 0, 1);
        sync.on_remote_input(1, 1, 1);
        sync.on_remote_input(1, 3, 1); // Frame 2 is a hole

        // confirmed_frame should be 1, because 2 is missing on P2
        // If current buggy logic remains, it might be 2 (min of max(P1=2) and max(P2=3))
        assert_eq!(
            sync.last_confirmed_frame, 1,
            "Confirmed frame should stop at the hole"
        );
    }

    #[test]
    fn overlapping_rollbacks() {
        let mut sync = RollbackSync::new(0);
        sync.set_port_active(0, true);
        sync.set_port_active(1, true);

        // Advance to frame 10
        for f in 0..=10 {
            sync.on_local_input(0, f, 0x01);
            sync.inputs_for_frame(f);
        }

        // Receive remote input for frame 8 (let's say we predicted 0, but it's 2)
        // This triggers a rollback to 8
        sync.on_remote_input(1, 8, 0x02);
        let rb = sync.pending_rollback().unwrap();
        assert_eq!(rb.target_frame, 8);

        // This should override the target frame to 5 because it's earlier
        sync.on_remote_input(1, 5, 0x03);
        let rb = sync.pending_rollback().unwrap();
        assert_eq!(rb.target_frame, 5);
    }

    #[test]
    fn rollback_pruning_safety() {
        let mut sync = RollbackSync::new(0);
        sync.set_port_active(0, true);
        sync.set_port_active(1, true);

        // 1. Fill inputs up to frame 200 for both players
        for f in 0..=200 {
            sync.on_local_input(0, f, 0x01);
            sync.on_remote_input(1, f, 0x02);
            sync.inputs_for_frame(f);
        }

        // confirmed_frame should be 200
        assert_eq!(sync.last_confirmed_frame, 200);

        // 2. Prune old inputs (keeps from 200-120 = 80)
        sync.prune_old_inputs(200);
        assert!(!sync.confirmed_inputs[0].contains_key(&50));

        // 3. Receive a "late" remote input for frame 50 (already pruned)
        // This input differs from what we had (was 0x02, now 0x03)
        // Since it's pruned, it should be ignored and NOT trigger a rollback
        sync.on_remote_input(1, 50, 0x03);

        assert!(
            sync.pending_rollback().is_none(),
            "Should not trigger rollback for pruned frame"
        );
    }
}
