use std::net::SocketAddr;

use nesium_netproto::{
    channel::ChannelKind,
    constants::SPECTATOR_PLAYER_INDEX,
    header::Header,
    messages::session::{JoinAck, JoinRoom, LoadRom, PlayerJoined},
    msg_id::MsgId,
};
use tracing::{error, info, warn};

use crate::ConnCtx;
use crate::net::inbound::ConnId;
use crate::net::outbound::send_msg_tcp;
use crate::proto_dispatch::error::{HandlerError, HandlerResult};
use crate::room::state::{ClientOutbounds, Player, RoomManager, Spectator};

pub(crate) async fn handle(
    ctx: &mut ConnCtx,
    conn_id: ConnId,
    peer: &SocketAddr,
    payload: &[u8],
    room_mgr: &mut RoomManager,
) -> HandlerResult {
    let join: JoinRoom = match postcard::from_bytes(payload) {
        Ok(v) => v,
        Err(e) => {
            warn!(%peer, error = %e, "Bad JoinRoom message");
            return Err(HandlerError::bad_message());
        }
    };

    if room_mgr.get_client_room(ctx.assigned_client_id).is_some() {
        warn!(
            client_id = ctx.assigned_client_id,
            "Client already in a room, ignoring join request"
        );
        return Err(HandlerError::already_in_room());
    }

    let room_id = if join.room_code == 0 {
        let id = room_mgr.create_room(ctx.assigned_client_id);
        info!(
            room_id = id,
            client_id = ctx.assigned_client_id,
            "Room created"
        );
        // The room's sync mode is decided at creation time by the host.
        if let Some(room) = room_mgr.get_room_mut(id)
            && let Some(preferred) = join.preferred_sync_mode
        {
            room.sync_mode = preferred;
        }
        id
    } else {
        match room_mgr.find_by_code(join.room_code) {
            Some(r) => r.id,
            None => {
                warn!(room_code = join.room_code, "Room code not found");
                return Err(HandlerError::room_not_found());
            }
        }
    };

    let (ok, player_index) = {
        let room = room_mgr
            .get_room_mut(room_id)
            .expect("room should exist as we either just created it or found it by code");
        let is_spectator = room.player_count() >= 2;

        let mut outbounds = ClientOutbounds::new(ctx.outbound.clone());
        if let Some(tx) = ctx.channels.get(&ChannelKind::Input) {
            outbounds.set_channel(ChannelKind::Input, tx.clone());
        }
        if let Some(tx) = ctx.channels.get(&ChannelKind::Bulk) {
            outbounds.set_channel(ChannelKind::Bulk, tx.clone());
        }

        if is_spectator {
            room.add_spectator(Spectator {
                conn_id,
                client_id: ctx.assigned_client_id,
                name: ctx.name.clone(),
                outbounds,
            });
            info!(
                client_id = ctx.assigned_client_id,
                room_id, "Added spectator to room"
            );
            (true, SPECTATOR_PLAYER_INDEX)
        } else {
            match room.add_player(Player {
                conn_id,
                client_id: ctx.assigned_client_id,
                player_index: 0,
                name: ctx.name.clone(),
                outbounds,
            }) {
                Some(idx) => {
                    info!(
                        client_id = ctx.assigned_client_id,
                        player_index = idx,
                        room_id,
                        "Player joined room"
                    );
                    (true, idx)
                }
                None => {
                    warn!(room_id, "Room is full, join rejected");
                    (false, 0)
                }
            }
        }
    };

    if ok {
        room_mgr.set_client_room(ctx.assigned_client_id, room_id);
    }

    // Determine start_frame for late joiners: use cached state frame if available
    let (start_frame, sync_mode) = {
        let room = room_mgr.get_room_mut(room_id).expect("room should exist");
        let frame = room.cached_state.as_ref().map(|(f, _)| *f).unwrap_or(0);
        (frame, room.sync_mode)
    };

    let ack = JoinAck {
        ok,
        player_index,
        start_frame,
        room_id,
        sync_mode,
    };

    let h = Header::new(MsgId::JoinAck as u8);

    if let Err(e) = send_msg_tcp(&ctx.outbound, h, MsgId::JoinAck, &ack).await {
        error!(%peer, error = %e, "Failed to send JoinAck");
    }

    // Broadcast PlayerJoined to existing players (not to the joiner).
    if ok && player_index != SPECTATOR_PLAYER_INDEX {
        let room = room_mgr.get_room_mut(room_id).expect("room should exist");
        let existing_outbounds: Vec<_> = room
            .players
            .values()
            .filter(|p| p.client_id != ctx.assigned_client_id)
            .map(|p| p.outbounds.outbound_for_msg(MsgId::PlayerJoined))
            .chain(
                room.spectators
                    .iter()
                    .map(|s| s.outbounds.outbound_for_msg(MsgId::PlayerJoined)),
            )
            .collect();

        if !existing_outbounds.is_empty() {
            let joined_msg = PlayerJoined {
                client_id: ctx.assigned_client_id,
                player_index,
                name: ctx.name.clone(),
            };
            let h = Header::new(MsgId::PlayerJoined as u8);

            for recipient in &existing_outbounds {
                let _ = send_msg_tcp(recipient, h, MsgId::PlayerJoined, &joined_msg).await;
            }
            info!(
                client_id = ctx.assigned_client_id,
                player_index,
                recipients = existing_outbounds.len(),
                "Broadcasted PlayerJoined"
            );
        }
    }

    // Inform the new joiner about all existing players and spectators.
    if ok {
        let room = room_mgr.get_room_mut(room_id).expect("room should exist");
        let mut existing_players = Vec::new();
        for p in room.players.values() {
            if p.client_id != ctx.assigned_client_id {
                existing_players.push((p.client_id, p.player_index, p.name.clone()));
            }
        }
        for s in &room.spectators {
            if s.client_id != ctx.assigned_client_id {
                existing_players.push((s.client_id, SPECTATOR_PLAYER_INDEX, s.name.clone()));
            }
        }

        for (cid, p_idx, name) in existing_players {
            let joined_msg = PlayerJoined {
                client_id: cid,
                player_index: p_idx,
                name,
            };
            let h = Header::new(MsgId::PlayerJoined as u8);

            let _ = send_msg_tcp(&ctx.outbound, h, MsgId::PlayerJoined, &joined_msg).await;
        }
    }

    // Late joiners: if there's a cached ROM, send it immediately.
    let Some(room) = room_mgr.get_room_mut(room_id) else {
        return Ok(());
    };

    if let Some(rom_data) = room.rom_data.clone() {
        info!(
            client_id = ctx.assigned_client_id,
            "Sending cached ROM to joiner"
        );
        let load_rom = LoadRom { data: rom_data };
        let h = Header::new(MsgId::LoadRom as u8);
        let tx = room
            .outbound_for_client_msg(ctx.assigned_client_id, MsgId::LoadRom)
            .unwrap_or_else(|| ctx.outbound.clone());
        let _ = send_msg_tcp(&tx, h, MsgId::LoadRom, &load_rom).await;
    }
    Ok(())
}
