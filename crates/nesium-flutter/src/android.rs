use std::{
    collections::VecDeque,
    ffi::{CStr, c_char, c_int, c_uint, c_void},
    os::unix::io::RawFd,
    sync::{
        Arc, Condvar, Mutex, OnceLock,
        atomic::{AtomicBool, AtomicI32, Ordering},
    },
    time::Duration,
};

use jni::{
    JNIEnv,
    objects::{GlobalRef, JClass, JObject},
    sys::{jint, jlong, jobject},
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
    buffers: [*mut AHardwareBuffer; 2],
    pitch_bytes: usize,
    sync_mu: Mutex<AhbSyncState>,
    gpu_busy_cv: Condvar,
    fallback_planes: [Box<[u8]>; 2],
}

#[derive(Clone, Copy)]
struct AhbSyncState {
    gpu_busy: [bool; 2],
    cpu_locked: [bool; 2],
}

// SAFETY: The swapchain buffers are stable native handles; access is coordinated via internal
// atomics/mutexes and the Android NDK AHardwareBuffer APIs are thread-safe.
unsafe impl Send for AhbSwapchain {}
unsafe impl Sync for AhbSwapchain {}

impl AhbSwapchain {
    pub fn new(width: u32, height: u32) -> Self {
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
                panic!("AHardwareBuffer_allocate failed: {res}");
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

        Self {
            buffers,
            pitch_bytes,
            sync_mu: Mutex::new(AhbSyncState {
                gpu_busy: [false; 2],
                cpu_locked: [false; 2],
            }),
            gpu_busy_cv: Condvar::new(),
            fallback_planes,
        }
    }

    pub fn pitch_bytes(&self) -> usize {
        self.pitch_bytes
    }

    pub fn buffer(&self, idx: usize) -> *mut AHardwareBuffer {
        self.buffers[idx]
    }

