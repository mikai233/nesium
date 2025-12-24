//! nesium-flutter
//!
//! Flutter bridge for the shared `nesium-runtime` backend.
//! - Flutter (via FRB) issues control commands (load/reset/input).
//! - The runtime owns a dedicated NES thread that renders frames into a
//!   double-buffered external 32-bit framebuffer (BGRA/RGBA depending on platform).
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
        // Platform-specific framebuffer pixel format.
        //
        // - macOS (CoreVideo/CVPixelBuffer paths) prefers BGRA.
        // - Windows render backends commonly prefer BGRA (e.g. DXGI/WGPU defaults).
        // - Android (GL upload) prefers RGBA.
        let color_format = {
            #[cfg(target_os = "macos")]
            {
                ColorFormat::Bgra8888
            }
            #[cfg(target_os = "windows")]
            {
                ColorFormat::Bgra8888
            }
            #[cfg(target_os = "android")]
            {
                ColorFormat::Rgba8888
            }
            #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "android")))]
            {
                ColorFormat::Bgra8888
            }
        };
        let len = FRAME_WIDTH * FRAME_HEIGHT * color_format.bytes_per_pixel();
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
                color_format,
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

/// Android-specific initialization module.
///
/// Compiled only on Android. Kotlin calls the exported JNI symbol to pass an
/// Android `Context` into Rust so native dependencies can access Android
/// system services via `ndk_context`.
#[cfg(target_os = "android")]
#[allow(non_snake_case)] // JNI symbol names are fixed by convention.
mod android_init {
    use jni::JNIEnv;
    use jni::objects::{GlobalRef, JObject};
    use std::{ffi::c_void, sync::OnceLock};

    /// A process-wide global reference to an Android `Context`.
    ///
    /// Keeping a `GlobalRef` alive prevents the JVM from collecting the object
    /// while native code still holds its raw `jobject` pointer.
    static GLOBAL_CONTEXT: OnceLock<GlobalRef> = OnceLock::new();

    /// Ensures `ndk_context::initialize_android_context` runs at most once.
    ///
    /// Flutter/Android may recreate the Activity (configuration changes, hot restart,
    /// etc.). From the native runtime's perspective, initialization should be
    /// effectively idempotent, so we guard it here.
    static NDK_CONTEXT_INIT: OnceLock<()> = OnceLock::new();

    /// JNI entry point called from Kotlin.
    ///
    /// Target Kotlin Class:  `io.github.mikai233.nesium.MainActivity`
    /// Target Kotlin Method: `init_android_context(context: Context)`
    ///
    /// Naming rules:
    /// - `.` in package names becomes `_`
    /// - `_` in identifiers becomes `_1`
    #[unsafe(no_mangle)]
    pub unsafe extern "system" fn Java_io_github_mikai233_nesium_MainActivity_init_1android_1context(
        env: JNIEnv,
        _class: JObject,
        context: JObject,
    ) {
        // 1) Capture the JavaVM pointer.
        //
        // Some native libraries attach native threads to the JVM; `ndk_context`
        // stores this pointer globally.
        let java_vm = env
            .get_java_vm()
            .expect("Failed to retrieve JavaVM instance");
        let vm_ptr = java_vm.get_java_vm_pointer() as *mut c_void;

        // 2) Promote the passed `Context` into a JVM GlobalRef.
        //
        // This yields a stable `jobject` for `ndk_context` as long as the GlobalRef lives.
        let global = GLOBAL_CONTEXT.get_or_init(|| {
            env.new_global_ref(&context)
                .expect("Failed to create global reference for Context")
        });

        let context_ptr = global.as_raw() as *mut c_void;

        // 3) Initialize `ndk_context` once per process.
        //
        // Repeated calls are ignored.
        NDK_CONTEXT_INIT.get_or_init(|| {
            // SAFETY: Pointers are backed by the process-wide GlobalRef and JavaVM.
            unsafe {
                ndk_context::initialize_android_context(vm_ptr, context_ptr);
            }
            println!("[Rust] Android Context initialized via ndk-context");
        });
    }
}
