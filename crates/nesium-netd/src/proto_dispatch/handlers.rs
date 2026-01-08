use std::net::SocketAddr;

use nesium_netproto::{header::Header, messages::session::ErrorMsg, msg_id::MsgId};
use tracing::warn;

use super::error::HandlerError;
use crate::ConnCtx;
use crate::net::framing::PacketOwned;
use crate::net::inbound::ConnId;
use crate::net::outbound::send_msg_tcp;
use crate::room::state::RoomManager;

mod hello;
mod input_batch;
mod join_room;
mod load_rom;
mod pause_game;
mod provide_state;
mod request_state;
mod reset_game;
mod rom_loaded;
mod switch_role;
mod sync_state;

/// Sends an error response to the client.
async fn send_error_response(ctx: &mut ConnCtx, error: HandlerError) {
    let msg = ErrorMsg { code: error.code };
    let mut h = Header::new(MsgId::Error as u8);
    h.client_id = ctx.assigned_client_id;
    h.room_id = 0;
    h.seq = ctx.server_seq;
    ctx.server_seq = ctx.server_seq.wrapping_add(1);
    let _ = send_msg_tcp(&ctx.outbound, h, MsgId::Error, &msg).await;
}

pub(crate) async fn dispatch_packet(
    ctx: &mut ConnCtx,
    conn_id: ConnId,
    peer: &SocketAddr,
    packet: &PacketOwned,
    room_mgr: &mut RoomManager,
) {
    let result = match packet.msg_id {
        MsgId::Hello => hello::handle(ctx, peer, &packet.payload).await,
        MsgId::JoinRoom => join_room::handle(ctx, conn_id, peer, &packet.payload, room_mgr).await,
        MsgId::InputBatch => input_batch::handle(ctx, peer, &packet.payload, room_mgr).await,
        MsgId::SwitchRole => {
            switch_role::handle(ctx, conn_id, peer, &packet.payload, room_mgr).await
        }
        MsgId::LoadRom => load_rom::handle(ctx, peer, &packet.payload, room_mgr).await,
        MsgId::RomLoaded => rom_loaded::handle(ctx, room_mgr).await,
        MsgId::PauseGame => pause_game::handle(ctx, &packet.payload, room_mgr).await,
        MsgId::ResetGame => reset_game::handle(ctx, &packet.payload, room_mgr).await,
        MsgId::RequestState => request_state::handle(ctx, room_mgr).await,
        MsgId::SyncState => sync_state::handle(ctx, &packet.payload, room_mgr).await,
        MsgId::ProvideState => provide_state::handle(ctx, &packet.payload, room_mgr).await,
        _ => {
            warn!(
                conn_id,
                client_id = ctx.assigned_client_id,
                room_id = packet.header.room_id,
                msg_id = ?packet.msg_id,
                payload_len = packet.payload.len(),
                %peer,
                "Unhandled message (ignored)"
            );
            return;
        }
    };

    if let Err(e) = result {
        send_error_response(ctx, e).await;
    }
}
