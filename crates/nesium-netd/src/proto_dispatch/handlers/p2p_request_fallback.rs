//! P2PRequestFallbackHandler - handles P2PRequestFallback messages.

use nesium_netproto::messages::session::{
    P2P_MAX_REASON_LEN, P2PFallbackNotice, P2PRequestFallback,
};
use tracing::info;

use super::{Handler, HandlerContext};
use crate::net::outbound::send_msg_tcp;
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

        let (recipients, notice) = {
            let Some(room) = ctx.room_mgr.get_room_mut(req.room_id) else {
                return Err(HandlerError::room_not_found());
            };

            room.request_p2p_fallback(ctx.conn_ctx.assigned_client_id, reason);
            let Some(fallback) = room.p2p_fallback.clone() else {
                return Err(HandlerError::invalid_state());
            };

            let recipients = room.p2p_watchers.values().cloned().collect::<Vec<_>>();
            (
                recipients,
                P2PFallbackNotice {
                    room_id: room.id,
                    reason: fallback.reason,
                    requested_by_client_id: fallback.requested_by_client_id,
                },
            )
        };
        for tx in &recipients {
            let _ = send_msg_tcp(tx, &notice).await;
        }

        info!(
            room_id = notice.room_id,
            requested_by_client_id = notice.requested_by_client_id,
            "P2P relay fallback requested"
        );

        Ok(())
    }
}
