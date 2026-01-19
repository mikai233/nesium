//! ProvideStateHandler - handles ProvideState messages.

use nesium_netproto::{
    messages::session::{BeginCatchUp, ProvideState, SyncState},
    msg_id::MsgId,
};
use tracing::{debug, info};

use super::{Handler, HandlerContext};
use crate::net::outbound::send_msg_tcp;
use crate::proto_dispatch::error::HandlerResult;
use crate::room::broadcast::broadcast_inputs_required;

/// Handler for ProvideState messages.
pub(crate) struct ProvideStateHandler;

impl Handler<ProvideState> for ProvideStateHandler {
    async fn handle(&self, ctx: &mut HandlerContext<'_>, msg: ProvideState) -> HandlerResult {
        let Some(room) = ctx
            .room_mgr
            .client_room_mut(ctx.conn_ctx.assigned_client_id)
        else {
            return Ok(()); // Not an error - host might not be in room yet
        };

        room.cache_state(msg.frame, msg.data);
        debug!(room_id = room.id, frame = msg.frame, "Cached game state");

        // Late joiners waiting for a fresh state: send SyncState + input history + BeginCatchUp now.
        if room.started && !room.pending_catch_up_clients.is_empty() {
            let Some((frame, state_data)) = room.cached_state.clone() else {
                return Ok(());
            };

            let pending = std::mem::take(&mut room.pending_catch_up_clients);
            let history = room.get_input_history(frame);
            let target_frame = room.current_frame.max(frame);
            let active_ports_mask = room.get_active_ports_mask();

            info!(
                room_id = room.id,
                frame,
                target_frame,
                recipients = pending.len(),
                "Delivering fresh state to pending late joiners"
            );

            for client_id in pending {
                let Some(state_outbound) =
                    room.outbound_for_client_msg(client_id, MsgId::SyncState)
                else {
                    continue;
                };
                let begin_outbound = room
                    .outbound_for_client_msg(client_id, MsgId::BeginCatchUp)
                    .unwrap_or_else(|| state_outbound.clone());

                let sync_state = SyncState {
                    frame,
                    data: state_data.clone(),
                };
                let _ = send_msg_tcp(&state_outbound, &sync_state).await;

                if let Some(input_outbound) =
                    room.outbound_for_client_msg(client_id, MsgId::RelayInputs)
                {
                    let recipients = vec![input_outbound];
                    for (p_idx, base, buttons) in &history {
                        broadcast_inputs_required(&recipients, *p_idx, *base, buttons).await;
                    }
                }

                let msg = BeginCatchUp {
                    snapshot_frame: frame,
                    target_frame,
                    active_ports_mask,
                };
                let _ = send_msg_tcp(&begin_outbound, &msg).await;
            }
        }
        Ok(())
    }
}
