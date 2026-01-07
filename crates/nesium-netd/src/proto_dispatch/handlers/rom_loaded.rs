use nesium_netproto::{
    constants::MAX_TCP_FRAME,
    header::Header,
    messages::session::{BeginCatchUp, StartGame, SyncState},
    msg_id::MsgId,
};
use tracing::{debug, info, warn};

use crate::ConnCtx;
use crate::room::broadcast::broadcast_inputs_required;
use crate::room::state::RoomManager;

pub(crate) async fn handle(ctx: &mut ConnCtx, room_mgr: &mut RoomManager) {
    let Some(room_id) = room_mgr.get_client_room(ctx.assigned_client_id) else {
        return;
    };
    let Some(room) = room_mgr.get_room_mut(room_id) else {
        return;
    };

    info!(
        client_id = ctx.assigned_client_id,
        room_id, "Client confirmed ROM loaded"
    );

    let sender_id = ctx.assigned_client_id;
    let was_started = room.started;
    let sender_was_loaded = room.loaded_players.contains(&sender_id);
    let sender_outbound = room.outbound_for_client(sender_id);

    let start_recipients = room.handle_rom_loaded(sender_id);
    if !start_recipients.is_empty() {
        info!(room_id, "All players loaded ROM, starting game");

        let msg = StartGame {
            active_ports_mask: room.get_active_ports_mask(),
        };
        let mut h = Header::new(MsgId::StartGame as u8);
        h.client_id = 0; // System message
        h.room_id = room_id;
        h.seq = 0;

        for recipient in &start_recipients {
            if let Err(e) =
                crate::net::outbound::send_msg_tcp(recipient, h, MsgId::StartGame, &msg, 4096).await
            {
                warn!(error = %e, "Failed to broadcast StartGame");
            }
        }
        return;
    }

    // Late joiner: room already started, but this client just finished loading the ROM.
    // Send them the latest cached state + input history, then BeginCatchUp to activate lockstep.
    if was_started && !sender_was_loaded {
        let Some(outbound) = sender_outbound else {
            return;
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
            let mut h = Header::new(MsgId::SyncState as u8);
            h.client_id = 0;
            h.room_id = room_id;
            h.seq = ctx.server_seq;
            ctx.server_seq = ctx.server_seq.wrapping_add(1);
            let _ = crate::net::outbound::send_msg_tcp(
                &outbound,
                h,
                MsgId::SyncState,
                &sync_state,
                MAX_TCP_FRAME,
            )
            .await;

            let history = room.get_input_history(frame);
            info!(
                client_id = sender_id,
                chunks = history.len(),
                "Sending input history to late joiner"
            );

            let recipients = vec![outbound.clone()];
            let mut seq = ctx.server_seq;
            for (p_idx, base, buttons) in history {
                broadcast_inputs_required(&recipients, p_idx, base, &buttons, room_id, &mut seq)
                    .await;
            }
            ctx.server_seq = seq;

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
            let mut h = Header::new(MsgId::BeginCatchUp as u8);
            h.client_id = 0;
            h.room_id = room_id;
            h.seq = ctx.server_seq;
            ctx.server_seq = ctx.server_seq.wrapping_add(1);
            let _ =
                crate::net::outbound::send_msg_tcp(&outbound, h, MsgId::BeginCatchUp, &msg, 4096)
                    .await;
        } else {
            debug!(
                client_id = sender_id,
                room_id, "No cached state available for late joiner"
            );
        }
    }
}
