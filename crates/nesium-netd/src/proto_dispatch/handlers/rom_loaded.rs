//! RomLoadedHandler - handles RomLoaded messages.

use nesium_netproto::messages::session::{
    BeginCatchUp, RequestState, RomLoaded, StartGame, SyncState,
};
use nesium_netproto::msg_id::MsgId;
use tracing::{debug, info, warn};

use super::{Handler, HandlerContext};
use crate::net::outbound::send_msg_tcp;
use crate::proto_dispatch::error::HandlerResult;
use crate::room::broadcast::broadcast_inputs_required;

/// Handler for RomLoaded messages.
pub(crate) struct RomLoadedHandler;

impl Handler<RomLoaded> for RomLoadedHandler {
    async fn handle(&self, ctx: &mut HandlerContext<'_>, _msg: RomLoaded) -> HandlerResult {
        let Some(room_id) = ctx
            .room_mgr
            .get_client_room(ctx.conn_ctx.assigned_client_id)
        else {
            // Not an error - just ignore if not in room
            return Ok(());
        };
        let Some(room) = ctx.room_mgr.get_room_mut(room_id) else {
            return Ok(());
        };

        info!(
            client_id = ctx.conn_ctx.assigned_client_id,
            room_id, "Client confirmed ROM loaded"
        );

        let sender_id = ctx.conn_ctx.assigned_client_id;
        let was_started = room.started;
        let sender_was_loaded = room.loaded_players.contains(&sender_id);
        let sender_state_outbound = room.outbound_for_client_msg(sender_id, MsgId::SyncState);
        let sender_input_outbound = room.outbound_for_client_msg(sender_id, MsgId::RelayInputs);

        let start_recipients = room.handle_rom_loaded(sender_id);
        if !start_recipients.is_empty() {
            info!(room_id, "All players loaded ROM, starting game");

            let msg = StartGame {
                active_ports_mask: room.get_active_ports_mask(),
            };

            for recipient in &start_recipients {
                if let Err(e) = send_msg_tcp(recipient, &msg).await {
                    warn!(error = %e, "Failed to broadcast StartGame");
                }
            }
            return Ok(());
        }

        // Late joiner: room already started, but this client just finished loading the ROM.
        // Send them the latest cached state + input history, then BeginCatchUp to activate lockstep.
        if was_started && !sender_was_loaded {
            let Some(state_outbound) = sender_state_outbound else {
                return Ok(());
            };
            let begin_outbound = room
                .outbound_for_client_msg(sender_id, MsgId::BeginCatchUp)
                .unwrap_or_else(|| state_outbound.clone());

            if let Some((frame, state_data)) = room.cached_state.clone() {
                info!(
                    client_id = sender_id,
                    frame, "Sending cached state to late joiner"
                );
                let sync_state = SyncState {
                    frame,
                    data: state_data,
                };
                let _ = send_msg_tcp(&state_outbound, &sync_state).await;

                let history = room.get_input_history(frame);
                let Some(input_outbound) = sender_input_outbound else {
                    return Ok(());
                };
                let recipients = vec![input_outbound.clone()];
                for (p_idx, base, buttons) in history {
                    broadcast_inputs_required(&recipients, p_idx, base, &buttons).await;
                }

                let active_ports_mask = room.get_active_ports_mask();
                let target_frame = room.current_frame.max(frame);
                let msg = BeginCatchUp {
                    snapshot_frame: frame,
                    target_frame,
                    active_ports_mask,
                };
                let _ = send_msg_tcp(&begin_outbound, &msg).await;
            } else {
                // No cached state yet: request a fresh snapshot from host and defer catch-up.
                let host_id = room.host_client_id;
                if let Some(host_outbound) =
                    room.outbound_for_client_msg(host_id, MsgId::RequestState)
                {
                    if !room.pending_catch_up_clients.contains(&sender_id) {
                        room.pending_catch_up_clients.push(sender_id);
                    }
                    let msg = RequestState {};
                    if let Err(e) = send_msg_tcp(&host_outbound, &msg).await {
                        warn!(error = %e, "Failed to request fresh state from host");
                        room.pending_catch_up_clients.retain(|&id| id != sender_id);
                    } else {
                        info!(
                            client_id = sender_id,
                            host_client_id = host_id,
                            room_id,
                            "Requested fresh state from host for late joiner"
                        );
                    }
                } else {
                    debug!(
                        client_id = sender_id,
                        room_id, "No cached state available for late joiner"
                    );
                }
            }
        }
        Ok(())
    }
}
