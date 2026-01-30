//! nesium-flutter
//!
//! Flutter bridge for the shared `nesium-runtime` backend.
//! - Flutter (via FRB) issues control commands (load/reset/input).
//! - The runtime owns a dedicated NES thread that renders frames into a
//!   double-buffered external 32-bit framebuffer (BGRA/RGBA depending on platform).
//! - The macOS runner registers a frame-ready callback and copies the
//!   latest buffer into a CVPixelBuffer.

#[cfg(target_os = "android")]
mod android;
pub mod api;
#[cfg(any(target_os = "macos", target_os = "ios"))]
mod apple;
pub mod aux_texture;
pub mod event_worker;
mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */
mod senders;
#[cfg(any(
    target_os = "android",
    target_os = "windows",
    any(target_os = "macos", target_os = "ios")
))]
mod shader_utils;
#[cfg(target_os = "windows")]
pub mod windows;

#[cfg(all(
    feature = "mimalloc",
    any(target_os = "android", target_os = "windows", target_os = "linux")
))]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::{
    os::raw::{c_uint, c_void},
    sync::{Arc, OnceLock},
};

use nesium_core::{
    controller::Button as CoreButton,
    ppu::{
        SCREEN_HEIGHT, SCREEN_WIDTH,
        buffer::{ColorFormat, ExternalFrameHandle, FrameReadyCallback},
    },
};
use nesium_runtime::{
    AudioMode, Runtime, RuntimeConfig, RuntimeHandle, VideoBackendConfig, VideoConfig,
};

pub const FRAME_WIDTH: usize = SCREEN_WIDTH;
pub const FRAME_HEIGHT: usize = SCREEN_HEIGHT;

#[cfg(target_os = "android")]
pub enum VideoBacking {
    Upload,
    Ahb(Arc<android::AhbSwapchain>),
}

struct RuntimeHolder {
    handle: RuntimeHandle,
    frame_handle: Option<Arc<ExternalFrameHandle>>,
    _runtime: Runtime,
    #[cfg(target_os = "android")]
    _video: VideoBacking,
}

static RUNTIME: OnceLock<RuntimeHolder> = OnceLock::new();

/// Returns the platform-specific pixel format for Flutter textures.
///
/// - macOS/iOS: BGRA (CVPixelBuffer)
/// - Android/Windows/Linux: RGBA (default - can be overridden at runtime on Windows)
#[inline]
pub fn platform_color_format() -> ColorFormat {
    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        ColorFormat::Bgra8888
    }
    #[cfg(not(any(target_os = "macos", target_os = "ios")))]
    {
        ColorFormat::Rgba8888
    }
}

fn ensure_runtime() -> &'static RuntimeHolder {
    RUNTIME.get_or_init(|| {
        let color_format = platform_color_format();
        let video_cfg = VideoConfig {
            color_format,
            output_width: FRAME_WIDTH as u32,
            output_height: FRAME_HEIGHT as u32,
            backend: VideoBackendConfig::Owned,
        };

        #[cfg(target_os = "android")]
        let (runtime, video_backing) = if android::use_ahb_video_backend() {
            let mut video_cfg = video_cfg;
            let swapchain = Arc::new(
                android::AhbSwapchain::new(video_cfg.output_width, video_cfg.output_height)
                    .expect("failed to initialize AHB swapchain"),
            );
            let user_data = Arc::as_ptr(&swapchain) as *mut c_void;
            video_cfg.backend = VideoBackendConfig::Swapchain {
                lock: android::ahb_lock_plane,
                unlock: android::ahb_unlock_plane,
                user_data,
            };
            let runtime = Runtime::start(RuntimeConfig {
                video: video_cfg,
                audio: AudioMode::Auto,
            })
            .expect("failed to start nesium runtime");
            (runtime, VideoBacking::Ahb(swapchain))
        } else {
            let runtime = Runtime::start(RuntimeConfig {
                video: video_cfg,
                audio: AudioMode::Auto,
            })
            .expect("failed to start nesium runtime");
            (runtime, VideoBacking::Upload)
        };

        #[cfg(not(target_os = "android"))]
        let runtime = Runtime::start(RuntimeConfig {
            video: video_cfg,
            audio: AudioMode::Auto,
        })
        .expect("failed to start nesium runtime");

        let handle = runtime.handle();
        #[cfg(target_os = "android")]
        {
            // Default frame-ready callback for Android, used by both backends:
            // - Upload backend: wakes Kotlin GL uploader (pipe)
            // - AHB backend: wakes Rust renderer (condvar) + pipe (optional)
            handle
                .set_frame_ready_callback(
                    Some(android::android_frame_ready_cb),
                    std::ptr::null_mut(),
                )
                .expect("failed to set android frame ready callback");
        }
        let frame_handle = handle.frame_handle().cloned();

        RuntimeHolder {
            handle,
            frame_handle,
            _runtime: runtime,
            #[cfg(target_os = "android")]
            _video: video_backing,
        }
    })
}

