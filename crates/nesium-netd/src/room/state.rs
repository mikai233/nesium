//! Room state management.
//!
//! Each room represents a netplay session with one or more players
//! and optional spectators.

use std::collections::{HashMap, VecDeque};
use std::net::SocketAddr;

use crate::net::inbound::ConnId;
use crate::net::outbound::OutboundTx;
use nesium_netproto::messages::session::SyncMode;
use nesium_netproto::{
    channel::{ChannelKind, channel_for_msg},
    constants::SPECTATOR_PLAYER_INDEX,
    msg_id::MsgId,
};
use rand::Rng;

/// Maximum number of players per room.
pub const MAX_PLAYERS: usize = 4;

/// Minimum 6-digit Room ID (inclusive).
pub const ROOM_ID_MIN: u32 = 100_000;
/// Maximum 6-digit Room ID (inclusive).
pub const ROOM_ID_MAX: u32 = 999_999;
/// Total number of available Room IDs in the 6-digit space.
pub const ROOM_ID_COUNT: u32 = ROOM_ID_MAX - ROOM_ID_MIN + 1;

/// Direct-connect information for Host-as-server P2P mode (netd as signaling).
#[derive(Debug, Clone)]
pub struct P2PHostInfo {
    pub host_signal_client_id: u32,
    pub host_addrs: Vec<SocketAddr>,
    pub host_room_id: u32,
    pub host_quic_cert_sha256_fingerprint: Option<String>,
    pub host_quic_server_name: Option<String>,
}

#[derive(Debug, Clone)]
pub struct P2PFallbackState {
    pub reason: String,
    pub requested_by_client_id: u32,
}

#[derive(Debug, Clone)]
pub struct ClientOutbounds {
    pub control: OutboundTx,
    pub input: Option<OutboundTx>,
    pub bulk: Option<OutboundTx>,
}

impl ClientOutbounds {
    pub fn new(control: OutboundTx) -> Self {
        Self {
            control,
            input: None,
            bulk: None,
        }
    }

    pub fn outbound_for_channel(&self, channel: ChannelKind) -> OutboundTx {
        match channel {
            ChannelKind::Control => self.control.clone(),
            ChannelKind::Input => self.input.clone().unwrap_or_else(|| self.control.clone()),
            ChannelKind::Bulk => self.bulk.clone().unwrap_or_else(|| self.control.clone()),
        }
    }

    pub fn outbound_for_msg(&self, msg_id: MsgId) -> OutboundTx {
        self.outbound_for_channel(channel_for_msg(msg_id))
    }

    pub fn set_channel(&mut self, channel: ChannelKind, outbound: OutboundTx) {
        match channel {
            ChannelKind::Control => self.control = outbound,
            ChannelKind::Input => self.input = Some(outbound),
            ChannelKind::Bulk => self.bulk = Some(outbound),
        }
    }

    pub fn clear_channel(&mut self, channel: ChannelKind) {
        match channel {
            ChannelKind::Control => {}
            ChannelKind::Input => self.input = None,
            ChannelKind::Bulk => self.bulk = None,
        }
    }
}

/// Player assignment in a room.
#[derive(Debug, Clone)]
pub struct Player {
    pub conn_id: ConnId,
    pub client_id: u32,
    pub player_index: u8,
    pub name: String,
    pub outbounds: ClientOutbounds,
}

/// Spectator in a room.
#[derive(Debug, Clone)]
pub struct Spectator {
    pub conn_id: ConnId,
    pub client_id: u32,
    pub name: String,
    pub outbounds: ClientOutbounds,
}

