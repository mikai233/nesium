use core::ffi::c_void;
use std::path::PathBuf;

use crossbeam_channel::{Receiver, Sender};
use nesium_core::{
    audio::bus::AudioBusConfig,
    interceptor::{
        palette_interceptor::CapturePoint as PaletteCapturePoint,
        sprite_interceptor::CapturePoint as SpriteCapturePoint,
        tile_viewer_interceptor::CapturePoint as TileViewerCapturePoint,
        tilemap_interceptor::CapturePoint as TilemapCapturePoint,
    },
    ppu::buffer::{ColorFormat, FrameReadyCallback},
    ppu::palette::{Palette, PaletteKind},
    reset_kind::ResetKind,
};

use super::debug::{DebugCommand, DebugEvent};
use super::types::{
    EventTopic, RuntimeError, RuntimeEventSender, TileViewerBackground, TileViewerLayout,
    TileViewerSource,
};

pub(crate) type ControlReplySender = Sender<Result<(), RuntimeError>>;

pub(crate) enum ControlMessage {
    Stop,
    LoadRom(PathBuf, ControlReplySender),
    LoadRomFromMemory(Vec<u8>, ControlReplySender),
    Reset(ResetKind, ControlReplySender),
    PowerOff(ControlReplySender),
    SetAudioConfig(AudioBusConfig, ControlReplySender),
    SetFrameReadyCallback(Option<FrameReadyCallback>, *mut c_void, ControlReplySender),
    SetColorFormat(ColorFormat, ControlReplySender),
    SetPaletteKind(PaletteKind, ControlReplySender),
    SetPalette(Palette, ControlReplySender),
    /// None = exact NTSC FPS, Some(60) = integer FPS (PAL reserved for future).
    SetIntegerFpsTarget(Option<u32>, ControlReplySender),
    SaveState(PathBuf, ControlReplySender),
    LoadState(PathBuf, ControlReplySender),
    SaveStateToMemory(Sender<Result<Vec<u8>, RuntimeError>>),
    LoadStateFromMemory(Vec<u8>, ControlReplySender),
    SetRewinding(bool, ControlReplySender),
    SetFastForwarding(bool, ControlReplySender),
    SetFastForwardSpeed(u16, ControlReplySender),
    SetRewindSpeed(u16, ControlReplySender),
    LoadMovie(nesium_support::tas::Movie, ControlReplySender),
    SubscribeEvent(EventTopic, Box<dyn RuntimeEventSender>, ControlReplySender),
    UnsubscribeEvent(EventTopic, ControlReplySender),
    // Per-viewer capture points
    SetTilemapCapturePoint(TilemapCapturePoint, ControlReplySender),
    SetTileViewerCapturePoint(TileViewerCapturePoint, ControlReplySender),
    SetSpriteCapturePoint(SpriteCapturePoint, ControlReplySender),
    // Tile viewer settings
    SetTileViewerSource(TileViewerSource, ControlReplySender),
    SetTileViewerStartAddress(u32, ControlReplySender),
    SetTileViewerSize {
        columns: u16,
        rows: u16,
        reply: ControlReplySender,
    },
    SetTileViewerLayout(TileViewerLayout, ControlReplySender),
    SetTileViewerBackground(TileViewerBackground, ControlReplySender),
    SetTileViewerPalette(u8, ControlReplySender),
    SetTileViewerUseGrayscalePalette(bool, ControlReplySender),
    // Palette viewer settings
    SetPaletteCapturePoint(PaletteCapturePoint, ControlReplySender),
    // Debugger control
    /// Enable the debugger with the given channels.
    EnableDebugger {
        debug_rx: Receiver<DebugCommand>,
        debug_tx: Sender<DebugEvent>,
        reply: ControlReplySender,
    },
    /// Disable and remove the debugger.
    DisableDebugger(ControlReplySender),

    // Netplay control
    /// Enable netplay with the given input provider.
    EnableNetplay {
        input_provider: std::sync::Arc<dyn nesium_netplay::NetplayInputProvider>,
        reply: ControlReplySender,
    },
    /// Disable netplay and return to local input.
    DisableNetplay(ControlReplySender),
}

// SAFETY: raw pointers and function pointers are forwarded to the runtime thread without
// dereferencing on the sending thread; the receiver owns and uses them.
unsafe impl Send for ControlMessage {}
