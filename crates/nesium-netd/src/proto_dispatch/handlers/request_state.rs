//! RequestStateHandler - handles RequestState messages.

use nesium_netproto::messages::session::{RequestState, SyncState};
use nesium_netproto::msg_id::MsgId;
use tracing::{debug, info, warn};

use super::{Handler, HandlerContext};
use crate::net::outbound::send_msg;
use crate::proto_dispatch::error::{HandlerError, HandlerResult};

/// Handler for RequestState messages.
pub(crate) struct RequestStateHandler;

impl Handler<RequestState> for RequestStateHandler {
    async fn handle(&self, ctx: &mut HandlerContext<'_>, _msg: RequestState) -> HandlerResult {
        let Some(room) = ctx
            .room_mgr
            .client_room_mut(ctx.conn_ctx.assigned_client_id)
        else {
            return Err(HandlerError::not_in_room());
        };

        if let Some((frame, state_data)) = room.cached_state.clone() {
            info!(
                client_id = ctx.conn_ctx.assigned_client_id,
                room_id = room.id,
                size = state_data.len(),
                frame,
                "Sending cached state to client"
            );

            let sync_msg = SyncState {
                frame,
                data: state_data,
            };

            let tx = room
                .outbound_for_client_msg(ctx.conn_ctx.assigned_client_id, MsgId::SyncState)
                .unwrap_or_else(|| ctx.conn_ctx.outbound.clone());

            if let Err(e) = send_msg(&tx, &sync_msg).await {
                warn!(error = %e, "Failed to send SyncState");
            }
        } else {
            debug!(
                client_id = ctx.conn_ctx.assigned_client_id,
                room_id = room.id,
                "No cached state available"
            );
        }
        Ok(())
    }
}
