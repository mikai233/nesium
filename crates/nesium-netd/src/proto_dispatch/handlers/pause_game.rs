//! PauseGameHandler - handles PauseGame messages.

use nesium_netproto::messages::session::{PauseGame, PauseSync};
use tracing::{info, warn};

use super::{Handler, HandlerContext};
use crate::net::outbound::send_msg;
use crate::proto_dispatch::error::{HandlerError, HandlerResult};

/// Handler for PauseGame messages.
pub(crate) struct PauseGameHandler;

impl Handler<PauseGame> for PauseGameHandler {
    async fn handle(&self, ctx: &mut HandlerContext<'_>, msg: PauseGame) -> HandlerResult {
        let Some(room) = ctx
            .room_mgr
            .client_room_mut(ctx.conn_ctx.assigned_client_id)
        else {
            return Err(HandlerError::not_in_room());
        };

        let recipients = room.handle_pause_game(ctx.conn_ctx.assigned_client_id, msg.paused);
        if recipients.is_empty() {
            return Ok(());
        }

        info!(
            client_id = ctx.conn_ctx.assigned_client_id,
            room_id = room.id,
            paused = msg.paused,
            "Broadcasting pause sync"
        );

        let sync_msg = PauseSync { paused: msg.paused };

        for recipient in &recipients {
            if let Err(e) = send_msg(recipient, &sync_msg).await {
                warn!(error = %e, "Failed to broadcast PauseSync");
            }
        }
        Ok(())
    }
}
