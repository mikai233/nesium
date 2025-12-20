//! nesium-flutter
//!
//! Flutter bridge for the shared `nesium-runtime` backend.
//! - Flutter (via FRB) issues control commands (load/reset/input).
//! - The runtime owns a dedicated NES thread that renders frames into a
//!   double-buffered external BGRA8888 framebuffer.
//! - The macOS runner registers a frame-ready callback and copies the
//!   latest buffer into a CVPixelBuffer.

pub mod api;
mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */

use std::{
    os::raw::{c_uint, c_void},
    sync::{Arc, OnceLock},
};

use nesium_core::{
    controller::Button as CoreButton,
    ppu::{
        SCREEN_HEIGHT, SCREEN_WIDTH,
        buffer::{ColorFormat, ExternalFrameHandle},
    },
};
use nesium_runtime::{
    AudioMode, FrameReadyCallback, Runtime, RuntimeConfig, RuntimeHandle, VideoConfig,
};

pub const FRAME_WIDTH: usize = SCREEN_WIDTH;
pub const FRAME_HEIGHT: usize = SCREEN_HEIGHT;
pub const BYTES_PER_PIXEL: usize = 4; // BGRA8888

struct VideoBackingStore {
    _plane0: Box<[u8]>,
    _plane1: Box<[u8]>,
}

struct RuntimeHolder {
    _video: VideoBackingStore,
    handle: RuntimeHandle,
    frame_handle: Arc<ExternalFrameHandle>,
    _runtime: Runtime,
}

static RUNTIME: OnceLock<RuntimeHolder> = OnceLock::new();

fn ensure_runtime() -> &'static RuntimeHolder {
    RUNTIME.get_or_init(|| {
        let len = FRAME_WIDTH * FRAME_HEIGHT * BYTES_PER_PIXEL;
        let plane0 = vec![0u8; len].into_boxed_slice();
        let plane1 = vec![0u8; len].into_boxed_slice();

        let mut video = VideoBackingStore {
            _plane0: plane0,
            _plane1: plane1,
        };

        // SAFETY: `video` keeps the two planes alive for the lifetime of the process.
        // The planes do not overlap and are sized to the NES framebuffer.
        let runtime = Runtime::start(RuntimeConfig {
            video: VideoConfig {
                color_format: ColorFormat::Bgra8888,
                plane0: video._plane0.as_mut_ptr(),
                plane1: video._plane1.as_mut_ptr(),
            },
            audio: AudioMode::Auto,
        })
        .expect("failed to start nesium runtime");

        let handle = runtime.handle();
        let frame_handle = handle.frame_handle();

        RuntimeHolder {
            _video: video,
            handle,
            frame_handle,
            _runtime: runtime,
        }
    })
}

pub(crate) fn runtime_handle() -> RuntimeHandle {
    ensure_runtime().handle.clone()
}

fn frame_handle_ref() -> &'static ExternalFrameHandle {
    ensure_runtime().frame_handle.as_ref()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PadButton {
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}

impl From<PadButton> for CoreButton {
    fn from(value: PadButton) -> Self {
        match value {
            PadButton::A => CoreButton::A,
            PadButton::B => CoreButton::B,
            PadButton::Select => CoreButton::Select,
            PadButton::Start => CoreButton::Start,
            PadButton::Up => CoreButton::Up,
            PadButton::Down => CoreButton::Down,
            PadButton::Left => CoreButton::Left,
            PadButton::Right => CoreButton::Right,
        }
    }
}

// === C ABI exposed to Swift/macOS =========================================

#[unsafe(no_mangle)]
pub extern "C" fn nesium_runtime_start() {
    let _ = ensure_runtime();
}

#[unsafe(no_mangle)]
pub extern "C" fn nesium_set_frame_ready_callback(
    cb: Option<FrameReadyCallback>,
    user_data: *mut c_void,
) {
    let handle = runtime_handle();
    let _ = handle.set_frame_ready_callback(cb, user_data);
}

/// Copy the current NES frame into a BGRA8888 destination buffer.
///
/// # Safety
/// - `dst` must be null or point to at least `dst_pitch * dst_height` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nesium_copy_frame(
    _buffer_index: c_uint,
    dst: *mut u8,
    dst_pitch: c_uint,
    dst_height: c_uint,
) {
    if dst.is_null() {
        return;
    }

    let frame_handle = frame_handle_ref();
    let idx = frame_handle.begin_front_copy();
    let src_slice = frame_handle.plane_slice(idx);

    let height = FRAME_HEIGHT.min(dst_height as usize);
    let src_pitch = FRAME_WIDTH * BYTES_PER_PIXEL;
    let dst_pitch = dst_pitch as usize;

    let dst_slice = unsafe {
        std::slice::from_raw_parts_mut(
            dst,
            dst_pitch
                .saturating_mul(dst_height as usize)
                .min(src_pitch * FRAME_HEIGHT),
        )
    };

    for y in 0..height {
        let src_off = y * src_pitch;
        let dst_off = y * dst_pitch;
        let src_row = &src_slice[src_off..src_off + src_pitch];
        let dst_row = &mut dst_slice[dst_off..dst_off + src_pitch.min(dst_pitch)];
        dst_row.copy_from_slice(&src_row[..dst_row.len()]);
    }

    frame_handle.end_front_copy();
}
