//! Session handler for processing server messages and managing state transitions.
//!
//! This module handles the protocol logic for:
//! - Hello/Welcome handshake
//! - Room joining
//! - Input relay
//! - State synchronization

use std::net::SocketAddr;
use std::sync::Arc;

use nesium_netproto::{
    channel::ChannelKind,
    constants::SPECTATOR_PLAYER_INDEX,
    messages::{
        input::{InputBatch, RelayInputs},
        session::{
            ActivatePort, BeginCatchUp, ErrorCode, ErrorMsg, FallbackToRelay, Hello, JoinAck,
            JoinRoom, LoadRom, PauseGame, PauseSync, ProvideState, QueryRoom, RejoinReady,
            RequestFallbackRelay, RequestState, ResetGame, ResetSync, RomLoaded, RoomInfo,
            StartGame, SyncMode as ProtoSyncMode, SyncState, TransportKind, Welcome,
        },
        sync::{Ping, Pong},
    },
    msg_id::MsgId,
};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info, warn};

use crate::{
    error::NetplayError,
    input_provider::{NetplayInputProvider, SharedInputProvider},
    session::SessionState,
    sync::SyncMode as ClientSyncMode,
    tcp_client::{PacketOwned, TcpClientEvent, TcpClientHandle},
};

/// Configuration for starting a netplay session.
#[derive(Debug, Clone)]
pub struct NetplayConfig {
    /// Player name to display.
    pub name: String,
    /// Preferred transport used in the Hello handshake.
    pub transport: TransportKind,
    /// Whether to join as spectator.
    pub spectator: bool,
    /// Room code to join (0 = create new room).
    pub room_code: u32,
    /// Desired role on join (0-3 player, 0xFE auto, 0xFF spectator).
    pub desired_role: u8,
    /// True if the local runtime already has the ROM loaded.
    pub has_rom: bool,
}

#[derive(Debug, Clone)]
pub enum NetplayEvent {
    LoadRom(Vec<u8>),
    StartGame,
    PauseSync {
        paused: bool,
    },
    ResetSync(u8),
    SyncState(u32, Vec<u8>),
    /// A player has left the room.
    PlayerLeft {
        player_index: u8,
    },
    /// Server sent an error.
    Error {
        code: ErrorCode,
    },
    /// Server instructed us to switch to relay mode on `relay_addr`.
    FallbackToRelay {
        relay_addr: SocketAddr,
        relay_room_code: u32,
        reason: String,
    },
}

#[derive(Debug)]
pub enum NetplayCommand {
    CreateRoom,
    JoinRoom {
        room_code: u32,
        desired_role: u8,
        has_rom: bool,
    },
    SwitchRole(u8),
    SendRom(Vec<u8>),
    RomLoaded,
    SendPause(bool),
    SendReset(u8),
    RequestState,
    ProvideState(u32, Vec<u8>),
    /// Send local input for a frame: (frame_number, buttons)
    SendInput(u32, u16),
    /// Lockstep reconnect: client finished catch-up and is ready for activation.
    RejoinReady(u32),
    /// Host-only: ask the server to instruct all clients to reconnect to a relay server.
    RequestFallbackRelay {
        relay_addr: SocketAddr,
        relay_room_code: u32,
        reason: String,
    },
    /// Query room occupancy/state before joining (pre-join UI).
    QueryRoom {
        room_code: u32,
        resp: oneshot::Sender<Result<RoomInfo, String>>,
    },
}

/// Session handler that processes events and updates session state.
pub struct SessionHandler {
    /// TCP client handle for sending messages.
    client: TcpClientHandle,
    /// Session configuration.
    config: NetplayConfig,
    /// Shared input provider.
    input_provider: Arc<SharedInputProvider>,
    /// Event receiver from TCP client.
    event_rx: mpsc::Receiver<TcpClientEvent>,
    /// Command receiver for external control.
    command_rx: mpsc::Receiver<NetplayCommand>,
    /// Channel to send game events up to the runtime/UI.
    game_event_tx: mpsc::Sender<NetplayEvent>,
    /// Cached per-join flag: whether the local runtime already has the ROM.
    last_join_has_rom: bool,
    next_query_id: u32,
    pending_query: Option<(u32, oneshot::Sender<Result<RoomInfo, String>>)>,
}

