//! Input provider interface for netplay.
//!
//! This trait allows the NES runtime to fetch controller inputs
//! from the network instead of the local UI.

use parking_lot::Mutex;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU32, Ordering},
};

use crate::session::NetplaySession;
use crate::sync::{SyncMode, SyncStrategy, lockstep::LockstepSync};

fn frame_offset(session: &NetplaySession) -> u32 {
    match session.state {
        crate::session::SessionState::Playing { start_frame, .. } => start_frame,
        crate::session::SessionState::Spectating { start_frame } => start_frame,
        _ => 0,
    }
}

/// Trait for providing controller inputs from the network.
///
/// Implementations of this trait bridge the netplay session
/// with the NES runtime's input system.
pub trait NetplayInputProvider: Send + Sync {
    /// Poll inputs for the given frame.
    ///
    /// Returns `Some([buttons; 4])` if inputs are available for all ports,
    /// or `None` if we need to wait for network data.
    fn poll_inputs(&self, frame: u32) -> Option<[u16; 4]>;

    /// Submit local input for the local player.
    fn submit_local_input(&self, pad: usize, buttons: u16);

    /// Check if the session is waiting for remote input.
    fn is_waiting(&self) -> bool;

    /// Check if netplay is currently active.
    fn is_active(&self) -> bool;

    /// Set netplay active state.
    fn set_active(&self, active: bool);

    /// Get local player index.
    fn local_player(&self) -> Option<u8>;

    /// Get configured input delay in frames.
    fn input_delay(&self) -> u32;

    /// Returns the synchronized rewind capacity.
    fn rewind_capacity(&self) -> u32;

    /// Send local input to server.
    fn send_input_to_server(&self, frame: u32, buttons: u16);

    /// Check if inputs are ready for the given frame (peek).
    fn is_frame_ready(&self, frame: u32) -> bool;

    /// Check if we should fast-forward (catch up).
    fn should_fast_forward(&self, frame: u32) -> bool;

    /// Consume a pending "state sync requested" flag (host-only).
    ///
    /// When true, the runtime should send a fresh state snapshot to the server for caching,
    /// to help late joiners/reconnects catch up quickly.
    fn take_state_sync_request(&self) -> bool {
        false
    }

    /// Send a state snapshot to the server.
    fn send_state(&self, frame: u32, data: &[u8]);

    /// Get current sync mode (Lockstep or Rollback).
    fn sync_mode(&self) -> SyncMode;

    /// Frame offset used to map the local (0-based) runtime timeline onto the network timeline.
    ///
    /// Late joiners start at a non-zero network frame, but the runtime still counts frames from 0.
    /// This offset makes `poll_inputs(frame)` refer to the correct network frame.
    fn frame_offset(&self) -> u32 {
        0
    }

    /// Convert a local runtime frame into a network/effective frame.
    fn to_effective_frame(&self, local_frame: u32) -> u32 {
        local_frame.wrapping_add(self.frame_offset())
    }

    /// Convert a network/effective frame into a local runtime frame.
    fn to_local_frame(&self, effective_frame: u32) -> u32 {
        effective_frame.wrapping_sub(self.frame_offset())
    }

    /// Check if a rollback is pending (Rollback mode only).
    fn pending_rollback(&self) -> Option<crate::sync::RollbackRequest>;

    /// Clear the pending rollback after it has been processed.
    fn clear_rollback(&self);
}

/// Shared netplay input provider implementation.
///
/// This implementation wraps a `NetplaySession` and provides
/// thread-safe access for both the NES runtime and the netplay client.
pub struct SharedInputProvider {
    /// The underlying session (protected by mutex for state machine).
    session: Mutex<NetplaySession>,

    /// Synchronization strategy (lockstep or rollback).
    sync_strategy: Mutex<Box<dyn SyncStrategy>>,

    /// Absolute (effective) frame to catch up to for late joiners.
    ///
    /// `u32::MAX` means no catch-up target.
    catch_up_target_frame: AtomicU32,

    /// Host-only: request an immediate state snapshot upload (ProvideState).
    state_sync_requested: AtomicBool,

    /// Local inputs are treated as 0 until this effective frame (inclusive start).
    ///
    /// This is used for lockstep reconnect: during catch-up, the local player must not
    /// contribute non-zero inputs until the server schedules activation.
    local_input_allowed_from_effective_frame: AtomicU32,

    /// Schedule: port becomes active from this effective frame.
    /// `u32::MAX` means no scheduled activation.
    scheduled_port_active_from: [AtomicU32; 4],

    /// Reconnect flow: once catch-up is done and fast-forward settles, send `RejoinReady`.
    rejoin_ready_armed: AtomicBool,
    rejoin_ready_waiting_for_settle: AtomicBool,

    /// Flag indicating we're waiting for remote input.
    waiting: AtomicBool,

