//! PingHandler - handles Ping messages for room keep-alive.

use nesium_netproto::messages::sync::{Ping, Pong};
use tracing::trace;

use super::{Handler, HandlerContext};
use crate::net::outbound::send_msg_tcp;
use crate::proto_dispatch::error::HandlerResult;

/// Handler for Ping messages.
///
/// Updates room activity timestamp and responds with Pong.
pub(crate) struct PingHandler;

impl Handler<Ping> for PingHandler {
    async fn handle(&self, ctx: &mut HandlerContext<'_>, msg: Ping) -> HandlerResult {
        // Get client's room and touch it to update activity timestamp
        if let Some(room_id) = ctx
            .room_mgr
            .get_client_room(ctx.conn_ctx.assigned_client_id)
        {
            if let Some(room) = ctx.room_mgr.get_room_mut(room_id) {
                room.touch();
                trace!(
                    room_id,
                    client_id = ctx.conn_ctx.assigned_client_id,
                    "Room activity updated via Ping"
                );
            }
        }

        // Respond with Pong
        let pong = Pong { t_ms: msg.t_ms };
        let _ = send_msg_tcp(&ctx.conn_ctx.outbound, &pong).await;

        Ok(())
    }
}
