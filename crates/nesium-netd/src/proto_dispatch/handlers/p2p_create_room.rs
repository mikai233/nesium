//! P2PCreateRoomHandler - handles P2PCreateRoom messages.

use nesium_netproto::messages::session::{P2P_MAX_HOST_ADDRS, P2PCreateRoom, P2PRoomCreated};
use tracing::{info, warn};

use super::{Handler, HandlerContext};
use crate::net::outbound::send_msg_tcp;
use crate::proto_dispatch::error::{HandlerError, HandlerResult};
use crate::room::state::P2PHostInfo;

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

        let room_id = ctx.room_mgr.create_room(ctx.conn_ctx.assigned_client_id);
        let Some(room) = ctx.room_mgr.get_room_mut(room_id) else {
            return Err(HandlerError::invalid_state());
        };

        room.set_p2p_host(P2PHostInfo {
            host_signal_client_id: ctx.conn_ctx.assigned_client_id,
            host_addrs: msg.host_addrs,
            host_room_code: msg.host_room_code,
            host_quic_cert_sha256_fingerprint: msg.host_quic_cert_sha256_fingerprint,
            host_quic_server_name: msg.host_quic_server_name,
        });
        room.upsert_p2p_watcher(
            ctx.conn_ctx.assigned_client_id,
            ctx.conn_ctx.outbound.clone(),
        );

        let resp = P2PRoomCreated {
            room_code: room.code,
        };
        send_msg_tcp(&ctx.conn_ctx.outbound, &resp)
            .await
            .map_err(|_| HandlerError::invalid_state())?;

        info!(
            client_id = ctx.conn_ctx.assigned_client_id,
            room_code = room.code,
            "Created P2P signaling room"
        );

        Ok(())
    }
}