    fn wait_gpu_idle(&self, idx: usize) {
        let mut guard = match self.sync_mu.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        while guard.gpu_busy[idx] {
            guard = match self.gpu_busy_cv.wait(guard) {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
        }
    }

    fn set_gpu_busy(&self, idx: usize, busy: bool) {
        let mut guard = match self.sync_mu.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        guard.gpu_busy[idx] = busy;
        if !busy {
            self.gpu_busy_cv.notify_all();
        }
    }

    fn lock_plane(&self, idx: usize) -> *mut u8 {
        self.wait_gpu_idle(idx);

        let mut out: *mut c_void;
        let mut last_err: c_int = 0;
        for attempt in 0..6u32 {
            out = std::ptr::null_mut();
            let res = unsafe {
                AHardwareBuffer_lock(
                    self.buffers[idx],
                    AHARDWAREBUFFER_USAGE_CPU_WRITE_OFTEN,
                    -1,
                    std::ptr::null(),
                    &mut out as *mut _,
                )
            };
            if res == 0 && !out.is_null() {
                let mut guard = match self.sync_mu.lock() {
                    Ok(guard) => guard,
                    Err(poisoned) => poisoned.into_inner(),
                };
                guard.cpu_locked[idx] = true;
                return out as *mut u8;
            }
            last_err = res;

            // Short backoff to tolerate transient failures; avoid spinning too aggressively.
            let backoff_ms = (1u64 << attempt).min(16);
            std::thread::sleep(Duration::from_millis(backoff_ms));
        }

        eprintln!(
            "AHardwareBuffer_lock failed for idx={idx} (err={last_err}); falling back to dummy buffer"
        );
        let mut guard = match self.sync_mu.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        guard.cpu_locked[idx] = false;
        self.fallback_planes[idx].as_ptr() as *mut u8
    }

    fn unlock_plane(&self, idx: usize) {
        let should_unlock = {
            let mut guard = match self.sync_mu.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            let was_locked = guard.cpu_locked[idx];
            guard.cpu_locked[idx] = false;
            was_locked
        };
        if !should_unlock {
            return;
        }

        let res = unsafe { AHardwareBuffer_unlock(self.buffers[idx], std::ptr::null_mut()) };
        if res != 0 {
            eprintln!("AHardwareBuffer_unlock failed: {res}");
        }
    }
}

impl Drop for AhbSwapchain {
    fn drop(&mut self) {
        for b in self.buffers {
            if !b.is_null() {
                unsafe { AHardwareBuffer_release(b) };
            }
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

struct RustRendererSignalState {
    queue: VecDeque<u32>,
    renderer_active: bool,
}

struct RustRendererSignal {
    mu: std::sync::Mutex<RustRendererSignalState>,
    cv: std::sync::Condvar,
}

static RUST_RENDERER_SIGNAL: OnceLock<RustRendererSignal> = OnceLock::new();

fn notify_rust_renderer(buffer_index: u32) {
    let signal = rust_renderer_signal();

    let mut state = match signal.mu.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };

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
        println!("[Rust] Android Context initialized via ndk-context");
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
    _width: c_uint,
    _height: c_uint,
    _pitch: c_uint,
    _user_data: *mut c_void,
) {
    // Must not panic here.
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
const EGL_CONTEXT_CLIENT_VERSION: EGLint = 0x3098;
const EGL_OPENGL_ES_API: EGLint = 0x30A0;

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
    let mut state = match signal.mu.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
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
    let Some(swapchain) = get_ahb_swapchain() else {
        eprintln!("nativeStartRustRenderer: AHB swapchain backend not active");
        return;
    };

    let env_ptr = env.get_native_interface();
    let window = unsafe { ANativeWindow_fromSurface(env_ptr, surface.as_raw()) };
    if window.is_null() {
        eprintln!("ANativeWindow_fromSurface failed");
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
    let join = std::thread::spawn(move || {
        let window = window_ptr as *mut ANativeWindow;
        let res = std::panic::catch_unwind(|| unsafe {
            run_rust_renderer(window, swapchain_ref, stop_for_thread);
        });
        if let Err(_) = res {
            eprintln!("Rust renderer thread panicked");
        }
        set_rust_renderer_active(false);
        unsafe { ANativeWindow_release(window) };
    });

    let mut slot = match rust_renderer_slot().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    *slot = Some(RustRendererHandle { stop, join });
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nativeStopRustRenderer(
    _env: JNIEnv,
    _class: JClass,
) {
    set_rust_renderer_active(false);
    let handle = {
        let mut slot = match rust_renderer_slot().lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
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
        eprintln!("shader compile failed: {}", String::from_utf8_lossy(&buf));
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
        eprintln!("program link failed: {}", String::from_utf8_lossy(&buf));
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

struct EglImagesCleanup {
    dpy: EGLDisplay,
    destroy_image: PFNEGLDESTROYIMAGEKHRPROC,
    images: [EGLImageKHR; 2],
}

impl Drop for EglImagesCleanup {
    fn drop(&mut self) {
        unsafe {
            for img in self.images {
                if img != EGL_NO_IMAGE_KHR {
                    let _ = (self.destroy_image)(self.dpy, img);
                }
            }
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
        eprintln!("eglGetDisplay failed");
        return;
    }
    egl.dpy = dpy;
    let mut major: EGLint = 0;
    let mut minor: EGLint = 0;
    if unsafe { eglInitialize(dpy, &mut major as *mut _, &mut minor as *mut _) } == EGL_FALSE {
        eprintln!("eglInitialize failed: 0x{:x}", unsafe { eglGetError() });
        return;
    }
    egl.initialized = true;
    if unsafe { eglBindAPI(EGL_OPENGL_ES_API) } == EGL_FALSE {
        eprintln!("eglBindAPI failed: 0x{:x}", unsafe { eglGetError() });
    }

    let attribs = [
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
    let mut config: EGLConfig = std::ptr::null_mut();
    let mut num: EGLint = 0;
    if unsafe {
        eglChooseConfig(
            dpy,
            attribs.as_ptr(),
            &mut config as *mut _,
            1,
            &mut num as *mut _,
        )
    } == EGL_FALSE
        || num <= 0
    {
        eprintln!("eglChooseConfig failed: 0x{:x}", unsafe { eglGetError() });
        return;
    }

    let ctx_attribs = [EGL_CONTEXT_CLIENT_VERSION, 2, EGL_NONE];
    let ctx = unsafe { eglCreateContext(dpy, config, EGL_NO_CONTEXT, ctx_attribs.as_ptr()) };
    if ctx == EGL_NO_CONTEXT {
        eprintln!("eglCreateContext failed: 0x{:x}", unsafe { eglGetError() });
        return;
    }
    egl.ctx = ctx;

    let surf = unsafe { eglCreateWindowSurface(dpy, config, window, [EGL_NONE].as_ptr()) };
    if surf == EGL_NO_SURFACE {
        eprintln!("eglCreateWindowSurface failed: 0x{:x}", unsafe {
            eglGetError()
        });
        return;
    }
    egl.surf = surf;

    if unsafe { eglMakeCurrent(dpy, surf, surf, ctx) } == EGL_FALSE {
        eprintln!("eglMakeCurrent failed: 0x{:x}", unsafe { eglGetError() });
        return;
    }

    let get_native_client_buffer: PFNEGLGETNATIVECLIENTBUFFERANDROIDPROC =
        match unsafe { egl_proc(b"eglGetNativeClientBufferANDROID\0") } {
            Some(p) => p,
            None => {
                eprintln!("missing eglGetNativeClientBufferANDROID");
                return;
            }
        };
    let egl_create_image: PFNEGLCREATEIMAGEKHRPROC =
        match unsafe { egl_proc(b"eglCreateImageKHR\0") } {
            Some(p) => p,
            None => {
                eprintln!("missing eglCreateImageKHR");
                return;
            }
        };
    let egl_destroy_image: PFNEGLDESTROYIMAGEKHRPROC =
        match unsafe { egl_proc(b"eglDestroyImageKHR\0") } {
            Some(p) => p,
            None => {
                eprintln!("missing eglDestroyImageKHR");
                return;
            }
        };
    let gl_egl_image_target_texture: PFNGLEGLIMAGETARGETTEXTURE2DOESPROC =
        match unsafe { egl_proc(b"glEGLImageTargetTexture2DOES\0") } {
            Some(p) => p,
            None => {
                eprintln!("missing glEGLImageTargetTexture2DOES");
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
        println!("[RustRenderer] GL_EXTENSIONS: {}", ext);
    }

    unsafe { glDisable(GL_DITHER) };
    let _ = unsafe { eglSwapInterval(dpy, 1) };

    // Create textures backed by the two AHardwareBuffers.
    let mut textures = [0u32; 2];
    unsafe { glGenTextures(2, textures.as_mut_ptr()) };

    let mut images = [EGL_NO_IMAGE_KHR; 2];
    for i in 0..2 {
        unsafe {
            glBindTexture(GL_TEXTURE_2D, textures[i]);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER, GL_NEAREST);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER, GL_NEAREST);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S, GL_CLAMP_TO_EDGE);
            glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T, GL_CLAMP_TO_EDGE);
        }

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
            eprintln!("eglCreateImageKHR failed: 0x{:x}", unsafe { eglGetError() });
            continue;
        }
        images[i] = image;
        unsafe { gl_egl_image_target_texture(GL_TEXTURE_2D, image as *const c_void) };
    }

    let _images_cleanup = EglImagesCleanup {
        dpy,
        destroy_image: egl_destroy_image,
        images,
    };

    let vs_src = CStr::from_bytes_with_nul(
        b"attribute vec4 a_position;\nattribute vec2 a_tex_coord;\nvarying vec2 v_tex_coord;\nvoid main() {\n  gl_Position = a_position;\n  v_tex_coord = a_tex_coord;\n}\n\0",
    )
    .expect("vertex shader source must be NUL-terminated");
    let fs_src = CStr::from_bytes_with_nul(
        b"precision mediump float;\nuniform sampler2D u_texture;\nvarying vec2 v_tex_coord;\nvoid main() {\n  gl_FragColor = texture2D(u_texture, v_tex_coord);\n}\n\0",
    )
    .expect("fragment shader source must be NUL-terminated");
    let Some(vs) = compile_shader(GL_VERTEX_SHADER, vs_src) else {
        return;
    };
    let Some(fs) = compile_shader(GL_FRAGMENT_SHADER, fs_src) else {
        return;
    };
    let Some(program) = link_program(vs, fs) else {
        return;
    };
    unsafe { glUseProgram(program) };

    let a_position =
        unsafe { glGetAttribLocation(program, b"a_position\0".as_ptr() as *const c_char) };
    let a_tex = unsafe { glGetAttribLocation(program, b"a_tex_coord\0".as_ptr() as *const c_char) };
    let u_tex = unsafe { glGetUniformLocation(program, b"u_texture\0".as_ptr() as *const c_char) };
    unsafe { glUniform1i(u_tex, 0) };

    let vertex_data: [f32; 16] = [
        // X,   Y,   U,   V
        -1.0, -1.0, 0.0, 1.0, // bottom-left
        1.0, -1.0, 1.0, 1.0, // bottom-right
        -1.0, 1.0, 0.0, 0.0, // top-left
        1.0, 1.0, 1.0, 0.0, // top-right
    ];
    let stride = (4 * std::mem::size_of::<f32>()) as c_int;
    let base_ptr = vertex_data.as_ptr() as *const c_void;
    unsafe {
        glEnableVertexAttribArray(a_position as u32);
        glVertexAttribPointer(
            a_position as u32,
            2,
            0x1406, /* GL_FLOAT */
            0,
            stride,
            base_ptr,
        );
        glEnableVertexAttribArray(a_tex as u32);
        glVertexAttribPointer(
            a_tex as u32,
            2,
            0x1406, /* GL_FLOAT */
            0,
            stride,
            (vertex_data.as_ptr().add(2)) as *const c_void,
        );
    }

    // Render loop: wait for frame-ready signals from `android_frame_ready_cb`.
    let signal = rust_renderer_signal();

    while !stop.load(Ordering::Acquire) {
        let msg = {
            let mut state = match signal.mu.lock() {
                Ok(guard) => guard,
                Err(poisoned) => poisoned.into_inner(),
            };
            while state.queue.is_empty() && !stop.load(Ordering::Acquire) {
                let (g, _) = match signal.cv.wait_timeout(state, Duration::from_millis(500)) {
                    Ok(res) => res,
                    Err(poisoned) => poisoned.into_inner(),
                };
                state = g;
            }
            if stop.load(Ordering::Acquire) {
                None
            } else {
                state.queue.pop_front()
            }
        };

        let Some(buffer_index) = msg else {
            break;
        };
        let idx = buffer_index as usize;

        let _busy = GpuBusyGuard::new(swapchain, idx);

        unsafe {
            glViewport(0, 0, FRAME_WIDTH as c_int, FRAME_HEIGHT as c_int);
            glClearColor(0.0, 0.0, 0.0, 1.0);
            glClear(GL_COLOR_BUFFER_BIT);
        }

        unsafe {
            glActiveTexture(GL_TEXTURE0);
            glBindTexture(GL_TEXTURE_2D, textures[idx]);
            glDrawArrays(GL_TRIANGLE_STRIP, 0, 4);
        }

        if unsafe { eglSwapBuffers(dpy, surf) } == EGL_FALSE {
            eprintln!("eglSwapBuffers failed: 0x{:x}", unsafe { eglGetError() });
            wait_for_gpu(dpy);
            break;
        }
        wait_for_gpu(dpy);
    }

    unsafe {
        glDeleteTextures(2, textures.as_ptr());
        glDeleteProgram(program);
        glFlush();
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
    let Some(h) = ensure_runtime().frame_handle.as_ref() else {
        return -1;
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

    let Some(h) = ensure_runtime().frame_handle.as_ref() else {
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
            eprintln!("Failed to create direct ByteBuffer: {e}");
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
    if let Some(h) = ensure_runtime().frame_handle.as_ref() {
        h.end_front_copy();
    }
}

/// Returns the fixed NES framebuffer width in pixels.
///
/// Kotlin: `NesiumNative.nativeFrameWidth(): Int`
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nativeFrameWidth(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    FRAME_WIDTH as jint
}

/// Returns the fixed NES framebuffer height in pixels.
///
/// Kotlin: `NesiumNative.nativeFrameHeight(): Int`
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nativeFrameHeight(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    FRAME_HEIGHT as jint
}
