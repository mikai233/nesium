use std::net::SocketAddr;

use nesium_netproto::{
    constants::SPECTATOR_PLAYER_INDEX,
    header::Header,
    messages::session::{JoinAck, JoinRoom, LoadRom, PlayerJoined},
    msg_id::MsgId,
};
use tracing::{error, info, warn};

use crate::net::inbound::ConnId;
use crate::room::state::{Player, RoomManager, Spectator};
use crate::{ConnCtx, net::outbound::send_msg_tcp};

pub(crate) async fn handle(
    ctx: &mut ConnCtx,
    conn_id: ConnId,
    peer: &SocketAddr,
    payload: &[u8],
    room_mgr: &mut RoomManager,
) {
    let join: JoinRoom = match postcard::from_bytes(payload) {
        Ok(v) => v,
        Err(e) => {
            warn!(%peer, error = %e, "Bad JoinRoom message");
            return;
        }
    };

    if room_mgr.get_client_room(ctx.assigned_client_id).is_some() {
        warn!(
            client_id = ctx.assigned_client_id,
            "Client already in a room, ignoring join request"
        );
        return;
    }

    let room_id = if join.room_code == 0 {
        let id = room_mgr.create_room(ctx.rom_hash, ctx.assigned_client_id);
        info!(
            room_id = id,
            client_id = ctx.assigned_client_id,
            "Room created"
        );
        id
    } else {
        match room_mgr.find_by_code(join.room_code) {
            Some(r) => r.id,
            None => {
                warn!(room_code = join.room_code, "Room code not found");
                return;
            }
        }
    };

    let (ok, player_index) = {
        let room = room_mgr.get_room_mut(room_id).unwrap();
        let is_spectator = room.player_count() >= 2;

        if is_spectator {
            room.add_spectator(Spectator {
                conn_id,
                client_id: ctx.assigned_client_id,
                name: ctx.name.clone(),
                outbound: ctx.outbound.clone(),
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
                outbound: ctx.outbound.clone(),
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
    let start_frame = {
        let room = room_mgr.get_room_mut(room_id).unwrap();
        room.cached_state.as_ref().map(|(f, _)| *f).unwrap_or(0)
    };

    let ack = JoinAck {
        ok,
        player_index,
        start_frame,
        room_id,
    };

    let mut h = Header::new(MsgId::JoinAck as u8);
    h.client_id = ctx.assigned_client_id;
    h.room_id = room_id;
    h.seq = ctx.server_seq;
    ctx.server_seq = ctx.server_seq.wrapping_add(1);

    if let Err(e) = send_msg_tcp(&ctx.outbound, h, MsgId::JoinAck, &ack).await {
        error!(%peer, error = %e, "Failed to send JoinAck");
    }

    // Broadcast PlayerJoined to existing players (not to the joiner).
    if ok && player_index != SPECTATOR_PLAYER_INDEX {
        let room = room_mgr.get_room_mut(room_id).unwrap();
        let existing_outbounds: Vec<_> = room
            .players
            .values()
            .filter(|p| p.client_id != ctx.assigned_client_id)
            .map(|p| p.outbound.clone())
            .chain(room.spectators.iter().map(|s| s.outbound.clone()))
            .collect();

        if !existing_outbounds.is_empty() {
            let joined_msg = PlayerJoined {
                client_id: ctx.assigned_client_id,
                player_index,
                name: ctx.name.clone(),
            };
            let mut h = Header::new(MsgId::PlayerJoined as u8);
            h.client_id = ctx.assigned_client_id;
            h.room_id = room_id;
            h.seq = 0;

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
        let room = room_mgr.get_room_mut(room_id).unwrap();
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
            let mut h = Header::new(MsgId::PlayerJoined as u8);
            h.client_id = cid;
            h.room_id = room_id;
            h.seq = 0;

            let _ = send_msg_tcp(&ctx.outbound, h, MsgId::PlayerJoined, &joined_msg).await;
        }
    }

    // Late joiners: if there's a cached ROM, send it immediately.
    let Some(room) = room_mgr.get_room_mut(room_id) else {
        return;
    };

    if let Some(rom_data) = room.rom_data.clone() {
        info!(
            client_id = ctx.assigned_client_id,
            "Sending cached ROM to joiner"
        );
        let load_rom = LoadRom { data: rom_data };
        let mut h = Header::new(MsgId::LoadRom as u8);
        h.client_id = 0; // System message
        h.room_id = room_id;
        h.seq = ctx.server_seq;
        ctx.server_seq = ctx.server_seq.wrapping_add(1);
        let _ = send_msg_tcp(&ctx.outbound, h, MsgId::LoadRom, &load_rom).await;
    }
}