/// Room state.
#[derive(Debug)]
pub struct Room {
    /// Unique room ID.
    pub id: u32,
    /// Host client ID (first player to join).
    pub host_client_id: u32,
    /// Players in the room (player_index -> Player).
    pub players: HashMap<u8, Player>,
    /// Whether each player port is considered active for lockstep progression.
    ///
    /// In lockstep reconnect, a player may be present but inactive (treated as 0 input)
    /// until the server schedules activation.
    pub active_ports: [bool; MAX_PLAYERS],
    /// Resume tokens for reclaiming each player slot after disconnect.
    /// Scheduled activations: player_index -> active_from_frame.
    pub scheduled_port_activations: HashMap<u8, u32>,
    /// Spectators in the room.
    pub spectators: Vec<Spectator>,
    /// Current authoritative frame number.
    pub current_frame: u32,
    /// Input buffer per player (frame -> buttons).
    pub input_buffers: [VecDeque<(u32, u16)>; MAX_PLAYERS],
    /// Whether the game started.
    pub started: bool,
    /// Players that have loaded the ROM.
    pub loaded_players: std::collections::HashSet<u32>,
    /// Whether the game is paused.
    pub paused: bool,
    /// Cached save state for reconnection/late join (frame, data).
    pub cached_state: Option<(u32, Vec<u8>)>,
    /// Late joiners waiting for an up-to-date cached state before starting catch-up.
    pub pending_catch_up_clients: Vec<u32>,
    /// ROM data for late joiners.
    pub rom_data: Option<Vec<u8>>,
    /// Host-as-server P2P direct-connect info (when this room ID is used for P2P matchmaking).
    pub p2p_host: Option<P2PHostInfo>,
    /// Clients currently watching this room via P2P signaling (client_id -> outbound).
    pub p2p_watchers: HashMap<u32, OutboundTx>,
    /// Whether relay fallback was requested for this room ID.
    pub p2p_fallback: Option<P2PFallbackState>,
    /// Synchronization mode for this room (Lockstep or Rollback).
    pub sync_mode: SyncMode,
}

impl Room {
    /// Create a new room.
    pub fn new(id: u32, host_client_id: u32) -> Self {
        Self {
            id,
            host_client_id,
            players: HashMap::new(),
            active_ports: [false; MAX_PLAYERS],
            scheduled_port_activations: HashMap::new(),
            spectators: Vec::new(),
            current_frame: 0,
            input_buffers: Default::default(),
            started: false,
            loaded_players: std::collections::HashSet::new(),
            paused: false,
            cached_state: None,
            pending_catch_up_clients: Vec::new(),
            rom_data: None,
            p2p_host: None,
            p2p_watchers: HashMap::new(),
            p2p_fallback: None,
            sync_mode: Default::default(),
        }
    }

    pub fn set_p2p_host(
        &mut self,
        host_signal_client_id: u32,
        host_addrs: Vec<SocketAddr>,
        host_room_id: u32,
        host_quic_cert_sha256_fingerprint: Option<String>,
        host_quic_server_name: Option<String>,
    ) {
        self.p2p_host = Some(P2PHostInfo {
            host_signal_client_id,
            host_addrs,
            host_room_id,
            host_quic_cert_sha256_fingerprint,
            host_quic_server_name,
        });
    }

    pub fn upsert_p2p_watcher(&mut self, client_id: u32, outbound: OutboundTx) {
        self.p2p_watchers.insert(client_id, outbound);
    }

    pub fn remove_p2p_watcher(&mut self, client_id: u32) {
        self.p2p_watchers.remove(&client_id);
    }

    pub fn request_p2p_fallback(&mut self, requested_by_client_id: u32, reason: String) {
        if self.p2p_fallback.is_some() {
            return;
        }
        self.p2p_fallback = Some(P2PFallbackState {
            reason,
            requested_by_client_id,
        });
    }

    /// Add a player to the room.
    pub fn add_player(&mut self, player: Player) -> Option<u8> {
        // Find first available slot
        for idx in 0..MAX_PLAYERS as u8 {
            if !self.players.contains_key(&idx) {
                let player_index = idx;
                self.players.insert(
                    player_index,
                    Player {
                        player_index,
                        ..player
                    },
                );
                if (player_index as usize) < MAX_PLAYERS {
                    self.active_ports[player_index as usize] = true;
                }
                return Some(player_index);
            }
        }
        None // Room is full
    }

