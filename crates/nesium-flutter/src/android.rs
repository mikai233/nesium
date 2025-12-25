use std::{
    ffi::{c_uint, c_void},
    os::unix::io::RawFd,
    sync::{
        OnceLock,
        atomic::{AtomicI32, Ordering},
    },
};

use jni::{
    JNIEnv,
    objects::{GlobalRef, JClass, JObject},
    sys::{jint, jlong, jobject},
};

use nesium_core::ppu::buffer::ColorFormat;

use crate::{FRAME_HEIGHT, FRAME_WIDTH, ensure_runtime, frame_handle_ref, nesium_runtime_start};

// Raw syscalls (fcntl/write) for the Android frame signal pipe.
use libc;

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

    // Ensure the runtime is started.
    let runtime = ensure_runtime();
    runtime
        .handle
        .set_frame_ready_callback(Some(android_frame_ready_cb), std::ptr::null_mut())
        .expect("Failed to set frame ready callback");
}

/// Stores the write-end FD for the frame signal pipe and makes it non-blocking.
fn set_frame_signal_fd(fd: RawFd) {
    if fd < 0 {
        return;
    }

    // Best-effort: make the pipe write FD non-blocking so the producer thread never stalls.
    // If this fails, we still store the FD; writes may block if the pipe becomes full.
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

    let seq = frame_handle_ref().frame_seq();
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
    _buffer_index: c_uint,
    _width: c_uint,
    _height: c_uint,
    _pitch: c_uint,
    _user_data: *mut c_void,
) {
    // Must not panic here.
    signal_frame_ready();
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
    let h = frame_handle_ref();
    h.frame_seq() as jlong
}

/// Begins a front-buffer copy and returns the active plane index.
///
/// Kotlin: `NesiumNative.nativeBeginFrontCopy(): Int`
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nativeBeginFrontCopy(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    let h = frame_handle_ref();
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

    let h = frame_handle_ref();
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
    let h = frame_handle_ref();
    h.end_front_copy();
}

/// Returns the current pixel format.
///
/// Kotlin: `NesiumNative.nativeColorFormat(): Int`
///
/// 0 = RGBA8888, 1 = BGRA8888, 2 = Unknown
#[unsafe(no_mangle)]
pub extern "system" fn Java_io_github_mikai233_nesium_NesiumNative_nativeColorFormat(
    _env: JNIEnv,
    _class: JClass,
) -> jint {
    nesium_runtime_start();

    let h = frame_handle_ref();
    match h.color_format() {
        Some(ColorFormat::Rgba8888) => 0,
        Some(ColorFormat::Bgra8888) => 1,
        _ => 2,
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
