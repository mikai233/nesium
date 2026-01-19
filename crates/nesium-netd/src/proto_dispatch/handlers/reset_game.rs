//! ResetGameHandler - handles ResetGame messages.

use bytes::Bytes;
use nesium_netproto::{
    codec::encode_message,
    messages::session::{ResetGame, ResetSync},
};
use tracing::{error, info};

use super::{Handler, HandlerContext};
use crate::proto_dispatch::error::{HandlerError, HandlerResult};

/// Handler for ResetGame messages.
pub(crate) struct ResetGameHandler;

impl Handler<ResetGame> for ResetGameHandler {
    async fn handle(&self, ctx: &mut HandlerContext<'_>, msg: ResetGame) -> HandlerResult {
        let client_id = ctx.require_client_id()?;
        let room = ctx.require_room_mut()?;

        let recipients = room.handle_reset_game(client_id);
        if recipients.is_empty() {
            return Ok(());
        }

        info!(
            client_id,
            room_id = room.id,
            kind = msg.kind,
            "Broadcasting reset sync"
        );

        let sync_msg = ResetSync { kind: msg.kind };

        let frame = match encode_message(&sync_msg) {
            Ok(f) => Bytes::from(f),
            Err(e) => {
                error!("Failed to serialize ResetSync: {}", e);
                return Err(HandlerError::invalid_state());
            }
        };

        for tx in recipients {
            let frame = frame.clone();
            tokio::spawn(async move {
                let _ = tx.send(frame).await;
            });
        }
        Ok(())
    }
}
