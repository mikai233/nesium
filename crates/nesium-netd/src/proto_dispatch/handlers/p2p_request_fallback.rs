use std::net::SocketAddr;

use nesium_netproto::{
    header::Header,
    messages::session::{P2P_MAX_REASON_LEN, P2PFallbackNotice, P2PRequestFallback},
    msg_id::MsgId,
};
use tracing::{info, warn};

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
    let req: P2PRequestFallback = match postcard::from_bytes(payload) {
        Ok(v) => v,
        Err(e) => {
            warn!(%peer, error = %e, "Bad P2PRequestFallback message");
            return Err(HandlerError::bad_message());
        }
    };

    if ctx.assigned_client_id == 0 {
        return Err(HandlerError::invalid_state());
    }

    // Truncate reason to prevent abuse
    let reason = if req.reason.len() > P2P_MAX_REASON_LEN {
        req.reason[..P2P_MAX_REASON_LEN].to_string()
    } else {
        req.reason
    };

    let (recipients, notice) = {
        let Some(room) = room_mgr.find_by_code_mut(req.room_code) else {
            return Err(HandlerError::room_not_found());
        };

        room.request_p2p_fallback(ctx.assigned_client_id, reason);
        let Some(fallback) = room.p2p_fallback.clone() else {
            return Err(HandlerError::invalid_state());
        };

        let recipients = room.p2p_watchers.values().cloned().collect::<Vec<_>>();
        (
            recipients,
            P2PFallbackNotice {
                room_code: room.code,
                reason: fallback.reason,
                requested_by_client_id: fallback.requested_by_client_id,
            },
        )
    };

    let h = Header::new(MsgId::P2PFallbackNotice as u8);
    for tx in &recipients {
        let _ = send_msg_tcp(tx, h, MsgId::P2PFallbackNotice, &notice).await;
    }

    info!(
        room_code = notice.room_code,
        requested_by_client_id = notice.requested_by_client_id,
        "P2P relay fallback requested"
    );

    Ok(())
}