impl SessionHandler {
    /// Create a new session handler.
    pub fn new(
        client: TcpClientHandle,
        config: NetplayConfig,
        input_provider: Arc<SharedInputProvider>,
        event_rx: mpsc::Receiver<TcpClientEvent>,
        game_event_tx: mpsc::Sender<NetplayEvent>,
    ) -> (Self, mpsc::Sender<NetplayCommand>) {
        let (tx, rx) = mpsc::channel(32);

        // Setup input sending callback
        let tx_clone = tx.clone();
        input_provider.set_on_send_input(Box::new(move |frame, buttons| {
            let _ = tx_clone.try_send(NetplayCommand::SendInput(frame, buttons));
        }));

        let tx_clone_state = tx.clone();
        input_provider.set_on_send_state(Box::new(move |frame, data| {
            let _ = tx_clone_state.try_send(NetplayCommand::ProvideState(frame, data.to_vec()));
        }));

        let tx_clone_rejoin = tx.clone();
        input_provider.set_on_send_rejoin_ready(Box::new(move |frame| {
            let _ = tx_clone_rejoin.try_send(NetplayCommand::RejoinReady(frame));
        }));

        input_provider.with_session_mut(|s| {
            s.local_name = config.name.clone();
        });

        (
            Self {
                client,
                config,
                input_provider,
                event_rx,
                command_rx: rx,
                game_event_tx,
                last_join_has_rom: false,
                next_query_id: 1,
                pending_query: None,
            },
            tx,
        )
    }

    /// Run the session handler loop.
    ///
    /// This processes events from the TCP client and updates session state accordingly.
    pub async fn run(&mut self) -> Result<(), NetplayError> {
        loop {
            // We use biased select to prioritize incoming network packets over outgoing commands.
            // This is crucial for lockstep netplay to reduce input latency.
            tokio::select! {
                biased;

                event = self.event_rx.recv() => {
                    match event {
                        Some(TcpClientEvent::Connected) => {
                            info!("Connected to server");
                            self.handle_connected().await?;
                        }
                        Some(TcpClientEvent::Disconnected { reason }) => {
                            self.handle_disconnected(&reason);
                            return Ok(());
                        }
                        Some(TcpClientEvent::Packet(packet)) => {
                            self.handle_packet(packet).await?;
                        }
                        Some(TcpClientEvent::Error(e)) => {
                            error!("Connection error: {}", e);
                            return Err(NetplayError::ConnectionFailed(e));
                        }
                        None => {
                            debug!("Event channel closed");
                            return Ok(());
                        }
                    }
                }
                cmd = self.command_rx.recv() => {
                    match cmd {
                        Some(NetplayCommand::CreateRoom) => {
                            self.send_join_room(0, 0, false).await?;
                        }
                        Some(NetplayCommand::JoinRoom { room_code, desired_role, has_rom }) => {
                            self.send_join_room(room_code, desired_role, has_rom).await?;
                        }
                        Some(NetplayCommand::SwitchRole(role)) => {
                            self.send_switch_role(role).await?;
                        }
                        Some(NetplayCommand::SendRom(data)) => {
                            self.send_load_rom(data).await?;
                        }
                        Some(NetplayCommand::RomLoaded) => {
                            self.send_rom_loaded().await?;
                        }
                        Some(NetplayCommand::SendPause(paused)) => {
                            self.send_pause_game(paused).await?;
                        }
                        Some(NetplayCommand::SendReset(kind)) => {
                            self.send_reset_game(kind).await?;
                        }
                        Some(NetplayCommand::RequestState) => {
                            self.send_request_state().await?;
                        }
                        Some(NetplayCommand::ProvideState(frame, data)) => {
                            self.send_provide_state(frame, data).await?;
                        }
                        Some(NetplayCommand::SendInput(frame, buttons)) => {
                            self.send_input(frame, buttons).await?;
                        }
                        Some(NetplayCommand::RejoinReady(frame)) => {
                            self.send_rejoin_ready(frame).await?;
                        }
                        Some(NetplayCommand::RequestFallbackRelay { relay_addr, relay_room_code, reason }) => {
                            self.send_request_fallback_relay(relay_addr, relay_room_code, reason).await?;
                        }
                        Some(NetplayCommand::QueryRoom { room_code, resp }) => {
                            self.send_query_room(room_code, resp).await?;
                        }
                        None => {
                            debug!("Command channel closed");
                        }
                    }
                }
            }
        }
    }

    /// Handle successful connection - send Hello message.
    async fn handle_connected(&mut self) -> Result<(), NetplayError> {
        self.input_provider.with_session(|session| {
            session.state = SessionState::Connecting;
            session.transport = Some(self.config.transport);
        });

        // Send Hello message
        let hello = Hello {
            client_nonce: rand_nonce(),
            transport: self.config.transport,
            proto_min: nesium_netproto::constants::VERSION,
            proto_max: nesium_netproto::constants::VERSION,
            name: self.config.name.clone(),
        };

        self.client.send_message(&hello).await?;

        self.input_provider.with_session(|session| {
            session.state = SessionState::Handshake;
        });

        Ok(())
    }

