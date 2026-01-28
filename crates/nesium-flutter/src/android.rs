use parking_lot::{Condvar, Mutex};
use std::{
    collections::VecDeque,
    ffi::{CStr, c_char, c_int, c_uint, c_void},
    num::NonZeroU32,
    os::unix::io::RawFd,
    sync::{
        Arc, OnceLock,
        atomic::{AtomicBool, AtomicI32, AtomicU32, Ordering},
    },
    time::Duration,
};

use glow::HasContext;
use jni::{
    JNIEnv,
    objects::{GlobalRef, JByteBuffer, JClass, JObject, JString},
    sys::{jboolean, jint, jlong, jobject},
};
use librashader::presets::ShaderFeatures as LibrashaderShaderFeatures;
use librashader::runtime::Size as LibrashaderSize;
use librashader::runtime::Viewport as LibrashaderViewport;
use librashader::runtime::gl::{
    FilterChain as LibrashaderFilterChain, FilterChainOptions as LibrashaderFilterChainOptions,
    FrameOptions as LibrashaderFrameOptions, GLImage as LibrashaderGlImage,
};

use crate::{FRAME_HEIGHT, FRAME_WIDTH, ensure_runtime, runtime_handle};

// Raw syscalls (fcntl/write) for the Android frame signal pipe.
use libc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum AndroidVideoBackend {
    #[allow(dead_code)]
    Upload = 0,
    AhbSwapchain = 1,
}

static VIDEO_BACKEND: AtomicI32 = AtomicI32::new(AndroidVideoBackend::AhbSwapchain as i32);

pub fn use_ahb_video_backend() -> bool {
    VIDEO_BACKEND.load(Ordering::Acquire) == AndroidVideoBackend::AhbSwapchain as i32
}

static RUST_RENDERER_TID: AtomicI32 = AtomicI32::new(-1);
static RUST_RENDERER_RUNNING: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Clone)]
struct AndroidShaderConfig {
    enabled: bool,
    preset_path: Option<String>,
    generation: u64,
}

static ANDROID_SHADER_CONFIG: OnceLock<Mutex<AndroidShaderConfig>> = OnceLock::new();

fn android_shader_config() -> &'static Mutex<AndroidShaderConfig> {
    ANDROID_SHADER_CONFIG.get_or_init(|| {
        Mutex::new(AndroidShaderConfig {
            enabled: false,
            preset_path: None,
            generation: 1,
        })
    })
}

fn android_shader_snapshot() -> AndroidShaderConfig {
    android_shader_config().lock().clone()
}

pub(crate) fn android_set_shader_enabled(enabled: bool) {
    let mut cfg = android_shader_config().lock();
    if cfg.enabled == enabled {
        return;
    }
    cfg.enabled = enabled;
    cfg.generation = cfg.generation.wrapping_add(1);
    // Wake renderer so it reloads promptly.
    rust_renderer_wake();
}

pub(crate) fn android_set_shader_preset_path(path: Option<String>) {
    let mut cfg = android_shader_config().lock();
    if cfg.preset_path == path {
        return;
    }
    cfg.preset_path = path;
    cfg.generation = cfg.generation.wrapping_add(1);
    rust_renderer_wake();
}

fn try_raise_current_thread_priority() {
    // Best-effort: Android apps may not be allowed to reduce nice (negative values).
    // If the call fails, we simply keep the default scheduler behavior.
    unsafe {
        let tid = libc::gettid() as i32;
        RUST_RENDERER_TID.store(tid, Ordering::Release);
        if !nesium_runtime::runtime::is_high_priority_enabled() {
            return;
        }
        let tid = tid as libc::id_t;
        let _ = libc::setpriority(libc::PRIO_PROCESS, tid, -2);
    }
}

pub(crate) fn apply_rust_renderer_priority(enabled: bool) {
    let tid = RUST_RENDERER_TID.load(Ordering::Acquire);
    if tid <= 0 {
        return;
    }
    unsafe {
        let tid = tid as libc::id_t;
        let nice = if enabled { -2 } else { 0 };
        let _ = libc::setpriority(libc::PRIO_PROCESS, tid, nice);
    }
}

// === AHardwareBuffer swapchain (Scheme B) ==================================

#[repr(C)]
pub struct AHardwareBuffer {
    _private: [u8; 0],
}

#[repr(C)]
pub struct AHardwareBuffer_Desc {
    pub width: u32,
    pub height: u32,
    pub layers: u32,
    pub format: u32,
    pub usage: u64,
    pub stride: u32,
    pub rfu0: u32,
    pub rfu1: u64,
}

#[repr(C)]
pub struct ARect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

// https://developer.android.com/ndk/reference/group/a-hardware-buffer
const AHARDWAREBUFFER_FORMAT_R8G8B8A8_UNORM: u32 = 1;
const AHARDWAREBUFFER_USAGE_CPU_WRITE_OFTEN: u64 = 0x30;
const AHARDWAREBUFFER_USAGE_GPU_SAMPLED_IMAGE: u64 = 0x100;

#[link(name = "android")]
unsafe extern "C" {
    fn AHardwareBuffer_allocate(
        desc: *const AHardwareBuffer_Desc,
        out: *mut *mut AHardwareBuffer,
    ) -> c_int;
    fn AHardwareBuffer_release(buffer: *mut AHardwareBuffer);
    fn AHardwareBuffer_describe(buffer: *const AHardwareBuffer, out: *mut AHardwareBuffer_Desc);
    fn AHardwareBuffer_lock(
        buffer: *mut AHardwareBuffer,
        usage: u64,
        fence: c_int,
        rect: *const ARect,
        out_virtual_address: *mut *mut c_void,
    ) -> c_int;
    fn AHardwareBuffer_unlock(buffer: *mut AHardwareBuffer, fence: *mut c_int) -> c_int;
}

pub struct AhbSwapchain {
    sync_mu: Mutex<AhbState>,
    sync_cv: Condvar,
    generation: AtomicU32,
}

struct AhbState {
    buffers: [*mut AHardwareBuffer; 2],
    width: u32,
    height: u32,
    pitch_bytes: usize,
    fallback_planes: [Box<[u8]>; 2],
    gpu_busy: [bool; 2],
    cpu_locked: [bool; 2],
    cpu_locked_ahb: [bool; 2],
    resizing: bool,
    retired_buffers: Vec<[*mut AHardwareBuffer; 2]>,
}

// SAFETY: The swapchain buffers are stable native handles; access is coordinated via internal
// atomics/mutexes and the Android NDK AHardwareBuffer APIs are thread-safe.
unsafe impl Send for AhbSwapchain {}
unsafe impl Sync for AhbSwapchain {}

impl AhbSwapchain {
    pub fn new(width: u32, height: u32) -> Self {
        let (buffers, pitch_bytes, fallback_planes) =
            allocate_buffers(width, height).expect("failed to allocate AHB swapchain buffers");

        Self {
            sync_mu: Mutex::new(AhbState {
                buffers,
                width,
                height,
                pitch_bytes,
                fallback_planes,
                gpu_busy: [false; 2],
                cpu_locked: [false; 2],
                cpu_locked_ahb: [false; 2],
                resizing: false,
                retired_buffers: Vec::new(),
            }),
            sync_cv: Condvar::new(),
            generation: AtomicU32::new(0),
        }
    }

    pub fn pitch_bytes(&self) -> usize {
        let mut state = self.sync_mu.lock();
        while state.resizing {
            self.sync_cv.wait(&mut state);
        }
        state.pitch_bytes
    }

    pub fn buffer(&self, idx: usize) -> *mut AHardwareBuffer {
        let mut state = self.sync_mu.lock();
        while state.resizing {
            self.sync_cv.wait(&mut state);
        }
        state.buffers[idx]
    }

    pub fn width(&self) -> u32 {
        let mut state = self.sync_mu.lock();
        while state.resizing {
            self.sync_cv.wait(&mut state);
        }
        state.width
    }

    pub fn height(&self) -> u32 {
        let mut state = self.sync_mu.lock();
        while state.resizing {
            self.sync_cv.wait(&mut state);
        }
        state.height
    }

    pub fn generation(&self) -> u32 {
        self.generation.load(Ordering::Acquire)
    }

