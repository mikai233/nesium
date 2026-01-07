use std::net::SocketAddr;

use nesium_netproto::messages::input::InputBatch;
use tracing::warn;

use crate::ConnCtx;
use crate::room::broadcast::{broadcast_inputs_best_effort, broadcast_inputs_required};
use crate::room::state::RoomManager;

pub(crate) async fn handle(
    ctx: &mut ConnCtx,
    peer: &SocketAddr,
    payload: &[u8],
    room_mgr: &mut RoomManager,
) {
    let batch: InputBatch = match postcard::from_bytes(payload) {
        Ok(v) => v,
        Err(e) => {
            warn!(%peer, error = %e, "Bad InputBatch message");
            return;
        }
    };

    let Some(room_id) = room_mgr.get_client_room(ctx.assigned_client_id) else {
        return;
    };

    let Some(room) = room_mgr.get_room_mut(room_id) else {
        return;
    };

    let player_index = room
        .players
        .values()
        .find(|p| p.client_id == ctx.assigned_client_id)
        .map(|p| p.player_index);

    let Some(player_index) = player_index else {
        return;
    };

    room.record_inputs(player_index, batch.start_frame, &batch.buttons);

    // Players are required recipients for lockstep; spectators are best-effort.
    let player_recipients: Vec<_> = room.players.values().map(|p| p.outbound.clone()).collect();
    let spectator_recipients: Vec<_> = room.spectators.iter().map(|s| s.outbound.clone()).collect();

    let mut server_seq = ctx.server_seq;
    broadcast_inputs_required(
        &player_recipients,
        player_index,
        batch.start_frame,
        &batch.buttons,
        room_id,
        &mut server_seq,
    )
    .await;
    broadcast_inputs_best_effort(
        &spectator_recipients,
        player_index,
        batch.start_frame,
        &batch.buttons,
        room_id,
        &mut server_seq,
    );
    ctx.server_seq = server_seq;
}
