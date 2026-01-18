//! QueryRoomHandler - handles QueryRoom messages.

use nesium_netproto::messages::session::{QueryRoom, RoomInfo};
use tracing::{info, warn};

use super::{Handler, HandlerContext};
use crate::net::outbound::send_msg_tcp;
use crate::proto_dispatch::error::HandlerResult;
use crate::room::state::MAX_PLAYERS;

/// Handler for QueryRoom messages.
pub(crate) struct QueryRoomHandler;

impl Handler<QueryRoom> for QueryRoomHandler {
    async fn handle(&self, ctx: &mut HandlerContext<'_>, msg: QueryRoom) -> HandlerResult {
        let mut resp = RoomInfo {
            request_id: msg.request_id,
            ok: false,
            room_id: 0,
            started: false,
            sync_mode: Default::default(),
            occupied_mask: 0,
        };

        if let Some(found) = ctx.room_mgr.find_by_code(msg.room_code) {
            resp.ok = true;
            resp.room_id = found.id;
            resp.started = found.started;
            resp.sync_mode = found.sync_mode;
            let mut mask = 0u8;
            for idx in 0..MAX_PLAYERS {
                if found.players.contains_key(&(idx as u8)) {
                    mask |= 1u8 << (idx as u8);
                }
            }
            resp.occupied_mask = mask;
        }
        if let Err(e) = send_msg_tcp(&ctx.conn_ctx.outbound, &resp).await {
            warn!(error = %e, "Failed to send RoomInfo");
        }

        info!(
            client_id = ctx.conn_ctx.assigned_client_id,
            room_code = msg.room_code,
            ok = resp.ok,
            occupied_mask = resp.occupied_mask,
            "QueryRoom handled"
        );

        Ok(())
    }
}
