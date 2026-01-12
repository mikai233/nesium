use nesium_netproto::{
    header::Header,
    messages::session::{BeginCatchUp, StartGame, SyncState},
    msg_id::MsgId,
};
use tracing::{debug, info, warn};

use crate::ConnCtx;
use crate::net::outbound::send_msg_tcp;
use crate::proto_dispatch::error::HandlerResult;
use crate::room::broadcast::broadcast_inputs_required;
use crate::room::state::RoomManager;

pub(crate) async fn handle(ctx: &mut ConnCtx, room_mgr: &mut RoomManager) -> HandlerResult {
    let Some(room_id) = room_mgr.get_client_room(ctx.assigned_client_id) else {
        // Not an error - just ignore if not in room
        return Ok(());
    };
    let Some(room) = room_mgr.get_room_mut(room_id) else {
        return Ok(());
    };

    info!(
        client_id = ctx.assigned_client_id,
        room_id, "Client confirmed ROM loaded"
    );

    let sender_id = ctx.assigned_client_id;
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
        let h = Header::new(MsgId::StartGame as u8);

        for recipient in &start_recipients {
            if let Err(e) = send_msg_tcp(recipient, h, MsgId::StartGame, &msg).await {
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

        if let Some((frame, state_data)) = room.cached_state.clone() {
            info!(
                client_id = sender_id,
                frame, "Sending cached state to late joiner"
            );
            let sync_state = SyncState {
                frame,
                data: state_data,
            };
            let h = Header::new(MsgId::SyncState as u8);
            let _ = send_msg_tcp(&state_outbound, h, MsgId::SyncState, &sync_state).await;

            let history = room.get_input_history(frame);
            info!(
                client_id = sender_id,
                chunks = history.len(),
                "Sending input history to late joiner"
            );

            let Some(input_outbound) = sender_input_outbound else {
                return Ok(());
            };
            let recipients = vec![input_outbound.clone()];
            for (p_idx, base, buttons) in history {
                broadcast_inputs_required(&recipients, p_idx, base, &buttons).await;
            }

            // After state + inputs are in flight, tell the joiner to begin catch-up.
            let mut active_ports_mask: u8 = 0;
            for idx in room.players.keys() {
                if *idx < 8 {
                    active_ports_mask |= 1u8 << *idx;
                }
            }
            let target_frame = room.current_frame.max(frame);
            let msg = BeginCatchUp {
                snapshot_frame: frame,
                target_frame,
                active_ports_mask,
            };
            let h = Header::new(MsgId::BeginCatchUp as u8);
            let _ = send_msg_tcp(&state_outbound, h, MsgId::BeginCatchUp, &msg).await;
        } else {
            debug!(
                client_id = sender_id,
                room_id, "No cached state available for late joiner"
            );
        }
    }
    Ok(())
}
