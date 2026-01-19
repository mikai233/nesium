//! P2PCreateRoomHandler - handles P2PCreateRoom messages.

use nesium_netproto::messages::session::{P2P_MAX_HOST_ADDRS, P2PCreateRoom, P2PRoomCreated};
use tracing::{info, warn};

use super::{Handler, HandlerContext};
use crate::net::outbound::send_msg_tcp;
use crate::proto_dispatch::error::{HandlerError, HandlerResult};

/// Handler for P2PCreateRoom messages.
pub(crate) struct P2PCreateRoomHandler;

impl Handler<P2PCreateRoom> for P2PCreateRoomHandler {
    async fn handle(&self, ctx: &mut HandlerContext<'_>, msg: P2PCreateRoom) -> HandlerResult {
        if ctx.conn_ctx.assigned_client_id == 0 {
            return Err(HandlerError::invalid_state());
        }

        // Validate input limits
        if msg.host_addrs.len() > P2P_MAX_HOST_ADDRS {
            warn!(%ctx.peer, addrs_len = msg.host_addrs.len(), "Too many host addresses");
            return Err(HandlerError::bad_message());
        }

        let Some(room_id) = ctx.room_mgr.create_room(ctx.conn_ctx.assigned_client_id) else {
            return Err(HandlerError::server_full());
        };
        let Some(room) = ctx.room_mgr.room_mut(room_id) else {
            return Err(HandlerError::invalid_state());
        };

        room.set_p2p_host(
            ctx.conn_ctx.assigned_client_id,
            msg.host_addrs,
            msg.host_room_id,
            msg.host_quic_cert_sha256_fingerprint,
            msg.host_quic_server_name,
        );
        room.upsert_p2p_watcher(
            ctx.conn_ctx.assigned_client_id,
            ctx.conn_ctx.outbound.clone(),
        );

        let resp = P2PRoomCreated { room_id: room.id };
        send_msg_tcp(&ctx.conn_ctx.outbound, &resp)
            .await
            .map_err(|_| HandlerError::invalid_state())?;

        info!(
            client_id = ctx.conn_ctx.assigned_client_id,
            room_id = room.id,
            "Created P2P signaling room"
        );

        Ok(())
    }
}
