use std::net::SocketAddr;
use std::sync::atomic::Ordering;

use nesium_netproto::{
    header::Header,
    messages::session::{Hello, Welcome},
    msg_id::MsgId,
};
use tracing::{error, info, warn};

use crate::{ConnCtx, NEXT_CLIENT_ID, NEXT_SERVER_NONCE, net::outbound::send_msg_tcp};

pub(crate) async fn handle(ctx: &mut ConnCtx, peer: &SocketAddr, payload: &[u8]) {
    let hello: Hello = match postcard::from_bytes(payload) {
        Ok(v) => v,
        Err(e) => {
            warn!(%peer, error = %e, "Bad Hello message");
            return;
        }
    };

    if ctx.assigned_client_id == 0 {
        ctx.assigned_client_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);
        ctx.rom_hash = hello.rom_hash;
        ctx.name = hello.name.clone();
    }

    let welcome = Welcome {
        server_nonce: NEXT_SERVER_NONCE.fetch_add(1, Ordering::Relaxed),
        assigned_client_id: ctx.assigned_client_id,
        room_id: 0,
        tick_hz: 60,
        input_delay_frames: 2,
        max_payload: 4096,
        rewind_capacity: 600,
    };

    let mut h = Header::new(MsgId::Welcome as u8);
    h.client_id = ctx.assigned_client_id;
    h.room_id = 0;
    h.seq = ctx.server_seq;
    ctx.server_seq = ctx.server_seq.wrapping_add(1);

    match send_msg_tcp(&ctx.outbound, h, MsgId::Welcome, &welcome, 4096).await {
        Ok(()) => {
            info!(
                client_id = ctx.assigned_client_id,
                name = %hello.name,
                "Hello/Welcome handshake completed"
            );
        }
        Err(e) => {
            error!(%peer, error = %e, "Failed to send Welcome");
        }
    }
}
