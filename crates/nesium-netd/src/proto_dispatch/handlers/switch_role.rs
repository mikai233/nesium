use std::net::SocketAddr;

use nesium_netproto::{
    header::Header,
    messages::session::{RoleChanged, SwitchRole},
    msg_id::MsgId,
};
use tracing::{info, warn};

use crate::ConnCtx;
use crate::net::inbound::ConnId;
use crate::net::outbound::send_msg_tcp;
use crate::proto_dispatch::error::{HandlerError, HandlerResult};
use crate::room::state::RoomManager;

pub(crate) async fn handle(
    ctx: &mut ConnCtx,
    _conn_id: ConnId,
    peer: &SocketAddr,
    payload: &[u8],
    room_mgr: &mut RoomManager,
) -> HandlerResult {
    let msg: SwitchRole = match postcard::from_bytes(payload) {
        Ok(v) => v,
        Err(e) => {
            warn!(%peer, error = %e, "Bad SwitchRole message");
            return Err(HandlerError::bad_message());
        }
    };

    let Some(room_id) = room_mgr.get_client_room(ctx.assigned_client_id) else {
        warn!(%peer, "SwitchRole: client not in a room");
        return Err(HandlerError::not_in_room());
    };
    let Some(room) = room_mgr.get_room_mut(room_id) else {
        return Err(HandlerError::not_in_room());
    };

    // Role switching during an active game can deadlock lockstep:
    // existing players will start waiting for inputs from the newly-promoted role,
    // but the switching client may still be catching up.
    if room.started {
        warn!(%peer, room_id, "Rejecting SwitchRole while game is running");
        return Err(HandlerError::game_already_started());
    }

    match room.switch_player_role(ctx.assigned_client_id, msg.new_role) {
        Ok(changes) => {
            let recipients = room.all_outbounds();
            for (cid, role) in changes {
                let broadcast = RoleChanged {
                    client_id: cid,
                    new_role: role,
                };
                let mut h = Header::new(MsgId::RoleChanged as u8);
                h.client_id = cid;
                h.room_id = room_id;
                h.seq = 0;

                for recipient in &recipients {
                    if let Err(e) = send_msg_tcp(recipient, h, MsgId::RoleChanged, &broadcast).await
                    {
                        warn!(error = %e, "Failed to broadcast RoleChanged");
                    }
                }

                info!(client_id = cid, room_id, new_role = role, "Role changed");
            }
        }
        Err(e) => {
            warn!(%peer, error = %e, "Failed to switch role");
            return Err(HandlerError::invalid_state());
        }
    }
    Ok(())
}
