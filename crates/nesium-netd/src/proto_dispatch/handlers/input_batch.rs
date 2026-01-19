//! InputBatchHandler - handles InputBatch messages.

use nesium_netproto::messages::input::InputBatch;
use nesium_netproto::msg_id::MsgId;

use super::{Handler, HandlerContext};
use crate::proto_dispatch::error::{HandlerError, HandlerResult};
use crate::room::broadcast::{broadcast_inputs_optional, broadcast_inputs_required};

/// Handler for InputBatch messages.
pub(crate) struct InputBatchHandler;

impl Handler<InputBatch> for InputBatchHandler {
    async fn handle(&self, ctx: &mut HandlerContext<'_>, batch: InputBatch) -> HandlerResult {
        let client_id = ctx.require_client_id()?;
        let room = ctx.require_room_mut()?;

        let player_index = room
            .players
            .values()
            .find(|p| p.client_id == client_id)
            .map(|p| p.player_index);

        let Some(player_index) = player_index else {
            // Client is not a player (spectator?), cannot send inputs
            return Err(HandlerError::permission_denied());
        };

        room.record_inputs(player_index, batch.start_frame, &batch.buttons);

        // Lockstep recipients:
        // - Active ports are "required" (await backpressure)
        // - Inactive/rejoining ports and spectators are best-effort (must not stall the room)
        let mut required_recipients = Vec::new();
        let mut best_effort_recipients = Vec::new();

        for p in room.players.values() {
            let active = (p.player_index as usize) < room.active_ports.len()
                && room.active_ports[p.player_index as usize];
            let tx = p.outbounds.outbound_for_msg(MsgId::RelayInputs);
            if active {
                required_recipients.push(tx);
            } else {
                best_effort_recipients.push(tx);
            }
        }
        best_effort_recipients.extend(
            room.spectators
                .iter()
                .map(|s| s.outbounds.outbound_for_msg(MsgId::RelayInputs)),
        );

        broadcast_inputs_required(
            &required_recipients,
            player_index,
            batch.start_frame,
            &batch.buttons,
        )
        .await;
        broadcast_inputs_optional(
            &best_effort_recipients,
            player_index,
            batch.start_frame,
            &batch.buttons,
        );
        Ok(())
    }
}