    pub fn add_player_at_index(&mut self, player_index: u8, player: Player, active: bool) -> bool {
        if player_index as usize >= MAX_PLAYERS {
            return false;
        }
        if self.players.contains_key(&player_index) {
            return false;
        }
        self.players.insert(
            player_index,
            Player {
                player_index,
                ..player
            },
        );
        self.active_ports[player_index as usize] = active;
        true
    }

    /// Add a spectator to the room.
    pub fn add_spectator(&mut self, spectator: Spectator) {
        self.spectators.push(spectator);
    }

    /// Remove a player from the room.
    /// Also clears their input buffer to prevent lockstep deadlock.
    pub fn remove_player(&mut self, client_id: u32) -> Option<Player> {
        let key = self
            .players
            .iter()
            .find(|(_, p)| p.client_id == client_id)
            .map(|(k, _)| *k);
        if let Some(k) = key {
            // Clear input buffer for this player to prevent lockstep deadlock
            if let Some(buffer) = self.input_buffers.get_mut(k as usize) {
                buffer.clear();
            }
            if (k as usize) < MAX_PLAYERS {
                self.active_ports[k as usize] = false;
                self.scheduled_port_activations.remove(&k);
            }
            self.players.remove(&k)
        } else {
            None
        }
    }

    /// Remove a spectator from the room.
    pub fn remove_spectator(&mut self, client_id: u32) -> Option<Spectator> {
        if let Some(pos) = self
            .spectators
            .iter()
            .position(|s| s.client_id == client_id)
        {
            Some(self.spectators.remove(pos))
        } else {
            None
        }
    }

    /// Check if room is empty.
    pub fn is_empty(&self) -> bool {
        self.players.is_empty() && self.spectators.is_empty()
    }

    /// Get input history for all players since `start_frame`.
    /// Returns Vec<(player_index, base_frame, buttons)>.
    pub fn get_input_history(&self, start_frame: u32) -> Vec<(u8, u32, Vec<u16>)> {
        let mut history = Vec::new();

        for (player_index, buffer) in self.input_buffers.iter().enumerate() {
            let mut current_base: Option<u32> = None;
            let mut current_buttons: Vec<u16> = Vec::new();
            let mut prev_frame: Option<u32> = None;

            for (frame, buttons) in buffer {
                if *frame < start_frame {
                    continue;
                }

                let contiguous = prev_frame
                    .and_then(|prev| prev.checked_add(1))
                    .map(|expected| expected == *frame)
                    .unwrap_or(false);

                if current_base.is_none() {
                    current_base = Some(*frame);
                } else if !contiguous {
                    history.push((player_index as u8, current_base.unwrap(), current_buttons));
                    current_base = Some(*frame);
                    current_buttons = Vec::new();
                }

                current_buttons.push(*buttons);
                prev_frame = Some(*frame);
            }

            if let Some(base_frame) = current_base {
                if !current_buttons.is_empty() {
                    history.push((player_index as u8, base_frame, current_buttons));
                }
            }
        }

        history
    }

    /// Get player count.
    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    /// Returns a bitmask of active player ports.
    /// Bit N is set if player index N is present.
    pub fn get_active_ports_mask(&self) -> u8 {
        let mut mask = 0u8;
        for idx in 0..MAX_PLAYERS {
            if self.active_ports[idx] && idx < 8 {
                mask |= 1u8 << (idx as u8);
            }
        }
        mask
    }

    pub fn schedule_port_activation(&mut self, player_index: u8, active_from_frame: u32) {
        if (player_index as usize) < MAX_PLAYERS {
            self.active_ports[player_index as usize] = false;
            self.scheduled_port_activations
                .insert(player_index, active_from_frame);
        }
    }

