use nesium_netproto::{header::Header, messages::session::SyncState, msg_id::MsgId};
use tracing::{debug, info, warn};

use crate::ConnCtx;
use crate::net::outbound::send_msg_tcp;
use crate::proto_dispatch::error::{HandlerError, HandlerResult};
use crate::room::state::RoomManager;

pub(crate) async fn handle(ctx: &mut ConnCtx, room_mgr: &mut RoomManager) -> HandlerResult {
    let Some(room_id) = room_mgr.get_client_room(ctx.assigned_client_id) else {
        return Err(HandlerError::not_in_room());
    };
    let Some(room) = room_mgr.get_room_mut(room_id) else {
        return Err(HandlerError::not_in_room());
    };

    if let Some((frame, state_data)) = room.cached_state.clone() {
        info!(
            client_id = ctx.assigned_client_id,
            room_id,
            size = state_data.len(),
            frame,
            "Sending cached state to client"
        );

        let sync_msg = SyncState {
            frame,
            data: state_data,
        };
        let h = Header::new(MsgId::SyncState as u8);

        let tx = room
            .outbound_for_client_msg(ctx.assigned_client_id, MsgId::SyncState)
            .unwrap_or_else(|| ctx.outbound.clone());

        if let Err(e) = send_msg_tcp(&tx, h, MsgId::SyncState, &sync_msg).await {
            warn!(error = %e, "Failed to send SyncState");
        }
    } else {
        debug!(
            client_id = ctx.assigned_client_id,
            room_id, "No cached state available"
        );
    }
    Ok(())
}
