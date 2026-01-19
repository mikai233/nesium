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
        let client_id = ctx.require_client_id()?;
        let room = ctx.require_room_mut()?;

        match room.handle_load_rom(client_id) {
            Ok(recipients) => {
                // Forward ROM to others
                info!(
                    client_id,
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
