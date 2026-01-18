//! JoinRoomHandler - handles JoinRoom messages.

use nesium_netproto::{
    channel::ChannelKind,
    constants::{AUTO_PLAYER_INDEX, SPECTATOR_PLAYER_INDEX},
    messages::session::{JoinAck, JoinRoom, LoadRom, PlayerJoined},
    msg_id::MsgId,
};
use tracing::{error, info, warn};

use super::{Handler, HandlerContext};
use crate::net::outbound::send_msg_tcp;
use crate::proto_dispatch::error::{HandlerError, HandlerResult};
use crate::room::state::{ClientOutbounds, Player, Spectator};

/// Handler for JoinRoom messages.
pub(crate) struct JoinRoomHandler;

impl Handler<JoinRoom> for JoinRoomHandler {
    async fn handle(&self, ctx: &mut HandlerContext<'_>, join: JoinRoom) -> HandlerResult {
        if ctx
            .room_mgr
            .get_client_room(ctx.conn_ctx.assigned_client_id)
            .is_some()
        {
            warn!(
                client_id = ctx.conn_ctx.assigned_client_id,
                "Client already in a room, ignoring join request"
            );
            return Err(HandlerError::already_in_room());
        }

        let room_id = if join.room_code == 0 {
            let id = ctx.room_mgr.create_room(ctx.conn_ctx.assigned_client_id);
            info!(
                room_id = id,
                client_id = ctx.conn_ctx.assigned_client_id,
                "Room created"
            );
            // The room's sync mode is decided at creation time by the host.
            if let Some(room) = ctx.room_mgr.get_room_mut(id) {
                if let Some(preferred) = join.preferred_sync_mode {
                    room.sync_mode = preferred;
                }
            }
            id
        } else {
            match ctx.room_mgr.find_by_code(join.room_code) {
                Some(r) => r.id,
                None => {
                    warn!(room_code = join.room_code, "Room code not found");
                    return Err(HandlerError::room_not_found());
                }
            }
        };

        let (ok, player_index, pending_activation) = {
            let room = ctx.room_mgr.get_room_mut(room_id).unwrap();

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
                (true, SPECTATOR_PLAYER_INDEX, false)
            } else {
                let desired = join.desired_role;
                if join.room_code == 0 {
                    // Creating room: host is always P1.
                    let ok = room.add_player_at_index(
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
                    (ok, 0, false)
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
                    let ok = room.add_player_at_index(
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
                    if ok {
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
        };

        if ok {
            ctx.room_mgr
                .set_client_room(ctx.conn_ctx.assigned_client_id, room_id);
        }

        // Determine start_frame for late joiners: use cached state frame if available
        let (start_frame, sync_mode) = {
            let room = ctx.room_mgr.get_room_mut(room_id).unwrap();
            let frame = room.cached_state.as_ref().map(|(f, _)| *f).unwrap_or(0);
            (frame, room.sync_mode)
        };

        let ack = JoinAck {
            ok,
            player_index,
            start_frame,
            room_id,
            sync_mode,
            pending_activation,
        };

        if let Err(e) = send_msg_tcp(&ctx.conn_ctx.outbound, &ack).await {
            error!(%ctx.peer, error = %e, "Failed to send JoinAck");
        }

        // Broadcast PlayerJoined to existing players (not to the joiner).
        if ok && player_index != SPECTATOR_PLAYER_INDEX {
            let room = ctx.room_mgr.get_room_mut(room_id).unwrap();
            let existing_outbounds: Vec<_> = room
                .players
                .values()
                .filter(|p| p.client_id != ctx.conn_ctx.assigned_client_id)
                .map(|p| p.outbounds.outbound_for_msg(MsgId::PlayerJoined))
                .chain(
                    room.spectators
                        .iter()
                        .map(|s| s.outbounds.outbound_for_msg(MsgId::PlayerJoined)),
                )
                .collect();

            if !existing_outbounds.is_empty() {
                let joined_msg = PlayerJoined {
                    client_id: ctx.conn_ctx.assigned_client_id,
                    player_index,
                    name: ctx.conn_ctx.name.clone(),
                };

                for recipient in &existing_outbounds {
                    let _ = send_msg_tcp(recipient, &joined_msg).await;
                }
                info!(
                    client_id = ctx.conn_ctx.assigned_client_id,
                    player_index,
                    recipients = existing_outbounds.len(),
                    "Broadcasted PlayerJoined"
                );
            }
        }

        // Inform the new joiner about all existing players and spectators.
        if ok {
            let room = ctx.room_mgr.get_room_mut(room_id).unwrap();
            let mut existing_players = Vec::new();
            for p in room.players.values() {
                if p.client_id != ctx.conn_ctx.assigned_client_id {
                    existing_players.push((p.client_id, p.player_index, p.name.clone()));
                }
            }
            for s in &room.spectators {
                if s.client_id != ctx.conn_ctx.assigned_client_id {
                    existing_players.push((s.client_id, SPECTATOR_PLAYER_INDEX, s.name.clone()));
                }
            }

            for (cid, p_idx, name) in existing_players {
                let joined_msg = PlayerJoined {
                    client_id: cid,
                    player_index: p_idx,
                    name,
                };

                let _ = send_msg_tcp(&ctx.conn_ctx.outbound, &joined_msg).await;
            }
        }

        // Late joiners: if there's a cached ROM, send it immediately.
        let Some(room) = ctx.room_mgr.get_room_mut(room_id) else {
            return Ok(());
        };

        if !join.has_rom {
            if let Some(rom_data) = room.rom_data.clone() {
                info!(
                    client_id = ctx.conn_ctx.assigned_client_id,
                    "Sending cached ROM to joiner"
                );
                let load_rom = LoadRom { data: rom_data };
                let tx = room
                    .outbound_for_client_msg(ctx.conn_ctx.assigned_client_id, MsgId::LoadRom)
                    .unwrap_or_else(|| ctx.conn_ctx.outbound.clone());
                let _ = send_msg_tcp(&tx, &load_rom).await;
            }
        }
        Ok(())
    }
}
