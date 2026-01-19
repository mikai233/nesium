//! JoinRoomHandler - handles JoinRoom messages.

use nesium_netproto::{
    channel::ChannelKind,
    constants::{AUTO_PLAYER_INDEX, SPECTATOR_PLAYER_INDEX},
    messages::session::{JoinAck, JoinRoom, LoadRom, PlayerJoined},
    msg_id::MsgId,
};
use tracing::{error, info, warn};

use super::{Handler, HandlerContext};
use crate::net::outbound::{OutboundTx, send_msg};
use crate::proto_dispatch::error::{HandlerError, HandlerResult};
use crate::room::state::{ClientOutbounds, Player, Room, Spectator};

/// Handler for JoinRoom messages.
pub(crate) struct JoinRoomHandler;

impl Handler<JoinRoom> for JoinRoomHandler {
    async fn handle(&self, ctx: &mut HandlerContext<'_>, join: JoinRoom) -> HandlerResult {
        // 1. Check if already in a room
        if ctx
            .room_mgr
            .client_room_id(ctx.conn_ctx.assigned_client_id)
            .is_some()
        {
            warn!(
                client_id = ctx.conn_ctx.assigned_client_id,
                "Client already in a room, ignoring join request"
            );
            return Err(HandlerError::already_in_room());
        }

        // 2. Resolve or create room
        let room_id = Self::resolve_or_create_room(ctx, &join)?;

        // 3. Attempt to add client to room (assign role)
        let (is_joined, player_index, pending_activation) =
            Self::try_add_client_to_room(ctx, room_id, &join);

        // 4. Register client in manager if successful
        if is_joined {
            ctx.room_mgr
                .register_client(ctx.conn_ctx.assigned_client_id, room_id);
        }

        // 5. Get room reference for Ack and Sync
        // This borrows ctx.room_mgr mutably. We cannot use ctx.conn_ctx or ctx.room_mgr again
        // while `room` is alive, unless we passed distinct references.
        let Some(room) = ctx.room_mgr.room_mut(room_id) else {
            warn!(room_id, "Room disappeared during join processing");
            return Err(HandlerError::room_not_found());
        };

        // 6. Send JoinAck
        let start_frame = room.cached_state.as_ref().map(|(f, _)| *f).unwrap_or(0);
        let sync_mode = room.sync_mode;

        let ack = JoinAck {
            ok: is_joined,
            player_index,
            start_frame,
            room_id,
            sync_mode,
            pending_activation,
        };

        if let Err(e) = send_msg(&ctx.conn_ctx.outbound, &ack).await {
            error!(peer = %ctx.conn_ctx.peer, error = %e, "Failed to send JoinAck");
        }

        if !is_joined {
            return Ok(());
        }

        // 7. Broadcast PlayerJoined to existing members
        Self::broadcast_new_player(
            ctx.conn_ctx.assigned_client_id,
            &ctx.conn_ctx.name,
            room,
            player_index,
        )
        .await;

        // 8. Sync existing state to new member (Other players + cached ROM)
        Self::sync_initial_state(
            ctx.conn_ctx.assigned_client_id,
            &ctx.conn_ctx.outbound,
            room,
            join.has_rom,
        )
        .await;

        Ok(())
    }
}

impl JoinRoomHandler {
    fn resolve_or_create_room(
        ctx: &mut HandlerContext<'_>,
        join: &JoinRoom,
    ) -> Result<u32, HandlerError> {
        if join.room_id == 0 {
            let Some(id) = ctx.room_mgr.create_room(ctx.conn_ctx.assigned_client_id) else {
                return Err(HandlerError::server_full());
            };
            info!(
                room_id = id,
                client_id = ctx.conn_ctx.assigned_client_id,
                "Room created"
            );
            // The room's sync mode is decided at creation time by the host.
            if let Some(room) = ctx.room_mgr.room_mut(id) {
                if let Some(preferred) = join.preferred_sync_mode {
                    room.sync_mode = preferred;
                }
            }
            Ok(id)
        } else {
            match ctx.room_mgr.room(join.room_id) {
                Some(r) => Ok(r.id),
                None => {
                    warn!(room_id = join.room_id, "Room not found");
                    Err(HandlerError::room_not_found())
                }
            }
        }
    }

