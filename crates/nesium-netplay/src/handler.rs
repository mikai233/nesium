//! Session handler for processing server messages and managing state transitions.
//!
//! This module handles the protocol logic for:
//! - Hello/Welcome handshake
//! - Room joining
//! - Input relay
//! - State synchronization

use std::sync::Arc;

use nesium_netproto::{
    constants::SPECTATOR_PLAYER_INDEX,
    header::Header,
    messages::{
        input::{InputBatch, RelayInputs},
        session::{
            BeginCatchUp, ErrorCode, ErrorMsg, Hello, JoinAck, JoinRoom, LoadRom, PauseGame,
            PauseSync, ProvideState, RequestState, ResetGame, ResetSync, RomLoaded, StartGame,
            SyncState, TransportKind, Welcome,
        },
        sync::{Ping, Pong},
    },
    msg_id::MsgId,
};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::{
    error::NetplayError,
    input_provider::SharedInputProvider,
    session::SessionState,
    tcp_client::{PacketOwned, TcpClientEvent, TcpClientHandle},
};

/// Configuration for starting a netplay session.
#[derive(Debug, Clone)]
pub struct NetplayConfig {
    /// Player name to display.
    pub name: String,
    /// ROM hash for validation.
    pub rom_hash: [u8; 16],
    /// Whether to join as spectator.
    pub spectator: bool,
    /// Room code to join (0 = create new room).
    pub room_code: u32,
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
}