    fn apply_scheduled_activations(&mut self) {
        let current = self.current_frame;
        let mut to_activate = Vec::<u8>::new();
        for (&idx, &from) in &self.scheduled_port_activations {
            if current >= from {
                to_activate.push(idx);
            }
        }
        for idx in to_activate {
            if (idx as usize) < MAX_PLAYERS {
                self.active_ports[idx as usize] = true;
            }
            self.scheduled_port_activations.remove(&idx);
        }
    }

    fn refresh_active_ports_from_players(&mut self) {
        for idx in 0..MAX_PLAYERS {
            self.active_ports[idx] = self.players.contains_key(&(idx as u8));
        }
        self.scheduled_port_activations.clear();
    }

    pub fn cache_state(&mut self, frame: u32, data: Vec<u8>) {
        self.cached_state = Some((frame, data));
        self.prune_inputs_before(frame);
    }

    pub fn cache_rom(&mut self, data: Vec<u8>) {
        self.rom_data = Some(data);
    }

    pub fn outbound_for_client_channel(
        &self,
        client_id: u32,
        channel: ChannelKind,
    ) -> Option<OutboundTx> {
        if let Some(p) = self.players.values().find(|p| p.client_id == client_id) {
            return Some(p.outbounds.outbound_for_channel(channel));
        }
        self.spectators
            .iter()
            .find(|s| s.client_id == client_id)
            .map(|s| s.outbounds.outbound_for_channel(channel))
    }

    pub fn outbound_for_client_msg(&self, client_id: u32, msg_id: MsgId) -> Option<OutboundTx> {
        self.outbound_for_client_channel(client_id, channel_for_msg(msg_id))
    }

    pub fn set_client_channel_outbound(
        &mut self,
        client_id: u32,
        channel: ChannelKind,
        outbound: OutboundTx,
    ) {
        if let Some(p) = self.players.values_mut().find(|p| p.client_id == client_id) {
            p.outbounds.set_channel(channel, outbound);
            return;
        }
        if let Some(s) = self
            .spectators
            .iter_mut()
            .find(|s| s.client_id == client_id)
        {
            s.outbounds.set_channel(channel, outbound);
        }
    }

    pub fn clear_client_channel_outbound(&mut self, client_id: u32, channel: ChannelKind) {
        if let Some(p) = self.players.values_mut().find(|p| p.client_id == client_id) {
            p.outbounds.clear_channel(channel);
            return;
        }
        if let Some(s) = self
            .spectators
            .iter_mut()
            .find(|s| s.client_id == client_id)
        {
            s.outbounds.clear_channel(channel);
        }
    }

    pub fn record_inputs(&mut self, player_index: u8, start_frame: u32, buttons: &[u16]) {
        let Some(buffer) = self.input_buffers.get_mut(player_index as usize) else {
            return;
        };

        for (offset, &mask) in buttons.iter().enumerate() {
            let frame = start_frame.wrapping_add(offset as u32);
            match buffer.back() {
                None => {
                    buffer.push_back((frame, mask));
                }
                Some((last_frame, _)) if frame > *last_frame => {
                    buffer.push_back((frame, mask));
                }
                _ => {
                    // Out-of-order or duplicate frame: insert in sorted order, overwriting if present.
                    if let Some(pos) = buffer.iter().position(|(f, _)| *f >= frame) {
                        if buffer[pos].0 == frame {
                            buffer[pos].1 = mask;
                        } else {
                            buffer.insert(pos, (frame, mask));
                        }
                    } else {
                        buffer.push_back((frame, mask));
                    }
                }
            }
        }

        if let Some((snap_frame, _)) = &self.cached_state {
            self.prune_inputs_before(*snap_frame);
        }

        if !buttons.is_empty() {
            let end_frame = start_frame.wrapping_add(buttons.len().saturating_sub(1) as u32);
            self.current_frame = self.current_frame.max(end_frame);
        }

        self.apply_scheduled_activations();
    }

    fn prune_inputs_before(&mut self, frame: u32) {
        for buffer in &mut self.input_buffers {
            while let Some((f, _)) = buffer.front() {
                if *f < frame {
                    buffer.pop_front();
                } else {
                    break;
                }
            }
        }
    }

