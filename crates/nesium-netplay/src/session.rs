use std::collections::{BTreeMap, VecDeque};

/// Netplay session state machine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionState {
    /// Not connected to any server.
    Disconnected,
    /// TCP connection established, waiting to send Hello.
    Connecting,
    /// Hello sent, waiting for Welcome.
    Handshake,
    /// Welcome received, waiting for room assignment.
    WaitingForRoom,
    /// Receiving save state for synchronization.
    Syncing {
        snapshot_id: u32,
        frags_received: u16,
    },
    /// Actively playing as a participant.
    Playing { start_frame: u32, player_index: u8 },
    /// Spectating (receive-only mode).
    Spectating { start_frame: u32 },
}

/// Information about a remote player in the room.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemotePlayer {
    pub client_id: u32,
    pub name: String,
    /// 0-3 for player index, or `SPECTATOR_PLAYER_INDEX` for spectator.
    pub player_index: u8,
}

impl Default for SessionState {
    fn default() -> Self {
        Self::Disconnected
    }
}

/// Netplay session context.
///
/// This struct maintains all state needed for a netplay session,
/// including connection metadata and input queues.
#[derive(Debug)]
pub struct NetplaySession {
    /// Current session state.
    pub state: SessionState,

    /// Server-assigned client ID (0 = not assigned yet).
    pub client_id: u32,

    /// Current room ID (0 = not in a room).
    pub room_id: u32,

    /// Local player name.
    pub local_name: String,

    /// Local player index (None = spectator).
    pub local_player_index: Option<u8>,

    /// Input queues per controller port.
    /// These hold confirmed inputs from the network.
    input_queues: [BTreeMap<u32, u16>; 4],

    /// Pending local inputs that have been sent but not yet confirmed.
    /// (frame, buttons) pairs.
    pending_local_inputs: VecDeque<(u32, u16)>,

    /// Active ports mask (true if port has received input).
    pub active_ports: [bool; 4],

    /// Network-configured input delay in frames.
    pub input_delay_frames: u8,

    /// Current frame number.
    pub current_frame: u32,

    /// ROM hash for validation.
    pub rom_hash: [u8; 16],

    /// Server nonce for handshake.
    pub server_nonce: u32,

    /// Sequence number for outgoing packets.
    pub local_seq: u32,

    /// Last acknowledged sequence from server.
    pub last_ack: u32,

    /// Rewind capacity (frames) negotiated with server.
    pub rewind_capacity: u32,

    /// Remote players in the room (client_id -> RemotePlayer).
    pub players: BTreeMap<u32, RemotePlayer>,
}

impl Default for NetplaySession {
    fn default() -> Self {
        Self::new()
    }
}

impl NetplaySession {
    /// Create a new disconnected session.
    pub fn new() -> Self {
        Self {
            state: SessionState::default(),
            client_id: 0,
            room_id: 0,
            local_name: String::new(),
            local_player_index: None,
            input_queues: [
                BTreeMap::new(),
                BTreeMap::new(),
                BTreeMap::new(),
                BTreeMap::new(),
            ],
            pending_local_inputs: VecDeque::new(),
            active_ports: [false; 4],
            input_delay_frames: 2,
            current_frame: 0,
            rom_hash: [0; 16],
            server_nonce: 0,
            local_seq: 1,
            last_ack: 0,
            rewind_capacity: 600,
            players: BTreeMap::new(),
        }
    }

    /// Reset session state for a new connection.
    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Check if the session is in a playable state.
    pub fn is_playing(&self) -> bool {
        matches!(
            self.state,
            SessionState::Playing { .. } | SessionState::Spectating { .. }
        )
    }

    /// Check if this session is a spectator.
    pub fn is_spectator(&self) -> bool {
        matches!(self.state, SessionState::Spectating { .. })
    }

    /// Push confirmed input into the queue for a specific port and frame.
    pub fn push_input(&mut self, port: usize, frame: u32, buttons: u16) {
        if let Some(queue) = self.input_queues.get_mut(port) {
            queue.insert(frame, buttons);
            if port < 4 {
                self.active_ports[port] = true;
            }
        }
    }

