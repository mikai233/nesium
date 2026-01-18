//! HelloHandler - handles Hello messages.

use std::sync::atomic::Ordering;

use nesium_netproto::messages::session::{Hello, Welcome};
use tracing::{error, info};

use super::{Handler, HandlerContext};
use crate::net::outbound::send_msg_tcp;
use crate::proto_dispatch::error::HandlerResult;
use crate::{NEXT_CLIENT_ID, NEXT_SERVER_NONCE, NEXT_SESSION_TOKEN};

/// Handler for Hello messages.
pub(crate) struct HelloHandler;

impl Handler<Hello> for HelloHandler {
    async fn handle(&self, ctx: &mut HandlerContext<'_>, msg: Hello) -> HandlerResult {
        if ctx.conn_ctx.assigned_client_id == 0 {
            ctx.conn_ctx.assigned_client_id = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);
            ctx.conn_ctx.name = msg.name.clone();
        }

        ctx.conn_ctx.role = crate::ConnRole::Control;

        let server_nonce = NEXT_SERVER_NONCE.fetch_add(1, Ordering::Relaxed);
        // If the client retries Hello on the same connection, keep the existing token.
        if ctx.conn_ctx.session_token == 0 {
            let counter = NEXT_SESSION_TOKEN.fetch_add(1, Ordering::Relaxed);
            let rand_low: u32 = rand::random();
            ctx.conn_ctx.session_token = (counter << 32) | (rand_low as u64);
        }

        let welcome = Welcome {
            server_nonce,
            session_token: ctx.conn_ctx.session_token,
            assigned_client_id: ctx.conn_ctx.assigned_client_id,
            room_id: 0,
            tick_hz: 60,
            input_delay_frames: 2,
            max_payload: 4096,
            rewind_capacity: 600,
        };

        match send_msg_tcp(&ctx.conn_ctx.outbound, &welcome).await {
            Ok(()) => {
                info!(
                    client_id = ctx.conn_ctx.assigned_client_id,
                    name = %msg.name,
                    "Hello/Welcome handshake completed"
                );
            }
            Err(e) => {
                error!(%ctx.peer, error = %e, "Failed to send Welcome");
            }
        }
        Ok(())
    }
}