    /// Handle disconnection.
    fn handle_disconnected(&mut self, reason: &str) {
        warn!("Session disconnected: {}", reason);
        if let Some((_id, resp)) = self.pending_query.take() {
            let _ = resp.send(Err(format!("Disconnected: {reason}")));
        }
        self.input_provider.set_active(false);
        self.input_provider.with_session(|session| {
            session.state = SessionState::Disconnected;
            session.transport = None;
            session.tcp_fallback_from_quic = false;
        });
    }

    /// Handle incoming packet.
    async fn handle_packet(&mut self, packet: PacketOwned) -> Result<(), NetplayError> {
        debug!("Received {:?}", packet.msg_id());

        match packet.msg_id() {
            MsgId::Welcome => self.handle_welcome(&packet).await?,
            MsgId::JoinAck => self.handle_join_ack(&packet).await?,
            MsgId::RoleChanged => self.handle_role_changed(&packet).await?,
            MsgId::PlayerJoined => self.handle_player_joined(&packet).await?,
            MsgId::RelayInputs => self.handle_relay_inputs(&packet)?,
            MsgId::Pong => self.handle_pong(&packet)?,
            MsgId::LoadRom => self.handle_load_rom(&packet).await?,
            MsgId::StartGame => self.handle_start_game(&packet).await?,
            MsgId::BeginCatchUp => self.handle_begin_catch_up(&packet).await?,
            MsgId::PauseSync => self.handle_pause_sync(&packet).await?,
            MsgId::ResetSync => self.handle_reset_sync(&packet).await?,
            MsgId::SyncState => self.handle_sync_state(&packet).await?,
            MsgId::RequestState => self.handle_request_state(&packet).await?,
            MsgId::PlayerLeft => self.handle_player_left(&packet).await?,
            MsgId::ActivatePort => self.handle_activate_port(&packet).await?,
            MsgId::FallbackToRelay => self.handle_fallback_to_relay(&packet).await?,
            MsgId::RoomInfo => self.handle_room_info(&packet).await?,
            MsgId::ErrorMsg => self.handle_error(&packet).await?,
            msg => {
                debug!("Ignoring unhandled message type: {:?}", msg);
            }
        }

        Ok(())
    }

    /// Handle Welcome message - server accepted our Hello.
    async fn handle_welcome(&mut self, packet: &PacketOwned) -> Result<(), NetplayError> {
        let welcome: Welcome =
            postcard::from_bytes(&packet.payload).map_err(|e| NetplayError::Protocol(e.into()))?;

        info!(
            "Received Welcome: client_id={}, room_id={}, input_delay={}",
            welcome.assigned_client_id, welcome.room_id, welcome.input_delay_frames
        );

        // Best-effort: attach secondary channels to avoid HOL blocking on large transfers.
        if let Err(e) = self
            .client
            .attach_channel(welcome.session_token, ChannelKind::Bulk)
            .await
        {
            warn!(error = %e, "Failed to attach bulk channel; falling back to control");
        }
        if let Err(e) = self
            .client
            .attach_channel(welcome.session_token, ChannelKind::Input)
            .await
        {
            warn!(error = %e, "Failed to attach input channel; falling back to control");
        }

        self.input_provider.with_session(|session| {
            session.client_id = welcome.assigned_client_id;
            session.room_id = welcome.room_id;
            session.server_nonce = welcome.server_nonce;
            session.input_delay_frames = welcome.input_delay_frames;
            session.rewind_capacity = welcome.rewind_capacity;
            session.state = SessionState::WaitingForRoom;
        });
        // Keep sync strategy's delay in sync with negotiated session delay.
        self.input_provider
            .set_input_delay(welcome.input_delay_frames as u32);

        // If we have a room code, join it; otherwise wait for assignment
        if self.config.room_code != 0 {
            self.send_join_room(
                self.config.room_code,
                self.config.desired_role,
                self.config.has_rom,
            )
            .await?;
        }

        Ok(())
    }

    /// Send JoinRoom request.
    async fn send_join_room(
        &mut self,
        room_code: u32,
        desired_role: u8,
        has_rom: bool,
    ) -> Result<(), NetplayError> {
        // Sync mode is decided by the room at creation time (host sets it once).
        // When joining an existing room, do not send any preference.
        let preferred_sync_mode = if room_code == 0 {
            Some(match self.input_provider.sync_mode() {
                ClientSyncMode::Lockstep => ProtoSyncMode::Lockstep,
                ClientSyncMode::Rollback => ProtoSyncMode::Rollback,
            })
        } else {
            None
        };

        // For room creation, the host is always P1 and ROM is handled via LoadRom.
        let (desired_role, has_rom) = if room_code == 0 {
            (0u8, false)
        } else {
            (
                if self.config.spectator {
                    SPECTATOR_PLAYER_INDEX
                } else {
                    desired_role
                },
                has_rom,
            )
        };
        self.last_join_has_rom = has_rom;

        let join = JoinRoom {
            room_code,
            preferred_sync_mode,
            desired_role,
            has_rom,
        };

        self.client.send_message(&join).await?;

        Ok(())
    }

