pub(crate) mod ahb;
pub(crate) mod chain;
pub(crate) mod gles;
mod renderer;
pub mod session;

pub use ahb::{AhbSwapchain, ahb_lock_plane, ahb_unlock_plane};
pub use renderer::apply_rust_renderer_priority;

use jni::{
    JNIEnv,
    objects::{GlobalRef, JByteBuffer, JClass, JObject},
    sys::jint,
    sys::jlong,
    sys::jobject,
};
use std::os::raw::{c_uint, c_void};
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering};
use std::sync::{Arc, OnceLock};

use gles::ANativeWindow_fromSurface;
use gles::ANativeWindow_release;
use renderer::{
    RustRendererHandle, notify_rust_renderer, rust_renderer_wake, set_rust_renderer_active,
    store_rust_renderer_tid,
};

use crate::{FRAME_HEIGHT, FRAME_WIDTH, ensure_runtime, runtime_handle};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum AndroidVideoBackend {
    #[allow(dead_code)]
    Upload = 0,
    AhbSwapchain = 1,
}

static VIDEO_BACKEND: AtomicI32 = AtomicI32::new(AndroidVideoBackend::AhbSwapchain as i32);
static FAST_FORWARD_FLAG: AtomicBool = AtomicBool::new(false);
pub(crate) static RUST_RENDERER_RUNNING: AtomicBool = AtomicBool::new(false);
pub(crate) static FRAME_SIGNAL_FD: AtomicI32 = AtomicI32::new(-1);
static CURRENT_OUTPUT_WIDTH: AtomicU32 = AtomicU32::new(0);
static CURRENT_OUTPUT_HEIGHT: AtomicU32 = AtomicU32::new(0);

static GLOBAL_CONTEXT: OnceLock<GlobalRef> = OnceLock::new();
static NDK_CONTEXT_INIT: OnceLock<()> = OnceLock::new();
static RUST_RENDERER: OnceLock<parking_lot::Mutex<Option<RustRendererHandle>>> = OnceLock::new();

pub fn use_ahb_video_backend() -> bool {
    VIDEO_BACKEND.load(Ordering::Acquire) == AndroidVideoBackend::AhbSwapchain as i32
}

pub fn set_android_fast_forward_flag(enabled: bool) {
    FAST_FORWARD_FLAG.store(enabled, Ordering::Release);
}

pub(crate) fn is_fast_forwarding() -> bool {
    FAST_FORWARD_FLAG.load(Ordering::Acquire)
}

fn set_current_output_size(width: u32, height: u32) {
    CURRENT_OUTPUT_WIDTH.store(width, Ordering::Release);
    CURRENT_OUTPUT_HEIGHT.store(height, Ordering::Release);
}

fn current_output_size() -> Option<(u32, u32)> {
    let w = CURRENT_OUTPUT_WIDTH.load(Ordering::Acquire);
    let h = CURRENT_OUTPUT_HEIGHT.load(Ordering::Acquire);
    if w > 0 && h > 0 { Some((w, h)) } else { None }
}

fn rust_renderer_slot() -> &'static parking_lot::Mutex<Option<RustRendererHandle>> {
    RUST_RENDERER.get_or_init(|| parking_lot::Mutex::new(None))
}

pub fn get_ahb_swapchain() -> Option<&'static AhbSwapchain> {
    #[allow(clippy::match_wildcard_for_single_variants)]
    match &ensure_runtime()._video {
        crate::VideoBacking::Ahb(swapchain) => Some(&**swapchain),
        _ => None,
    }
}

pub fn resize_ahb_swapchain(width: u32, height: u32) -> Result<(), String> {
    set_current_output_size(width, height);
    if !use_ahb_video_backend() {
        return Ok(());
    }
    let Some(swapchain) = get_ahb_swapchain() else {
        return Ok(());
    };
    swapchain.resize(width, height)
}

