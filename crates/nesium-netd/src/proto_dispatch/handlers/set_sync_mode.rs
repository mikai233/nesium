//! SetSyncModeHandler - handles SetSyncMode messages.

use nesium_netproto::messages::session::{SetSyncMode, SyncModeChanged};
use tracing::{info, warn};

use super::{Handler, HandlerContext};
use crate::net::outbound::send_msg;
use crate::proto_dispatch::error::HandlerResult;

/// Handler for SetSyncMode messages.
pub(crate) struct SetSyncModeHandler;

impl Handler<SetSyncMode> for SetSyncModeHandler {
    async fn handle(&self, ctx: &mut HandlerContext<'_>, msg: SetSyncMode) -> HandlerResult {
        let client_id = ctx.require_client_id()?;
        let room = ctx.require_room_mut()?;

        let recipients = room.handle_set_sync_mode(client_id, msg.mode)?;

        if recipients.is_empty() {
            return Ok(());
        }

        info!(
            client_id,
            room_id = room.id,
            sync_mode = ?msg.mode,
            "Broadcasting sync mode change"
        );

        let sync_msg = SyncModeChanged { mode: msg.mode };

        for recipient in &recipients {
            if let Err(e) = send_msg(recipient, &sync_msg).await {
                warn!(error = %e, "Failed to broadcast SyncModeChanged");
            }
        }
        Ok(())
    }
}