    /// Get all outbound channels for broadcast (routed by message type).
    pub fn all_outbounds_msg(&self, msg_id: MsgId) -> Vec<OutboundTx> {
        let mut result: Vec<_> = self
            .players
            .values()
            .map(|p| p.outbounds.outbound_for_msg(msg_id))
            .collect();
        result.extend(
            self.spectators
                .iter()
                .map(|s| s.outbounds.outbound_for_msg(msg_id)),
        );
        result
    }

    /// Switch player role.
    ///
    /// Returns a list of (client_id, new_role) for broadcast.
    pub fn switch_player_role(
        &mut self,
        client_id: u32,
        new_role: u8,
    ) -> Result<Vec<(u32, u8)>, &'static str> {
        // 1. Identify current role and validate request
        let current_spectator_pos = self
            .spectators
            .iter()
            .position(|s| s.client_id == client_id);
        let current_player_index = self
            .players
            .iter()
            .find(|(_, p)| p.client_id == client_id)
            .map(|(idx, _)| *idx);

        if current_spectator_pos.is_none() && current_player_index.is_none() {
            return Err("Client not in room");
        }

        // Early return if role is already the same
        if new_role == SPECTATOR_PLAYER_INDEX {
            if current_spectator_pos.is_some() {
                return Ok(vec![]);
            }
        } else if let Some(idx) = current_player_index
            && idx == new_role
        {
            return Ok(vec![]);
        }

        if new_role == SPECTATOR_PLAYER_INDEX {
            // Switch to spectator
            if let Some(p_idx) = current_player_index
                && let Some(p) = self.players.remove(&p_idx)
            {
                self.spectators.push(Spectator {
                    conn_id: p.conn_id,
                    client_id: p.client_id,
                    name: p.name,
                    outbounds: p.outbounds,
                });
                self.refresh_active_ports_from_players();
                return Ok(vec![(client_id, SPECTATOR_PLAYER_INDEX)]);
            }
        } else if new_role < MAX_PLAYERS as u8 {
            // Switch to player slot
            if let Some(occupant) = self.players.remove(&new_role) {
                // Target slot is occupied -> Swap
                // 1. We removed the occupant already.
                // 2. We need to remove the requestor from their current spot.
                let requestor = if let Some(p_idx) = current_player_index {
                    // Requestor was a player (swapping slots)
                    self.players
                        .remove(&p_idx)
                        .ok_or("Failed to remove requestor from current slot")?
                } else {
                    // Requestor was a spectator
                    let pos = current_spectator_pos.ok_or("Requestor spectator not found")?;
                    let s = self.spectators.remove(pos);
                    Player {
                        conn_id: s.conn_id,
                        client_id: s.client_id,
                        player_index: 0,
                        name: s.name,
                        outbounds: s.outbounds,
                    }
                };

                // 3. Put requestor in new_role
                let requestor_cid = requestor.client_id;
                self.players.insert(
                    new_role,
                    Player {
                        player_index: new_role,
                        ..requestor
                    },
                );

                // 4. Put occupant in requestor's old spot
                let occupant_new_role = if let Some(old_p_idx) = current_player_index {
                    // Swap to player slot
                    self.players.insert(
                        old_p_idx,
                        Player {
                            player_index: old_p_idx,
                            ..occupant
                        },
                    );
                    old_p_idx
                } else {
                    // Swap to spectator list
                    self.spectators.push(Spectator {
                        conn_id: occupant.conn_id,
                        client_id: occupant.client_id,
                        name: occupant.name,
                        outbounds: occupant.outbounds,
                    });
                    SPECTATOR_PLAYER_INDEX
                };

                self.refresh_active_ports_from_players();
                return Ok(vec![
                    (requestor_cid, new_role),
                    (occupant.client_id, occupant_new_role),
                ]);
            } else {
                // Target slot is vacant
                let requestor = if let Some(p_idx) = current_player_index {
                    self.players
                        .remove(&p_idx)
                        .ok_or("Failed to remove requestor from vacant slot flip")?
                } else {
                    // Requestor is spectator
                    let pos = current_spectator_pos
                        .ok_or("Requestor spectator not found for vacant slot")?;
                    let s = self.spectators.remove(pos);
                    Player {
                        conn_id: s.conn_id,
                        client_id: s.client_id,
                        player_index: 0,
                        name: s.name,
                        outbounds: s.outbounds,
                    }
                };

                self.players.insert(
                    new_role,
                    Player {
                        player_index: new_role,
                        ..requestor
                    },
                );
                self.refresh_active_ports_from_players();
                return Ok(vec![(client_id, new_role)]);
            }
        }