    /// Send SwitchRole request.
    async fn send_switch_role(&mut self, new_role: u8) -> Result<(), NetplayError> {
        let req = nesium_netproto::messages::session::SwitchRole { new_role };

        self.client.send_message(&req).await?;

        Ok(())
    }

    /// Handle JoinAck - room join succeeded.
    async fn handle_join_ack(&mut self, packet: &PacketOwned) -> Result<(), NetplayError> {
        let ack: JoinAck =
            postcard::from_bytes(&packet.payload).map_err(|e| NetplayError::Protocol(e.into()))?;

        if !ack.ok {
            return Err(NetplayError::RoomJoinFailed("Server rejected join".into()));
        }

        info!(
            "Joined room: player_index={}, start_frame={}, sync_mode={:?}",
            ack.player_index, ack.start_frame, ack.sync_mode
        );

        // Room is authoritative: always switch to the server-selected sync mode.
        self.input_provider.set_sync_mode(match ack.sync_mode {
            ProtoSyncMode::Lockstep => ClientSyncMode::Lockstep,
            ProtoSyncMode::Rollback => ClientSyncMode::Rollback,
        });

        let is_spectator = ack.player_index == SPECTATOR_PLAYER_INDEX;
        if is_spectator {
            self.input_provider
                .set_local_input_allowed_from_effective_frame(0);
            self.input_provider.with_session_mut(|session| {
                session.state = SessionState::Spectating {
                    start_frame: ack.start_frame,
                };
                session.local_player_index = None;
                session.current_frame = ack.start_frame;
                session.room_id = ack.room_id;
            });
        } else {
            self.input_provider.with_session_mut(|session| {
                session.state = SessionState::Playing {
                    start_frame: ack.start_frame,
                    player_index: ack.player_index,
                };
                session.local_player_index = Some(ack.player_index);
                session.current_frame = ack.start_frame;
                session.room_id = ack.room_id;
            });
            if ack.pending_activation {
                // Lockstep reconnect: stay inactive (0 input) until the server schedules activation.
                self.input_provider.arm_rejoin_ready();
                self.input_provider
                    .set_port_active(ack.player_index as usize, false);
            } else {
                self.input_provider
                    .set_local_input_allowed_from_effective_frame(0);
                // Mark own port as active to prevent lockstep deadlock in solo play
                self.input_provider
                    .set_port_active(ack.player_index as usize, true);
            }

            // If we joined as player 2 (index 1), we know player 1 (index 0) exists.
            if ack.player_index == 1 {
                self.input_provider.set_port_active(0, true);
            }
        }

        self.input_provider.set_local_player(if is_spectator {
            None
        } else {
            Some(ack.player_index)
        });
        // NOTE: Do NOT call set_active(true) here. Lockstep should only start after StartGame.

        // If we already have the ROM loaded locally, confirm immediately so the server can begin
        // late-join catch-up (started rooms) or pre-start synchronization without waiting for LoadRom.
        if self.last_join_has_rom {
            self.send_rom_loaded().await?;
        }

        Ok(())
    }

    async fn send_query_room(
        &mut self,
        room_code: u32,
        resp: oneshot::Sender<Result<RoomInfo, String>>,
    ) -> Result<(), NetplayError> {
        // Only one outstanding query is supported (UI can debounce).
        let request_id = self.next_query_id;
        self.next_query_id = self.next_query_id.wrapping_add(1).max(1);
        self.pending_query = Some((request_id, resp));

        let msg = QueryRoom {
            request_id,
            room_code,
        };
        self.client.send_message(&msg).await?;
        Ok(())
    }

    fn build_input_batches(items: Vec<(u32, u16)>) -> Vec<InputBatch> {
        use std::collections::BTreeMap;

        if items.is_empty() {
            return Vec::new();
        }

        // Ensure deterministic ordering and collapse duplicates (last write wins).
        let mut by_frame = BTreeMap::<u32, u16>::new();
        for (frame, buttons) in items {
            by_frame.insert(frame, buttons);
        }

        let mut batches = Vec::<InputBatch>::new();
        let mut current_start: Option<u32> = None;
        let mut current_buttons: Vec<u16> = Vec::new();
        let mut prev_frame: Option<u32> = None;

        for (frame, buttons) in by_frame {
            let contiguous = prev_frame
                .and_then(|prev| prev.checked_add(1))
                .map(|expected| expected == frame)
                .unwrap_or(false);

            if current_start.is_none() {
                current_start = Some(frame);
            } else if !contiguous {
                batches.push(InputBatch {
                    start_frame: current_start.unwrap(),
                    buttons: std::mem::take(&mut current_buttons),
                });
                current_start = Some(frame);
            }

            current_buttons.push(buttons);
            prev_frame = Some(frame);
        }

        if let Some(start_frame) = current_start {
            batches.push(InputBatch {
                start_frame,
                buttons: current_buttons,
            });
        }

        batches
    }

