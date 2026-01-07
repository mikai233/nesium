use bytes::Bytes;
use nesium_netproto::{
    codec_tcp::encode_tcp_frame,
    header::Header,
    messages::session::{ResetGame, ResetSync},
    msg_id::MsgId,
};
use tracing::{error, info, warn};

use crate::ConnCtx;
use crate::room::state::RoomManager;

pub(crate) async fn handle(ctx: &mut ConnCtx, payload: &[u8], room_mgr: &mut RoomManager) {
    let msg: ResetGame = match postcard::from_bytes(payload) {
        Ok(v) => v,
        Err(e) => {
            warn!(error = %e, "Bad ResetGame message");
            return;
        }
    };

    let Some(room_id) = room_mgr.get_client_room(ctx.assigned_client_id) else {
        return;
    };
    let Some(room) = room_mgr.get_room_mut(room_id) else {
        return;
    };

    let recipients = room.handle_reset_game(ctx.assigned_client_id);
    if recipients.is_empty() {
        return;
    }

    info!(
        client_id = ctx.assigned_client_id,
        room_id,
        kind = msg.kind,
        "Broadcasting reset sync"
    );

    let sync_msg = ResetSync { kind: msg.kind };
    let mut h = Header::new(MsgId::ResetSync as u8);
    h.client_id = ctx.assigned_client_id;
    h.room_id = room_id;
    h.seq = 0;

    let frame = match encode_tcp_frame(h, MsgId::ResetSync, &sync_msg, 4096) {
        Ok(f) => Bytes::from(f),
        Err(e) => {
            error!("Failed to serialize ResetSync: {}", e);
            return;
        }
    };

    for tx in recipients {
        let frame = frame.clone();
        tokio::spawn(async move {
            let _ = tx.send(frame).await;
        });
    }
}