    /// Flag indicating netplay is active.
    active: AtomicBool,

    /// Current frame number.
    current_frame: AtomicU32,

    /// Local player index (if assigned).
    local_player: Mutex<Option<u8>>,

    /// Pending local buttons for each pad.
    local_buttons: [std::sync::atomic::AtomicU16; 4],

    /// Callback to send input to server.
    on_send_input: Mutex<Option<Box<dyn Fn(u32, u16) + Send + Sync>>>,

    /// Callback to send state to server.
    on_send_state: Mutex<Option<Box<dyn Fn(u32, &[u8]) + Send + Sync>>>,

    /// Callback to notify server we're ready to reactivate (lockstep reconnect).
    on_send_rejoin_ready: Mutex<Option<Box<dyn Fn(u32) + Send + Sync>>>,
}

impl Default for SharedInputProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedInputProvider {
    /// Create a new shared input provider with default Lockstep sync mode.
    pub fn new() -> Self {
        Self::with_sync_mode(SyncMode::Lockstep)
    }

    /// Create a new shared input provider with the specified sync mode.
    pub fn with_sync_mode(mode: SyncMode) -> Self {
        let strategy: Box<dyn SyncStrategy> = match mode {
            SyncMode::Lockstep => Box::new(LockstepSync::new(2)),
            SyncMode::Rollback => Box::new(crate::sync::rollback::RollbackSync::new(2)),
        };
        Self {
            session: Mutex::new(NetplaySession::new()),
            sync_strategy: Mutex::new(strategy),
            catch_up_target_frame: AtomicU32::new(u32::MAX),
            state_sync_requested: AtomicBool::new(false),
            local_input_allowed_from_effective_frame: AtomicU32::new(0),
            scheduled_port_active_from: [
                AtomicU32::new(u32::MAX),
                AtomicU32::new(u32::MAX),
                AtomicU32::new(u32::MAX),
                AtomicU32::new(u32::MAX),
            ],
            rejoin_ready_armed: AtomicBool::new(false),
            rejoin_ready_waiting_for_settle: AtomicBool::new(false),
            waiting: AtomicBool::new(false),
            active: AtomicBool::new(false),
            current_frame: AtomicU32::new(0),
            local_player: Mutex::new(None),
            local_buttons: Default::default(),
            on_send_input: Mutex::new(None),
            on_send_state: Mutex::new(None),
            on_send_rejoin_ready: Mutex::new(None),
        }
    }

    /// Host-only: request uploading a fresh state snapshot to the server.
    pub fn request_state_sync(&self) {
        self.state_sync_requested.store(true, Ordering::Release);
    }

    pub fn set_local_input_allowed_from_effective_frame(&self, frame: u32) {
        self.local_input_allowed_from_effective_frame
            .store(frame, Ordering::Release);
    }

    pub fn schedule_port_active_from(&self, port: usize, active_from_frame: u32) {
        if port >= 4 {
            return;
        }
        let prev = self.scheduled_port_active_from[port].load(Ordering::Acquire);
        let next = if prev == u32::MAX {
            active_from_frame
        } else {
            prev.min(active_from_frame)
        };
        self.scheduled_port_active_from[port].store(next, Ordering::Release);
    }

    pub fn arm_rejoin_ready(&self) {
        self.rejoin_ready_armed.store(true, Ordering::Release);
        self.rejoin_ready_waiting_for_settle
            .store(false, Ordering::Release);
        self.set_local_input_allowed_from_effective_frame(u32::MAX);
    }

    /// Set/clear the late-join catch-up target frame (absolute/effective frame number).
    pub fn set_catch_up_target_frame(&self, target_frame: Option<u32>) {
        let value = target_frame.unwrap_or(u32::MAX);
        self.catch_up_target_frame.store(value, Ordering::Release);
    }

    /// Get the current sync mode.
    pub fn sync_mode(&self) -> SyncMode {
        self.sync_strategy.lock().mode()
    }

    /// Set the sync mode (clears existing input state).
    pub fn set_sync_mode(&self, mode: SyncMode) {
        let (input_delay, active_ports) =
            self.with_session(|s| (s.input_delay_frames as u32, s.active_ports));
        let mut new_strategy: Box<dyn SyncStrategy> = match mode {
            SyncMode::Lockstep => Box::new(LockstepSync::new(input_delay)),
            SyncMode::Rollback => Box::new(crate::sync::rollback::RollbackSync::new(input_delay)),
        };

        // Transfer active ports to the new strategy
        for (i, &active) in active_ports.iter().enumerate() {
            new_strategy.set_port_active(i, active);
        }

        let mut strategy = self.sync_strategy.lock();
        *strategy = new_strategy;

        let mut session = self.session.lock();
        session.clear_inputs();
    }