    /// Handle RoleChanged - server notified us of a role change.
    async fn handle_role_changed(&mut self, packet: &PacketOwned) -> Result<(), NetplayError> {
        let change: nesium_netproto::messages::session::RoleChanged =
            postcard::from_bytes(&packet.payload).map_err(|e| NetplayError::Protocol(e.into()))?;

        info!(
            "Role changed: client_id={}, new_role={}",
            change.client_id, change.new_role
        );

        let my_client_id = self.input_provider.with_session(|s| s.client_id);
        if change.client_id == my_client_id {
            // It's me!
            self.input_provider.with_session_mut(|session| {
                if change.new_role == SPECTATOR_PLAYER_INDEX {
                    // Became spectator
                    if let SessionState::Playing { start_frame, .. } = session.state {
                        session.state = SessionState::Spectating { start_frame };
                    }
                    session.local_player_index = None;
                } else {
                    // Became player
                    if let SessionState::Spectating { start_frame } = session.state {
                        session.state = SessionState::Playing {
                            start_frame,
                            player_index: change.new_role,
                        };
                    } else if let SessionState::Playing { start_frame, .. } = session.state {
                        // Changed player index
                        session.state = SessionState::Playing {
                            start_frame,
                            player_index: change.new_role,
                        };
                    }
                    session.local_player_index = Some(change.new_role);
                }
            });

            self.input_provider
                .set_local_player(if change.new_role == SPECTATOR_PLAYER_INDEX {
                    None
                } else {
                    Some(change.new_role)
                });
        } else {
            // It's someone else
            self.input_provider.with_session_mut(|session| {
                if let Some(player) = session.players.get_mut(&change.client_id) {
                    player.player_index = change.new_role;
                }
            });
        }

        Ok(())
    }

    /// Handle PlayerJoined - mark that player's port as active to prevent silent desync.
    ///
    /// Without this, an existing client may keep treating the new port as "inactive" and
    /// continue advancing frames using implicit 0 inputs until the first input packet arrives,
    /// which can drift by 1–2 frames and cause divergence.
    async fn handle_player_joined(&mut self, packet: &PacketOwned) -> Result<(), NetplayError> {
        let msg: nesium_netproto::messages::session::PlayerJoined =
            postcard::from_bytes(&packet.payload).map_err(|e| NetplayError::Protocol(e.into()))?;

        info!(
            client_id = msg.client_id,
            player_index = msg.player_index,
            name = %msg.name,
            "Player joined"
        );

        self.input_provider.with_session_mut(|session| {
            // Add to player list
            session.players.insert(
                msg.client_id,
                crate::session::RemotePlayer {
                    client_id: msg.client_id,
                    name: msg.name,
                    player_index: msg.player_index,
                },
            );
        });

        // If the game is already running, do not activate immediately; wait for server `ActivatePort`.
        if msg.player_index != SPECTATOR_PLAYER_INDEX && !self.input_provider.is_active() {
            self.input_provider
                .set_port_active(msg.player_index as usize, true);
        }

        Ok(())
    }

    /// Handle RelayInputs message (inputs from other players).
    fn handle_relay_inputs(&mut self, packet: &PacketOwned) -> Result<(), NetplayError> {
        let relay: RelayInputs =
            postcard::from_bytes(&packet.payload).map_err(|e| NetplayError::Protocol(e.into()))?;

        // Push inputs to provider
        let player_index = relay.player_index as usize;
        for (i, buttons) in relay.buttons.iter().enumerate() {
            let frame = relay.base_frame + i as u32;
            self.input_provider
                .push_remote_input(player_index, frame, *buttons);
        }

        Ok(())
    }

    /// Handle Pong - RTT measurement response.
    fn handle_pong(&mut self, packet: &PacketOwned) -> Result<(), NetplayError> {
        let pong: Pong =
            postcard::from_bytes(&packet.payload).map_err(|e| NetplayError::Protocol(e.into()))?;

        let now_ms = current_time_ms();
        let rtt = now_ms.saturating_sub(pong.t_ms);
        debug!("Ping RTT: {}ms", rtt);

        // TODO: Update adaptive input delay based on RTT

        Ok(())
    }

