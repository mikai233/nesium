use std::net::SocketAddr;

use nesium_netproto::{
    constants::MAX_TCP_FRAME, header::Header, messages::session::LoadRom, msg_id::MsgId,
};
use tracing::{info, warn};

use crate::ConnCtx;
use crate::room::state::RoomManager;

pub(crate) async fn handle(
    ctx: &mut ConnCtx,
    peer: &SocketAddr,
    payload: &[u8],
    room_mgr: &mut RoomManager,
) {
    let msg: LoadRom = match postcard::from_bytes(payload) {
        Ok(v) => v,
        Err(e) => {
            warn!(%peer, error = %e, "Bad LoadRom message");
            return;
        }
    };

    let Some(room_id) = room_mgr.get_client_room(ctx.assigned_client_id) else {
        return;
    };
    let Some(room) = room_mgr.get_room_mut(room_id) else {
        return;
    };

    match room.handle_load_rom(ctx.assigned_client_id) {
        Ok(recipients) => {
            // Forward ROM to others
            info!(
                client_id = ctx.assigned_client_id,
                room_id, "Host loaded ROM, forwarding..."
            );

            room.cache_rom(msg.data.clone());

            let mut h = Header::new(MsgId::LoadRom as u8);
            h.client_id = ctx.assigned_client_id;
            h.room_id = room_id;
            h.seq = 0;

            for recipient in &recipients {
                if let Err(e) = crate::net::outbound::send_msg_tcp(
                    recipient,
                    h,
                    MsgId::LoadRom,
                    &msg,
                    MAX_TCP_FRAME,
                )
                .await
                {
                    warn!(error = %e, "Failed to forward LoadRom");
                }
            }
        }
        Err(e) => {
            warn!(%peer, error = %e, "LoadRom rejected");
        }
    }
}