#[derive(Debug)]
pub enum NetplayCommand {
    CreateRoom,
    JoinRoom(u32),
    SwitchRole(u8),
    SendRom(Vec<u8>),
    RomLoaded,
    SendPause(bool),
    SendReset(u8),
    RequestState,
    ProvideState(u32, Vec<u8>),
    /// Send local input for a frame: (frame_number, buttons)
    SendInput(u32, u16),
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
            let _ = tx_clone.blocking_send(NetplayCommand::SendInput(frame, buttons));
        }));

        let tx_clone_state = tx.clone();
        input_provider.set_on_send_state(Box::new(move |frame, data| {
            let _ =
                tx_clone_state.blocking_send(NetplayCommand::ProvideState(frame, data.to_vec()));
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
                            self.send_join_room(0).await?;
                        }
                        Some(NetplayCommand::JoinRoom(code)) => {
                            self.send_join_room(code).await?;
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
        });

        // Send Hello message
        let hello = Hello {
            client_nonce: rand_nonce(),
            transport: TransportKind::Tcp,
            proto_min: 1,
            proto_max: 1,
            rom_hash: self.config.rom_hash,
            name: self.config.name.clone(),
        };

        let mut header = Header::new(MsgId::Hello as u8);
        header.seq = self.input_provider.with_session(|s| s.next_seq());

        self.client
            .send_message(header, MsgId::Hello, &hello)
            .await?;

        self.input_provider.with_session(|session| {
            session.state = SessionState::Handshake;
            session.rom_hash = self.config.rom_hash;
        });

        Ok(())
    }

    /// Handle disconnection.
    fn handle_disconnected(&mut self, reason: &str) {
        warn!("Session disconnected: {}", reason);
        self.input_provider.set_active(false);
        self.input_provider.with_session(|session| {
            session.state = SessionState::Disconnected;
        });
    }

    /// Handle incoming packet.
    async fn handle_packet(&mut self, packet: PacketOwned) -> Result<(), NetplayError> {
        debug!("Received {:?}", packet.msg_id);

        match packet.msg_id {
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
            MsgId::PlayerLeft => self.handle_player_left(&packet).await?,
            MsgId::Error => self.handle_error(&packet).await?,
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

        self.input_provider.with_session(|session| {
            session.client_id = welcome.assigned_client_id;
            session.room_id = welcome.room_id;
            session.server_nonce = welcome.server_nonce;
            session.input_delay_frames = welcome.input_delay_frames;
            session.rewind_capacity = welcome.rewind_capacity;
            session.state = SessionState::WaitingForRoom;
        });

        // If we have a room code, join it; otherwise wait for assignment
        if self.config.room_code != 0 {
            self.send_join_room(self.config.room_code).await?;
        }

        Ok(())
    }

    /// Send JoinRoom request.
    async fn send_join_room(&mut self, room_code: u32) -> Result<(), NetplayError> {
        let join = JoinRoom { room_code };

        let mut header = Header::new(MsgId::JoinRoom as u8);
        header.seq = self.input_provider.with_session(|s| s.next_seq());
        header.client_id = self.input_provider.with_session(|s| s.client_id);

        self.client
            .send_message(header, MsgId::JoinRoom, &join)
            .await?;

        Ok(())
    }

    /// Send SwitchRole request.
    async fn send_switch_role(&mut self, new_role: u8) -> Result<(), NetplayError> {
        let req = nesium_netproto::messages::session::SwitchRole { new_role };

        let mut header = Header::new(MsgId::SwitchRole as u8);
        header.seq = self.input_provider.with_session(|s| s.next_seq());
        header.client_id = self.input_provider.with_session(|s| s.client_id);
        header.room_id = self.input_provider.with_session(|s| s.room_id);

        self.client
            .send_message(header, MsgId::SwitchRole, &req)
            .await?;

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
            "Joined room: player_index={}, start_frame={}",
            ack.player_index, ack.start_frame
        );

        // The server is authoritative on role assignment: it may assign us as spectator if all
        // player slots are taken, even if we didn't request it.
        let is_spectator = ack.player_index == SPECTATOR_PLAYER_INDEX;
        if self.config.spectator != is_spectator {
            warn!(
                requested_spectator = self.config.spectator,
                assigned_spectator = is_spectator,
                assigned_player_index = ack.player_index,
                "Server role assignment differs from requested role"
            );
        }
        self.input_provider.with_session_mut(|session| {
            if is_spectator {
                session.state = SessionState::Spectating {
                    start_frame: ack.start_frame,
                };
                session.local_player_index = None;
                // Current server logic only assigns spectators when there are already >=2 players.
                // Mark known player ports active so we don't advance using implicit 0 inputs.
                if session.active_ports.len() >= 2 {
                    session.active_ports[0] = true;
                    session.active_ports[1] = true;
                } else if !session.active_ports.is_empty() {
                    session.active_ports[0] = true;
                }
            } else {
                session.state = SessionState::Playing {
                    start_frame: ack.start_frame,
                    player_index: ack.player_index,
                };
                session.local_player_index = Some(ack.player_index);
                // Mark own port as active to prevent lockstep deadlock in solo play
                if (ack.player_index as usize) < session.active_ports.len() {
                    session.active_ports[ack.player_index as usize] = true;
                }
                // If we joined as player 2 (index 1), we know player 1 (index 0) exists.
                // Mark it active immediately so we block until its inputs arrive, preventing
                // a 1–2 frame drift window during late-join catch-up.
                if ack.player_index == 1 && !session.active_ports.is_empty() {
                    session.active_ports[0] = true;
                }
            }
            session.current_frame = ack.start_frame;
            session.room_id = ack.room_id;
        });

        self.input_provider.set_local_player(if is_spectator {
            None
        } else {
            Some(ack.player_index)
        });
        // NOTE: Do NOT call set_active(true) here. Lockstep should only start after StartGame.

        Ok(())
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

            if msg.player_index != SPECTATOR_PLAYER_INDEX {
                let idx = msg.player_index as usize;
                if idx < session.active_ports.len() {
                    session.active_ports[idx] = true;
                }
            }
        });

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
        let (should_send, batch) = self.input_provider.with_session_mut(|session| {
            session.queue_local_input(frame, buttons);

            // Check if we have enough pending inputs to send a batch
            // Send in batches to reduce overhead while maintaining low latency.
            let pending_count = session.pending_inputs_count();
            if pending_count >= 1 {
                // Drain and send
                let items = session.drain_pending_inputs(30); // Max batch size
                if items.is_empty() {
                    return (false, None);
                }

                let start_frame = items[0].0;
                let buttons: Vec<u16> = items.iter().map(|(_, b)| *b).collect();

                (
                    true,
                    Some(InputBatch {
                        start_frame,
                        buttons,
                    }),
                )
            } else {
                (false, None)
            }
        });

        if should_send {
            if let Some(batch) = batch {
                let mut header = Header::new(MsgId::InputBatch as u8);
                header.seq = self.input_provider.with_session(|s| s.next_seq());
                header.client_id = self.input_provider.with_session(|s| s.client_id);
                header.room_id = self.input_provider.with_session(|s| s.room_id);

                // Use UDP if enabled, else TCP
                // TODO: Implement UDP support. For now always TCP.
                self.client
                    .send_message(header, MsgId::InputBatch, &batch)
                    .await?;
            }
        }
        Ok(())
    }

    /// Send a ping for RTT measurement.
    pub async fn send_ping(&mut self) -> Result<(), NetplayError> {
        let ping = Ping {
            t_ms: current_time_ms(),
        };

        let mut header = Header::new(MsgId::Ping as u8);
        header.seq = self.input_provider.with_session(|s| s.next_seq());
        header.client_id = self.input_provider.with_session(|s| s.client_id);

        self.client.send_message(header, MsgId::Ping, &ping).await?;

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
        // This prevents the deadlock where is_frame_ready() returns false because
        // no ports are marked active yet.
        self.input_provider.with_session_mut(|session| {
            for i in 0..session.active_ports.len().min(8) {
                session.active_ports[i] = (msg.active_ports_mask & (1u8 << i)) != 0;
            }
            // Always keep local port active
            if let Some(idx) = session.local_player_index {
                let i = idx as usize;
                if i < session.active_ports.len() {
                    session.active_ports[i] = true;
                }
            }
        });

        // NOW activate lockstep - game is ready to begin
        self.input_provider.set_active(true);

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

        self.input_provider.with_session_mut(|session| {
            for i in 0..session.active_ports.len().min(8) {
                session.active_ports[i] = (msg.active_ports_mask & (1u8 << i)) != 0;
            }
            // Always keep the local port active if we're a player.
            if let Some(idx) = session.local_player_index {
                let i = idx as usize;
                if i < session.active_ports.len() {
                    session.active_ports[i] = true;
                }
            }
        });

        self.input_provider.set_active(true);

        // Reuse StartGame event: it tells the UI/runtime to unpause and begin running frames.
        if let Err(e) = self.game_event_tx.send(NetplayEvent::StartGame).await {
            error!("Failed to send StartGame event: {}", e);
        }

        Ok(())
    }

    /// Send LoadRom message.
    async fn send_load_rom(&mut self, data: Vec<u8>) -> Result<(), NetplayError> {
        let req = LoadRom { data };

        let mut header = Header::new(MsgId::LoadRom as u8);
        header.seq = self.input_provider.with_session(|s| s.next_seq());
        header.client_id = self.input_provider.with_session(|s| s.client_id);
        header.room_id = self.input_provider.with_session(|s| s.room_id);

        self.client
            .send_message(header, MsgId::LoadRom, &req)
            .await?;

        Ok(())
    }

    /// Send RomLoaded message.
    async fn send_rom_loaded(&mut self) -> Result<(), NetplayError> {
        let req = RomLoaded {};

        let mut header = Header::new(MsgId::RomLoaded as u8);
        header.seq = self.input_provider.with_session(|s| s.next_seq());
        header.client_id = self.input_provider.with_session(|s| s.client_id);
        header.room_id = self.input_provider.with_session(|s| s.room_id);

        self.client
            .send_message(header, MsgId::RomLoaded, &req)
            .await?;

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
        let _ = self
            .game_event_tx
            .send(NetplayEvent::SyncState(msg.frame, msg.data))
            .await;
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

        let _ = self
            .game_event_tx
            .send(NetplayEvent::PlayerLeft {
                player_index: msg.player_index,
            })
            .await;

        Ok(())
    }

    /// Send PauseGame message.
    async fn send_pause_game(&mut self, paused: bool) -> Result<(), NetplayError> {
        let req = PauseGame { paused };

        let mut header = Header::new(MsgId::PauseGame as u8);
        header.seq = self.input_provider.with_session(|s| s.next_seq());
        header.client_id = self.input_provider.with_session(|s| s.client_id);
        header.room_id = self.input_provider.with_session(|s| s.room_id);

        self.client
            .send_message(header, MsgId::PauseGame, &req)
            .await?;
        Ok(())
    }

    /// Send ResetGame message.
    async fn send_reset_game(&mut self, kind: u8) -> Result<(), NetplayError> {
        let req = ResetGame { kind };

        let mut header = Header::new(MsgId::ResetGame as u8);
        header.seq = self.input_provider.with_session(|s| s.next_seq());
        header.client_id = self.input_provider.with_session(|s| s.client_id);
        header.room_id = self.input_provider.with_session(|s| s.room_id);

        self.client
            .send_message(header, MsgId::ResetGame, &req)
            .await?;
        Ok(())
    }

    /// Send RequestState message.
    async fn send_request_state(&mut self) -> Result<(), NetplayError> {
        let req = RequestState {};

        let mut header = Header::new(MsgId::RequestState as u8);
        header.seq = self.input_provider.with_session(|s| s.next_seq());
        header.client_id = self.input_provider.with_session(|s| s.client_id);
        header.room_id = self.input_provider.with_session(|s| s.room_id);

        self.client
            .send_message(header, MsgId::RequestState, &req)
            .await?;
        Ok(())
    }

    /// Send ProvideState (provide state to server for caching).
    async fn send_provide_state(&mut self, frame: u32, data: Vec<u8>) -> Result<(), NetplayError> {
        let msg = ProvideState { frame, data };
        let mut header = Header::new(MsgId::ProvideState as u8);
        header.seq = self.input_provider.with_session(|s| s.next_seq());
        header.client_id = self.input_provider.with_session(|s| s.client_id);
        header.room_id = self.input_provider.with_session(|s| s.room_id);

        self.client
            .send_message(header, MsgId::ProvideState, &msg)
            .await?;
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
}
