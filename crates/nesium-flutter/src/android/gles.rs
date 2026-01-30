use super::ahb::AHardwareBuffer;
use jni::sys::{JNIEnv, jobject};
use std::ffi::{c_char, c_int, c_void};

pub type EGLDisplay = *mut c_void;
pub type EGLContext = *mut c_void;
pub type EGLSurface = *mut c_void;
pub type EGLConfig = *mut c_void;
pub type EGLClientBuffer = *mut c_void;
pub type EGLImageKHR = *mut c_void;
pub type EGLSyncKHR = *mut c_void;
pub type EGLBoolean = c_int;
pub type EGLint = c_int;
pub type EGLTimeKHR = u64;
pub type EGLNativeDisplayType = *mut c_void;

#[repr(C)]
pub struct ANativeWindow {
    _private: [u8; 0],
}
pub type EGLNativeWindowType = *mut ANativeWindow;

pub const EGL_FALSE: EGLBoolean = 0;
pub const EGL_TRUE: EGLBoolean = 1;
pub const EGL_DEFAULT_DISPLAY: EGLNativeDisplayType = std::ptr::null_mut();
pub const EGL_NO_DISPLAY: EGLDisplay = std::ptr::null_mut();
pub const EGL_NO_CONTEXT: EGLContext = std::ptr::null_mut();
pub const EGL_NO_SURFACE: EGLSurface = std::ptr::null_mut();
pub const EGL_NO_IMAGE_KHR: EGLImageKHR = std::ptr::null_mut();
pub const EGL_NO_SYNC_KHR: EGLSyncKHR = std::ptr::null_mut();

pub const EGL_NONE: EGLint = 0x3038;
pub const EGL_RED_SIZE: EGLint = 0x3024;
pub const EGL_GREEN_SIZE: EGLint = 0x3023;
pub const EGL_BLUE_SIZE: EGLint = 0x3022;
pub const EGL_ALPHA_SIZE: EGLint = 0x3021;
pub const EGL_RENDERABLE_TYPE: EGLint = 0x3040;
pub const EGL_SURFACE_TYPE: EGLint = 0x3033;
pub const EGL_WINDOW_BIT: EGLint = 0x0004;
pub const EGL_OPENGL_ES2_BIT: EGLint = 0x0004;
pub const EGL_OPENGL_ES3_BIT_KHR: EGLint = 0x00000040;
pub const EGL_CONTEXT_CLIENT_VERSION: EGLint = 0x3098;
pub const EGL_OPENGL_ES_API: EGLint = 0x30A0;
pub const EGL_WIDTH: EGLint = 0x3057;
pub const EGL_HEIGHT: EGLint = 0x3056;

pub const EGL_NATIVE_BUFFER_ANDROID: EGLint = 0x3140;
pub const EGL_IMAGE_PRESERVED_KHR: EGLint = 0x30D2;

pub const EGL_SYNC_FENCE_KHR: EGLint = 0x30F9;
pub const EGL_SYNC_FLUSH_COMMANDS_BIT_KHR: EGLint = 0x0001;
pub const EGL_FOREVER_KHR: EGLTimeKHR = 0xFFFFFFFFFFFFFFFF;

#[link(name = "android")]
unsafe extern "C" {
    pub fn ANativeWindow_fromSurface(env: *mut JNIEnv, surface: jobject) -> *mut ANativeWindow;
    pub fn ANativeWindow_release(window: *mut ANativeWindow);
}