    /// Set a port as active or inactive.
    pub fn set_port_active(&self, port: usize, active: bool) {
        self.sync_strategy.lock().set_port_active(port, active);
        if port < 4 {
            self.scheduled_port_active_from[port].store(u32::MAX, Ordering::Release);
        }
        let mut session = self.session.lock();
        if port < session.active_ports.len() {
            session.active_ports[port] = active;
        }
    }

    fn maybe_activate_scheduled_ports(&self, effective_frame: u32) {
        for port in 0..4 {
            let from = self.scheduled_port_active_from[port].load(Ordering::Acquire);
            if from != u32::MAX && effective_frame >= from {
                self.set_port_active(port, true);
                self.scheduled_port_active_from[port].store(u32::MAX, Ordering::Release);
            }
        }
    }
    /// Get mutable access to the sync strategy.
    pub fn with_sync<R>(&self, f: impl FnOnce(&mut dyn SyncStrategy) -> R) -> R {
        let mut strategy = self.sync_strategy.lock();
        f(strategy.as_mut())
    }

    /// Get mutable access to the session.
    pub fn with_session<R>(&self, f: impl FnOnce(&mut NetplaySession) -> R) -> R {
        let mut session = self.session.lock();
        f(&mut session)
    }

    /// Get mutable access to the session (mut variant).
    pub fn with_session_mut<R>(&self, f: impl FnOnce(&mut NetplaySession) -> R) -> R {
        let mut session = self.session.lock();
        f(&mut session)
    }

    /// Set the active state.
    pub fn set_active(&self, active: bool) {
        self.active.store(active, Ordering::Release);
    }

    /// Set the local player index.
    pub fn set_local_player(&self, player: Option<u8>) {
        let mut guard = self.local_player.lock();
        *guard = player;
    }

    /// Get the local player index.
    pub fn local_player(&self) -> Option<u8> {
        let guard = self.local_player.lock();
        *guard
    }

    /// Set current frame.
    pub fn set_current_frame(&self, frame: u32) {
        self.current_frame.store(frame, Ordering::Release);
    }

    /// Push confirmed input from network into the queue.
    pub fn push_remote_input(&self, port: usize, frame: u32, buttons: u16) {
        // Forward to sync strategy
        self.sync_strategy
            .lock()
            .on_remote_input(port as u8, frame, buttons);

        // Also push to session for backward compatibility
        let mut session = self.session.lock();
        session.push_input(port, frame, buttons);

        // Signal that we might no longer be waiting
        self.waiting.store(false, Ordering::Release);
    }

    /// Clear all input queues.
    pub fn clear_queues(&self) {
        self.sync_strategy.lock().clear();
        let mut session = self.session.lock();
        session.clear_inputs();
    }

    /// Get the last confirmed frame.
    pub fn last_confirmed_frame(&self) -> u32 {
        self.sync_strategy.lock().last_confirmed_frame()
    }

    /// Set the input delay in frames.
    pub fn set_input_delay(&self, delay: u32) {
        self.sync_strategy.lock().set_input_delay(delay);
        let mut session = self.session.lock();
        session.input_delay_frames = delay as u8;
    }

    /// Set callback for sending inputs.
    pub fn set_on_send_input(&self, cb: Box<dyn Fn(u32, u16) + Send + Sync>) {
        let mut guard = self.on_send_input.lock();
        *guard = Some(cb);
    }

    /// Set callback for sending state.
    pub fn set_on_send_state(&self, cb: Box<dyn Fn(u32, &[u8]) + Send + Sync>) {
        let mut guard = self.on_send_state.lock();
        *guard = Some(cb);
    }

    pub fn set_on_send_rejoin_ready(&self, cb: Box<dyn Fn(u32) + Send + Sync>) {
        let mut guard = self.on_send_rejoin_ready.lock();
        *guard = Some(cb);
    }
}

impl NetplayInputProvider for SharedInputProvider {
    fn poll_inputs(&self, frame: u32) -> Option<[u16; 4]> {
        let effective_frame =
            self.with_session(|session| frame.wrapping_add(frame_offset(session)));

        self.maybe_activate_scheduled_ports(effective_frame);

        // Delegate to sync strategy
        let result = self.sync_strategy.lock().inputs_for_frame(effective_frame);

        if result.is_none() {
            self.waiting.store(true, Ordering::Release);
        } else {
            self.waiting.store(false, Ordering::Release);
        }

        result
    }

    fn submit_local_input(&self, pad: usize, buttons: u16) {
        if pad < 4 {
            self.local_buttons[pad].store(buttons, Ordering::Release);
        }
    }

    fn is_waiting(&self) -> bool {
        self.waiting.load(Ordering::Acquire)
    }

    fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    fn set_active(&self, active: bool) {
        self.active.store(active, Ordering::Release);
    }