    /// Get input for a specific port and frame.
    /// Returns None if input is not available.
    /// Returns Some(0) if port is not active.
    pub fn get_input(&mut self, port: usize, frame: u32) -> Option<u16> {
        let is_local = self
            .local_player_index
            .map(|idx| idx as usize == port)
            .unwrap_or(false);

        // If port is inactive and not local, we might treat it as empty or wait?
        // Current logic: min_queue_depth filtered inactive ports.
        // Here, if it's inactive, we assume 0 input?
        // BETTER: The runner logic checks "min_queue_depth" to decide if it should BLOCK.
        // If we change this to "get_input", we need to know if we *should* have input.
        // If active_ports[port] is false, and not local, we return Some(0).
        if !self.active_ports[port] && !is_local {
            return Some(0);
        }

        // Check if we have the specific frame
        self.input_queues
            .get_mut(port)
            .and_then(|q| q.remove(&frame))
    }

    /// Get the current queue depth for a port.
    pub fn queue_depth(&self, port: usize) -> usize {
        self.input_queues.get(port).map(|q| q.len()).unwrap_or(0)
    }

    /// Check if inputs are available for the given frame for all active ports.
    pub fn is_frame_ready(&self, frame: u32) -> bool {
        let mut any_active = false;
        for (i, queue) in self.input_queues.iter().enumerate() {
            let is_local = self
                .local_player_index
                .map(|idx| idx as usize == i)
                .unwrap_or(false);
            if self.active_ports[i] || is_local {
                any_active = true;
                if !queue.contains_key(&frame) {
                    return false;
                }
            }
        }
        // If we have verified all active ports have data, we return true.
        // If there are NO active ports (e.g. start of session), we should wait (return false).
        any_active
    }

    /// Check if we should fast-forward (catch up).
    /// This is true if we have buffered inputs significantly ahead of the current frame.
    /// This prevents fast-forwarding during normal play where we only have the input delay buffer.
    pub fn should_fast_forward(&self, current_frame: u32) -> bool {
        // We only fast forward if we have inputs for frame `current_frame + input_delay + 1`.
        // This ensures we maintain the input delay buffer but drain any excess.
        let target_frame = current_frame + self.input_delay_frames as u32 + 1;

        let mut any_active = false;
        for (i, queue) in self.input_queues.iter().enumerate() {
            let is_local = self
                .local_player_index
                .map(|idx| idx as usize == i)
                .unwrap_or(false);

            if self.active_ports[i] || is_local {
                any_active = true;
                // Strict check: we must have the target frame.
                // Assuming continuous inputs, if we have target_frame, we have everything before it.
                if !queue.contains_key(&target_frame) {
                    return false;
                }
            }
        }
        any_active
    }

    /// Queue a local input to be sent.
    pub fn queue_local_input(&mut self, frame: u32, buttons: u16) {
        self.pending_local_inputs.push_back((frame, buttons));
    }

    /// Drain pending local inputs up to a count.
    pub fn drain_pending_inputs(&mut self, count: usize) -> Vec<(u32, u16)> {
        let n = count.min(self.pending_local_inputs.len());
        self.pending_local_inputs.drain(..n).collect()
    }

    /// Get number of pending local inputs.
    pub fn pending_inputs_count(&self) -> usize {
        self.pending_local_inputs.len()
    }

    /// Clear all input queues (e.g., on resync).
    pub fn clear_inputs(&mut self) {
        for queue in &mut self.input_queues {
            queue.clear();
        }
        self.pending_local_inputs.clear();
    }

    /// Clear a specific port's input queue and mark it inactive.
    pub fn clear_port(&mut self, port: usize) {
        if let Some(queue) = self.input_queues.get_mut(port) {
            queue.clear();
        }
        if port < self.active_ports.len() {
            self.active_ports[port] = false;
        }
    }

    /// Increment and return the next sequence number.
    pub fn next_seq(&mut self) -> u32 {
        let seq = self.local_seq;
        self.local_seq = self.local_seq.wrapping_add(1);
        seq
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_starts_disconnected() {
        let session = NetplaySession::new();
        assert_eq!(session.state, SessionState::Disconnected);
        assert!(!session.is_playing());
    }

    #[test]
    fn input_queue_operations() {
        let mut session = NetplaySession::new();

        session.push_input(0, 100, 0x01);
        session.push_input(0, 101, 0x02);
        assert_eq!(session.queue_depth(0), 2);

        // Inputs should be retrievable by frame
        assert_eq!(session.get_input(0, 100), Some(0x01));
        assert_eq!(session.get_input(0, 101), Some(0x02));

        // And consumed
        assert_eq!(session.queue_depth(0), 0);
        assert_eq!(session.get_input(0, 100), None);
    }
}