        Err("Invalid role")
    }

    /// Handle LoadRom from a client.
    ///
    /// Returns list of recipients to forward the ROM to.
    /// Any player can load ROMs (no host restriction).
    pub fn handle_load_rom(&mut self, sender_id: u32) -> Result<Vec<OutboundTx>, &'static str> {
        // Check if the sender is a player (not a spectator)
        let is_player = self.players.values().any(|p| p.client_id == sender_id);
        if !is_player {
            return Err("Only players can load a ROM");
        }

        self.started = false;
        self.loaded_players.clear();
        self.loaded_players.insert(sender_id); // Sender has obviously loaded it

        // Broadcast to everyone else
        let mut recipients = Vec::new();
        for p in self.players.values() {
            if p.client_id != sender_id {
                recipients.push(p.outbounds.outbound_for_msg(MsgId::LoadRom));
            }
        }
        for s in &self.spectators {
            if s.client_id != sender_id {
                recipients.push(s.outbounds.outbound_for_msg(MsgId::LoadRom));
            }
        }

        Ok(recipients)
    }

    /// Handle RomLoaded from a client.
    ///
    /// If all players have loaded, returns list of recipients to broadcast StartGame to.
    pub fn handle_rom_loaded(&mut self, sender_id: u32) -> Vec<OutboundTx> {
        self.loaded_players.insert(sender_id);

        if self.started {
            return Vec::new();
        }

        // Check if all players (not spectators) have loaded
        let all_loaded = self
            .players
            .values()
            .all(|p| self.loaded_players.contains(&p.client_id));

        if all_loaded && !self.players.is_empty() {
            self.started = true;
            // Broadcast StartGame to everyone
            self.all_outbounds_msg(MsgId::StartGame)
        } else {
            Vec::new()
        }
    }

    /// Handle PauseGame from a client.
    ///
    /// Returns list of recipients to broadcast PauseSync to.
    pub fn handle_pause_game(&mut self, sender_id: u32, paused: bool) -> Vec<OutboundTx> {
        // Check if the sender is a player (not a spectator)
        let is_player = self.players.values().any(|p| p.client_id == sender_id);
        if !is_player {
            return Vec::new();
        }

        self.paused = paused;
        self.all_outbounds_msg(MsgId::PauseSync)
    }

    /// Handle ResetGame from a client.
    ///
    /// Returns list of recipients to broadcast ResetSync to.
    /// Also clears cached state and input buffers to prevent stale data for late joiners.
    pub fn handle_reset_game(&mut self, sender_id: u32) -> Vec<OutboundTx> {
        // Check if the sender is a player (not a spectator)
        let is_player = self.players.values().any(|p| p.client_id == sender_id);
        if !is_player {
            return Vec::new();
        }

        // Clear cached state - late joiners should not receive pre-reset state
        self.cached_state = None;

        // Clear input buffers - all previous inputs are now invalid
        for buffer in &mut self.input_buffers {
            buffer.clear();
        }

        tracing::info!(
            room_id = self.id,
            "Cleared cached state and input buffers on reset"
        );

        self.all_outbounds_msg(MsgId::ResetSync)
    }

    /// Handle SetSyncMode from a client.
    ///
    /// Only the host can change sync mode, and only before the game starts.
    /// Returns list of recipients to broadcast SyncModeChanged to, or error.
    pub fn handle_set_sync_mode(
        &mut self,
        sender_id: u32,
        mode: nesium_netproto::messages::session::SyncMode,
    ) -> Result<Vec<OutboundTx>, crate::proto_dispatch::error::HandlerError> {
        use crate::proto_dispatch::error::HandlerError;

        // Only host can change sync mode
        if sender_id != self.host_client_id {
            return Err(HandlerError::permission_denied());
        }

        // Cannot change sync mode after game has started
        if self.started {
            return Err(HandlerError::game_already_started());
        }

        // Update room sync mode
        self.sync_mode = mode;

        tracing::info!(
            room_id = self.id,
            sync_mode = ?mode,
            "Sync mode changed"
        );

        // Broadcast to all players
        Ok(self.all_outbounds_msg(MsgId::SyncModeChanged))
    }
}

