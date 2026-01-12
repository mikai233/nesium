use std::net::SocketAddr;

use nesium_netproto::messages::input::InputBatch;
use nesium_netproto::msg_id::MsgId;
use tracing::warn;

use crate::ConnCtx;
use crate::proto_dispatch::error::{HandlerError, HandlerResult};
use crate::room::broadcast::{broadcast_inputs_best_effort, broadcast_inputs_required};
use crate::room::state::RoomManager;

pub(crate) async fn handle(
    ctx: &mut ConnCtx,
    peer: &SocketAddr,
    payload: &[u8],
    room_mgr: &mut RoomManager,
) -> HandlerResult {
    let batch: InputBatch = match postcard::from_bytes(payload) {
        Ok(v) => v,
        Err(e) => {
            warn!(%peer, error = %e, "Bad InputBatch message");
            return Err(HandlerError::bad_message());
        }
    };

    let Some(room_id) = room_mgr.get_client_room(ctx.assigned_client_id) else {
        return Err(HandlerError::not_in_room());
    };

    let Some(room) = room_mgr.get_room_mut(room_id) else {
        return Err(HandlerError::not_in_room());
    };

    let player_index = room
        .players
        .values()
        .find(|p| p.client_id == ctx.assigned_client_id)
        .map(|p| p.player_index);

    let Some(player_index) = player_index else {
        // Client is not a player (spectator?), cannot send inputs
        return Err(HandlerError::permission_denied());
    };

    room.record_inputs(player_index, batch.start_frame, &batch.buttons);

    // Players are required recipients for lockstep; spectators are best-effort.
    let player_recipients: Vec<_> = room
        .players
        .values()
        .map(|p| p.outbounds.outbound_for_msg(MsgId::RelayInputs))
        .collect();
    let spectator_recipients: Vec<_> = room
        .spectators
        .iter()
        .map(|s| s.outbounds.outbound_for_msg(MsgId::RelayInputs))
        .collect();

    broadcast_inputs_required(
        &player_recipients,
        player_index,
        batch.start_frame,
        &batch.buttons,
    )
    .await;
    broadcast_inputs_best_effort(
        &spectator_recipients,
        player_index,
        batch.start_frame,
        &batch.buttons,
    );
    Ok(())
}
