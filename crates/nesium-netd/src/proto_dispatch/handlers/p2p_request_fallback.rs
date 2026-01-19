//! P2PRequestFallbackHandler - handles P2PRequestFallback messages.

use nesium_netproto::messages::session::{
    P2P_MAX_REASON_LEN, P2PFallbackNotice, P2PRequestFallback,
};
use tracing::info;

use super::{Handler, HandlerContext};
use crate::net::outbound::send_msg;
use crate::proto_dispatch::error::{HandlerError, HandlerResult};

/// Handler for P2PRequestFallback messages.
pub(crate) struct P2PRequestFallbackHandler;

impl Handler<P2PRequestFallback> for P2PRequestFallbackHandler {
    async fn handle(&self, ctx: &mut HandlerContext<'_>, req: P2PRequestFallback) -> HandlerResult {
        if ctx.conn_ctx.assigned_client_id == 0 {
            return Err(HandlerError::invalid_state());
        }

        // Truncate reason to prevent abuse
        let reason = if req.reason.len() > P2P_MAX_REASON_LEN {
            req.reason[..P2P_MAX_REASON_LEN].to_string()
        } else {
            req.reason
        };

        let notice = P2PFallbackNotice {
            room_id: req.room_id,
            reason: reason.clone(),
            requested_by_client_id: ctx.conn_ctx.assigned_client_id,
        };

        let Some(room) = ctx.room_mgr.room_mut(req.room_id) else {
            return Err(HandlerError::room_not_found());
        };

        room.request_p2p_fallback(ctx.conn_ctx.assigned_client_id, reason);

        for tx in room.p2p_watchers.values() {
            let _ = send_msg(tx, &notice).await;
        }

        info!(
            room_id = notice.room_id,
            requested_by_client_id = notice.requested_by_client_id,
            "P2P relay fallback requested"
        );

        Ok(())
    }
}
