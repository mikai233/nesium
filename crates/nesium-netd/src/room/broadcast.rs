//! Input broadcast utilities.
//!
//! Broadcasts input data to room participants.

use bytes::Bytes;
use nesium_netproto::codec::encode_message;
use nesium_netproto::messages::input::RelayInputs;
use tracing::error;

use crate::net::outbound::OutboundTx;

// Conservative chunking to guarantee RelayInputs fits within the control/input payload limit (4KB).
// This prevents late-join/reconnect catch-up from stalling when input history is large.
const RELAY_INPUTS_MAX_BUTTONS: usize = 512;

fn build_relay_inputs_frame(player_index: u8, start_frame: u32, buttons: &[u16]) -> Option<Bytes> {
    let relay = RelayInputs {
        player_index,
        base_frame: start_frame,
        buttons: buttons.to_vec(),
    };

    match encode_message(&relay) {
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
) {
    // Split into RELAY_INPUTS_MAX_BUTTONS-button chunks -- avoids hitting the 4KB limit.
    for (chunk_idx, chunk) in buttons.chunks(RELAY_INPUTS_MAX_BUTTONS).enumerate() {
        let chunk_start = start_frame + (chunk_idx * RELAY_INPUTS_MAX_BUTTONS) as u32;
        if let Some(frame) = build_relay_inputs_frame(player_index, chunk_start, chunk) {
            for tx in recipients {
                // Best-effort send (if channel is full, try_send would drop).
                // Using `send(..).await` applies backpressure.
                let _ = tx.send(frame.clone()).await;
            }
        }
    }
}

/// Broadcast input to optional recipients (spectators).
///
/// Non-blocking; drops if queues are full.
pub fn broadcast_inputs_optional(
    recipients: &[OutboundTx],
    player_index: u8,
    start_frame: u32,
    buttons: &[u16],
) {
    for (chunk_idx, chunk) in buttons.chunks(RELAY_INPUTS_MAX_BUTTONS).enumerate() {
        let chunk_start = start_frame + (chunk_idx * RELAY_INPUTS_MAX_BUTTONS) as u32;
        if let Some(frame) = build_relay_inputs_frame(player_index, chunk_start, chunk) {
            for tx in recipients {
                let _ = tx.try_send(frame.clone());
            }
        }
    }
}

/// Broadcast already-encoded relay frames.
#[allow(dead_code)]
pub async fn broadcast_relay_frames(recipients: &[OutboundTx], frames: &[Bytes]) {
    for frame in frames {
        for tx in recipients {
            let _ = tx.send(frame.clone()).await;
        }
    }
}

/// Send a single relay frame to one recipient.
#[allow(dead_code)]
pub async fn send_relay_frame(tx: &OutboundTx, frame: Bytes) {
    let _ = tx.send(frame).await;
}

/// Send input history to a single recipient (e.g., late joiner or reconnect).
#[allow(dead_code)]
pub async fn send_input_history(
    tx: &OutboundTx,
    player_index: u8,
    start_frame: u32,
    buttons: &[u16],
) {
    for (chunk_idx, chunk) in buttons.chunks(RELAY_INPUTS_MAX_BUTTONS).enumerate() {
        let chunk_start = start_frame + (chunk_idx * RELAY_INPUTS_MAX_BUTTONS) as u32;
        if let Some(frame) = build_relay_inputs_frame(player_index, chunk_start, chunk) {
            let _ = tx.send(frame).await;
        }
    }
}
