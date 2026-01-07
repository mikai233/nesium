//! Room state management.
//!
//! Each room represents a netplay session with one or more players
//! and optional spectators.

use std::collections::{HashMap, VecDeque};

use crate::net::inbound::ConnId;
use crate::net::outbound::OutboundTx;
use nesium_netproto::constants::SPECTATOR_PLAYER_INDEX;

/// Maximum number of players per room.
pub const MAX_PLAYERS: usize = 4;

/// Player assignment in a room.
#[derive(Debug, Clone)]
pub struct Player {
    pub conn_id: ConnId,
    pub client_id: u32,
    pub player_index: u8,
    pub name: String,
    pub outbound: OutboundTx,
}

/// Spectator in a room.
#[derive(Debug, Clone)]
pub struct Spectator {
    pub conn_id: ConnId,
    pub client_id: u32,
    pub name: String,
    pub outbound: OutboundTx,
}

/// Room state.
#[derive(Debug)]
pub struct Room {
    /// Unique room ID.
    pub id: u32,
    /// Room code for joining.
    pub code: u32,
    /// ROM hash for validation.
    pub rom_hash: [u8; 16],
    /// Host client ID (first player to join).
    pub host_client_id: u32,
    /// Players in the room (player_index -> Player).
    pub players: HashMap<u8, Player>,
    /// Spectators in the room.
    pub spectators: Vec<Spectator>,
    /// Current authoritative frame number.
    pub current_frame: u32,
    /// Input buffer per player (frame -> buttons).
    pub input_buffers: [VecDeque<(u32, u16)>; MAX_PLAYERS],
    /// Last confirmed frame for each player.
    pub last_confirmed_frame: [u32; MAX_PLAYERS],
    /// Configured input delay frames.
    pub input_delay_frames: u8,
    /// Whether the game started.
    pub started: bool,
    /// Players that have loaded the ROM.
    pub loaded_players: std::collections::HashSet<u32>,
    /// Whether the game is paused.
    pub paused: bool,
    /// Cached save state for reconnection/late join (frame, data).
    pub cached_state: Option<(u32, Vec<u8>)>,
    /// ROM data for late joiners.
    pub rom_data: Option<Vec<u8>>,
}

