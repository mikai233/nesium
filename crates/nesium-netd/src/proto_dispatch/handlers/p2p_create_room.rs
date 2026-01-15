use std::net::SocketAddr;

use nesium_netproto::{
    header::Header,
    messages::session::{P2P_MAX_HOST_ADDRS, P2PCreateRoom, P2PRoomCreated},
    msg_id::MsgId,
};
use tracing::{info, warn};

use crate::{
    ConnCtx,
    net::outbound::send_msg_tcp,
    proto_dispatch::error::{HandlerError, HandlerResult},
    room::state::{P2PHostInfo, RoomManager},
};

pub(crate) async fn handle(
    ctx: &mut ConnCtx,
    peer: &SocketAddr,
    payload: &[u8],
    room_mgr: &mut RoomManager,
) -> HandlerResult {
    let msg: P2PCreateRoom = match postcard::from_bytes(payload) {
        Ok(v) => v,
        Err(e) => {
            warn!(%peer, error = %e, "Bad P2PCreateRoom message");
            return Err(HandlerError::bad_message());
        }
    };

    if ctx.assigned_client_id == 0 {
        return Err(HandlerError::invalid_state());
    }

    // Validate input limits
    if msg.host_addrs.len() > P2P_MAX_HOST_ADDRS {
        warn!(%peer, addrs_len = msg.host_addrs.len(), "Too many host addresses");
        return Err(HandlerError::bad_message());
    }

    let room_id = room_mgr.create_room(ctx.assigned_client_id);
    let Some(room) = room_mgr.get_room_mut(room_id) else {
        return Err(HandlerError::invalid_state());
    };

    room.set_p2p_host(P2PHostInfo {
        host_signal_client_id: ctx.assigned_client_id,
        host_addrs: msg.host_addrs,
        host_room_code: msg.host_room_code,
        host_quic_cert_sha256_fingerprint: msg.host_quic_cert_sha256_fingerprint,
        host_quic_server_name: msg.host_quic_server_name,
    });
    room.upsert_p2p_watcher(ctx.assigned_client_id, ctx.outbound.clone());

    let resp = P2PRoomCreated {
        room_code: room.code,
    };
    let h = Header::new(MsgId::P2PRoomCreated as u8);
    send_msg_tcp(&ctx.outbound, h, MsgId::P2PRoomCreated, &resp)
        .await
        .map_err(|_| HandlerError::invalid_state())?;

    info!(
        client_id = ctx.assigned_client_id,
        room_code = room.code,
        "Created P2P signaling room"
    );

    Ok(())
}