    /// Handle Error message from server.
    async fn handle_error(&mut self, packet: &PacketOwned) -> Result<(), NetplayError> {
        let msg: ErrorMsg = match postcard::from_bytes(&packet.payload) {
            Ok(m) => m,
            Err(e) => {
                warn!("Failed to decode ErrorMsg: {:?}", e);
                return Ok(());
            }
        };

        warn!(code = ?msg.code, "Received error from server");

        // Notify UI layer
        let _ = self
            .game_event_tx
            .send(NetplayEvent::Error { code: msg.code })
            .await;

        Ok(())
    }

    /// Process and buffer local input.
    async fn send_input(&mut self, frame: u32, buttons: u16) -> Result<(), NetplayError> {
        let batches = self.input_provider.with_session_mut(|session| {
            session.queue_local_input(frame, buttons);

            // Check if we have enough pending inputs to send a batch
            // Send in batches to reduce overhead while maintaining low latency.
            let pending_count = session.pending_inputs_count();
            if pending_count >= 1 {
                // Drain and send
                let items = session.drain_pending_inputs(30); // Max batch size
                Self::build_input_batches(items)
            } else {
                Vec::new()
            }
        });

        for batch in batches {
            self.client.send_message(&batch).await?;
        }
        Ok(())
    }

    /// Send a ping for RTT measurement.
    pub async fn send_ping(&mut self) -> Result<(), NetplayError> {
        let ping = Ping {
            t_ms: current_time_ms(),
        };

        self.client.send_message(&ping).await?;

        Ok(())
    }

    /// Handle LoadRom message from key.
    async fn handle_load_rom(&mut self, packet: &PacketOwned) -> Result<(), NetplayError> {
        let load: LoadRom =
            postcard::from_bytes(&packet.payload).map_err(|e| NetplayError::Protocol(e.into()))?;

        info!("Received LoadRom: {} bytes", load.data.len());
        if let Err(e) = self
            .game_event_tx
            .send(NetplayEvent::LoadRom(load.data))
            .await
        {
            error!("Failed to send LoadRom event: {}", e);
        }

        Ok(())
    }

    /// Handle StartGame message.
    async fn handle_start_game(&mut self, packet: &PacketOwned) -> Result<(), NetplayError> {
        let msg: StartGame =
            postcard::from_bytes(&packet.payload).map_err(|e| NetplayError::Protocol(e.into()))?;

        info!(
            active_ports_mask = msg.active_ports_mask,
            "Received StartGame - activating lockstep"
        );

        // Initialize active ports from server-provided mask BEFORE activating lockstep.
        // Always clear stale port state first (important across reconnects / role changes).
        for i in 0..4 {
            self.input_provider.set_port_active(i, false);
        }
        for i in 0..4 {
            let active = (msg.active_ports_mask & (1u8 << (i as u8))) != 0;
            self.input_provider.set_port_active(i, active);
        }

        // NOW activate lockstep - game is ready to begin
        self.input_provider.set_active(true);
        self.input_provider.set_catch_up_target_frame(None);

        if let Err(e) = self.game_event_tx.send(NetplayEvent::StartGame).await {
            error!("Failed to send StartGame event: {}", e);
        }
        Ok(())
    }

    /// Handle BeginCatchUp (late join resync).
    ///
    /// This is the "activation" signal for late joiners (instead of StartGame).
    async fn handle_begin_catch_up(&mut self, packet: &PacketOwned) -> Result<(), NetplayError> {
        let msg: BeginCatchUp =
            postcard::from_bytes(&packet.payload).map_err(|e| NetplayError::Protocol(e.into()))?;

        info!(
            snapshot_frame = msg.snapshot_frame,
            target_frame = msg.target_frame,
            active_ports_mask = msg.active_ports_mask,
            "Received BeginCatchUp - activating lockstep"
        );

        // Always clear stale port state first (important across reconnects / role changes).
        for i in 0..4 {
            self.input_provider.set_port_active(i, false);
        }
        for i in 0..4 {
            let active = (msg.active_ports_mask & (1u8 << (i as u8))) != 0;
            self.input_provider.set_port_active(i, active);
        }

        self.input_provider.set_active(true);
        self.input_provider
            .set_catch_up_target_frame(Some(msg.target_frame));

        // Reuse StartGame event: it tells the UI/runtime to unpause and begin running frames.
        if let Err(e) = self.game_event_tx.send(NetplayEvent::StartGame).await {
            error!("Failed to send StartGame event: {}", e);
        }

        Ok(())
    }

    /// Send LoadRom message.
    async fn send_load_rom(&mut self, data: Vec<u8>) -> Result<(), NetplayError> {
        let req = LoadRom { data };

        self.client.send_message(&req).await?;

        Ok(())
    }