pub(crate) fn runtime_handle() -> &'static RuntimeHandle {
    &ensure_runtime().handle
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

// === C ABI exposed to platform runners ====================================

#[unsafe(no_mangle)]
pub extern "C" fn nesium_runtime_start() {
    let _ = ensure_runtime();
}

/// Set the color format for frame rendering at runtime.
///
/// On Windows, D3D11 GPU textures require BGRA, while CPU fallback uses RGBA.
/// Call this after runtime start but before creating textures.
///
/// `use_bgra`: if true, uses BGRA; if false, uses RGBA.
#[cfg(target_os = "windows")]
#[unsafe(no_mangle)]
pub extern "C" fn nesium_set_color_format(use_bgra: bool) {
    use nesium_core::ppu::buffer::ColorFormat;
    let format = if use_bgra {
        ColorFormat::Bgra8888
    } else {
        ColorFormat::Rgba8888
    };
    let _ = runtime_handle().set_color_format(format);
}

#[unsafe(no_mangle)]
pub extern "C" fn nesium_set_frame_ready_callback(
    cb: Option<FrameReadyCallback>,
    user_data: *mut c_void,
) {
    let handle = runtime_handle();
    let _ = handle.set_frame_ready_callback(cb, user_data);
}

/// Copy the current NES frame into a destination buffer.
///
/// The pixel format is a platform-specific compile-time default.
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

    let Some(frame_handle) = ensure_runtime().frame_handle.as_deref() else {
        return;
    };
    let idx = frame_handle.begin_front_copy();
    let src_slice = frame_handle.plane_slice(idx);

    let src_width = frame_handle.width();
    let src_height = frame_handle.height();
    let bpp = frame_handle.bytes_per_pixel();
    let src_pitch = frame_handle.pitch_bytes();
    let dst_pitch = dst_pitch as usize;

    let dst_len = match dst_pitch.checked_mul(dst_height as usize) {
        Some(v) => v,
        None => {
            frame_handle.end_front_copy();
            return;
        }
    };
    let dst_slice = unsafe { std::slice::from_raw_parts_mut(dst, dst_len) };

    let height = src_height.min(dst_height as usize);
    if height == 0 {
        frame_handle.end_front_copy();
        return;
    }

    let row_bytes = src_width * bpp;

    if dst_pitch == src_pitch {
        let bytes = src_pitch.saturating_mul(height);
        let src_len = src_slice.len();
        let dst_len = dst_slice.len();
        let bytes = bytes.min(src_len).min(dst_len);
        let src = &src_slice[..bytes];
        let dst = &mut dst_slice[..bytes];
        dst.copy_from_slice(src);
    } else {
        let row_copy = row_bytes.min(dst_pitch).min(src_pitch);
        if row_copy == 0 {
            frame_handle.end_front_copy();
            return;
        }
        for y in 0..height {
            let src_off = y * src_pitch;
            let dst_off = y * dst_pitch;
            if src_off + row_copy > src_slice.len() || dst_off + row_copy > dst_slice.len() {
                break;
            }
            let src_row = &src_slice[src_off..src_off + row_copy];
            let dst_row = &mut dst_slice[dst_off..dst_off + row_copy];
            dst_row.copy_from_slice(src_row);
        }
    }

    frame_handle.end_front_copy();
}
