//! Input broadcast utilities.
//!
//! Broadcasts input data to room participants.

use bytes::Bytes;
use nesium_netproto::{
    codec_tcp::encode_tcp_frame, header::Header, messages::input::RelayInputs, msg_id::MsgId,
};
use tracing::error;

use crate::net::outbound::OutboundTx;

fn build_relay_inputs_frame(
    player_index: u8,
    start_frame: u32,
    buttons: &[u16],
    room_id: u32,
    seq: u32,
) -> Option<Bytes> {
    let relay = RelayInputs {
        player_index,
        base_frame: start_frame,
        buttons: buttons.to_vec(),
    };

    let mut header = Header::new(MsgId::RelayInputs as u8);
    header.room_id = room_id;
    header.seq = seq;

    match encode_tcp_frame(header, MsgId::RelayInputs, &relay, 4096) {
        Ok(f) => Some(Bytes::from(f)),
        Err(e) => {
            error!("Failed to encode RelayInputs: {}", e);
            None
        }
    }
}

/// Broadcast input to "required" recipients (players).
///
/// This awaits backpressure; lockstep correctness depends on reliable delivery.
pub async fn broadcast_inputs_required(
    recipients: &[OutboundTx],
    player_index: u8,
    start_frame: u32,
    buttons: &[u16],
    room_id: u32,
    server_seq: &mut u32,
) {
    let seq = *server_seq;
    *server_seq = server_seq.wrapping_add(1);

    let Some(frame) = build_relay_inputs_frame(player_index, start_frame, buttons, room_id, seq)
    else {
        return;
    };

    for tx in recipients {
        let _ = tx.send(frame.clone()).await;
    }
}

/// Broadcast input to "best-effort" recipients (spectators).
///
/// This never awaits; a slow spectator must not stall the entire room.
pub fn broadcast_inputs_best_effort(
    recipients: &[OutboundTx],
    player_index: u8,
    start_frame: u32,
    buttons: &[u16],
    room_id: u32,
    server_seq: &mut u32,
) {
    let seq = *server_seq;
    *server_seq = server_seq.wrapping_add(1);

    let Some(frame) = build_relay_inputs_frame(player_index, start_frame, buttons, room_id, seq)
    else {
        return;
    };

    for tx in recipients {
        let _ = tx.try_send(frame.clone());
    }
}