    fn local_player(&self) -> Option<u8> {
        let guard = self.local_player.lock();
        *guard
    }

    fn input_delay(&self) -> u32 {
        let session = self.session.lock();
        session.input_delay_frames as u32
    }

    fn rewind_capacity(&self) -> u32 {
        let session = self.session.lock();
        session.rewind_capacity
    }

    fn send_input_to_server(&self, frame: u32, buttons: u16) {
        let (effective_frame, local_player) = self.with_session(|session| {
            (
                frame.wrapping_add(frame_offset(session)),
                session.local_player_index,
            )
        });
        let allowed_from = self
            .local_input_allowed_from_effective_frame
            .load(Ordering::Acquire);
        if effective_frame < allowed_from {
            return;
        }

        self.with_session_mut(|session| {
            if let Some(idx) = session.local_player_index {
                session.push_input(idx as usize, effective_frame, buttons);
            }
        });

        if let Some(idx) = local_player {
            self.sync_strategy
                .lock()
                .on_local_input(idx, effective_frame, buttons);
        }

        // CRITICAL: Push to own queue immediately to prevent lockstep deadlock.
        // Without this, the game waits for server relay which causes latency-induced freeze.
        // Then send to server for relay to other players
        let cb = self.on_send_input.lock();
        if let Some(f) = cb.as_ref() {
            f(effective_frame, buttons);
        }
    }

    fn is_frame_ready(&self, frame: u32) -> bool {
        let effective_frame =
            self.with_session(|session| frame.wrapping_add(frame_offset(session)));
        self.sync_strategy.lock().can_advance(effective_frame)
    }

    fn should_fast_forward(&self, frame: u32) -> bool {
        let effective_frame =
            self.with_session(|session| frame.wrapping_add(frame_offset(session)));

        let target = self.catch_up_target_frame.load(Ordering::Acquire);
        if target != u32::MAX {
            if effective_frame < target {
                return true;
            }
            self.catch_up_target_frame
                .store(u32::MAX, Ordering::Release);
            if self.rejoin_ready_armed.load(Ordering::Acquire) {
                self.rejoin_ready_waiting_for_settle
                    .store(true, Ordering::Release);
            }
        }

        let should_ff = self
            .sync_strategy
            .lock()
            .should_fast_forward(effective_frame);
        if !should_ff
            && self.rejoin_ready_waiting_for_settle.load(Ordering::Acquire)
            && self
                .rejoin_ready_waiting_for_settle
                .swap(false, Ordering::AcqRel)
            && self.rejoin_ready_armed.swap(false, Ordering::AcqRel)
        {
            let cb = self.on_send_rejoin_ready.lock();
            if let Some(f) = cb.as_ref() {
                f(effective_frame);
            }
        }

        should_ff
    }

    fn take_state_sync_request(&self) -> bool {
        self.state_sync_requested.swap(false, Ordering::AcqRel)
    }

    fn send_state(&self, frame: u32, data: &[u8]) {
        let effective_frame =
            self.with_session(|session| frame.wrapping_add(frame_offset(session)));
        let cb = self.on_send_state.lock();
        if let Some(f) = cb.as_ref() {
            f(effective_frame, data);
        }
    }

    fn sync_mode(&self) -> SyncMode {
        self.sync_strategy.lock().mode()
    }

    fn frame_offset(&self) -> u32 {
        self.with_session(|session| frame_offset(session))
    }

    fn pending_rollback(&self) -> Option<crate::sync::RollbackRequest> {
        self.sync_strategy.lock().pending_rollback()
    }

    fn clear_rollback(&self) {
        self.sync_strategy.lock().clear_rollback();
    }
}

/// Create a new shared input provider wrapped in an Arc.
pub fn create_input_provider() -> Arc<SharedInputProvider> {
    Arc::new(SharedInputProvider::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_starts_inactive() {
        let provider = SharedInputProvider::new();
        assert!(!provider.is_active());
        assert!(!provider.is_waiting());
    }

    #[test]
    fn poll_returns_none_when_empty() {
        let provider = SharedInputProvider::new();
        provider.set_active(true);
        provider.set_port_active(0, true);
        assert!(provider.poll_inputs(0).is_none());
        assert!(provider.is_waiting());
    }

    #[test]
    fn poll_returns_inputs_when_available() {
        let provider = SharedInputProvider::new();
        provider.set_active(true);

        // Push inputs for all ports
        for port in 0..4 {
            provider.set_port_active(port, true);
            provider.push_remote_input(port, 0, (port + 1) as u16);
        }

        let inputs = provider.poll_inputs(0);
        assert!(inputs.is_some());
        assert_eq!(inputs.unwrap(), [1, 2, 3, 4]);
        assert!(!provider.is_waiting());
    }
}