pub extern "C" fn android_frame_ready_cb(
    buffer_index: c_uint,
    width: c_uint,
    height: c_uint,
    _pitch: c_uint,
    _user_data: *mut c_void,
) {
    set_current_output_size(width as u32, height as u32);
    renderer::signal_frame_ready();
    notify_rust_renderer(buffer_index);
}

// === JNI Entry Points ======================================================

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_init_1android_1context(
    env: JNIEnv,
    _class: JClass,
    context: JObject,
) {
    let java_vm = env
        .get_java_vm()
        .expect("Failed to retrieve JavaVM instance");
    let vm_ptr = java_vm.get_java_vm_pointer() as *mut c_void;

    let global = GLOBAL_CONTEXT.get_or_init(|| {
        env.new_global_ref(&context)
            .expect("Failed to create global reference for Context")
    });
    let context_ptr = global.as_raw() as *mut c_void;

    NDK_CONTEXT_INIT.get_or_init(|| {
        // SAFETY: VM and context pointers are stable and backed by GLOBAL_CONTEXT.
        unsafe {
            ndk_context::initialize_android_context(vm_ptr, context_ptr);
        }
        tracing::info!("Android Context initialized via ndk-context");
    });

    let runtime = ensure_runtime();
    runtime
        .handle
        .set_frame_ready_callback(Some(android_frame_ready_cb), std::ptr::null_mut())
        .expect("Failed to set frame ready callback");
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nativeSetVideoBackend(
    _env: JNIEnv,
    _class: JClass,
    mode: jint,
) {
    VIDEO_BACKEND.store(mode as i32, Ordering::Release);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nativeRegisterRendererTid(
    _env: JNIEnv,
    _class: JClass,
    tid: jint,
) {
    store_rust_renderer_tid(tid as i32);
    apply_rust_renderer_priority(nesium_runtime::runtime::is_high_priority_enabled());
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nativeStartRustRenderer(
    env: JNIEnv,
    _class: JClass,
    surface: JObject,
) {
    let Some(swapchain) = get_ahb_swapchain() else {
        tracing::error!("nativeStartRustRenderer: AHB swapchain backend not active");
        return;
    };

    let env_ptr = env.get_native_interface();
    // SAFETY: surface is a valid JObject representing an Android Surface.
    let window = unsafe { ANativeWindow_fromSurface(env_ptr, surface.as_raw()) };
    if window.is_null() {
        tracing::error!("ANativeWindow_fromSurface failed");
        return;
    }

    let stop = Arc::new(AtomicBool::new(false));
    let window_ptr = window as usize;
    let swapchain_ref: &'static AhbSwapchain = swapchain;

    Java_io_github_mikai233_nesium_NesiumNative_nativeStopRustRenderer(env, _class);
    set_rust_renderer_active(true);

    let stop_for_thread = stop.clone();
    let swapchain_ref = std::panic::AssertUnwindSafe(swapchain_ref);
    let join = std::thread::spawn(move || {
        renderer::try_raise_current_thread_priority();
        RUST_RENDERER_RUNNING.store(true, Ordering::Release);
        let window = window_ptr as *mut gles::ANativeWindow;
        let res = std::panic::catch_unwind(|| unsafe {
            // Internal renderer loop
            renderer::run_rust_renderer(window, *swapchain_ref, stop_for_thread);
        });
        if res.is_err() {
            tracing::error!("Rust renderer thread panicked");
        }
        RUST_RENDERER_RUNNING.store(false, Ordering::Release);
        set_rust_renderer_active(false);
        // SAFETY: window was acquired via ANativeWindow_fromSurface.
        unsafe { ANativeWindow_release(window) };
    });

    let mut slot = rust_renderer_slot().lock();
    *slot = Some(RustRendererHandle { stop, join });
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nativeStopRustRenderer(
    _env: JNIEnv,
    _class: JClass,
) {
    set_rust_renderer_active(false);
    let handle = rust_renderer_slot().lock().take();
    if let Some(handle) = handle {
        handle.stop.store(true, Ordering::Release);
        rust_renderer_wake();
        let _ = handle.join.join();
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nativeSetFrameSignalFd(
    _env: JNIEnv,
    _class: JClass,
    fd: jint,
) {
    let fd = fd as i32;
    if fd < 0 {
        FRAME_SIGNAL_FD.store(-1, Ordering::Release);
        return;
    }
    // SAFETY: fd is a valid file descriptor provided by the Android OS.
    unsafe {
        let flags = libc::fcntl(fd, libc::F_GETFL);
        if flags >= 0 {
            let _ = libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        }
    }
    FRAME_SIGNAL_FD.store(fd, Ordering::Release);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nativeFrameSeq(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    runtime_handle().frame_seq() as jlong
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nativeBeginFrontCopy(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    ensure_runtime()
        .frame_handle
        .as_deref()
        .map(|h| h.begin_front_copy() as jint)
        .unwrap_or(0)
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nativePlaneBuffer(
    mut env: JNIEnv,
    _class: JClass,
    idx: jint,
) -> jobject {
    if idx < 0 || idx > 1 {
        return std::ptr::null_mut();
    }
    let Some(h) = ensure_runtime().frame_handle.as_deref() else {
        return std::ptr::null_mut();
    };
    let slice = h.plane_slice(idx as usize);
    // SAFETY: The direct byte buffer references NES runtime memory, which remains valid
    // until the matching nativeEndFrontCopy call.
    let res = unsafe { env.new_direct_byte_buffer(slice.as_ptr() as *mut u8, slice.len()) };
    match res {
        Ok(buf) => buf.into_raw(),
        Err(e) => {
            tracing::error!("Failed to create direct ByteBuffer: {e}");
            std::ptr::null_mut()
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nativeEndFrontCopy(
    _env: JNIEnv,
    _class: JClass,
) {
    if let Some(h) = ensure_runtime().frame_handle.as_deref() {
        h.end_front_copy();
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nativeFrameWidth(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    if let Some(h) = ensure_runtime().frame_handle.as_deref() {
        return h.width() as jint;
    }
    if let Some((w, _)) = current_output_size() {
        return w as jint;
    }
    if let Some(swapchain) = get_ahb_swapchain() {
        return swapchain.width() as jint;
    }
    FRAME_WIDTH as jint
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nativeFrameHeight(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    if let Some(h) = ensure_runtime().frame_handle.as_deref() {
        return h.height() as jint;
    }
    if let Some((_, h)) = current_output_size() {
        return h as jint;
    }
    if let Some(swapchain) = get_ahb_swapchain() {
        return swapchain.height() as jint;
    }
    FRAME_HEIGHT as jint
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nesiumAuxCreate(
    _env: JNIEnv,
    _class: JClass,
    id: jint,
    width: jint,
    height: jint,
) {
    crate::aux_texture::aux_create(id as u32, width as u32, height as u32);
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nesiumAuxCopy(
    env: JNIEnv,
    _class: JClass,
    id: jint,
    dst: jobject,
    dst_pitch: jint,
    dst_height: jint,
) -> jint {
    // SAFETY: dst must be a valid DirectByteBuffer.
    let Ok(ptr) = env.get_direct_buffer_address(&unsafe { JByteBuffer::from_raw(dst) }) else {
        return 0;
    };
    if ptr.is_null() {
        return 0;
    }
    let len = (dst_pitch as usize) * (dst_height as usize);
    let slice = unsafe { std::slice::from_raw_parts_mut(ptr, len) };
    crate::aux_texture::aux_copy(id as u32, slice, dst_pitch as usize) as jint
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nesiumAuxDestroy(
    _env: JNIEnv,
    _class: JClass,
    id: jint,
) {
    crate::aux_texture::aux_destroy(id as u32);
}
