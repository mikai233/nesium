//! RejoinReadyHandler - handles RejoinReady messages.

use nesium_netproto::{
    messages::session::{ActivatePort, RejoinReady},
    msg_id::MsgId,
};
use tracing::info;

use super::{Handler, HandlerContext};
use crate::net::outbound::send_msg_tcp;
use crate::proto_dispatch::error::{HandlerError, HandlerResult};

const ACTIVATION_LEAD_FRAMES: u32 = 8;

/// Handler for RejoinReady messages.
pub(crate) struct RejoinReadyHandler;

impl Handler<RejoinReady> for RejoinReadyHandler {
    async fn handle(&self, ctx: &mut HandlerContext<'_>, ready: RejoinReady) -> HandlerResult {
        let Some(room) = ctx
            .room_mgr
            .client_room_mut(ctx.conn_ctx.assigned_client_id)
        else {
            return Err(HandlerError::not_in_room());
        };

        if !room.started {
            return Ok(());
        }

        let player_index = room
            .players
            .values()
            .find(|p| p.client_id == ctx.conn_ctx.assigned_client_id)
            .map(|p| p.player_index)
            .ok_or_else(HandlerError::permission_denied)?;

        if (player_index as usize) >= room.active_ports.len() {
            return Ok(());
        }
        if room.active_ports[player_index as usize] {
            return Ok(());
        }

        let active_from_frame = room.current_frame.saturating_add(ACTIVATION_LEAD_FRAMES);
        room.schedule_port_activation(player_index, active_from_frame);

        let msg = ActivatePort {
            player_index,
            active_from_frame,
        };

        for tx in room.all_outbounds_msg(MsgId::ActivatePort) {
            let _ = send_msg_tcp(&tx, &msg).await;
        }

        info!(
            room_id = room.id,
            client_id = ctx.conn_ctx.assigned_client_id,
            player_index,
            caught_up_to_frame = ready.caught_up_to_frame,
            active_from_frame,
            "Scheduled port activation"
        );

        Ok(())
    }
}
