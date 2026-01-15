use std::net::SocketAddr;

use nesium_netproto::{
    header::Header,
    messages::session::{P2PJoinAck, P2PJoinRoom},
    msg_id::MsgId,
};
use tracing::warn;

use crate::{
    ConnCtx,
    net::outbound::send_msg_tcp,
    proto_dispatch::error::{HandlerError, HandlerResult},
    room::state::RoomManager,
};

pub(crate) async fn handle(
    ctx: &mut ConnCtx,
    peer: &SocketAddr,
    payload: &[u8],
    room_mgr: &mut RoomManager,
) -> HandlerResult {
    let join: P2PJoinRoom = match postcard::from_bytes(payload) {
        Ok(v) => v,
        Err(e) => {
            warn!(%peer, error = %e, "Bad P2PJoinRoom message");
            return Err(HandlerError::bad_message());
        }
    };

    if ctx.assigned_client_id == 0 {
        return Err(HandlerError::invalid_state());
    }

    let Some(room) = room_mgr.find_by_code_mut(join.room_code) else {
        return Err(HandlerError::room_not_found());
    };

    room.upsert_p2p_watcher(ctx.assigned_client_id, ctx.outbound.clone());

    let Some(host) = room.p2p_host.clone() else {
        return Err(HandlerError::host_not_available());
    };

    let fallback_required = room.p2p_fallback.is_some();
    let fallback_reason = room.p2p_fallback.as_ref().map(|s| s.reason.clone());

    let ack = P2PJoinAck {
        ok: true,
        room_code: room.code,
        host_addrs: host.host_addrs,
        host_room_code: host.host_room_code,
        host_quic_cert_sha256_fingerprint: host.host_quic_cert_sha256_fingerprint,
        host_quic_server_name: host.host_quic_server_name,
        fallback_required,
        fallback_reason,
    };

    let h = Header::new(MsgId::P2PJoinAck as u8);
    send_msg_tcp(&ctx.outbound, h, MsgId::P2PJoinAck, &ack)
        .await
        .map_err(|_| HandlerError::invalid_state())?;
    Ok(())
}