#[link(name = "EGL")]
unsafe extern "C" {
    pub fn eglGetDisplay(display_id: EGLNativeDisplayType) -> EGLDisplay;
    pub fn eglInitialize(dpy: EGLDisplay, major: *mut EGLint, minor: *mut EGLint) -> EGLBoolean;
    pub fn eglTerminate(dpy: EGLDisplay) -> EGLBoolean;
    pub fn eglBindAPI(api: EGLint) -> EGLBoolean;
    pub fn eglChooseConfig(
        dpy: EGLDisplay,
        attrib_list: *const EGLint,
        configs: *mut EGLConfig,
        config_size: EGLint,
        num_config: *mut EGLint,
    ) -> EGLBoolean;
    pub fn eglCreateContext(
        dpy: EGLDisplay,
        config: EGLConfig,
        share_context: EGLContext,
        attrib_list: *const EGLint,
    ) -> EGLContext;
    pub fn eglDestroyContext(dpy: EGLDisplay, ctx: EGLContext) -> EGLBoolean;
    pub fn eglCreateWindowSurface(
        dpy: EGLDisplay,
        config: EGLConfig,
        win: EGLNativeWindowType,
        attrib_list: *const EGLint,
    ) -> EGLSurface;
    pub fn eglDestroySurface(dpy: EGLDisplay, surface: EGLSurface) -> EGLBoolean;
    pub fn eglMakeCurrent(
        dpy: EGLDisplay,
        draw: EGLSurface,
        read: EGLSurface,
        ctx: EGLContext,
    ) -> EGLBoolean;
    pub fn eglQuerySurface(
        dpy: EGLDisplay,
        surface: EGLSurface,
        attribute: EGLint,
        value: *mut EGLint,
    ) -> EGLBoolean;
    pub fn eglSwapInterval(dpy: EGLDisplay, interval: EGLint) -> EGLBoolean;
    pub fn eglSwapBuffers(dpy: EGLDisplay, surface: EGLSurface) -> EGLBoolean;
    pub fn eglGetProcAddress(procname: *const c_char) -> *const c_void;
    pub fn eglGetError() -> EGLint;
}

#[link(name = "GLESv2")]
unsafe extern "C" {
    pub fn glGenTextures(n: c_int, textures: *mut u32);
    pub fn glDeleteTextures(n: c_int, textures: *const u32);
    pub fn glBindTexture(target: u32, texture: u32);
    pub fn glTexParameteri(target: u32, pname: u32, param: c_int);
    pub fn glFinish();
}

pub const GL_TEXTURE_2D: u32 = 0x0DE1;
pub const GL_TEXTURE_MIN_FILTER: u32 = 0x2801;
pub const GL_TEXTURE_MAG_FILTER: u32 = 0x2800;
pub const GL_TEXTURE_WRAP_S: u32 = 0x2802;
pub const GL_TEXTURE_WRAP_T: u32 = 0x2803;
pub const GL_NEAREST: c_int = 0x2600;
pub const GL_CLAMP_TO_EDGE: c_int = 0x812F;
pub const GL_RGBA8: u32 = 0x8058;
pub const GL_RGBA: u32 = 0x1908;
pub const GL_UNSIGNED_BYTE: u32 = 0x1401;

pub type PFNEGLGETNATIVECLIENTBUFFERANDROIDPROC =
    unsafe extern "C" fn(buffer: *mut AHardwareBuffer) -> EGLClientBuffer;
pub type PFNEGLCREATEIMAGEKHRPROC = unsafe extern "C" fn(
    dpy: EGLDisplay,
    ctx: EGLContext,
    target: EGLint,
    buffer: EGLClientBuffer,
    attrib_list: *const EGLint,
) -> EGLImageKHR;
pub type PFNEGLDESTROYIMAGEKHRPROC =
    unsafe extern "C" fn(dpy: EGLDisplay, image: EGLImageKHR) -> EGLBoolean;
pub type PFNGLEGLIMAGETARGETTEXTURE2DOESPROC =
    unsafe extern "C" fn(target: u32, image: *const c_void);
pub type PFNEGLCREATESYNCKHRPROC =
    unsafe extern "C" fn(dpy: EGLDisplay, ty: EGLint, attrib_list: *const EGLint) -> EGLSyncKHR;
pub type PFNEGLDESTROYSYNCKHRPROC =
    unsafe extern "C" fn(dpy: EGLDisplay, sync: EGLSyncKHR) -> EGLBoolean;
pub type PFNEGLCLIENTWAITSYNCKHRPROC = unsafe extern "C" fn(
    dpy: EGLDisplay,
    sync: EGLSyncKHR,
    flags: EGLint,
    timeout: EGLTimeKHR,
) -> EGLint;

pub unsafe fn egl_proc<T>(name: &'static [u8]) -> Option<T> {
    debug_assert!(name.last() == Some(&0));
    let ptr = unsafe { eglGetProcAddress(name.as_ptr() as *const c_char) };
    if ptr.is_null() {
        return None;
    }
    Some(unsafe { std::mem::transmute_copy::<*const c_void, T>(&ptr) })
}

pub struct EglCleanup {
    pub dpy: EGLDisplay,
    pub ctx: EGLContext,
    pub surf: EGLSurface,
    pub initialized: bool,
}

impl EglCleanup {
    pub fn new() -> Self {
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
