use std::{ffi::c_void, sync::OnceLock};

use jni::{
    objects::{GlobalRef, JClass, JObject},
    sys::{jint, jlong, jobject},
    JNIEnv,
};

use nesium_core::ppu::buffer::ColorFormat;

use crate::{FRAME_HEIGHT, FRAME_WIDTH, frame_handle_ref, nesium_runtime_start};

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
    nesium_runtime_start();
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
