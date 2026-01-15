//! Synchronization strategies for netplay.
//!
//! This module provides two sync modes:
//! - **Lockstep**: Wait for all players' inputs before advancing (low latency networks)
//! - **Rollback**: Predict inputs and rollback on misprediction (high latency networks)

pub mod lockstep;
pub mod rollback;
pub mod snapshot;

use serde::{Deserialize, Serialize};

/// Synchronization mode for netplay sessions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SyncMode {
    /// Wait for all players' confirmed inputs before advancing each frame.
    /// Best for low-latency networks (LAN, same region).
    #[default]
    Lockstep,
    /// Predict remote inputs and rollback/resimulate on misprediction.
    /// Best for high-latency networks (cross-region, internet).
    Rollback,
}

/// Request to rollback to a specific frame and resimulate.
#[derive(Debug, Clone)]
pub struct RollbackRequest {
    /// Target frame to restore snapshot from.
    pub target_frame: u32,
    /// Current frame before rollback.
    pub current_frame: u32,
}

/// Strategy trait for different synchronization modes.
///
/// Implementations handle input buffering, prediction, and rollback detection.
pub trait SyncStrategy: Send + Sync {
    /// Get inputs for the given frame.
    ///
    /// For Lockstep: returns confirmed inputs or None if waiting.
    /// For Rollback: returns confirmed or predicted inputs, never None.
    fn inputs_for_frame(&mut self, frame: u32) -> Option<[u16; 4]>;

    /// Check if the given frame can be advanced.
    ///
    /// For Lockstep: true only if all active ports have confirmed inputs.
    /// For Rollback: always true (uses prediction).
    fn can_advance(&self, frame: u32) -> bool;

    /// Called when local input is generated.
    fn on_local_input(&mut self, player: u8, frame: u32, buttons: u16);

    /// Called when remote confirmed input is received from network.
    ///
    /// For Rollback: may trigger rollback if prediction was wrong.
    fn on_remote_input(&mut self, player: u8, frame: u32, buttons: u16);

    /// Check if a rollback is pending.
    ///
    /// For Lockstep: always None.
    /// For Rollback: Some if prediction mismatch detected.
    fn pending_rollback(&self) -> Option<RollbackRequest>;

    /// Clear the pending rollback after it has been processed.
    fn clear_rollback(&mut self);

    /// Check if we should fast-forward to catch up.
    fn should_fast_forward(&self, current_frame: u32) -> bool;

    /// Mark a port as active/inactive.
    fn set_port_active(&mut self, port: usize, active: bool);

    /// Get the current sync mode.
    fn mode(&self) -> SyncMode;

    /// Clear all buffered inputs (e.g., on game reset).
    fn clear(&mut self);

    /// Get the largest frame for which all players have confirmed inputs.
    fn last_confirmed_frame(&self) -> u32;

    /// Set the input delay in frames.
    fn set_input_delay(&mut self, delay: u32);
}