    /// Send RomLoaded message.
    async fn send_rom_loaded(&mut self) -> Result<(), NetplayError> {
        let req = RomLoaded {};

        self.client.send_message(&req).await?;

        Ok(())
    }

    /// Handle PauseSync message.
    async fn handle_pause_sync(&mut self, packet: &PacketOwned) -> Result<(), NetplayError> {
        let msg: PauseSync =
            postcard::from_bytes(&packet.payload).map_err(|e| NetplayError::Protocol(e.into()))?;

        info!(paused = msg.paused, "Received PauseSync");
        let _ = self
            .game_event_tx
            .send(NetplayEvent::PauseSync { paused: msg.paused })
            .await;
        Ok(())
    }

    /// Handle ResetSync message.
    async fn handle_reset_sync(&mut self, packet: &PacketOwned) -> Result<(), NetplayError> {
        info!("Received ResetSync");
        let msg: ResetSync =
            postcard::from_bytes(&packet.payload).map_err(|e| NetplayError::Protocol(e.into()))?;

        let _ = self
            .game_event_tx
            .send(NetplayEvent::ResetSync(msg.kind))
            .await;
        Ok(())
    }

    /// Handle SyncState message.
    async fn handle_sync_state(&mut self, packet: &PacketOwned) -> Result<(), NetplayError> {
        let msg: SyncState =
            postcard::from_bytes(&packet.payload).map_err(|e| NetplayError::Protocol(e.into()))?;

        info!(
            size = msg.data.len(),
            frame = msg.frame,
            "Received SyncState"
        );

        // Late-join resync: update the network frame offset to match the incoming snapshot frame.
        // We only do this when netplay is not yet active to avoid disrupting a running session.
        if !self.input_provider.is_active() {
            self.input_provider.with_sync(|sync| sync.clear());
            self.input_provider.with_session_mut(|session| {
                session.clear_inputs();
                session.current_frame = msg.frame;
                match &session.state {
                    SessionState::Playing { player_index, .. } => {
                        session.state = SessionState::Playing {
                            start_frame: msg.frame,
                            player_index: *player_index,
                        };
                    }
                    SessionState::Spectating { .. } => {
                        session.state = SessionState::Spectating {
                            start_frame: msg.frame,
                        };
                    }
                    _ => {}
                }
            });
        }

        let _ = self
            .game_event_tx
            .send(NetplayEvent::SyncState(msg.frame, msg.data))
            .await;
        Ok(())
    }

    /// Handle RequestState message (server -> host).
    async fn handle_request_state(&mut self, packet: &PacketOwned) -> Result<(), NetplayError> {
        let _msg: RequestState =
            postcard::from_bytes(&packet.payload).map_err(|e| NetplayError::Protocol(e.into()))?;

        // Only the host (P1) can provide authoritative state snapshots.
        if self.input_provider.local_player() == Some(0) {
            info!("Received RequestState from server, setting state_sync_requested flag");
            self.input_provider.request_state_sync();
        } else {
            info!(
                local_player = ?self.input_provider.local_player(),
                "Ignoring RequestState (not host)"
            );
        }
        Ok(())
    }

    /// Handle PlayerLeft message - mark player as inactive.
    async fn handle_player_left(&mut self, packet: &PacketOwned) -> Result<(), NetplayError> {
        let msg: nesium_netproto::messages::session::PlayerLeft =
            postcard::from_bytes(&packet.payload).map_err(|e| NetplayError::Protocol(e.into()))?;

        info!(
            client_id = msg.client_id,
            player_index = msg.player_index,
            "Player left room"
        );

        // Mark player's port as inactive so we don't wait for their inputs
        self.input_provider.with_session_mut(|session| {
            session.players.remove(&msg.client_id);
            session.clear_port(msg.player_index as usize);
        });
        if msg.player_index != SPECTATOR_PLAYER_INDEX {
            self.input_provider
                .set_port_active(msg.player_index as usize, false);
        }

        let _ = self
            .game_event_tx
            .send(NetplayEvent::PlayerLeft {
                player_index: msg.player_index,
            })
            .await;

        Ok(())
    }

    async fn handle_activate_port(&mut self, packet: &PacketOwned) -> Result<(), NetplayError> {
        let msg: ActivatePort =
            postcard::from_bytes(&packet.payload).map_err(|e| NetplayError::Protocol(e.into()))?;

        info!(
            player_index = msg.player_index,
            active_from_frame = msg.active_from_frame,
            "ActivatePort"
        );

        if msg.player_index != SPECTATOR_PLAYER_INDEX {
            self.input_provider
                .schedule_port_active_from(msg.player_index as usize, msg.active_from_frame);

            if self.input_provider.local_player() == Some(msg.player_index) {
                self.input_provider
                    .set_local_input_allowed_from_effective_frame(msg.active_from_frame);
            }
        }

        Ok(())
    }

