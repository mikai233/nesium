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

    /// Send a state snapshot to the server.
    fn send_state(&self, frame: u32, data: &[u8]);
}

/// Shared netplay input provider implementation.
///
/// This implementation wraps a `NetplaySession` and provides
/// thread-safe access for both the NES runtime and the netplay client.
pub struct SharedInputProvider {
    /// The underlying session (protected by mutex for state machine).
    session: Mutex<NetplaySession>,

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
}

impl Default for SharedInputProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SharedInputProvider {
    /// Create a new shared input provider.
    pub fn new() -> Self {
        Self {
            session: Mutex::new(NetplaySession::new()),
            waiting: AtomicBool::new(false),
            active: AtomicBool::new(false),
            current_frame: AtomicU32::new(0),
            local_player: Mutex::new(None),
            local_buttons: Default::default(),
            on_send_input: Mutex::new(None),
            on_send_state: Mutex::new(None),
        }
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
        let mut session = self.session.lock();
        session.push_input(port, frame, buttons);

        // Signal that we might no longer be waiting
        // We can't easily check specific frame readiness without current frame context,
        // but any input might unblock the waiter.
        self.waiting.store(false, Ordering::Release);
    }

    /// Clear all input queues.
    pub fn clear_queues(&self) {
        let mut session = self.session.lock();
        session.clear_inputs();
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
}

impl NetplayInputProvider for SharedInputProvider {
    fn poll_inputs(&self, frame: u32) -> Option<[u16; 4]> {
        let mut session = self.session.lock();
        let effective_frame = frame.wrapping_add(frame_offset(&session));

        // Check if we have valid inputs for THIS specific frame
        if !session.is_frame_ready(effective_frame) {
            self.waiting.store(true, Ordering::Release);
            return None;
        }

        // Retrieve inputs for this frame
        let mut inputs = [0u16; 4];
        for (i, input) in inputs.iter_mut().enumerate() {
            // get_input returns Some(buttons) or Some(0) if inactive,
            // or None if data missing (but is_frame_ready checked that).
            // So unwrap_or(0) is safe fallback.
            *input = session.get_input(i, effective_frame).unwrap_or(0);
        }

        self.waiting.store(false, Ordering::Release);
        Some(inputs)
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
        let effective_frame = self.with_session_mut(|session| {
            let effective = frame.wrapping_add(frame_offset(session));
            if let Some(idx) = session.local_player_index {
                session.push_input(idx as usize, effective, buttons);
            }
            effective
        });

        // CRITICAL: Push to own queue immediately to prevent lockstep deadlock.
        // Without this, the game waits for server relay which causes latency-induced freeze.
        // Then send to server for relay to other players
        let cb = self.on_send_input.lock();
        if let Some(f) = cb.as_ref() {
            f(effective_frame, buttons);
        }
    }

    fn is_frame_ready(&self, frame: u32) -> bool {
        let session = self.session.lock();
        session.is_frame_ready(frame.wrapping_add(frame_offset(&session)))
    }

    fn should_fast_forward(&self, frame: u32) -> bool {
        let session = self.session.lock();
        session.should_fast_forward(frame.wrapping_add(frame_offset(&session)))
    }

    fn send_state(&self, frame: u32, data: &[u8]) {
        let effective_frame =
            self.with_session(|session| frame.wrapping_add(frame_offset(session)));
        let cb = self.on_send_state.lock();
        if let Some(f) = cb.as_ref() {
            f(effective_frame, data);
        }
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
        assert!(provider.poll_inputs(0).is_none());
        assert!(provider.is_waiting());
    }

    #[test]
    fn poll_returns_inputs_when_available() {
        let provider = SharedInputProvider::new();
        provider.set_active(true);

        // Push inputs for all ports
        for port in 0..4 {
            provider.push_remote_input(port, 0, (port + 1) as u16);
        }

        let inputs = provider.poll_inputs(0);
        assert!(inputs.is_some());
        assert_eq!(inputs.unwrap(), [1, 2, 3, 4]);
        assert!(!provider.is_waiting());
    }
}
