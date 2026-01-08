use nesium_netproto::{header::Header, messages::session::SyncState, msg_id::MsgId};
use tracing::{debug, info, warn};

use crate::ConnCtx;
use crate::net::outbound::send_msg_tcp;
use crate::room::state::RoomManager;

pub(crate) async fn handle(ctx: &mut ConnCtx, room_mgr: &mut RoomManager) {
    let Some(room_id) = room_mgr.get_client_room(ctx.assigned_client_id) else {
        return;
    };
    let Some(room) = room_mgr.get_room_mut(room_id) else {
        return;
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
        let mut h = Header::new(MsgId::SyncState as u8);
        h.client_id = 0; // System message
        h.room_id = room_id;
        h.seq = 0;

        if let Err(e) = send_msg_tcp(&ctx.outbound, h, MsgId::SyncState, &sync_msg).await {
            warn!(error = %e, "Failed to send SyncState");
        }
    } else {
        debug!(
            client_id = ctx.assigned_client_id,
            room_id, "No cached state available"
        );
    }
}