    async fn handle_room_info(&mut self, packet: &PacketOwned) -> Result<(), NetplayError> {
        let msg: RoomInfo =
            postcard::from_bytes(&packet.payload).map_err(|e| NetplayError::Protocol(e.into()))?;

        let Some((pending_id, resp)) = self.pending_query.take() else {
            return Ok(());
        };
        if pending_id != msg.request_id {
            let _ = resp.send(Err("Stale RoomInfo response".to_string()));
            return Ok(());
        }

        let _ = resp.send(Ok(msg));
        Ok(())
    }

    async fn send_rejoin_ready(&mut self, frame: u32) -> Result<(), NetplayError> {
        let msg = RejoinReady {
            caught_up_to_frame: frame,
        };
        self.client.send_message(&msg).await?;
        Ok(())
    }

    /// Send PauseGame message.
    async fn send_pause_game(&mut self, paused: bool) -> Result<(), NetplayError> {
        let req = PauseGame { paused };

        self.client.send_message(&req).await?;
        Ok(())
    }

    /// Send ResetGame message.
    async fn send_reset_game(&mut self, kind: u8) -> Result<(), NetplayError> {
        let req = ResetGame { kind };

        self.client.send_message(&req).await?;
        Ok(())
    }

    /// Send RequestState message.
    async fn send_request_state(&mut self) -> Result<(), NetplayError> {
        let req = RequestState {};

        self.client.send_message(&req).await?;
        Ok(())
    }

    /// Send ProvideState (provide state to server for caching).
    async fn send_provide_state(&mut self, frame: u32, data: Vec<u8>) -> Result<(), NetplayError> {
        let msg = ProvideState { frame, data };
        self.client.send_message(&msg).await?;
        Ok(())
    }

    /// Host-only: ask server to broadcast a relay fallback instruction.
    async fn send_request_fallback_relay(
        &mut self,
        relay_addr: SocketAddr,
        relay_room_code: u32,
        reason: String,
    ) -> Result<(), NetplayError> {
        let msg = RequestFallbackRelay {
            relay_addr,
            relay_room_code,
            reason,
        };
        self.client.send_message(&msg).await?;
        Ok(())
    }

    async fn handle_fallback_to_relay(&mut self, packet: &PacketOwned) -> Result<(), NetplayError> {
        let msg: FallbackToRelay =
            postcard::from_bytes(&packet.payload).map_err(|e| NetplayError::Protocol(e.into()))?;

        warn!(
            relay_addr = %msg.relay_addr,
            relay_room_code = msg.relay_room_code,
            reason = %msg.reason,
            "Received FallbackToRelay"
        );

        let _ = self
            .game_event_tx
            .send(NetplayEvent::FallbackToRelay {
                relay_addr: msg.relay_addr,
                relay_room_code: msg.relay_room_code,
                reason: msg.reason.clone(),
            })
            .await;

        // Stop lockstep immediately; the caller is expected to reconnect.
        self.input_provider.set_active(false);
        self.input_provider.with_session(|s| {
            s.state = SessionState::Disconnected;
        });
        let _ = self.client.disconnect().await;
        Ok(())
    }
}

/// Generate a random nonce.
fn rand_nonce() -> u32 {
    // Simple random using time - in production use proper RNG
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    (now.as_nanos() as u32) ^ (now.as_secs() as u32)
}

/// Get current time in milliseconds.
fn current_time_ms() -> u32 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rand_nonce_is_nonzero() {
        // Just a sanity check
        let n1 = rand_nonce();
        std::thread::sleep(std::time::Duration::from_millis(1));
        let n2 = rand_nonce();
        // They should differ (with very high probability)
        assert_ne!(n1, n2);
    }

    #[test]
    fn build_input_batches_splits_on_holes() {
        let batches = SessionHandler::build_input_batches(vec![(0, 1), (1, 2), (3, 4)]);
        assert_eq!(batches.len(), 2);
        assert_eq!(batches[0].start_frame, 0);
        assert_eq!(batches[0].buttons, vec![1, 2]);
        assert_eq!(batches[1].start_frame, 3);
        assert_eq!(batches[1].buttons, vec![4]);
    }

    #[test]
    fn build_input_batches_sorts_and_dedupes() {
        let batches = SessionHandler::build_input_batches(vec![(2, 20), (0, 1), (2, 21), (1, 2)]);
        assert_eq!(batches.len(), 1);
        assert_eq!(batches[0].start_frame, 0);
        assert_eq!(batches[0].buttons, vec![1, 2, 21]);
    }
}
