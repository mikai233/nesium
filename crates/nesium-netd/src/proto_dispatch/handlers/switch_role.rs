//! SwitchRoleHandler - handles SwitchRole messages.

use nesium_netproto::{
    messages::session::{RoleChanged, SwitchRole},
    msg_id::MsgId,
};
use tracing::{info, warn};

use super::{Handler, HandlerContext};
use crate::net::outbound::send_msg;
use crate::proto_dispatch::error::{HandlerError, HandlerResult};

/// Handler for SwitchRole messages.
pub(crate) struct SwitchRoleHandler;

impl Handler<SwitchRole> for SwitchRoleHandler {
    async fn handle(&self, ctx: &mut HandlerContext<'_>, msg: SwitchRole) -> HandlerResult {
        let peer = *ctx.peer;
        let client_id = ctx.require_client_id()?;
        let room = ctx.require_room_mut()?;

        // Role switching during an active game can deadlock lockstep:
        // existing players will start waiting for inputs from the newly-promoted role,
        // but the switching client may still be catching up.
        if room.started {
            warn!(%peer, room_id = room.id, "Rejecting SwitchRole while game is running");
            return Err(HandlerError::game_already_started());
        }

        match room.switch_player_role(client_id, msg.new_role) {
            Ok(changes) => {
                let recipients = room.all_outbounds_msg(MsgId::RoleChanged);
                for (cid, role) in changes {
                    let broadcast = RoleChanged {
                        client_id: cid,
                        new_role: role,
                    };

                    for recipient in &recipients {
                        if let Err(e) = send_msg(recipient, &broadcast).await {
                            warn!(error = %e, "Failed to broadcast RoleChanged");
                        }
                    }

                    info!(
                        client_id = cid,
                        room_id = room.id,
                        new_role = role,
                        "Role changed"
                    );
                }
            }
            Err(e) => {
                warn!(%peer, error = %e, "Failed to switch role");
                return Err(HandlerError::invalid_state());
            }
        }
        Ok(())
    }
}
