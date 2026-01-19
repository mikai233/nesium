//! RequestFallbackRelayHandler - handles RequestFallbackRelay messages.

use nesium_netproto::{
    messages::session::{FallbackToRelay, RequestFallbackRelay},
    msg_id::MsgId,
};

use super::{Handler, HandlerContext};
use crate::net::outbound::send_msg;
use crate::proto_dispatch::error::{HandlerError, HandlerResult};

/// Handler for RequestFallbackRelay messages.
pub(crate) struct RequestFallbackRelayHandler;

impl Handler<RequestFallbackRelay> for RequestFallbackRelayHandler {
    async fn handle(
        &self,
        ctx: &mut HandlerContext<'_>,
        req: RequestFallbackRelay,
    ) -> HandlerResult {
        let sender_id = ctx.require_client_id()?;
        let room = ctx.require_room_mut()?;

        // Only the room host can request forcing clients to reconnect to relay.
        if room.host_client_id != sender_id {
            return Err(HandlerError::permission_denied());
        }

        let is_player = room.players.values().any(|p| p.client_id == sender_id);
        if !is_player {
            return Err(HandlerError::permission_denied());
        }

        let msg = FallbackToRelay {
            relay_addr: req.relay_addr,
            relay_room_id: req.relay_room_id,
            reason: req.reason,
        };

        let recipients = room
            .players
            .values()
            .filter(|p| p.client_id != sender_id)
            .map(|p| p.outbounds.outbound_for_msg(MsgId::FallbackToRelay))
            .chain(
                room.spectators
                    .iter()
                    .map(|s| s.outbounds.outbound_for_msg(MsgId::FallbackToRelay)),
            )
            .collect::<Vec<_>>();
        for tx in &recipients {
            let _ = send_msg(tx, &msg).await;
        }

        Ok(())
    }
}
