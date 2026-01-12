use std::net::SocketAddr;
use std::sync::atomic::Ordering;

use nesium_netproto::{
    header::Header,
    messages::session::{Hello, Welcome},
    msg_id::MsgId,
};
use tracing::{error, info, warn};

use crate::proto_dispatch::error::{HandlerError, HandlerResult};
use crate::{
    ConnCtx, NEXT_CLIENT_ID, NEXT_SERVER_NONCE, NEXT_SESSION_TOKEN, net::outbound::send_msg_tcp,
};

pub(crate) async fn handle(ctx: &mut ConnCtx, peer: &SocketAddr, payload: &[u8]) -> HandlerResult {
    let hello: Hello = match postcard::from_bytes(payload) {
        Ok(v) => v,
        Err(e) => {
            warn!(%peer, error = %e, "Bad Hello message");
            return Err(HandlerError::bad_message());
        }
    };

    if ctx.assigned_client_id == 0 {
        ctx.assigned_client_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);
        ctx.name = hello.name.clone();
    }

    ctx.role = crate::ConnRole::Control;

    let server_nonce = NEXT_SERVER_NONCE.fetch_add(1, Ordering::Relaxed);
    // If the client retries Hello on the same connection, keep the existing token.
    if ctx.session_token == 0 {
        let counter = NEXT_SESSION_TOKEN.fetch_add(1, Ordering::Relaxed);
        let rand_low: u32 = rand::random();
        ctx.session_token = (counter << 32) | (rand_low as u64);
    }

    let welcome = Welcome {
        server_nonce,
        session_token: ctx.session_token,
        assigned_client_id: ctx.assigned_client_id,
        room_id: 0,
        tick_hz: 60,
        input_delay_frames: 2,
        max_payload: 4096,
        rewind_capacity: 600,
    };

    let h = Header::new(MsgId::Welcome as u8);

    match send_msg_tcp(&ctx.outbound, h, MsgId::Welcome, &welcome).await {
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
    Ok(())
}
