//! P2PJoinRoomHandler - handles P2PJoinRoom messages.

use nesium_netproto::messages::session::{P2PJoinAck, P2PJoinRoom};

use super::{Handler, HandlerContext};
use crate::net::outbound::send_msg_tcp;
use crate::proto_dispatch::error::{HandlerError, HandlerResult};

/// Handler for P2PJoinRoom messages.
pub(crate) struct P2PJoinRoomHandler;

impl Handler<P2PJoinRoom> for P2PJoinRoomHandler {
    async fn handle(&self, ctx: &mut HandlerContext<'_>, join: P2PJoinRoom) -> HandlerResult {
        if ctx.conn_ctx.assigned_client_id == 0 {
            return Err(HandlerError::invalid_state());
        }

        let Some(room) = ctx.room_mgr.get_room_mut(join.room_id) else {
            return Err(HandlerError::room_not_found());
        };

        room.upsert_p2p_watcher(
            ctx.conn_ctx.assigned_client_id,
            ctx.conn_ctx.outbound.clone(),
        );

        let Some(host) = room.p2p_host.clone() else {
            return Err(HandlerError::host_not_available());
        };

        let fallback_required = room.p2p_fallback.is_some();
        let fallback_reason = room.p2p_fallback.as_ref().map(|s| s.reason.clone());

        let ack = P2PJoinAck {
            ok: true,
            room_id: room.id,
            host_addrs: host.host_addrs,
            host_room_id: host.host_room_id,
            host_quic_cert_sha256_fingerprint: host.host_quic_cert_sha256_fingerprint,
            host_quic_server_name: host.host_quic_server_name,
            fallback_required,
            fallback_reason,
        };
        send_msg_tcp(&ctx.conn_ctx.outbound, &ack)
            .await
            .map_err(|_| HandlerError::invalid_state())?;
        Ok(())
    }
}
