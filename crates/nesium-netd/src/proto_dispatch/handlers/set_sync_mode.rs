//! SetSyncModeHandler - handles SetSyncMode messages.

use nesium_netproto::messages::session::{SetSyncMode, SyncModeChanged};
use tracing::{info, warn};

use super::{Handler, HandlerContext};
use crate::net::outbound::send_msg_tcp;
use crate::proto_dispatch::error::{HandlerError, HandlerResult};

/// Handler for SetSyncMode messages.
pub(crate) struct SetSyncModeHandler;

impl Handler<SetSyncMode> for SetSyncModeHandler {
    async fn handle(&self, ctx: &mut HandlerContext<'_>, msg: SetSyncMode) -> HandlerResult {
        let Some(room) = ctx
            .room_mgr
            .client_room_mut(ctx.conn_ctx.assigned_client_id)
        else {
            return Err(HandlerError::not_in_room());
        };

        let recipients = room.handle_set_sync_mode(ctx.conn_ctx.assigned_client_id, msg.mode)?;

        if recipients.is_empty() {
            return Ok(());
        }

        info!(
            client_id = ctx.conn_ctx.assigned_client_id,
            room_id = room.id,
            sync_mode = ?msg.mode,
            "Broadcasting sync mode change"
        );

        let sync_msg = SyncModeChanged { mode: msg.mode };

        for recipient in &recipients {
            if let Err(e) = send_msg_tcp(recipient, &sync_msg).await {
                warn!(error = %e, "Failed to broadcast SyncModeChanged");
            }
        }
        Ok(())
    }
}
