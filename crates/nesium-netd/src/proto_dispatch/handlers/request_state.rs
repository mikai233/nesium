//! RequestStateHandler - handles RequestState messages.

use nesium_netproto::messages::session::{RequestState, SyncState};
use nesium_netproto::msg_id::MsgId;
use tracing::{debug, info, warn};

use super::{Handler, HandlerContext};
use crate::net::outbound::send_msg;
use crate::proto_dispatch::error::HandlerResult;

/// Handler for RequestState messages.
pub(crate) struct RequestStateHandler;

impl Handler<RequestState> for RequestStateHandler {
    async fn handle(&self, ctx: &mut HandlerContext<'_>, _msg: RequestState) -> HandlerResult {
        let client_id = ctx.require_client_id()?;
        let room = ctx.require_room_mut()?;

        if let Some((frame, state_data)) = room.cached_state.clone() {
            info!(
                client_id,
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
                .outbound_for_client_msg(client_id, MsgId::SyncState)
                .unwrap_or_else(|| ctx.conn_ctx.outbound.clone());

            if let Err(e) = send_msg(&tx, &sync_msg).await {
                warn!(error = %e, "Failed to send SyncState");
            }
        } else {
            debug!(client_id, room_id = room.id, "No cached state available");
        }
        Ok(())
    }
}