    pub fn resize(&self, width: u32, height: u32) -> Result<(), String> {
        if width == 0 || height == 0 {
            return Err("invalid output size".to_string());
        }

        let (old_buffers, should_retire) = {
            let mut state = self.sync_mu.lock();
            if state.width == width && state.height == height {
                return Ok(());
            }

            state.resizing = true;
            self.sync_cv.notify_all();

            while state.cpu_locked.iter().any(|&b| b) || state.gpu_busy.iter().any(|&b| b) {
                self.sync_cv.wait(&mut state);
            }

            (state.buffers, RUST_RENDERER_RUNNING.load(Ordering::Acquire))
        };

        let (new_buffers, pitch_bytes, fallback_planes) = match allocate_buffers(width, height) {
            Ok(v) => v,
            Err(e) => {
                let mut state = self.sync_mu.lock();
                state.resizing = false;
                self.sync_cv.notify_all();
                return Err(e);
            }
        };

        let to_release = {
            let mut state = self.sync_mu.lock();
            if should_retire {
                state.retired_buffers.push(old_buffers);
            }
            state.buffers = new_buffers;
            state.width = width;
            state.height = height;
            state.pitch_bytes = pitch_bytes;
            state.fallback_planes = fallback_planes;
            state.resizing = false;
            self.generation.fetch_add(1, Ordering::AcqRel);
            self.sync_cv.notify_all();
            if should_retire {
                None
            } else {
                Some(old_buffers)
            }
        };

        if let Some(buffers) = to_release {
            unsafe { release_buffers(buffers) };
        }

        rust_renderer_wake();
        Ok(())
    }

    pub fn take_retired_buffers(&self) -> Vec<[*mut AHardwareBuffer; 2]> {
        let mut state = self.sync_mu.lock();
        std::mem::take(&mut state.retired_buffers)
    }

    fn wait_gpu_idle(&self, idx: usize) {
        let mut state = self.sync_mu.lock();
        while state.resizing || state.gpu_busy[idx] {
            self.sync_cv.wait(&mut state);
        }
    }

    fn set_gpu_busy(&self, idx: usize, busy: bool) {
        let mut state = self.sync_mu.lock();
        if busy {
            while state.resizing {
                self.sync_cv.wait(&mut state);
            }
            state.gpu_busy[idx] = true;
            return;
        }

        // Clearing busy must never block on `resizing`, otherwise we can deadlock:
        // resize waits for `gpu_busy=false`, while the renderer waits to clear it.
        state.gpu_busy[idx] = false;
        self.sync_cv.notify_all();
    }

    fn lock_plane(&self, idx: usize) -> *mut u8 {
        let (buffer, fallback_ptr) = {
            let mut state = self.sync_mu.lock();
            while state.resizing || state.gpu_busy[idx] {
                self.sync_cv.wait(&mut state);
            }
            state.cpu_locked[idx] = true;
            state.cpu_locked_ahb[idx] = true;
            (
                state.buffers[idx],
                state.fallback_planes[idx].as_ptr() as *mut u8,
            )
        };

        let mut out: *mut c_void;
        let mut last_err: c_int = 0;
        for attempt in 0..6u32 {
            out = std::ptr::null_mut();
            let res = unsafe {
                AHardwareBuffer_lock(
                    buffer,
                    AHARDWAREBUFFER_USAGE_CPU_WRITE_OFTEN,
                    -1,
                    std::ptr::null(),
                    &mut out as *mut _,
                )
            };
            if res == 0 && !out.is_null() {
                return out as *mut u8;
            }
            last_err = res;

            // Short backoff to tolerate transient failures; avoid spinning too aggressively.
            let backoff_ms = (1u64 << attempt).min(16);
            std::thread::sleep(Duration::from_millis(backoff_ms));
        }

        tracing::error!(
            "AHardwareBuffer_lock failed for idx={idx} (err={last_err}); falling back to dummy buffer"
        );
        let mut state = self.sync_mu.lock();
        state.cpu_locked_ahb[idx] = false;
        fallback_ptr
    }

    fn unlock_plane(&self, idx: usize) {
        let (buffer, should_unlock) = {
            let state = self.sync_mu.lock();
            if !state.cpu_locked[idx] {
                return;
            }
            (state.buffers[idx], state.cpu_locked_ahb[idx])
        };

        if should_unlock {
            let res = unsafe { AHardwareBuffer_unlock(buffer, std::ptr::null_mut()) };
            if res != 0 {
                tracing::error!("AHardwareBuffer_unlock failed: {res}");
            }
        }

        let mut state = self.sync_mu.lock();
        state.cpu_locked[idx] = false;
        state.cpu_locked_ahb[idx] = false;
        self.sync_cv.notify_all();
    }
}

impl Drop for AhbSwapchain {
    fn drop(&mut self) {
        let state = self.sync_mu.get_mut();
        unsafe {
            release_buffers(state.buffers);
            for retired in state.retired_buffers.drain(..) {
                release_buffers(retired);
            }
        }
    }
}

fn allocate_buffers(
    width: u32,
    height: u32,
) -> Result<([*mut AHardwareBuffer; 2], usize, [Box<[u8]>; 2]), String> {
    let mut buffers: [*mut AHardwareBuffer; 2] = [std::ptr::null_mut(), std::ptr::null_mut()];
    let desc = AHardwareBuffer_Desc {
        width,
        height,
        layers: 1,
        format: AHARDWAREBUFFER_FORMAT_R8G8B8A8_UNORM,
        usage: AHARDWAREBUFFER_USAGE_CPU_WRITE_OFTEN | AHARDWAREBUFFER_USAGE_GPU_SAMPLED_IMAGE,
        stride: 0,
        rfu0: 0,
        rfu1: 0,
    };

    for slot in &mut buffers {
        let mut out: *mut AHardwareBuffer = std::ptr::null_mut();
        let res = unsafe { AHardwareBuffer_allocate(&desc as *const _, &mut out as *mut _) };
        if res != 0 || out.is_null() {
            unsafe { release_buffers(buffers) };
            return Err(format!("AHardwareBuffer_allocate failed: {res}"));
        }
        *slot = out;
    }

    let mut described = AHardwareBuffer_Desc {
        width: 0,
        height: 0,
        layers: 0,
        format: 0,
        usage: 0,
        stride: 0,
        rfu0: 0,
        rfu1: 0,
    };
    unsafe { AHardwareBuffer_describe(buffers[0] as *const _, &mut described as *mut _) };
    let stride_pixels = described.stride.max(width);
    let pitch_bytes = stride_pixels as usize * 4;
    let fallback_len = pitch_bytes * height as usize;
    let fallback_planes = [
        vec![0u8; fallback_len].into_boxed_slice(),
        vec![0u8; fallback_len].into_boxed_slice(),
    ];

    Ok((buffers, pitch_bytes, fallback_planes))
}

