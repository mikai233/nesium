use flutter_rust_bridge::frb;
use std::sync::atomic::{AtomicU8, Ordering};

use crate::runtime_handle;

const MAX_PORTS: usize = 2;

static KEYBOARD_PAD_MASKS: [AtomicU8; MAX_PORTS] = [AtomicU8::new(0), AtomicU8::new(0)];
static KEYBOARD_TURBO_MASKS: [AtomicU8; MAX_PORTS] = [AtomicU8::new(0), AtomicU8::new(0)];

static GAMEPAD_PAD_MASKS: [AtomicU8; MAX_PORTS] = [AtomicU8::new(0), AtomicU8::new(0)];
static GAMEPAD_TURBO_MASKS: [AtomicU8; MAX_PORTS] = [AtomicU8::new(0), AtomicU8::new(0)];

pub(crate) fn update_runtime_input(pad: usize) {
    if pad >= MAX_PORTS {
        return;
    }

    let k_pad = KEYBOARD_PAD_MASKS[pad].load(Ordering::Acquire);
    let g_pad = GAMEPAD_PAD_MASKS[pad].load(Ordering::Acquire);
    runtime_handle().set_pad_mask(pad, k_pad | g_pad);

    let k_turbo = KEYBOARD_TURBO_MASKS[pad].load(Ordering::Acquire);
    let g_turbo = GAMEPAD_TURBO_MASKS[pad].load(Ordering::Acquire);
    runtime_handle().set_turbo_mask(pad, k_turbo | g_turbo);
}

pub(crate) fn set_gamepad_masks(pad: usize, pad_mask: u8, turbo_mask: u8) {
    if pad < MAX_PORTS {
        GAMEPAD_PAD_MASKS[pad].store(pad_mask, Ordering::Release);
        GAMEPAD_TURBO_MASKS[pad].store(turbo_mask, Ordering::Release);
        update_runtime_input(pad);
    }
}

#[frb]
pub fn set_pad_mask(pad: u8, mask: u8) -> Result<(), String> {
    let pad = pad as usize;
    if pad < MAX_PORTS {
        KEYBOARD_PAD_MASKS[pad].store(mask, Ordering::Release);
        update_runtime_input(pad);
    }
    Ok(())
}

#[frb]
pub fn set_turbo_mask(pad: u8, mask: u8) -> Result<(), String> {
    let pad = pad as usize;
    if pad < MAX_PORTS {
        KEYBOARD_TURBO_MASKS[pad].store(mask, Ordering::Release);
        update_runtime_input(pad);
    }
    Ok(())
}

#[frb]
pub fn set_turbo_frames_per_toggle(frames: u8) -> Result<(), String> {
    runtime_handle().set_turbo_timing(frames, frames);
    Ok(())
}

#[frb]
pub fn set_turbo_timing(on_frames: u8, off_frames: u8) -> Result<(), String> {
    runtime_handle().set_turbo_timing(on_frames, off_frames);
    Ok(())
}
