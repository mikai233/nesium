use std::net::SocketAddr;

use nesium_netproto::{
    header::Header,
    messages::session::{FallbackToRelay, RequestFallbackRelay},
    msg_id::MsgId,
};
use tracing::warn;

use crate::{
    ConnCtx,
    net::outbound::send_msg_tcp,
    proto_dispatch::error::{HandlerError, HandlerResult},
    room::state::RoomManager,
};

pub(crate) async fn handle(
    ctx: &mut ConnCtx,
    peer: &SocketAddr,
    payload: &[u8],
    room_mgr: &mut RoomManager,
) -> HandlerResult {
    let req: RequestFallbackRelay = match postcard::from_bytes(payload) {
        Ok(v) => v,
        Err(e) => {
            warn!(%peer, error = %e, "Bad RequestFallbackRelay message");
            return Err(HandlerError::bad_message());
        }
    };

    let sender_id = ctx.assigned_client_id;
    if sender_id == 0 {
        return Err(HandlerError::invalid_state());
    }

    let Some(room_id) = room_mgr.get_client_room(sender_id) else {
        return Err(HandlerError::not_in_room());
    };
    let Some(room) = room_mgr.get_room_mut(room_id) else {
        return Err(HandlerError::not_in_room());
    };

    // Only the room host can request forcing clients to reconnect to relay.
    if room.host_client_id != sender_id {
        return Err(HandlerError::permission_denied());
    }

    let is_player = room.players.values().any(|p| p.client_id == sender_id);
    if !is_player {
        return Err(HandlerError::permission_denied());
    }

    let msg = FallbackToRelay {
        relay_addr: req.relay_addr,
        relay_room_code: req.relay_room_code,
        reason: req.reason,
    };

    let recipients = room
        .players
        .values()
        .filter(|p| p.client_id != sender_id)
        .map(|p| p.outbounds.outbound_for_msg(MsgId::FallbackToRelay))
        .chain(
            room.spectators
                .iter()
                .map(|s| s.outbounds.outbound_for_msg(MsgId::FallbackToRelay)),
        )
        .collect::<Vec<_>>();

    let h = Header::new(MsgId::FallbackToRelay as u8);
    for tx in &recipients {
        let _ = send_msg_tcp(tx, h, MsgId::FallbackToRelay, &msg).await;
    }

    Ok(())
}