unsafe fn release_buffers(buffers: [*mut AHardwareBuffer; 2]) {
    for b in buffers {
        if !b.is_null() {
            unsafe { AHardwareBuffer_release(b) };
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ahb_lock_plane(
    buffer_index: c_uint,
    pitch_out: *mut c_uint,
    user_data: *mut c_void,
) -> *mut u8 {
    let Some(pitch_out) = (unsafe { pitch_out.as_mut() }) else {
        return std::ptr::null_mut();
    };
    if user_data.is_null() {
        return std::ptr::null_mut();
    }
    let swapchain = unsafe { &*(user_data as *const AhbSwapchain) };
    *pitch_out = swapchain.pitch_bytes() as c_uint;
    swapchain.lock_plane(buffer_index as usize)
}

#[unsafe(no_mangle)]
pub extern "C" fn ahb_unlock_plane(buffer_index: c_uint, user_data: *mut c_void) {
    if user_data.is_null() {
        return;
    }
    let swapchain = unsafe { &*(user_data as *const AhbSwapchain) };
    swapchain.unlock_plane(buffer_index as usize);
}

/// A process-wide global reference to an Android `Context`.
///
/// Keeping a `GlobalRef` alive prevents the JVM from collecting the object
/// while native code still holds its raw `jobject` pointer.
static GLOBAL_CONTEXT: OnceLock<GlobalRef> = OnceLock::new();

/// Ensures `ndk_context::initialize_android_context` runs at most once.
///
/// Flutter/Android may recreate the Activity (configuration changes, hot restart, etc.).
/// From the native runtime's perspective, initialization should be effectively idempotent.
static NDK_CONTEXT_INIT: OnceLock<()> = OnceLock::new();

/// A process-wide write-end file descriptor for the "frame ready" signal pipe.
///
/// Kotlin creates a pipe and passes the write-end FD via `nativeSetFrameSignalFd(fd)`.
/// The NES runtime thread writes a small token to this FD when a new frame is published.
///
/// Note: the FD is owned by Kotlin (via `ParcelFileDescriptor`) and may be closed during shutdown.
static FRAME_SIGNAL_FD: AtomicI32 = AtomicI32::new(-1);
static CURRENT_OUTPUT_WIDTH: AtomicU32 = AtomicU32::new(0);
static CURRENT_OUTPUT_HEIGHT: AtomicU32 = AtomicU32::new(0);

fn set_current_output_size(width: u32, height: u32) {
    CURRENT_OUTPUT_WIDTH.store(width, Ordering::Release);
    CURRENT_OUTPUT_HEIGHT.store(height, Ordering::Release);
}

fn current_output_size() -> Option<(u32, u32)> {
    let w = CURRENT_OUTPUT_WIDTH.load(Ordering::Acquire);
    let h = CURRENT_OUTPUT_HEIGHT.load(Ordering::Acquire);
    if w > 0 && h > 0 { Some((w, h)) } else { None }
}

struct RustRendererSignalState {
    queue: VecDeque<u32>,
    renderer_active: bool,
}

struct RustRendererSignal {
    mu: Mutex<RustRendererSignalState>,
    cv: Condvar,
}

static RUST_RENDERER_SIGNAL: OnceLock<RustRendererSignal> = OnceLock::new();

fn notify_rust_renderer(buffer_index: u32) {
    let signal = rust_renderer_signal();

    let mut state = signal.mu.lock();

    if !state.renderer_active {
        return;
    }

    // Latest-only: keep only the most recent frame.
    state.queue.clear();

    state.queue.push_back(buffer_index);
    signal.cv.notify_one();
}

/// JNI entry point called from Kotlin.
///
/// Target Kotlin Class:  `io.github.mikai233.nesium.NesiumNative`
/// Target Kotlin Method: `init_android_context(context: Context)`
///
/// Naming rules:
/// - `.` in package names becomes `_`
/// - `_` in identifiers becomes `_1`
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_init_1android_1context(
    env: JNIEnv,
    _class: JClass,
    context: JObject,
) {
    // Capture the JavaVM pointer.
    let java_vm = env
        .get_java_vm()
        .expect("Failed to retrieve JavaVM instance");
    let vm_ptr = java_vm.get_java_vm_pointer() as *mut c_void;

    // Promote the passed `Context` into a JVM GlobalRef.
    // This yields a stable `jobject` for `ndk_context` as long as the GlobalRef lives.
    let global = GLOBAL_CONTEXT.get_or_init(|| {
        env.new_global_ref(&context)
            .expect("Failed to create global reference for Context")
    });

    let context_ptr = global.as_raw() as *mut c_void;

    // Initialize `ndk_context` once per process.
    NDK_CONTEXT_INIT.get_or_init(|| {
        // SAFETY: Pointers are backed by the process-wide GlobalRef and JavaVM.
        unsafe {
            ndk_context::initialize_android_context(vm_ptr, context_ptr);
        }
        tracing::info!("Android Context initialized via ndk-context");
    });

    // Ensure the runtime is started (video backend must be selected beforehand).
    let runtime = ensure_runtime();
    runtime
        .handle
        .set_frame_ready_callback(Some(android_frame_ready_cb), std::ptr::null_mut())
        .expect("Failed to set frame ready callback");
}

/// Selects the Android video backend for this process.
///
/// Must be called before `init_android_context` triggers runtime initialization.
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nativeSetVideoBackend(
    _env: JNIEnv,
    _class: JClass,
    mode: jint,
) {
    VIDEO_BACKEND.store(mode as i32, Ordering::Release);
}

/// Registers the calling thread as a renderer thread to receive dynamic priority updates.
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nativeRegisterRendererTid(
    _env: JNIEnv,
    _class: JClass,
    tid: jint,
) {
    RUST_RENDERER_TID.store(tid, Ordering::Release);
    // Apply current priority immediately to the newly registered thread.
    apply_rust_renderer_priority(nesium_runtime::runtime::is_high_priority_enabled());
}

/// Stores the write-end FD for the frame signal pipe and makes it non-blocking.
fn set_frame_signal_fd(fd: RawFd) {
    if fd < 0 {
        FRAME_SIGNAL_FD.store(-1, Ordering::Release);
        return;
    }

    unsafe {
        let flags = libc::fcntl(fd, libc::F_GETFL);
        if flags >= 0 {
            let _ = libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        }
    }

    FRAME_SIGNAL_FD.store(fd as i32, Ordering::Release);
}

/// Writes a small wakeup token into the frame signal pipe.
///
/// This function is designed to be called from a non-JVM thread (e.g. the NES runtime thread).
/// It must not perform any JNI work.
///
/// If the pipe is full (EAGAIN/EWOULDBLOCK), the token is dropped on purpose.
/// Kotlin uses a "latest-only" frame pull (`nativeFrameSeq` + begin/end copy), so losing
/// individual wakeup tokens is acceptable.
pub(crate) fn signal_frame_ready() {
    let fd = FRAME_SIGNAL_FD.load(Ordering::Acquire);
    if fd < 0 {
        return;
    }

    let seq = runtime_handle().frame_seq();
    let token = seq.to_le_bytes();

    let mut written = 0usize;
    while written < token.len() {
        let ptr = unsafe { token.as_ptr().add(written) } as *const c_void;
        let len = token.len() - written;

        let res = unsafe { libc::write(fd as RawFd, ptr, len) };
        if res > 0 {
            written += res as usize;
            continue;
        }

        // res == -1
        let err = std::io::Error::last_os_error();
        match err.raw_os_error() {
            Some(code) if code == libc::EINTR => {
                // Interrupted by a signal; retry.
                continue;
            }
            Some(code) if code == libc::EAGAIN || code == libc::EWOULDBLOCK => {
                // Pipe full; drop the signal (latest-only pull is fine).
                return;
            }
            Some(code) if code == libc::EBADF || code == libc::EPIPE => {
                // FD closed/invalid; disable it.
                FRAME_SIGNAL_FD.store(-1, Ordering::Release);
                return;
            }
            _ => {
                // Unknown error: ignore to avoid impacting the producer thread.
                return;
            }
        }
    }
}

pub extern "C" fn android_frame_ready_cb(
    buffer_index: c_uint,
    width: c_uint,
    height: c_uint,
    _pitch: c_uint,
    _user_data: *mut c_void,
) {
    // Must not panic here.
    set_current_output_size(width as u32, height as u32);
    signal_frame_ready();
    notify_rust_renderer(buffer_index);
}

// === Rust EGL/GL renderer (Scheme B) =======================================

#[repr(C)]
pub struct ANativeWindow {
    _private: [u8; 0],
}

#[link(name = "android")]
unsafe extern "C" {
    fn ANativeWindow_fromSurface(
        env: *mut jni::sys::JNIEnv,
        surface: jobject,
    ) -> *mut ANativeWindow;
    fn ANativeWindow_release(window: *mut ANativeWindow);
}

type EGLDisplay = *mut c_void;
type EGLContext = *mut c_void;
type EGLSurface = *mut c_void;
type EGLConfig = *mut c_void;
type EGLClientBuffer = *mut c_void;
type EGLImageKHR = *mut c_void;
type EGLSyncKHR = *mut c_void;
type EGLBoolean = c_int;
type EGLint = c_int;
type EGLTimeKHR = u64;
type EGLNativeDisplayType = *mut c_void;
type EGLNativeWindowType = *mut ANativeWindow;

const EGL_FALSE: EGLBoolean = 0;
const EGL_TRUE: EGLBoolean = 1;
const EGL_DEFAULT_DISPLAY: EGLNativeDisplayType = std::ptr::null_mut();
const EGL_NO_DISPLAY: EGLDisplay = std::ptr::null_mut();
const EGL_NO_CONTEXT: EGLContext = std::ptr::null_mut();
const EGL_NO_SURFACE: EGLSurface = std::ptr::null_mut();
const EGL_NO_IMAGE_KHR: EGLImageKHR = std::ptr::null_mut();
const EGL_NO_SYNC_KHR: EGLSyncKHR = std::ptr::null_mut();

const EGL_NONE: EGLint = 0x3038;
const EGL_RED_SIZE: EGLint = 0x3024;
const EGL_GREEN_SIZE: EGLint = 0x3023;
const EGL_BLUE_SIZE: EGLint = 0x3022;
const EGL_ALPHA_SIZE: EGLint = 0x3021;
const EGL_RENDERABLE_TYPE: EGLint = 0x3040;
const EGL_SURFACE_TYPE: EGLint = 0x3033;
const EGL_WINDOW_BIT: EGLint = 0x0004;
const EGL_OPENGL_ES2_BIT: EGLint = 0x0004;
// EGL_KHR_create_context
const EGL_OPENGL_ES3_BIT_KHR: EGLint = 0x00000040;
const EGL_CONTEXT_CLIENT_VERSION: EGLint = 0x3098;
const EGL_OPENGL_ES_API: EGLint = 0x30A0;
const EGL_WIDTH: EGLint = 0x3057;
const EGL_HEIGHT: EGLint = 0x3056;

const EGL_NATIVE_BUFFER_ANDROID: EGLint = 0x3140;
const EGL_IMAGE_PRESERVED_KHR: EGLint = 0x30D2;

const EGL_SYNC_FENCE_KHR: EGLint = 0x30F9;
const EGL_SYNC_FLUSH_COMMANDS_BIT_KHR: EGLint = 0x0001;
const EGL_FOREVER_KHR: EGLTimeKHR = 0xFFFFFFFFFFFFFFFF;

#[link(name = "EGL")]
unsafe extern "C" {
    fn eglGetDisplay(display_id: EGLNativeDisplayType) -> EGLDisplay;
    fn eglInitialize(dpy: EGLDisplay, major: *mut EGLint, minor: *mut EGLint) -> EGLBoolean;
    fn eglTerminate(dpy: EGLDisplay) -> EGLBoolean;
    fn eglBindAPI(api: EGLint) -> EGLBoolean;
    fn eglChooseConfig(
        dpy: EGLDisplay,
        attrib_list: *const EGLint,
        configs: *mut EGLConfig,
        config_size: EGLint,
        num_config: *mut EGLint,
    ) -> EGLBoolean;
    fn eglCreateContext(
        dpy: EGLDisplay,
        config: EGLConfig,
        share_context: EGLContext,
        attrib_list: *const EGLint,
    ) -> EGLContext;
    fn eglDestroyContext(dpy: EGLDisplay, ctx: EGLContext) -> EGLBoolean;
    fn eglCreateWindowSurface(
        dpy: EGLDisplay,
        config: EGLConfig,
        win: EGLNativeWindowType,
        attrib_list: *const EGLint,
    ) -> EGLSurface;
    fn eglDestroySurface(dpy: EGLDisplay, surface: EGLSurface) -> EGLBoolean;
    fn eglMakeCurrent(
        dpy: EGLDisplay,
        draw: EGLSurface,
        read: EGLSurface,
        ctx: EGLContext,
    ) -> EGLBoolean;
    fn eglQuerySurface(
        dpy: EGLDisplay,
        surface: EGLSurface,
        attribute: EGLint,
        value: *mut EGLint,
    ) -> EGLBoolean;
    fn eglSwapInterval(dpy: EGLDisplay, interval: EGLint) -> EGLBoolean;
    fn eglSwapBuffers(dpy: EGLDisplay, surface: EGLSurface) -> EGLBoolean;
    fn eglGetProcAddress(procname: *const c_char) -> *const c_void;
    fn eglGetError() -> EGLint;
}

#[link(name = "GLESv2")]
unsafe extern "C" {
    fn glDisable(cap: u32);
    fn glFlush();
    fn glViewport(x: c_int, y: c_int, width: c_int, height: c_int);
    fn glClearColor(r: f32, g: f32, b: f32, a: f32);
    fn glClear(mask: u32);
    fn glGenTextures(n: c_int, textures: *mut u32);
    fn glDeleteTextures(n: c_int, textures: *const u32);
    fn glBindTexture(target: u32, texture: u32);
    fn glTexParameteri(target: u32, pname: u32, param: c_int);
    fn glActiveTexture(texture: u32);
    fn glCreateShader(ty: u32) -> u32;
    fn glShaderSource(
        shader: u32,
        count: c_int,
        string: *const *const c_char,
        length: *const c_int,
    );
    fn glCompileShader(shader: u32);
    fn glGetShaderiv(shader: u32, pname: u32, params: *mut c_int);
    fn glGetShaderInfoLog(shader: u32, buf_size: c_int, length: *mut c_int, info_log: *mut c_char);
    fn glCreateProgram() -> u32;
    fn glAttachShader(program: u32, shader: u32);
    fn glLinkProgram(program: u32);
    fn glGetProgramiv(program: u32, pname: u32, params: *mut c_int);
    fn glGetProgramInfoLog(
        program: u32,
        buf_size: c_int,
        length: *mut c_int,
        info_log: *mut c_char,
    );
    fn glUseProgram(program: u32);
    fn glDeleteProgram(program: u32);
    fn glGetAttribLocation(program: u32, name: *const c_char) -> c_int;
    fn glGetUniformLocation(program: u32, name: *const c_char) -> c_int;
    fn glEnableVertexAttribArray(index: u32);
    fn glVertexAttribPointer(
        index: u32,
        size: c_int,
        ty: u32,
        normalized: u8,
        stride: c_int,
        pointer: *const c_void,
    );
    fn glUniform1i(location: c_int, v0: c_int);
    fn glDrawArrays(mode: u32, first: c_int, count: c_int);
    fn glFinish();
    fn glGetString(name: u32) -> *const u8;
}

const GL_DITHER: u32 = 0x0BD0;
const GL_COLOR_BUFFER_BIT: u32 = 0x00004000;
const GL_TEXTURE_2D: u32 = 0x0DE1;
const GL_TEXTURE0: u32 = 0x84C0;
const GL_TEXTURE_MIN_FILTER: u32 = 0x2801;
const GL_TEXTURE_MAG_FILTER: u32 = 0x2800;
const GL_TEXTURE_WRAP_S: u32 = 0x2802;
const GL_TEXTURE_WRAP_T: u32 = 0x2803;
const GL_NEAREST: c_int = 0x2600;
const GL_CLAMP_TO_EDGE: c_int = 0x812F;
const GL_VERTEX_SHADER: u32 = 0x8B31;
const GL_FRAGMENT_SHADER: u32 = 0x8B30;
const GL_COMPILE_STATUS: u32 = 0x8B81;
const GL_LINK_STATUS: u32 = 0x8B82;
const GL_INFO_LOG_LENGTH: u32 = 0x8B84;
const GL_TRIANGLE_STRIP: u32 = 0x0005;

const GL_EXTENSIONS: u32 = 0x1F03;

type PFNEGLGETNATIVECLIENTBUFFERANDROIDPROC =
    unsafe extern "C" fn(buffer: *mut AHardwareBuffer) -> EGLClientBuffer;
type PFNEGLCREATEIMAGEKHRPROC = unsafe extern "C" fn(
    dpy: EGLDisplay,
    ctx: EGLContext,
    target: EGLint,
    buffer: EGLClientBuffer,
    attrib_list: *const EGLint,
) -> EGLImageKHR;
type PFNEGLDESTROYIMAGEKHRPROC =
    unsafe extern "C" fn(dpy: EGLDisplay, image: EGLImageKHR) -> EGLBoolean;
type PFNGLEGLIMAGETARGETTEXTURE2DOESPROC = unsafe extern "C" fn(target: u32, image: *const c_void);
type PFNEGLCREATESYNCKHRPROC =
    unsafe extern "C" fn(dpy: EGLDisplay, ty: EGLint, attrib_list: *const EGLint) -> EGLSyncKHR;
type PFNEGLDESTROYSYNCKHRPROC =
    unsafe extern "C" fn(dpy: EGLDisplay, sync: EGLSyncKHR) -> EGLBoolean;
type PFNEGLCLIENTWAITSYNCKHRPROC = unsafe extern "C" fn(
    dpy: EGLDisplay,
    sync: EGLSyncKHR,
    flags: EGLint,
    timeout: EGLTimeKHR,
) -> EGLint;

unsafe fn egl_proc<T>(name: &'static [u8]) -> Option<T> {
    debug_assert!(name.last() == Some(&0));
    let ptr = unsafe { eglGetProcAddress(name.as_ptr() as *const c_char) };
    if ptr.is_null() {
        return None;
    }
    Some(unsafe { std::mem::transmute_copy::<*const c_void, T>(&ptr) })
}

struct RustRendererHandle {
    stop: Arc<AtomicBool>,
    join: std::thread::JoinHandle<()>,
}

static RUST_RENDERER: OnceLock<Mutex<Option<RustRendererHandle>>> = OnceLock::new();

fn rust_renderer_slot() -> &'static Mutex<Option<RustRendererHandle>> {
    RUST_RENDERER.get_or_init(|| Mutex::new(None))
}

fn rust_renderer_signal() -> &'static RustRendererSignal {
    RUST_RENDERER_SIGNAL.get_or_init(|| RustRendererSignal {
        mu: Mutex::new(RustRendererSignalState {
            queue: VecDeque::new(),
            renderer_active: false,
        }),
        cv: Condvar::new(),
    })
}

fn rust_renderer_wake() {
    let signal = rust_renderer_signal();
    signal.cv.notify_all();
}

fn set_rust_renderer_active(active: bool) {
    let signal = rust_renderer_signal();
    let mut state = signal.mu.lock();
    state.renderer_active = active;
    if !active {
        state.queue.clear();
    }
    signal.cv.notify_all();
}

fn get_ahb_swapchain() -> Option<&'static AhbSwapchain> {
    // This relies on `RuntimeHolder` keeping the backing store alive for the entire process lifetime.
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

struct GpuBusyGuard {
    swapchain: &'static AhbSwapchain,
    idx: usize,
}

impl GpuBusyGuard {
    fn new(swapchain: &'static AhbSwapchain, idx: usize) -> Self {
        swapchain.set_gpu_busy(idx, true);
        Self { swapchain, idx }
    }
}

impl Drop for GpuBusyGuard {
    fn drop(&mut self) {
        self.swapchain.set_gpu_busy(self.idx, false);
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nativeStartRustRenderer(
    env: JNIEnv,
    _class: JClass,
    surface: JObject,
) {
    // Ensure the runtime is started (video backend must be selected beforehand).
    let Some(swapchain) = get_ahb_swapchain() else {
        tracing::error!("nativeStartRustRenderer: AHB swapchain backend not active");
        return;
    };

    let env_ptr = env.get_native_interface();
    let window = unsafe { ANativeWindow_fromSurface(env_ptr, surface.as_raw()) };
    if window.is_null() {
        tracing::error!("ANativeWindow_fromSurface failed");
        return;
    }

    let stop = Arc::new(AtomicBool::new(false));
    // Closure capture in Rust 2024 may capture tuple struct fields; keep only `usize` / references.
    let window_ptr = window as usize;
    let swapchain_ref: &'static AhbSwapchain = swapchain;

    // Replace any existing renderer.
    Java_io_github_mikai233_nesium_NesiumNative_nativeStopRustRenderer(env, _class);
    set_rust_renderer_active(true);

    let stop_for_thread = stop.clone();
    let swapchain_ref = std::panic::AssertUnwindSafe(swapchain_ref);
    let join = std::thread::spawn(move || {
        try_raise_current_thread_priority();
        RUST_RENDERER_RUNNING.store(true, Ordering::Release);
        let window = window_ptr as *mut ANativeWindow;
        let res = std::panic::catch_unwind(|| unsafe {
            run_rust_renderer(window, *swapchain_ref, stop_for_thread);
        });
        if let Err(_) = res {
            tracing::error!("Rust renderer thread panicked");
        }
        RUST_RENDERER_RUNNING.store(false, Ordering::Release);
        set_rust_renderer_active(false);
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
    let handle = {
        let mut slot = rust_renderer_slot().lock();
        slot.take()
    };
    if let Some(handle) = handle {
        handle.stop.store(true, Ordering::Release);
        rust_renderer_wake();
        let _ = handle.join.join();
    }
}

fn compile_shader(kind: u32, src: &CStr) -> Option<u32> {
    unsafe {
        let shader = glCreateShader(kind);
        if shader == 0 {
            return None;
        }
        let ptrs = [src.as_ptr()];
        glShaderSource(shader, 1, ptrs.as_ptr(), std::ptr::null());
        glCompileShader(shader);
        let mut ok: c_int = 0;
        glGetShaderiv(shader, GL_COMPILE_STATUS, &mut ok as *mut _);
        if ok != 0 {
            return Some(shader);
        }

        let mut log_len: c_int = 0;
        glGetShaderiv(shader, GL_INFO_LOG_LENGTH, &mut log_len as *mut _);
        let mut buf = vec![0u8; log_len.max(1) as usize];
        let mut written: c_int = 0;
        glGetShaderInfoLog(
            shader,
            buf.len() as c_int,
            &mut written as *mut _,
            buf.as_mut_ptr() as *mut c_char,
        );
        tracing::error!("shader compile failed: {}", String::from_utf8_lossy(&buf));
        None
    }
}

fn link_program(vs: u32, fs: u32) -> Option<u32> {
    unsafe {
        let program = glCreateProgram();
        if program == 0 {
            return None;
        }
        glAttachShader(program, vs);
        glAttachShader(program, fs);
        glLinkProgram(program);
        let mut ok: c_int = 0;
        glGetProgramiv(program, GL_LINK_STATUS, &mut ok as *mut _);
        if ok != 0 {
            return Some(program);
        }

        let mut log_len: c_int = 0;
        glGetProgramiv(program, GL_INFO_LOG_LENGTH, &mut log_len as *mut _);
        let mut buf = vec![0u8; log_len.max(1) as usize];
        let mut written: c_int = 0;
        glGetProgramInfoLog(
            program,
            buf.len() as c_int,
            &mut written as *mut _,
            buf.as_mut_ptr() as *mut c_char,
        );
        tracing::error!("program link failed: {}", String::from_utf8_lossy(&buf));
        None
    }
}

struct EglCleanup {
    dpy: EGLDisplay,
    ctx: EGLContext,
    surf: EGLSurface,
    initialized: bool,
}

impl EglCleanup {
    fn new() -> Self {
        Self {
            dpy: EGL_NO_DISPLAY,
            ctx: EGL_NO_CONTEXT,
            surf: EGL_NO_SURFACE,
            initialized: false,
        }
    }
}

impl Drop for EglCleanup {
    fn drop(&mut self) {
        unsafe {
            if self.dpy == EGL_NO_DISPLAY || !self.initialized {
                return;
            }
            let _ = eglMakeCurrent(self.dpy, EGL_NO_SURFACE, EGL_NO_SURFACE, EGL_NO_CONTEXT);
            if self.surf != EGL_NO_SURFACE {
                let _ = eglDestroySurface(self.dpy, self.surf);
            }
            if self.ctx != EGL_NO_CONTEXT {
                let _ = eglDestroyContext(self.dpy, self.ctx);
            }
            let _ = eglTerminate(self.dpy);
        }
    }
}

unsafe fn run_rust_renderer(
    window: *mut ANativeWindow,
    swapchain: &'static AhbSwapchain,
    stop: Arc<AtomicBool>,
) {
    let mut egl = EglCleanup::new();

    let dpy = unsafe { eglGetDisplay(EGL_DEFAULT_DISPLAY) };
    if dpy == EGL_NO_DISPLAY {
        tracing::error!("eglGetDisplay failed");
        return;
    }
    egl.dpy = dpy;
    let mut major: EGLint = 0;
    let mut minor: EGLint = 0;
    if unsafe { eglInitialize(dpy, &mut major as *mut _, &mut minor as *mut _) } == EGL_FALSE {
        tracing::error!("eglInitialize failed: 0x{:x}", unsafe { eglGetError() });
        return;
    }
    egl.initialized = true;
    if unsafe { eglBindAPI(EGL_OPENGL_ES_API) } == EGL_FALSE {
        tracing::error!("eglBindAPI failed: 0x{:x}", unsafe { eglGetError() });
    }

    let attribs_es3 = [
        EGL_RENDERABLE_TYPE,
        EGL_OPENGL_ES3_BIT_KHR,
        EGL_SURFACE_TYPE,
        EGL_WINDOW_BIT,
        EGL_RED_SIZE,
        8,
        EGL_GREEN_SIZE,
        8,
        EGL_BLUE_SIZE,
        8,
        EGL_ALPHA_SIZE,
        8,
        EGL_NONE,
    ];
    let mut config: EGLConfig = std::ptr::null_mut();
    let mut num: EGLint = 0;

    let choose_ok = unsafe {
        eglChooseConfig(
            dpy,
            attribs_es3.as_ptr(),
            &mut config as *mut _,
            1,
            &mut num as *mut _,
        )
    } != EGL_FALSE
        && num > 0;

    if !choose_ok {
        // Fallback: ES2 config (keeps the old non-shader renderer working).
        let attribs_es2 = [
            EGL_RENDERABLE_TYPE,
            EGL_OPENGL_ES2_BIT,
            EGL_SURFACE_TYPE,
            EGL_WINDOW_BIT,
            EGL_RED_SIZE,
            8,
            EGL_GREEN_SIZE,
            8,
            EGL_BLUE_SIZE,
            8,
            EGL_ALPHA_SIZE,
            8,
            EGL_NONE,
        ];
        if unsafe {
            eglChooseConfig(
                dpy,
                attribs_es2.as_ptr(),
                &mut config as *mut _,
                1,
                &mut num as *mut _,
            )
        } == EGL_FALSE
            || num <= 0
        {
            tracing::error!("eglChooseConfig failed: 0x{:x}", unsafe { eglGetError() });
            return;
        }
    }

    // Prefer GLES3 for librashader; fall back to GLES2 if needed.
    let mut ctx_version: EGLint = 3;
    let ctx = unsafe {
        eglCreateContext(
            dpy,
            config,
            EGL_NO_CONTEXT,
            [EGL_CONTEXT_CLIENT_VERSION, 3, EGL_NONE].as_ptr(),
        )
    };
    let ctx = if ctx == EGL_NO_CONTEXT {
        ctx_version = 2;
        unsafe {
            eglCreateContext(
                dpy,
                config,
                EGL_NO_CONTEXT,
                [EGL_CONTEXT_CLIENT_VERSION, 2, EGL_NONE].as_ptr(),
            )
        }
    } else {
        ctx
    };
    if ctx == EGL_NO_CONTEXT {
        tracing::error!("eglCreateContext failed: 0x{:x}", unsafe { eglGetError() });
        return;
    }
    egl.ctx = ctx;

    let surf = unsafe { eglCreateWindowSurface(dpy, config, window, [EGL_NONE].as_ptr()) };
    if surf == EGL_NO_SURFACE {
        tracing::error!("eglCreateWindowSurface failed: 0x{:x}", unsafe {
            eglGetError()
        });
        return;
    }
    egl.surf = surf;

    if unsafe { eglMakeCurrent(dpy, surf, surf, ctx) } == EGL_FALSE {
        tracing::error!("eglMakeCurrent failed: 0x{:x}", unsafe { eglGetError() });
        return;
    }

    // Create a glow context for librashader and for our blit path.
    let glow_ctx = Arc::new(unsafe {
        glow::Context::from_loader_function(|name| {
            // `eglGetProcAddress` expects a NUL-terminated string.
            let name = match std::ffi::CString::new(name) {
                Ok(v) => v,
                Err(_) => return std::ptr::null(),
            };
            unsafe { eglGetProcAddress(name.as_ptr()) as *const c_void }
        })
    });

    let get_native_client_buffer: PFNEGLGETNATIVECLIENTBUFFERANDROIDPROC =
        match unsafe { egl_proc(b"eglGetNativeClientBufferANDROID\0") } {
            Some(p) => p,
            None => {
                tracing::error!("missing eglGetNativeClientBufferANDROID");
                return;
            }
        };
    let egl_create_image: PFNEGLCREATEIMAGEKHRPROC =
        match unsafe { egl_proc(b"eglCreateImageKHR\0") } {
            Some(p) => p,
            None => {
                tracing::error!("missing eglCreateImageKHR");
                return;
            }
        };
    let egl_destroy_image: PFNEGLDESTROYIMAGEKHRPROC =
        match unsafe { egl_proc(b"eglDestroyImageKHR\0") } {
            Some(p) => p,
            None => {
                tracing::error!("missing eglDestroyImageKHR");
                return;
            }
        };
    let gl_egl_image_target_texture: PFNGLEGLIMAGETARGETTEXTURE2DOESPROC =
        match unsafe { egl_proc(b"glEGLImageTargetTexture2DOES\0") } {
            Some(p) => p,
            None => {
                tracing::error!("missing glEGLImageTargetTexture2DOES");
                return;
            }
        };

    let fence_sync_procs = match (
        unsafe { egl_proc::<PFNEGLCREATESYNCKHRPROC>(b"eglCreateSyncKHR\0") },
        unsafe { egl_proc::<PFNEGLDESTROYSYNCKHRPROC>(b"eglDestroySyncKHR\0") },
        unsafe { egl_proc::<PFNEGLCLIENTWAITSYNCKHRPROC>(b"eglClientWaitSyncKHR\0") },
    ) {
        (Some(create), Some(destroy), Some(wait)) => Some((create, destroy, wait)),
        _ => None,
    };

    let wait_for_gpu = |dpy: EGLDisplay| {
        if let Some((egl_create_sync, egl_destroy_sync, egl_client_wait_sync)) = fence_sync_procs {
            let sync = unsafe { egl_create_sync(dpy, EGL_SYNC_FENCE_KHR, [EGL_NONE].as_ptr()) };
            if sync != EGL_NO_SYNC_KHR {
                let _ = unsafe {
                    egl_client_wait_sync(
                        dpy,
                        sync,
                        EGL_SYNC_FLUSH_COMMANDS_BIT_KHR,
                        EGL_FOREVER_KHR,
                    )
                };
                let _ = unsafe { egl_destroy_sync(dpy, sync) };
                return;
            }
        }
        unsafe { glFinish() };
    };

    // Log extensions once (useful when debugging device compatibility).
    let ext_ptr = unsafe { glGetString(GL_EXTENSIONS) };
    if !ext_ptr.is_null() {
        let ext = unsafe { CStr::from_ptr(ext_ptr as *const c_char) }.to_string_lossy();
        tracing::info!("GL_EXTENSIONS: {}", ext);
    }

    unsafe { glDisable(GL_DITHER) };
    let _ = unsafe { eglSwapInterval(dpy, 1) };

    let mut textures = [0u32; 2];
    let mut images = [EGL_NO_IMAGE_KHR; 2];
    let mut seen_generation = swapchain.generation();

    let mut destroy_images = |images: &mut [EGLImageKHR; 2]| unsafe {
        for img in images.iter_mut() {
            if *img != EGL_NO_IMAGE_KHR {
                let _ = egl_destroy_image(dpy, *img);
                *img = EGL_NO_IMAGE_KHR;
            }
        }
    };

    let mut recreate_textures_and_images =
        |textures: &mut [u32; 2], images: &mut [EGLImageKHR; 2]| {
            unsafe { glGenTextures(2, textures.as_mut_ptr()) };

            for i in 0..2 {
                unsafe { glBindTexture(GL_TEXTURE_2D, textures[i]) };

                let client_buf = unsafe { get_native_client_buffer(swapchain.buffer(i)) };
                let img_attribs = [EGL_IMAGE_PRESERVED_KHR, EGL_TRUE, EGL_NONE];
                let image = unsafe {
                    egl_create_image(
                        dpy,
                        EGL_NO_CONTEXT,
                        EGL_NATIVE_BUFFER_ANDROID,
                        client_buf,
                        img_attribs.as_ptr(),
                    )
                };
                if image == EGL_NO_IMAGE_KHR {
                    tracing::error!("eglCreateImageKHR failed: 0x{:x}", unsafe { eglGetError() });
                    continue;
                }
                images[i] = image;
                unsafe { gl_egl_image_target_texture(GL_TEXTURE_2D, image as *const c_void) };

                // Set texture parameters AFTER binding the EGLImage. Some Android GPU drivers
                // reset texture parameters to defaults (GL_LINEAR) when glEGLImageTargetTexture2DOES
                // is called, causing blurry rendering if set beforehand.
                unsafe {
                    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
                    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
                    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
                    glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
                }
            }
        };

    recreate_textures_and_images(&mut textures, &mut images);

    // Create a simple blit shader program to draw a texture to the EGL surface.
    let is_gles = glow_ctx.version().is_embedded;
    let is_gles3 = is_gles && glow_ctx.version().major >= 3 && ctx_version >= 3;
    tracing::info!(
        "GL Version: {:?}, is_gles: {}, ctx_version: {}, is_gles3: {}",
        glow_ctx.version(),
        is_gles,
        ctx_version,
        is_gles3
    );

    let (vs_src, fs_src) = if is_gles3 {
        (
            r#"#version 300 es
precision highp float;
layout(location = 0) in vec2 a_position;
layout(location = 1) in vec2 a_tex_coord;
out vec2 v_tex_coord;
void main() {
  gl_Position = vec4(a_position, 0.0, 1.0);
  v_tex_coord = a_tex_coord;
}
"#,
            r#"#version 300 es
precision mediump float;
uniform sampler2D u_texture;
in vec2 v_tex_coord;
out vec4 o_color;
void main() {
  o_color = texture(u_texture, v_tex_coord);
}
"#,
        )
    } else {
        (
            r#"attribute vec2 a_position;
attribute vec2 a_tex_coord;
varying vec2 v_tex_coord;
void main() {
  gl_Position = vec4(a_position, 0.0, 1.0);
  v_tex_coord = a_tex_coord;
}
"#,
            r#"precision mediump float;
uniform sampler2D u_texture;
varying vec2 v_tex_coord;
void main() {
  gl_FragColor = texture2D(u_texture, v_tex_coord);
}
"#,
        )
    };

    let blit_program = unsafe {
        let vs = match glow_ctx.create_shader(glow::VERTEX_SHADER) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("blit create vertex shader failed: {e}");
                return;
            }
        };
        glow_ctx.shader_source(vs, vs_src);
        glow_ctx.compile_shader(vs);
        if !glow_ctx.get_shader_compile_status(vs) {
            tracing::error!("blit vertex shader compile failed");
            glow_ctx.delete_shader(vs);
            return;
        }

        let fs = match glow_ctx.create_shader(glow::FRAGMENT_SHADER) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("blit create fragment shader failed: {e}");
                glow_ctx.delete_shader(vs);
                return;
            }
        };
        glow_ctx.shader_source(fs, fs_src);
        glow_ctx.compile_shader(fs);
        if !glow_ctx.get_shader_compile_status(fs) {
            tracing::error!("blit fragment shader compile failed");
            glow_ctx.delete_shader(vs);
            glow_ctx.delete_shader(fs);
            return;
        }

        let program = match glow_ctx.create_program() {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("blit create program failed: {e}");
                glow_ctx.delete_shader(vs);
                glow_ctx.delete_shader(fs);
                return;
            }
        };

        glow_ctx.attach_shader(program, vs);
        glow_ctx.attach_shader(program, fs);

        if !is_gles3 {
            glow_ctx.bind_attrib_location(program, 0, "a_position");
            glow_ctx.bind_attrib_location(program, 1, "a_tex_coord");
        }

        glow_ctx.link_program(program);
        glow_ctx.delete_shader(vs);
        glow_ctx.delete_shader(fs);

        if !glow_ctx.get_program_link_status(program) {
            tracing::error!("blit program link failed");
            glow_ctx.delete_program(program);
            return;
        }

        glow_ctx.use_program(Some(program));
        if let Some(loc) = glow_ctx.get_uniform_location(program, "u_texture") {
            glow_ctx.uniform_1_i32(Some(&loc), 0);
        }
        glow_ctx.use_program(None);
        program
    };

    // Fullscreen quad VBO/VAO.
    let quad: [f32; 16] = [
        // X,   Y,   U,   V
        -1.0, -1.0, 0.0, 1.0, // bottom-left
        1.0, -1.0, 1.0, 1.0, // bottom-right
        -1.0, 1.0, 0.0, 0.0, // top-left
        1.0, 1.0, 1.0, 0.0, // top-right
    ];

    let quad_vbo = unsafe {
        let vbo = match glow_ctx.create_buffer() {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("blit create buffer failed: {e}");
                glow_ctx.delete_program(blit_program);
                return;
            }
        };
        let bytes =
            std::slice::from_raw_parts(quad.as_ptr() as *const u8, std::mem::size_of_val(&quad));
        glow_ctx.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
        glow_ctx.buffer_data_u8_slice(glow::ARRAY_BUFFER, bytes, glow::STATIC_DRAW);
        glow_ctx.bind_buffer(glow::ARRAY_BUFFER, None);
        vbo
    };

    let quad_vao = if is_gles3 {
        unsafe {
            let vao = match glow_ctx.create_vertex_array() {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("blit create vertex array failed: {e}");
                    glow_ctx.delete_buffer(quad_vbo);
                    glow_ctx.delete_program(blit_program);
                    return;
                }
            };
            glow_ctx.bind_vertex_array(Some(vao));
            glow_ctx.bind_buffer(glow::ARRAY_BUFFER, Some(quad_vbo));
            let stride = (4 * std::mem::size_of::<f32>()) as i32;
            glow_ctx.enable_vertex_attrib_array(0);
            glow_ctx.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);
            glow_ctx.enable_vertex_attrib_array(1);
            glow_ctx.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, stride, 2 * 4);
            glow_ctx.bind_vertex_array(None);
            glow_ctx.bind_buffer(glow::ARRAY_BUFFER, None);
            Some(vao)
        }
    } else {
        None
    };

    // Librashader state (optional; only enabled when we successfully created a GLES3 context).
    let mut shader_seen_generation: u64 = 0;
    let mut shader_chain: Option<LibrashaderFilterChain> = None;
    let shader_features = LibrashaderShaderFeatures::ORIGINAL_ASPECT_UNIFORMS
        | LibrashaderShaderFeatures::FRAMETIME_UNIFORMS;
    let shader_chain_options = LibrashaderFilterChainOptions {
        // Auto-detect version; our git dependency now has the fix to detect GLES correctly.
        glsl_version: 0,
        use_dsa: false,
        force_no_mipmaps: false,
        disable_cache: true,
    };

    // Offscreen output texture for shader rendering (we then blit it to the EGL surface).
    let mut shader_output_tex: Option<glow::Texture> = None;
    let mut shader_output_size: LibrashaderSize<u32> = LibrashaderSize {
        width: 0,
        height: 0,
    };
    let mut frame_count: usize = 0;

    // Render loop: wait for frame-ready signals from `android_frame_ready_cb`.
    let signal = rust_renderer_signal();

    while !stop.load(Ordering::Acquire) {
        let generation = swapchain.generation();
        if generation != seen_generation {
            wait_for_gpu(dpy);
            unsafe { glBindTexture(GL_TEXTURE_2D, 0) };
            unsafe { glDeleteTextures(2, textures.as_ptr()) };
            wait_for_gpu(dpy);
            destroy_images(&mut images);
            wait_for_gpu(dpy);
            textures = [0u32; 2];
            images = [EGL_NO_IMAGE_KHR; 2];
            for retired in swapchain.take_retired_buffers() {
                unsafe { release_buffers(retired) };
            }
            recreate_textures_and_images(&mut textures, &mut images);
            seen_generation = generation;
        }

        let msg = {
            let mut state = signal.mu.lock();
            while state.queue.is_empty() && !stop.load(Ordering::Acquire) {
                signal.cv.wait_for(&mut state, Duration::from_millis(500));
                break;
            }
            if stop.load(Ordering::Acquire) {
                None
            } else {
                state.queue.pop_front()
            }
        };

        let Some(buffer_index) = msg else { continue };
        let idx = buffer_index as usize;

        let _busy = GpuBusyGuard::new(swapchain, idx);

        // Query the actual EGL surface size. When using a SurfaceView without
        // setFixedSize(), the surface matches the view's pixel dimensions. This allows
        // our GL renderer to scale the NES frame to fill the screen using NEAREST sampling,
        // avoiding the system compositor's bilinear scaling.
        let mut surf_w: EGLint = FRAME_WIDTH as EGLint;
        let mut surf_h: EGLint = FRAME_HEIGHT as EGLint;
        unsafe {
            let _ = eglQuerySurface(dpy, surf, EGL_WIDTH, &mut surf_w as *mut _);
            let _ = eglQuerySurface(dpy, surf, EGL_HEIGHT, &mut surf_h as *mut _);
        }

        let surf_w = (surf_w as i32).max(1) as u32;
        let surf_h = (surf_h as i32).max(1) as u32;

        // Reload shader chain if config changed.
        if is_gles3 {
            let cfg = android_shader_snapshot();
            if cfg.generation != shader_seen_generation {
                shader_seen_generation = cfg.generation;
                shader_chain = None;
                if cfg.enabled {
                    if let Some(path) = cfg.preset_path {
                        let res = unsafe {
                            LibrashaderFilterChain::load_from_path(
                                path,
                                shader_features,
                                Arc::clone(&glow_ctx),
                                Some(&shader_chain_options),
                            )
                        };
                        match res {
                            Ok(chain) => {
                                tracing::info!("Shader chain loaded successfully");
                                shader_chain = Some(chain);
                            }
                            Err(e) => tracing::error!("Failed to load shader preset: {e:?}"),
                        }
                    }
                }
            }
        }

        // Ensure shader output texture is allocated for the current surface size.
        if is_gles3 && shader_chain.is_some() {
            if shader_output_size.width != surf_w || shader_output_size.height != surf_h {
                shader_output_size = LibrashaderSize {
                    width: surf_w,
                    height: surf_h,
                };
                unsafe {
                    if let Some(tex) = shader_output_tex.take() {
                        glow_ctx.delete_texture(tex);
                    }
                    let tex = match glow_ctx.create_texture() {
                        Ok(v) => v,
                        Err(e) => {
                            tracing::error!("create shader output texture failed: {e}");
                            shader_chain = None;
                            continue;
                        }
                    };
                    glow_ctx.bind_texture(glow::TEXTURE_2D, Some(tex));
                    glow_ctx.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_MIN_FILTER,
                        glow::NEAREST as i32,
                    );
                    glow_ctx.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_MAG_FILTER,
                        glow::NEAREST as i32,
                    );
                    glow_ctx.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_WRAP_S,
                        glow::CLAMP_TO_EDGE as i32,
                    );
                    glow_ctx.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_WRAP_T,
                        glow::CLAMP_TO_EDGE as i32,
                    );
                    glow_ctx.tex_image_2d(
                        glow::TEXTURE_2D,
                        0,
                        glow::RGBA8 as i32,
                        surf_w as i32,
                        surf_h as i32,
                        0,
                        glow::RGBA,
                        glow::UNSIGNED_BYTE,
                        glow::PixelUnpackData::Slice(None),
                    );
                    glow_ctx.bind_texture(glow::TEXTURE_2D, None);
                    shader_output_tex = Some(tex);
                }
            }
        }

        // Optional shader pass: render into offscreen texture first.
        let mut present_tex_id = textures[idx];
        if is_gles3 {
            if let (Some(chain), Some(out_tex)) = (shader_chain.as_mut(), shader_output_tex) {
                let in_tex = NonZeroU32::new(textures[idx]).map(glow::NativeTexture);
                let input = LibrashaderGlImage {
                    handle: in_tex,
                    format: glow::RGBA8,
                    size: LibrashaderSize {
                        width: swapchain.width(),
                        height: swapchain.height(),
                    },
                };

                let output = LibrashaderGlImage {
                    handle: Some(out_tex),
                    format: glow::RGBA8,
                    size: shader_output_size,
                };

                let viewport =
                    LibrashaderViewport::new_render_target_sized_origin(&output, None).unwrap();

                let frame_options = LibrashaderFrameOptions {
                    frames_per_second: 60.0,
                    frametime_delta: 17,
                    ..Default::default()
                };

                if let Err(e) =
                    unsafe { chain.frame(&input, &viewport, frame_count, Some(&frame_options)) }
                {
                    tracing::error!("Shader frame failed: {e:?}");
                } else {
                    present_tex_id = out_tex.0.get();
                }
            }
        }

        frame_count = frame_count.wrapping_add(1);

        // Present: blit to EGL surface.
        unsafe {
            glow_ctx.viewport(0, 0, surf_w as i32, surf_h as i32);
            glow_ctx.clear_color(0.0, 0.0, 0.0, 1.0);
            glow_ctx.clear(glow::COLOR_BUFFER_BIT);

            glow_ctx.disable(glow::CULL_FACE);
            glow_ctx.disable(glow::BLEND);
            glow_ctx.disable(glow::DEPTH_TEST);

            glow_ctx.use_program(Some(blit_program));
            glow_ctx.active_texture(glow::TEXTURE0);
            let tex = NonZeroU32::new(present_tex_id).map(glow::NativeTexture);
            glow_ctx.bind_texture(glow::TEXTURE_2D, tex);

            if let Some(vao) = quad_vao {
                glow_ctx.bind_vertex_array(Some(vao));
            } else {
                glow_ctx.bind_buffer(glow::ARRAY_BUFFER, Some(quad_vbo));
                let stride = (4 * std::mem::size_of::<f32>()) as i32;
                glow_ctx.enable_vertex_attrib_array(0);
                glow_ctx.vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);
                glow_ctx.enable_vertex_attrib_array(1);
                glow_ctx.vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, stride, 2 * 4);
            }

            glow_ctx.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);

            if quad_vao.is_some() {
                glow_ctx.bind_vertex_array(None);
            } else {
                glow_ctx.bind_buffer(glow::ARRAY_BUFFER, None);
            }
            glow_ctx.bind_texture(glow::TEXTURE_2D, None);
            glow_ctx.use_program(None);
        }

        if unsafe { eglSwapBuffers(dpy, surf) } == EGL_FALSE {
            tracing::error!("eglSwapBuffers failed: 0x{:x}", unsafe { eglGetError() });
            wait_for_gpu(dpy);
            break;
        }
        wait_for_gpu(dpy);
    }

    wait_for_gpu(dpy);
    unsafe { glBindTexture(GL_TEXTURE_2D, 0) };
    unsafe { glDeleteTextures(2, textures.as_ptr()) };
    wait_for_gpu(dpy);
    destroy_images(&mut images);
    wait_for_gpu(dpy);
    for retired in swapchain.take_retired_buffers() {
        unsafe { release_buffers(retired) };
    }
    unsafe {
        // Best-effort cleanup (renderer thread exiting).
        if let Some(tex) = shader_output_tex.take() {
            glow_ctx.delete_texture(tex);
        }
        if let Some(vao) = quad_vao {
            glow_ctx.delete_vertex_array(vao);
        }
        glow_ctx.delete_buffer(quad_vbo);
        glow_ctx.delete_program(blit_program);
        glow_ctx.flush();
    }
}

