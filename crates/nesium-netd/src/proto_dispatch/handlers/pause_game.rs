use nesium_netproto::{
    header::Header,
    messages::session::{PauseGame, PauseSync},
    msg_id::MsgId,
};
use tracing::{info, warn};

use crate::ConnCtx;
use crate::net::outbound::send_msg_tcp;
use crate::proto_dispatch::error::{HandlerError, HandlerResult};
use crate::room::state::RoomManager;

pub(crate) async fn handle(
    ctx: &mut ConnCtx,
    payload: &[u8],
    room_mgr: &mut RoomManager,
) -> HandlerResult {
    let msg: PauseGame = match postcard::from_bytes(payload) {
        Ok(v) => v,
        Err(_) => return Err(HandlerError::bad_message()),
    };

    let Some(room_id) = room_mgr.get_client_room(ctx.assigned_client_id) else {
        return Err(HandlerError::not_in_room());
    };
    let Some(room) = room_mgr.get_room_mut(room_id) else {
        return Err(HandlerError::not_in_room());
    };

    let recipients = room.handle_pause_game(ctx.assigned_client_id, msg.paused);
    if recipients.is_empty() {
        return Ok(());
    }

    info!(
        client_id = ctx.assigned_client_id,
        room_id,
        paused = msg.paused,
        "Broadcasting pause sync"
    );

    let sync_msg = PauseSync { paused: msg.paused };
    let mut h = Header::new(MsgId::PauseSync as u8);
    h.client_id = ctx.assigned_client_id;
    h.room_id = room_id;
    h.seq = 0;

    for recipient in &recipients {
        if let Err(e) = send_msg_tcp(recipient, h, MsgId::PauseSync, &sync_msg).await {
            warn!(error = %e, "Failed to broadcast PauseSync");
        }
    }
    Ok(())
}