/// Room manager.
#[derive(Default)]
pub struct RoomManager {
    rooms: HashMap<u32, Room>,
    /// Map client_id -> room_id for quick lookup.
    client_rooms: HashMap<u32, u32>,
    /// Random seed for ID permutation.
    id_seed: u32,
    /// Counter for stateless ID generation.
    next_room_index: u32,
}

impl RoomManager {
    pub fn new() -> Self {
        let mut rng = rand::rng();
        Self {
            rooms: HashMap::new(),
            client_rooms: HashMap::new(),
            id_seed: rng.random(),
            next_room_index: 0,
        }
    }

    /// Create a new room with a random-looking 6-digit ID.
    /// Returns None if the ID space is exhausted.
    pub fn create_room(&mut self, host_client_id: u32) -> Option<u32> {
        if self.rooms.len() >= ROOM_ID_COUNT as usize {
            return None;
        }

        // We use a Feistel Network (Format-Preserving Encryption) to map
        // 0..ROOM_ID_COUNT to a unique 6-digit ID sequence.
        // This is O(1) time and O(1) memory.
        let mut attempts = 0;
        let id = loop {
            let index = self.next_room_index;
            self.next_room_index = (self.next_room_index + 1) % ROOM_ID_COUNT;

            let permuted = self.permute_id(index);
            let candidate = ROOM_ID_MIN + permuted;

            if !self.rooms.contains_key(&candidate) {
                break candidate;
            }

            attempts += 1;
            if attempts >= ROOM_ID_COUNT {
                return None;
            }
        };

        let room = Room::new(id, host_client_id);
        self.rooms.insert(id, room);
        Some(id)
    }

    /// 20-bit Feistel Network to shuffle IDs in the range 0..ROOM_ID_COUNT.
    /// maps 0..2^20-1 to 0..2^20-1 uniquely.
    fn permute_id(&self, index: u32) -> u32 {
        let mut val = index;

        // Space: 2^20 = 1,048,576.
        // We cycle-walk if the result is >= ROOM_ID_COUNT.
        loop {
            let mut l = (val >> 10) & 0x3FF;
            let mut r = val & 0x3FF;

            for round in 0..3 {
                // Round function: simple mix of R, seed, and round.
                let f = ((r ^ self.id_seed ^ round)
                    .wrapping_mul(0x45d9f3b)
                    .wrapping_add(0x9e3779b9))
                    & 0x3FF;
                let next_l = r;
                let next_r = l ^ f;
                l = next_l;
                r = next_r;
            }

            val = (l << 10) | r;

            if val < ROOM_ID_COUNT {
                return val;
            }
            // Cycle walk: if outside range, permute again.
            // This is guaranteed to eventually hit a value in range.
        }
    }

    /// Get room by ID.
    pub fn get_room(&self, room_id: u32) -> Option<&Room> {
        self.rooms.get(&room_id)
    }

