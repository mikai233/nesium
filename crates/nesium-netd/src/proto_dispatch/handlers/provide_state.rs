use nesium_netproto::messages::session::ProvideState;
use tracing::{debug, warn};

use crate::ConnCtx;
use crate::room::state::RoomManager;

pub(crate) async fn handle(ctx: &mut ConnCtx, payload: &[u8], room_mgr: &mut RoomManager) {
    let msg: ProvideState = match postcard::from_bytes(payload) {
        Ok(v) => v,
        Err(e) => {
            warn!(error = %e, "Bad ProvideState message");
            return;
        }
    };

    let Some(room_id) = room_mgr.get_client_room(ctx.assigned_client_id) else {
        return;
    };
    let Some(room) = room_mgr.get_room_mut(room_id) else {
        return;
    };

    room.cache_state(msg.frame, msg.data);
    debug!(room_id, frame = msg.frame, "Cached game state");
}