impl Room {
    /// Create a new room.
    pub fn new(id: u32, code: u32, rom_hash: [u8; 16], host_client_id: u32) -> Self {
        Self {
            id,
            code,
            rom_hash,
            host_client_id,
            players: HashMap::new(),
            spectators: Vec::new(),
            current_frame: 0,
            input_buffers: Default::default(),
            last_confirmed_frame: [0; MAX_PLAYERS],
            input_delay_frames: 2,
            started: false,
            loaded_players: std::collections::HashSet::new(),
            paused: false,
            cached_state: None,
            rom_data: None,
        }
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
                return Some(player_index);
            }
        }
        None // Room is full
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
        for (i, buffer) in self.input_buffers.iter().enumerate() {
            let mut buttons = Vec::new();
            let mut base_frame = start_frame;
            let mut first = true;

            for (f, b) in buffer {
                if *f >= start_frame {
                    if first {
                        base_frame = *f;
                        first = false;
                    }
                    buttons.push(*b);
                }
            }

            if !buttons.is_empty() {
                history.push((i as u8, base_frame, buttons));
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
        for idx in self.players.keys() {
            if (*idx as usize) < 8 {
                mask |= 1u8 << *idx;
            }
        }
        mask
    }

    pub fn cache_state(&mut self, frame: u32, data: Vec<u8>) {
        self.cached_state = Some((frame, data));
        self.prune_inputs_before(frame);
    }

    pub fn cache_rom(&mut self, data: Vec<u8>) {
        self.rom_data = Some(data);
    }

    pub fn outbound_for_client(&self, client_id: u32) -> Option<OutboundTx> {
        if let Some(p) = self.players.values().find(|p| p.client_id == client_id) {
            return Some(p.outbound.clone());
        }
        self.spectators
            .iter()
            .find(|s| s.client_id == client_id)
            .map(|s| s.outbound.clone())
    }

    pub fn record_inputs(&mut self, player_index: u8, start_frame: u32, buttons: &[u16]) {
        let Some(buffer) = self.input_buffers.get_mut(player_index as usize) else {
            return;
        };

        let mut last = buffer.back().map(|(f, _)| *f).unwrap_or(u32::MIN);
        for (offset, &mask) in buttons.iter().enumerate() {
            let frame = start_frame.wrapping_add(offset as u32);
            if frame <= last {
                continue;
            }
            buffer.push_back((frame, mask));
            last = frame;
        }

        if let Some((snap_frame, _)) = &self.cached_state {
            self.prune_inputs_before(*snap_frame);
        }

        if !buttons.is_empty() {
            let end_frame = start_frame.wrapping_add(buttons.len().saturating_sub(1) as u32);
            self.current_frame = self.current_frame.max(end_frame);
        }
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

    /// Get all outbound channels for broadcast.
    pub fn all_outbounds(&self) -> Vec<OutboundTx> {
        let mut result: Vec<_> = self.players.values().map(|p| p.outbound.clone()).collect();
        result.extend(self.spectators.iter().map(|s| s.outbound.clone()));
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
        let current_role_is_spectator = self.spectators.iter().any(|s| s.client_id == client_id);
        let current_player_index = self
            .players
            .iter()
            .find(|(_, p)| p.client_id == client_id)
            .map(|(algo, _)| *algo);

        if !current_role_is_spectator && current_player_index.is_none() {
            return Err("Client not in room");
        }

        if new_role == SPECTATOR_PLAYER_INDEX {
            // Switch to spectator
            if current_role_is_spectator {
                return Ok(vec![]); // No change
            }
            if let Some(p_idx) = current_player_index {
                let p = self.players.remove(&p_idx).unwrap();
                self.spectators.push(Spectator {
                    conn_id: p.conn_id,
                    client_id: p.client_id,
                    name: p.name,
                    outbound: p.outbound,
                });
                return Ok(vec![(client_id, SPECTATOR_PLAYER_INDEX)]);
            }
        } else if new_role < MAX_PLAYERS as u8 {
            // Switch to player slot
            if let Some(occupant) = self.players.remove(&new_role) {
                // Target slot is occupied -> Swap
                // 1. We removed the occupant temporarily.
                // 2. We need to remove the requestor from their current spot.

                let requestor = if let Some(p_idx) = current_player_index {
                    self.players.remove(&p_idx).unwrap()
                } else {
                    // Requestor is spectator
                    let pos = self
                        .spectators
                        .iter()
                        .position(|s| s.client_id == client_id)
                        .unwrap();
                    let s = self.spectators.remove(pos);
                    Player {
                        conn_id: s.conn_id,
                        client_id: s.client_id,
                        player_index: 0,
                        name: s.name,
                        outbound: s.outbound,
                    }
                };

                // 3. Put requestor in new_role
                self.players.insert(
                    new_role,
                    Player {
                        player_index: new_role,
                        ..requestor
                    },
                );

                // 4. Put occupant in requestor's old spot
                let occupant_new_role = if let Some(old_idx) = current_player_index {
                    // Swap to player
                    self.players.insert(
                        old_idx,
                        Player {
                            player_index: old_idx,
                            ..occupant
                        },
                    );
                    old_idx
                } else {
                    // Swap to spectator
                    self.spectators.push(Spectator {
                        conn_id: occupant.conn_id,
                        client_id: occupant.client_id,
                        name: occupant.name,
                        outbound: occupant.outbound,
                    });
                    SPECTATOR_PLAYER_INDEX
                };

                return Ok(vec![
                    (client_id, new_role),
                    (occupant.client_id, occupant_new_role),
                ]);
            } else {
                // Target slot is vacant
                let requestor = if let Some(p_idx) = current_player_index {
                    if p_idx == new_role {
                        return Ok(vec![]); // Same role
                    }
                    self.players.remove(&p_idx).unwrap()
                } else {
                    // Requestor is spectator
                    let pos = self
                        .spectators
                        .iter()
                        .position(|s| s.client_id == client_id)
                        .unwrap();
                    let s = self.spectators.remove(pos);
                    Player {
                        conn_id: s.conn_id,
                        client_id: s.client_id,
                        player_index: 0,
                        name: s.name,
                        outbound: s.outbound,
                    }
                };

                self.players.insert(
                    new_role,
                    Player {
                        player_index: new_role,
                        ..requestor
                    },
                );
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
                recipients.push(p.outbound.clone());
            }
        }
        for s in &self.spectators {
            if s.client_id != sender_id {
                recipients.push(s.outbound.clone());
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
            self.all_outbounds()
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
        self.all_outbounds()
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

        self.all_outbounds()
    }
}

/// Room manager.
#[derive(Default)]
pub struct RoomManager {
    rooms: HashMap<u32, Room>,
    next_room_id: u32,
    /// Map client_id -> room_id for quick lookup.
    client_rooms: HashMap<u32, u32>,
}

impl RoomManager {
    pub fn new() -> Self {
        Self {
            rooms: HashMap::new(),
            next_room_id: 1,
            client_rooms: HashMap::new(),
        }
    }

    /// Create a new room.
    pub fn create_room(&mut self, rom_hash: [u8; 16], host_client_id: u32) -> u32 {
        let id = self.next_room_id;
        self.next_room_id += 1;
        let code = id; // Simple: room code = room id for now
        let room = Room::new(id, code, rom_hash, host_client_id);
        self.rooms.insert(id, room);
        id
    }

    /// Get room by ID (mutable).
    pub fn get_room_mut(&mut self, room_id: u32) -> Option<&mut Room> {
        self.rooms.get_mut(&room_id)
    }

    /// Find room by code.
    pub fn find_by_code(&self, code: u32) -> Option<&Room> {
        self.rooms.values().find(|r| r.code == code)
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
}
