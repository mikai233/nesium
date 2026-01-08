use std::net::SocketAddr;

use nesium_netproto::{
    header::Header,
    messages::session::{ErrorMsg, RoleChanged, SwitchRole},
    msg_id::MsgId,
};
use tracing::{info, warn};

use crate::ConnCtx;
use crate::net::inbound::ConnId;
use crate::net::outbound::send_msg_tcp;
use crate::room::state::RoomManager;

pub(crate) async fn handle(
    ctx: &mut ConnCtx,
    _conn_id: ConnId,
    peer: &SocketAddr,
    payload: &[u8],
    room_mgr: &mut RoomManager,
) {
    let msg: SwitchRole = match postcard::from_bytes(payload) {
        Ok(v) => v,
        Err(e) => {
            warn!(%peer, error = %e, "Bad SwitchRole message");
            return;
        }
    };

    let Some(room_id) = room_mgr.get_client_room(ctx.assigned_client_id) else {
        return;
    };
    let Some(room) = room_mgr.get_room_mut(room_id) else {
        return;
    };

    // Role switching during an active game can deadlock lockstep:
    // existing players will start waiting for inputs from the newly-promoted role,
    // but the switching client may still be catching up.
    if room.started {
        warn!(%peer, room_id, "Rejecting SwitchRole while game is running");
        let msg = ErrorMsg {
            code: 1,
            message: 1,
        };
        let mut h = Header::new(MsgId::Error as u8);
        h.client_id = 0;
        h.room_id = room_id;
        h.seq = ctx.server_seq;
        ctx.server_seq = ctx.server_seq.wrapping_add(1);
        let _ = send_msg_tcp(&ctx.outbound, h, MsgId::Error, &msg).await;
        return;
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
        }
    }
}