    fn try_add_client_to_room(
        ctx: &mut HandlerContext<'_>,
        room_id: u32,
        join: &JoinRoom,
    ) -> (bool, u8, bool) {
        let room = ctx.room_mgr.room_mut(room_id).unwrap();
        let is_spectator = join.desired_role == SPECTATOR_PLAYER_INDEX;

        let mut outbounds = ClientOutbounds::new(ctx.conn_ctx.outbound.clone());
        if let Some(tx) = ctx.conn_ctx.channels.get(&ChannelKind::Input) {
            outbounds.set_channel(ChannelKind::Input, tx.clone());
        }
        if let Some(tx) = ctx.conn_ctx.channels.get(&ChannelKind::Bulk) {
            outbounds.set_channel(ChannelKind::Bulk, tx.clone());
        }

        if is_spectator {
            room.add_spectator(Spectator {
                conn_id: ctx.conn_id,
                client_id: ctx.conn_ctx.assigned_client_id,
                name: ctx.conn_ctx.name.clone(),
                outbounds,
            });
            info!(
                client_id = ctx.conn_ctx.assigned_client_id,
                room_id, "Added spectator to room"
            );
            return (true, SPECTATOR_PLAYER_INDEX, false);
        }

        let desired = join.desired_role;
        if join.room_id == 0 {
            // Creating room: host is always P1.
            let added = room.add_player_at_index(
                0,
                Player {
                    conn_id: ctx.conn_id,
                    client_id: ctx.conn_ctx.assigned_client_id,
                    player_index: 0,
                    name: ctx.conn_ctx.name.clone(),
                    outbounds,
                },
                true,
            );
            (added, 0, false)
        } else if desired == AUTO_PLAYER_INDEX {
            // Auto-assign is only allowed before game start.
            if room.started {
                room.add_spectator(Spectator {
                    conn_id: ctx.conn_id,
                    client_id: ctx.conn_ctx.assigned_client_id,
                    name: ctx.conn_ctx.name.clone(),
                    outbounds,
                });
                (true, SPECTATOR_PLAYER_INDEX, false)
            } else {
                match room.add_player(Player {
                    conn_id: ctx.conn_id,
                    client_id: ctx.conn_ctx.assigned_client_id,
                    player_index: 0,
                    name: ctx.conn_ctx.name.clone(),
                    outbounds: outbounds.clone(),
                }) {
                    Some(idx) => (true, idx, false),
                    None => {
                        room.add_spectator(Spectator {
                            conn_id: ctx.conn_id,
                            client_id: ctx.conn_ctx.assigned_client_id,
                            name: ctx.conn_ctx.name.clone(),
                            outbounds,
                        });
                        (true, SPECTATOR_PLAYER_INDEX, false)
                    }
                }
            }
        } else if desired < crate::room::state::MAX_PLAYERS as u8
            && !room.players.contains_key(&desired)
        {
            let pending_activation = room.started;
            let added = room.add_player_at_index(
                desired,
                Player {
                    conn_id: ctx.conn_id,
                    client_id: ctx.conn_ctx.assigned_client_id,
                    player_index: desired,
                    name: ctx.conn_ctx.name.clone(),
                    outbounds: outbounds.clone(),
                },
                !pending_activation,
            );
            if added {
                info!(
                    client_id = ctx.conn_ctx.assigned_client_id,
                    player_index = desired,
                    pending_activation,
                    room_id,
                    "Player joined room"
                );
                (true, desired, pending_activation)
            } else {
                room.add_spectator(Spectator {
                    conn_id: ctx.conn_id,
                    client_id: ctx.conn_ctx.assigned_client_id,
                    name: ctx.conn_ctx.name.clone(),
                    outbounds,
                });
                (true, SPECTATOR_PLAYER_INDEX, false)
            }
        } else {
            // Requested slot is occupied or invalid -> spectator fallback.
            room.add_spectator(Spectator {
                conn_id: ctx.conn_id,
                client_id: ctx.conn_ctx.assigned_client_id,
                name: ctx.conn_ctx.name.clone(),
                outbounds,
            });
            (true, SPECTATOR_PLAYER_INDEX, false)
        }
    }

    async fn broadcast_new_player(client_id: u32, name: &str, room: &Room, player_index: u8) {
        if player_index == SPECTATOR_PLAYER_INDEX {
            return;
        }

        let existing_outbounds: Vec<_> = room
            .players
            .values()
            .filter(|p| p.client_id != client_id)
            .map(|p| p.outbounds.outbound_for_msg(MsgId::PlayerJoined))
            .chain(
                room.spectators
                    .iter()
                    .map(|s| s.outbounds.outbound_for_msg(MsgId::PlayerJoined)),
            )
            .collect();

        if !existing_outbounds.is_empty() {
            let joined_msg = PlayerJoined {
                client_id,
                player_index,
                name: name.to_string(),
            };

            for recipient in &existing_outbounds {
                let _ = send_msg(recipient, &joined_msg).await;
            }
            info!(
                client_id,
                player_index,
                recipients = existing_outbounds.len(),
                "Broadcasted PlayerJoined"
            );
        }
    }

    async fn sync_initial_state(
        client_id: u32,
        outbound: &OutboundTx,
        room: &Room,
        join_has_rom: bool,
    ) {
        // 1. Inform the new joiner about all existing players and spectators.
        let mut existing_players = Vec::new();
        for p in room.players.values() {
            if p.client_id != client_id {
                existing_players.push((p.client_id, p.player_index, p.name.clone()));
            }
        }
        for s in &room.spectators {
            if s.client_id != client_id {
                existing_players.push((s.client_id, SPECTATOR_PLAYER_INDEX, s.name.clone()));
            }
        }

        for (cid, p_idx, p_name) in existing_players {
            let joined_msg = PlayerJoined {
                client_id: cid,
                player_index: p_idx,
                name: p_name,
            };

            let _ = send_msg(outbound, &joined_msg).await;
        }

        // 2. Late joiners: if there's a cached ROM, send it immediately.
        if !join_has_rom {
            if let Some(rom_data) = room.rom_data.clone() {
                info!(client_id, "Sending cached ROM to joiner");
                let load_rom = LoadRom { data: rom_data };
                let tx = room
                    .outbound_for_client_msg(client_id, MsgId::LoadRom)
                    .unwrap_or_else(|| outbound.clone());
                let _ = send_msg(&tx, &load_rom).await;
            }
        }
    }
}
