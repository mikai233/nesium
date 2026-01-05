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
pub mod aux_texture;
pub mod event_worker;
mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */
mod senders;

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
    AudioMode, Runtime, RuntimeConfig, RuntimeHandle, VideoConfig, VideoExternalConfig,
};

#[cfg(target_os = "android")]
use nesium_runtime::VideoSwapchainConfig;

pub const FRAME_WIDTH: usize = SCREEN_WIDTH;
pub const FRAME_HEIGHT: usize = SCREEN_HEIGHT;

struct VideoBackingStore {
    _plane0: Box<[u8]>,
    _plane1: Box<[u8]>,
}

#[cfg(target_os = "android")]
enum VideoBacking {
    Cpu(#[allow(dead_code)] VideoBackingStore),
    Ahb(Box<android::AhbSwapchain>),
}

struct RuntimeHolder {
    #[cfg(target_os = "android")]
    _video: VideoBacking,
    #[cfg(not(target_os = "android"))]
    _video: VideoBackingStore,
    handle: RuntimeHandle,
    frame_handle: Option<Arc<ExternalFrameHandle>>,
    _runtime: Runtime,
}

static RUNTIME: OnceLock<RuntimeHolder> = OnceLock::new();

/// Returns the platform-specific pixel format for Flutter textures.
///
/// - macOS/iOS: BGRA (CVPixelBuffer)
/// - Android/Windows/Linux: RGBA (OpenGL/FlPixelBufferTexture)
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
        #[cfg(target_os = "android")]
        let (video_cfg, video_backing): (VideoConfig, VideoBacking) = {
            if android::use_ahb_video_backend() {
                let swapchain = Box::new(android::AhbSwapchain::new(
                    FRAME_WIDTH as u32,
                    FRAME_HEIGHT as u32,
                ));
                let user_data = (&*swapchain as *const android::AhbSwapchain) as *mut c_void;
                let cfg = VideoConfig::Swapchain(VideoSwapchainConfig {
                    color_format,
                    lock: android::ahb_lock_plane,
                    unlock: android::ahb_unlock_plane,
                    user_data,
                });
                // NOTE: callbacks own a clone-like handle; keep the backing store alive too.
                (cfg, VideoBacking::Ahb(swapchain))
            } else {
                let len = FRAME_WIDTH * FRAME_HEIGHT * color_format.bytes_per_pixel();
                let plane0 = vec![0u8; len].into_boxed_slice();
                let plane1 = vec![0u8; len].into_boxed_slice();
                let mut video = VideoBackingStore {
                    _plane0: plane0,
                    _plane1: plane1,
                };
                let cfg = VideoConfig::External(VideoExternalConfig {
                    color_format,
                    pitch_bytes: FRAME_WIDTH * color_format.bytes_per_pixel(),
                    plane0: video._plane0.as_mut_ptr(),
                    plane1: video._plane1.as_mut_ptr(),
                });
                (cfg, VideoBacking::Cpu(video))
            }
        };

        #[cfg(not(target_os = "android"))]
        let (video_cfg, video_backing): (VideoConfig, VideoBackingStore) = {
            let len = FRAME_WIDTH * FRAME_HEIGHT * color_format.bytes_per_pixel();
            let plane0 = vec![0u8; len].into_boxed_slice();
            let plane1 = vec![0u8; len].into_boxed_slice();
            let mut video = VideoBackingStore {
                _plane0: plane0,
                _plane1: plane1,
            };
            let cfg = VideoConfig::External(VideoExternalConfig {
                color_format,
                pitch_bytes: FRAME_WIDTH * color_format.bytes_per_pixel(),
                plane0: video._plane0.as_mut_ptr(),
                plane1: video._plane1.as_mut_ptr(),
            });
            (cfg, video)
        };

        // SAFETY:
        // - CPU backend: `video_backing` keeps the two planes alive for the lifetime of the process.
        // - AHB backend: the swapchain callbacks manage per-frame locking and pointer validity.

        let runtime = Runtime::start_pending(RuntimeConfig {
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
            _video: video_backing,
            handle,
            frame_handle,
            _runtime: runtime,
        }
    })
}

pub(crate) fn runtime_handle() -> &'static RuntimeHandle {
    &ensure_runtime().handle
}

fn frame_handle_ref() -> &'static ExternalFrameHandle {
    ensure_runtime()
        .frame_handle
        .as_ref()
        .expect("frame handle not available for this video backend")
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

    let frame_handle = frame_handle_ref();
    let idx = frame_handle.begin_front_copy();
    let src_slice = frame_handle.plane_slice(idx);

    let height = FRAME_HEIGHT.min(dst_height as usize);
    let src_pitch = FRAME_WIDTH * frame_handle.bytes_per_pixel();
    let dst_pitch = dst_pitch as usize;

    let dst_slice = unsafe {
        std::slice::from_raw_parts_mut(
            dst,
            dst_pitch
                .saturating_mul(dst_height as usize)
                .min(src_pitch * FRAME_HEIGHT),
        )
    };

    // Fast path when destination is tightly packed.
    if dst_pitch == src_pitch {
        let bytes = src_pitch * height;
        let src_len = src_slice.len();
        let dst_len = dst_slice.len();
        let bytes = bytes.min(src_len).min(dst_len);
        let src = &src_slice[..bytes];
        let dst = &mut dst_slice[..bytes];
        dst.copy_from_slice(src);
        frame_handle.end_front_copy();
        return;
    }

    for y in 0..height {
        let src_off = y * src_pitch;
        let dst_off = y * dst_pitch;
        let src_row = &src_slice[src_off..src_off + src_pitch];
        let dst_row = &mut dst_slice[dst_off..dst_off + src_pitch.min(dst_pitch)];
        dst_row.copy_from_slice(&src_row[..dst_row.len()]);
    }

    frame_handle.end_front_copy();
}