/// Receives the write-end FD of the Kotlin-created frame signal pipe.
///
/// Kotlin: `NesiumNative.nativeSetFrameSignalFd(fd: Int)`
///
/// The FD is stored globally and used by `signal_frame_ready(seq)`.
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nativeSetFrameSignalFd(
    _env: JNIEnv,
    _class: JClass,
    fd: jint,
) {
    set_frame_signal_fd(fd as RawFd);
}

/// Returns the current monotonic frame sequence number.
///
/// Kotlin: `NesiumNative.nativeFrameSeq(): Long`
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nativeFrameSeq(
    _env: JNIEnv,
    _class: JClass,
) -> jlong {
    runtime_handle().frame_seq() as jlong
}

/// Begins a front-buffer copy and returns the active plane index.
///
/// Kotlin: `NesiumNative.nativeBeginFrontCopy(): Int`
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nativeBeginFrontCopy(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    let Some(h) = ensure_runtime().frame_handle.as_deref() else {
        return 0;
    };
    h.begin_front_copy() as jint
}

/// Returns a direct `java.nio.ByteBuffer` backed by the requested plane.
///
/// Kotlin: `NesiumNative.nativePlaneBuffer(idx: Int): ByteBuffer`
///
/// The returned buffer points into the runtime-owned backing store. The caller must ensure
/// `nativeEndFrontCopy()` is called after the upload is finished.
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nativePlaneBuffer(
    mut env: JNIEnv,
    _class: JClass,
    idx: jint,
) -> jobject {
    // ExternalFrameHandle currently uses a small fixed number of planes.
    // Guard against invalid indices to avoid panics.
    if idx < 0 || idx > 1 {
        return std::ptr::null_mut();
    }

    let Some(h) = ensure_runtime().frame_handle.as_deref() else {
        return std::ptr::null_mut();
    };
    let slice = h.plane_slice(idx as usize);

    // DirectByteBuffer enables zero-copy access on the Kotlin side.
    // SAFETY: The backing memory is owned by the runtime and remains valid until
    // `nativeEndFrontCopy()` is called.
    let res = unsafe { env.new_direct_byte_buffer(slice.as_ptr() as *mut u8, slice.len()) };

    match res {
        Ok(buf) => buf.into_raw(),
        Err(e) => {
            tracing::error!("Failed to create direct ByteBuffer: {e}");
            std::ptr::null_mut()
        }
    }
}

