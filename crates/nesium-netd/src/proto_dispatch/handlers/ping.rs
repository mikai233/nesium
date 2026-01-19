//! PingHandler - handles Ping messages for connection keep-alive.

use nesium_netproto::messages::sync::{Ping, Pong};
use tracing::trace;

use super::{Handler, HandlerContext};
use crate::net::outbound::send_msg_tcp;
use crate::proto_dispatch::error::HandlerResult;

/// Handler for Ping messages.
///
/// Connection activity is tracked at the connection level in handle_packet().
/// This handler simply responds with Pong for RTT measurement.
pub(crate) struct PingHandler;

impl Handler<Ping> for PingHandler {
    async fn handle(&self, ctx: &mut HandlerContext<'_>, msg: Ping) -> HandlerResult {
        trace!(
            client_id = ctx.conn_ctx.assigned_client_id,
            "Received Ping, responding with Pong"
        );

        // Respond with Pong
        let pong = Pong { t_ms: msg.t_ms };
        let _ = send_msg_tcp(&ctx.conn_ctx.outbound, &pong).await;

        Ok(())
    }
}
