//! LoadRomHandler - handles LoadRom messages.

use nesium_netproto::messages::session::LoadRom;
use tracing::{info, warn};

use super::{Handler, HandlerContext};
use crate::net::outbound::send_msg;
use crate::proto_dispatch::error::{HandlerError, HandlerResult};

/// Handler for LoadRom messages.
pub(crate) struct LoadRomHandler;

impl Handler<LoadRom> for LoadRomHandler {
    async fn handle(&self, ctx: &mut HandlerContext<'_>, msg: LoadRom) -> HandlerResult {
        let Some(room) = ctx
            .room_mgr
            .client_room_mut(ctx.conn_ctx.assigned_client_id)
        else {
            warn!(%ctx.peer, "LoadRom: client not in a room");
            return Err(HandlerError::not_in_room());
        };

        match room.handle_load_rom(ctx.conn_ctx.assigned_client_id) {
            Ok(recipients) => {
                // Forward ROM to others
                info!(
                    client_id = ctx.conn_ctx.assigned_client_id,
                    room_id = room.id,
                    "Host loaded ROM, forwarding..."
                );

                room.cache_rom(msg.data.clone());

                for recipient in &recipients {
                    if let Err(e) = send_msg(recipient, &msg).await {
                        warn!(error = %e, "Failed to forward LoadRom");
                    }
                }
                Ok(())
            }
            Err(e) => {
                warn!(%ctx.peer, error = %e, "LoadRom rejected");
                Err(HandlerError::permission_denied())
            }
        }
    }
}
