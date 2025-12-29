use core::ffi::c_void;
use std::path::PathBuf;

use crossbeam_channel::Sender;
use nesium_core::{
    audio::bus::AudioBusConfig,
    ppu::buffer::FrameReadyCallback,
    ppu::palette::{Palette, PaletteKind},
    reset_kind::ResetKind,
};

use super::types::RuntimeError;

pub(crate) type ControlReplySender = Sender<Result<(), RuntimeError>>;

pub(crate) enum ControlMessage {
    Stop,
    LoadRom(PathBuf, ControlReplySender),
    Reset(ResetKind, ControlReplySender),
    Eject(ControlReplySender),
    SetAudioConfig(AudioBusConfig, ControlReplySender),
    SetFrameReadyCallback(Option<FrameReadyCallback>, *mut c_void, ControlReplySender),
    SetPaletteKind(PaletteKind, ControlReplySender),
    SetPalette(Palette, ControlReplySender),
    /// None = exact NTSC FPS, Some(60) = integer FPS (PAL reserved for future).
    SetIntegerFpsTarget(Option<u32>, ControlReplySender),
    SaveState(PathBuf, ControlReplySender),
    LoadState(PathBuf, ControlReplySender),
    SaveStateToMemory(Sender<Result<Vec<u8>, RuntimeError>>),
    LoadStateFromMemory(Vec<u8>, ControlReplySender),
}

// SAFETY: raw pointers and function pointers are forwarded to the runtime thread without
// dereferencing on the sending thread; the receiver owns and uses them.
unsafe impl Send for ControlMessage {}