/// Ends the front-buffer copy.
///
/// Kotlin: `NesiumNative.nativeEndFrontCopy()`
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nativeEndFrontCopy(
    _env: JNIEnv,
    _class: JClass,
) {
    let Some(h) = ensure_runtime().frame_handle.as_deref() else {
        return;
    };
    h.end_front_copy();
}

/// Returns the current output framebuffer width in pixels.
///
/// Kotlin: `NesiumNative.nativeFrameWidth(): Int`
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

/// Returns the current output framebuffer height in pixels.
///
/// Kotlin: `NesiumNative.nativeFrameHeight(): Int`
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

// --- Auxiliary Texture System ---

/// Creates an auxiliary texture with a specific ID and dimensions.
///
/// Kotlin: `NesiumNative.nesiumAuxCreate(id: Int, width: Int, height: Int)`
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

/// Copies the latest complete frame from an auxiliary texture into a direct ByteBuffer.
///
/// Returns the number of bytes copied.
///
/// Kotlin: `NesiumNative.nesiumAuxCopy(id: Int, dst: ByteBuffer, dstPitch: Int, dstHeight: Int): Int`
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nesiumAuxCopy(
    env: JNIEnv,
    _class: JClass,
    id: jint,
    dst: jobject,
    dst_pitch: jint,
    dst_height: jint,
) -> jint {
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

/// Destroys an auxiliary texture and releases its memory.
///
/// Kotlin: `NesiumNative.nesiumAuxDestroy(id: Int)`
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nesiumAuxDestroy(
    _env: JNIEnv,
    _class: JClass,
    id: jint,
) {
    crate::aux_texture::aux_destroy(id as u32);
}