    /// Get room by ID (mutable).
    pub fn get_room_mut(&mut self, room_id: u32) -> Option<&mut Room> {
        self.rooms.get_mut(&room_id)
    }

    /// Get room for a client.
    pub fn get_client_room(&self, client_id: u32) -> Option<u32> {
        self.client_rooms.get(&client_id).copied()
    }

    /// Associate client with room.
    pub fn set_client_room(&mut self, client_id: u32, room_id: u32) {
        self.client_rooms.insert(client_id, room_id);
    }

    /// Remove client from tracking.
    pub fn remove_client(&mut self, client_id: u32) {
        self.client_rooms.remove(&client_id);
    }

    /// Remove empty room.
    pub fn remove_room(&mut self, room_id: u32) {
        self.rooms.remove(&room_id);
    }

    /// Remove a client from all P2P signaling watcher lists.
    pub fn remove_p2p_watcher(&mut self, client_id: u32) {
        let mut to_remove = Vec::new();
        for (room_id, room) in self.rooms.iter_mut() {
            room.remove_p2p_watcher(client_id);
            if room.is_empty() && room.p2p_watchers.is_empty() {
                // If this room was only used for signaling, drop it when unused.
                to_remove.push(*room_id);
            }
        }
        for room_id in to_remove {
            self.rooms.remove(&room_id);
        }
    }

    /// Clear P2P host info for rooms where the given client was the host.
    ///
    /// Returns a list of (room_id, watchers) for each room where the host was cleared,
    /// so the caller can broadcast `P2PHostDisconnected` to all watchers.
    pub fn clear_p2p_host_for_client(&mut self, client_id: u32) -> Vec<(u32, Vec<OutboundTx>)> {
        let mut result = Vec::new();
        for room in self.rooms.values_mut() {
            if let Some(host) = &room.p2p_host {
                if host.host_signal_client_id == client_id {
                    let room_id = room.id;
                    let watchers: Vec<OutboundTx> = room.p2p_watchers.values().cloned().collect();
                    room.p2p_host = None;
                    if !watchers.is_empty() {
                        result.push((room_id, watchers));
                    }
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_history_splits_on_holes() {
        let mut room = Room::new(1, 1);

        room.record_inputs(0, 0, &[1, 2]); // frames 0,1
        room.record_inputs(0, 3, &[4, 5]); // frames 3,4 (hole at 2)

        let history = room.get_input_history(0);
        assert_eq!(history.len(), 2);
        assert_eq!(history[0], (0, 0, vec![1, 2]));
        assert_eq!(history[1], (0, 3, vec![4, 5]));
    }

    #[test]
    fn record_inputs_inserts_out_of_order_and_overwrites() {
        let mut room = Room::new(1, 1);

        // Receive later frames first.
        room.record_inputs(0, 3, &[4, 5]); // frames 3,4

        // Fill earlier frames out-of-order.
        room.record_inputs(0, 0, &[1, 2, 3]); // frames 0,1,2

        // Overwrite an existing frame (rollback correction).
        room.record_inputs(0, 2, &[99]); // frame 2

        // History should be continuous and reflect the overwrite.
        let history = room.get_input_history(0);
        assert_eq!(history, vec![(0, 0, vec![1, 2, 99, 4, 5])]);
    }

    #[test]
    fn room_id_uniqueness_and_range() {
        let mgr = RoomManager::new();
        let mut seen = std::collections::HashSet::new();

        // Testing all 900k is slow in debug, let's test a sample.
        // But for verification, let's at least test 10,000 consecutive ones.
        for i in 0..10000 {
            let id = mgr.permute_id(i);
            assert!(id < ROOM_ID_COUNT, "ID {} out of range: {}", i, id);
            assert!(
                seen.insert(id),
                "Duplicate ID produced at index {}: {}",
                i,
                id
            );

            let room_id = ROOM_ID_MIN + id;
            assert!(room_id >= ROOM_ID_MIN && room_id <= ROOM_ID_MAX);
        }
    }
}
