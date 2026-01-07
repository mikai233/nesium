use std::net::SocketAddr;

use nesium_netproto::msg_id::MsgId;
use tracing::warn;

use crate::net::framing::PacketOwned;
use crate::room::state::RoomManager;
use crate::{ConnCtx, net::inbound::ConnId};

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

pub(crate) async fn dispatch_packet(
    ctx: &mut ConnCtx,
    conn_id: ConnId,
    peer: &SocketAddr,
    packet: &PacketOwned,
    room_mgr: &mut RoomManager,
) {
    match packet.msg_id {
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
        }
    }
}
