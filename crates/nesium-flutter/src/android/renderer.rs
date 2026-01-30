use librashader::runtime::Size as LibrashaderSize;
use librashader::runtime::Viewport as LibrashaderViewport;
use librashader::runtime::gl::FilterChainOptions as LibrashaderFilterChainOptions;
use librashader::runtime::gl::FrameOptions as LibrashaderFrameOptions;
use librashader::runtime::gl::GLImage as LibrashaderGlImage;

use crate::api::video::{ShaderParameter, ShaderParameters};
use crate::runtime_handle;
use glow::HasContext;
use std::num::NonZeroU32;

use parking_lot::{Condvar, Mutex};
use std::collections::VecDeque;
use std::ffi::c_void;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::time::Duration;

use super::ahb::{AhbSwapchain, GpuBusyGuard, release_buffers};
use super::gles::*;
use super::session::{
    ANDROID_SHADER_SESSION, PENDING_SHADER_DATA, ShaderSession, android_shader_snapshot,
};

pub struct RustRendererHandle {
    pub stop: Arc<AtomicBool>,
    pub join: std::thread::JoinHandle<()>,
}

pub struct RustRendererSignalState {
    pub queue: VecDeque<u32>,
    pub renderer_active: bool,
}

pub struct RustRendererSignal {
    pub mu: Mutex<RustRendererSignalState>,
    pub cv: Condvar,
}

static RUST_RENDERER_SIGNAL: std::sync::OnceLock<RustRendererSignal> = std::sync::OnceLock::new();

pub fn rust_renderer_signal() -> &'static RustRendererSignal {
    RUST_RENDERER_SIGNAL.get_or_init(|| RustRendererSignal {
        mu: Mutex::new(RustRendererSignalState {
            queue: VecDeque::new(),
            renderer_active: false,
        }),
        cv: Condvar::new(),
    })
}

pub fn rust_renderer_wake() {
    let signal = rust_renderer_signal();
    signal.cv.notify_all();
}

pub fn notify_rust_renderer(buffer_index: u32) {
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

pub fn set_rust_renderer_active(active: bool) {
    let signal = rust_renderer_signal();
    let mut state = signal.mu.lock();
    state.renderer_active = active;
    if !active {
        state.queue.clear();
    }
    signal.cv.notify_all();
}

static RUST_RENDERER_TID: AtomicI32 = AtomicI32::new(-1);

pub fn try_raise_current_thread_priority() {
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

pub fn apply_rust_renderer_priority(enabled: bool) {
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

pub fn store_rust_renderer_tid(tid: i32) {
    RUST_RENDERER_TID.store(tid, Ordering::Release);
}

pub fn signal_frame_ready() {
    let fd = super::FRAME_SIGNAL_FD.load(Ordering::Acquire);
    if fd < 0 {
        return;
    }

    let seq = runtime_handle().frame_seq();
    let token = seq.to_le_bytes();

    let mut written = 0usize;
    while written < token.len() {
        let ptr = unsafe { token.as_ptr().add(written) } as *const std::ffi::c_void;
        let len = token.len() - written;

        let res = unsafe { libc::write(fd as libc::c_int, ptr, len) };
        if res > 0 {
            written += res as usize;
            continue;
        }

        let err = std::io::Error::last_os_error();
        match err.raw_os_error() {
            Some(code) if code == libc::EINTR => continue,
            Some(code) if code == libc::EAGAIN || code == libc::EWOULDBLOCK => return,
            Some(code) if code == libc::EBADF || code == libc::EPIPE => {
                super::FRAME_SIGNAL_FD.store(-1, Ordering::Release);
                return;
            }
            _ => return,
        }
    }
}

pub unsafe fn run_rust_renderer(
    window: *mut ANativeWindow,
    swapchain: &'static AhbSwapchain,
    stop: Arc<AtomicBool>,
) {
    let mut renderer = match GlesRenderer::new(window, swapchain) {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Failed to initialize GLES renderer: {}", e);
            return;
        }
    };

    if let Err(e) = renderer.run(swapchain, stop) {
        tracing::error!("GLES renderer error: {}", e);
    }
}

struct GlesRenderer {
    egl: super::gles::EglCleanup,
    glow_ctx: Arc<glow::Context>,
    blit_program: Option<glow::Program>,
    quad_vbo: Option<glow::Buffer>,
    quad_vao: Option<glow::VertexArray>,

    // Textures and images managed by the swapchain
    textures: [u32; 2],
    images: [super::gles::EGLImageKHR; 2],
    seen_generation: u32,

    // Shader related
    shader_seen_generation: u64,
    shader_output_tex: Option<glow::Texture>,
    shader_output_size: LibrashaderSize<u32>,
    frame_count: usize,

    // FFI pointers
    egl_procs: EglProcs,
}

struct EglProcs {
    get_native_client_buffer: PFNEGLGETNATIVECLIENTBUFFERANDROIDPROC,
    egl_create_image: PFNEGLCREATEIMAGEKHRPROC,
    egl_destroy_image: PFNEGLDESTROYIMAGEKHRPROC,
    gl_egl_image_target_texture: PFNGLEGLIMAGETARGETTEXTURE2DOESPROC,
    fence_sync: Option<(
        PFNEGLCREATESYNCKHRPROC,
        PFNEGLDESTROYSYNCKHRPROC,
        PFNEGLCLIENTWAITSYNCKHRPROC,
    )>,
}

impl EglProcs {
    unsafe fn load() -> Result<Self, String> {
        let get_native_client_buffer = unsafe { egl_proc(b"eglGetNativeClientBufferANDROID\0") }
            .ok_or_else(|| "missing eglGetNativeClientBufferANDROID".to_string())?;
        let egl_create_image = unsafe { egl_proc(b"eglCreateImageKHR\0") }
            .ok_or_else(|| "missing eglCreateImageKHR".to_string())?;
        let egl_destroy_image = unsafe { egl_proc(b"eglDestroyImageKHR\0") }
            .ok_or_else(|| "missing eglDestroyImageKHR".to_string())?;
        let gl_egl_image_target_texture = unsafe { egl_proc(b"glEGLImageTargetTexture2DOES\0") }
            .ok_or_else(|| "missing glEGLImageTargetTexture2DOES".to_string())?;

        let fence_sync = match unsafe {
            (
                egl_proc::<PFNEGLCREATESYNCKHRPROC>(b"eglCreateSyncKHR\0"),
                egl_proc::<PFNEGLDESTROYSYNCKHRPROC>(b"eglDestroySyncKHR\0"),
                egl_proc::<PFNEGLCLIENTWAITSYNCKHRPROC>(b"eglClientWaitSyncKHR\0"),
            )
        } {
            (Some(create), Some(destroy), Some(wait)) => Some((create, destroy, wait)),
            _ => None,
        };

        Ok(Self {
            get_native_client_buffer,
            egl_create_image,
            egl_destroy_image,
            gl_egl_image_target_texture,
            fence_sync,
        })
    }
}

impl GlesRenderer {
    fn new(window: *mut ANativeWindow, swapchain: &'static AhbSwapchain) -> Result<Self, String> {
        let mut egl = EglCleanup::new();

        // SAFETY: eglGetDisplay with EGL_DEFAULT_DISPLAY is standard.
        let dpy = unsafe { eglGetDisplay(EGL_DEFAULT_DISPLAY) };
        if dpy == EGL_NO_DISPLAY {
            return Err("eglGetDisplay failed".to_string());
        }
        egl.dpy = dpy;

        let mut major: EGLint = 0;
        let mut minor: EGLint = 0;
        // SAFETY: Pointer to local EGLint is safe.
        if unsafe { eglInitialize(dpy, &mut major as *mut _, &mut minor as *mut _) } == EGL_FALSE {
            return Err(format!("eglInitialize failed: 0x{:x}", unsafe {
                eglGetError()
            }));
        }
        egl.initialized = true;

        // SAFETY: Constant value EGL_OPENGL_ES_API.
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

        let mut egl_config: EGLConfig = std::ptr::null_mut();
        let mut num: EGLint = 0;
        // SAFETY: Pointer to local variables or static array is safe.
        let choose_ok = unsafe {
            eglChooseConfig(
                dpy,
                attribs_es3.as_ptr(),
                &mut egl_config as *mut _,
                1,
                &mut num as *mut _,
            )
        } != EGL_FALSE
            && num > 0;

        let egl_config = if choose_ok {
            egl_config
        } else {
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
            // SAFETY: Pointer to local variables or static array is safe.
            if unsafe {
                eglChooseConfig(
                    dpy,
                    attribs_es2.as_ptr(),
                    &mut egl_config as *mut _,
                    1,
                    &mut num as *mut _,
                )
            } == EGL_FALSE
                || num <= 0
            {
                return Err(format!("eglChooseConfig failed: 0x{:x}", unsafe {
                    eglGetError()
                }));
            }
            egl_config
        };

        let mut ctx_version: EGLint = 3;
        // SAFETY: Constant values and shared dpy/config.
        let ctx = unsafe {
            eglCreateContext(
                dpy,
                egl_config,
                EGL_NO_CONTEXT,
                [EGL_CONTEXT_CLIENT_VERSION, 3, EGL_NONE].as_ptr(),
            )
        };
        let ctx = if ctx == EGL_NO_CONTEXT {
            ctx_version = 2;
            // SAFETY: Fallback to ES2.
            unsafe {
                eglCreateContext(
                    dpy,
                    egl_config,
                    EGL_NO_CONTEXT,
                    [EGL_CONTEXT_CLIENT_VERSION, 2, EGL_NONE].as_ptr(),
                )
            }
        } else {
            ctx
        };

        if ctx == EGL_NO_CONTEXT {
            return Err(format!("eglCreateContext failed: 0x{:x}", unsafe {
                eglGetError()
            }));
        }
        egl.ctx = ctx;

        // SAFETY: window pointer is assumed valid for the duration of the renderer thread.
        let surf = unsafe { eglCreateWindowSurface(dpy, egl_config, window, [EGL_NONE].as_ptr()) };
        if surf == EGL_NO_SURFACE {
            return Err(format!("eglCreateWindowSurface failed: 0x{:x}", unsafe {
                eglGetError()
            }));
        }
        egl.surf = surf;

        // SAFETY: Basic context activation.
        if unsafe { eglMakeCurrent(dpy, surf, surf, ctx) } == EGL_FALSE {
            return Err(format!("eglMakeCurrent failed: 0x{:x}", unsafe {
                eglGetError()
            }));
        }

        // SAFETY: glow creation from loader function.
        let glow_ctx = Arc::new(unsafe {
            glow::Context::from_loader_function(|name| {
                let name = match std::ffi::CString::new(name) {
                    Ok(v) => v,
                    Err(_) => return std::ptr::null(),
                };
                eglGetProcAddress(name.as_ptr()) as *const c_void
            })
        });

        // SAFETY: proc loading is internal.
        let egl_procs = unsafe { EglProcs::load()? };

        // SAFETY: Basic GL configuration.
        unsafe { glow_ctx.disable(glow::DITHER) };
        let _ = unsafe { eglSwapInterval(dpy, 1) };

        let textures = [0u32; 2];
        let images = [EGL_NO_IMAGE_KHR; 2];
        let seen_generation = swapchain.generation();

        let mut this = Self {
            egl,
            glow_ctx,
            blit_program: None,
            quad_vbo: None,
            quad_vao: None,
            textures,
            images,
            seen_generation,
            shader_seen_generation: 0,
            shader_output_tex: None,
            shader_output_size: LibrashaderSize {
                width: 0,
                height: 0,
            },
            frame_count: 0,
            egl_procs,
        };

        this.recreate_textures_and_images(swapchain)?;

        let is_gles3 = this.glow_ctx.version().is_embedded
            && this.glow_ctx.version().major >= 3
            && ctx_version >= 3;

        this.initialize_blit_resources(is_gles3)?;

        Ok(this)
    }

    fn recreate_textures_and_images(
        &mut self,
        swapchain: &'static AhbSwapchain,
    ) -> Result<(), String> {
        let dpy = self.egl.dpy;

        // Cleanup retired buffers if any.
        for retired in swapchain.take_retired_buffers() {
            // SAFETY: AHB release is safe for retired buffers.
            unsafe { release_buffers(retired) };
        }

        // SAFETY: Basic resource cleanup.
        unsafe {
            if self.textures[0] != 0 {
                glDeleteTextures(2, self.textures.as_ptr());
                self.textures = [0, 0];
            }
            for img in self.images.iter_mut() {
                if *img != EGL_NO_IMAGE_KHR {
                    (self.egl_procs.egl_destroy_image)(dpy, *img);
                    *img = EGL_NO_IMAGE_KHR;
                }
            }
        }

        // SAFETY: glGenTextures with array of 2.
        unsafe { glGenTextures(2, self.textures.as_mut_ptr()) };

        for i in 0..2 {
            // SAFETY: Binding generated textures.
            unsafe { glBindTexture(GL_TEXTURE_2D, self.textures[i]) };

            // SAFETY: AHB buffer handle from swapchain.
            let client_buf =
                unsafe { (self.egl_procs.get_native_client_buffer)(swapchain.buffer(i)) };
            let img_attribs = [EGL_IMAGE_PRESERVED_KHR, EGL_TRUE, EGL_NONE];

            // SAFETY: egl_create_image with AHB buffer.
            let image = unsafe {
                (self.egl_procs.egl_create_image)(
                    dpy,
                    EGL_NO_CONTEXT,
                    EGL_NATIVE_BUFFER_ANDROID,
                    client_buf,
                    img_attribs.as_ptr(),
                )
            };

            if image == EGL_NO_IMAGE_KHR {
                return Err(format!("eglCreateImageKHR failed: 0x{:x}", unsafe {
                    eglGetError()
                }));
            }
            self.images[i] = image;

            // SAFETY: target texture from EGL image.
            unsafe {
                (self.egl_procs.gl_egl_image_target_texture)(GL_TEXTURE_2D, image as *const c_void)
            };

            // SAFETY: Basic texture parameters.
            unsafe {
                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MIN_FILTER as u32, GL_NEAREST);
                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_MAG_FILTER as u32, GL_NEAREST);
                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_S as u32, GL_CLAMP_TO_EDGE);
                glTexParameteri(GL_TEXTURE_2D, GL_TEXTURE_WRAP_T as u32, GL_CLAMP_TO_EDGE);
            }
        }
        Ok(())
    }

    fn initialize_blit_resources(&mut self, is_gles3: bool) -> Result<(), String> {
        let (vs_src, fs_src) = if is_gles3 {
            (
                "#version 300 es\nprecision highp float; layout(location = 0) in vec2 a_position; layout(location = 1) in vec2 a_tex_coord; out vec2 v_tex_coord; void main() { gl_Position = vec4(a_position, 0.0, 1.0); v_tex_coord = a_tex_coord; }",
                "#version 300 es\nprecision mediump float; uniform sampler2D u_texture; in vec2 v_tex_coord; out vec4 o_color; void main() { o_color = texture(u_texture, v_tex_coord); }",
            )
        } else {
            (
                "attribute vec2 a_position; attribute vec2 a_tex_coord; varying vec2 v_tex_coord; void main() { gl_Position = vec4(a_position, 0.0, 1.0); v_tex_coord = a_tex_coord; }",
                "precision mediump float; uniform sampler2D u_texture; varying vec2 v_tex_coord; void main() { gl_FragColor = texture2D(u_texture, v_tex_coord); }",
            )
        };

        // SAFETY: glow operations are safe unless specified.
        unsafe {
            let vs = self.glow_ctx.create_shader(glow::VERTEX_SHADER)?;
            self.glow_ctx.shader_source(vs, vs_src);
            self.glow_ctx.compile_shader(vs);
            if !self.glow_ctx.get_shader_compile_status(vs) {
                return Err(format!(
                    "VS compile error: {}",
                    self.glow_ctx.get_shader_info_log(vs)
                ));
            }

            let fs = self.glow_ctx.create_shader(glow::FRAGMENT_SHADER)?;
            self.glow_ctx.shader_source(fs, fs_src);
            self.glow_ctx.compile_shader(fs);
            if !self.glow_ctx.get_shader_compile_status(fs) {
                return Err(format!(
                    "FS compile error: {}",
                    self.glow_ctx.get_shader_info_log(fs)
                ));
            }

            let program = self.glow_ctx.create_program()?;
            self.glow_ctx.attach_shader(program, vs);
            self.glow_ctx.attach_shader(program, fs);
            if !is_gles3 {
                self.glow_ctx.bind_attrib_location(program, 0, "a_position");
                self.glow_ctx
                    .bind_attrib_location(program, 1, "a_tex_coord");
            }
            self.glow_ctx.link_program(program);
            if !self.glow_ctx.get_program_link_status(program) {
                return Err(format!(
                    "Program link error: {}",
                    self.glow_ctx.get_program_info_log(program)
                ));
            }

            self.glow_ctx.delete_shader(vs);
            self.glow_ctx.delete_shader(fs);

            self.glow_ctx.use_program(Some(program));
            if let Some(loc) = self.glow_ctx.get_uniform_location(program, "u_texture") {
                self.glow_ctx.uniform_1_i32(Some(&loc), 0);
            }
            self.glow_ctx.use_program(None);
            self.blit_program = Some(program);

            let quad: [f32; 16] = [
                -1.0, -1.0, 0.0, 1.0, 1.0, -1.0, 1.0, 1.0, -1.0, 1.0, 0.0, 0.0, 1.0, 1.0, 1.0, 0.0,
            ];
            let quad_vbo = self.glow_ctx.create_buffer()?;
            let bytes = std::slice::from_raw_parts(
                quad.as_ptr() as *const u8,
                std::mem::size_of_val(&quad),
            );
            self.glow_ctx
                .bind_buffer(glow::ARRAY_BUFFER, Some(quad_vbo));
            self.glow_ctx
                .buffer_data_u8_slice(glow::ARRAY_BUFFER, bytes, glow::STATIC_DRAW);
            self.quad_vbo = Some(quad_vbo);

            if is_gles3 {
                let vao = self.glow_ctx.create_vertex_array()?;
                self.glow_ctx.bind_vertex_array(Some(vao));
                self.glow_ctx
                    .bind_buffer(glow::ARRAY_BUFFER, Some(quad_vbo));
                let stride = 4 * 4;
                self.glow_ctx.enable_vertex_attrib_array(0);
                self.glow_ctx
                    .vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);
                self.glow_ctx.enable_vertex_attrib_array(1);
                self.glow_ctx
                    .vertex_attrib_pointer_f32(1, 2, glow::FLOAT, false, stride, 2 * 4);
                self.glow_ctx.bind_vertex_array(None);
                self.quad_vao = Some(vao);
            }
        }
        Ok(())
    }

    fn run(
        &mut self,
        swapchain: &'static AhbSwapchain,
        stop: Arc<AtomicBool>,
    ) -> Result<(), String> {
        let signal = rust_renderer_signal();
        let is_gles3 = self.quad_vao.is_some();

        while !stop.load(Ordering::Acquire) {
            let generation = swapchain.generation();
            if generation != self.seen_generation {
                self.wait_for_gpu();
                self.recreate_textures_and_images(swapchain)?;
                self.seen_generation = generation;
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

            let Some(buffer_index) = msg else {
                continue;
            };
            let idx = buffer_index as usize;

            // SAFETY: swapchain buffer index is from signal queue.
            let _busy = GpuBusyGuard::new(swapchain, idx as u32);

            let mut surf_w: EGLint = 0;
            let mut surf_h: EGLint = 0;
            // SAFETY: display and surface are established for the lifetime of run_rust_renderer.
            unsafe {
                let _ = eglQuerySurface(self.egl.dpy, self.egl.surf, EGL_WIDTH, &mut surf_w);
                let _ = eglQuerySurface(self.egl.dpy, self.egl.surf, EGL_HEIGHT, &mut surf_h);
            }
            let surf_w = (surf_w as i32).max(1) as u32;
            let surf_h = (surf_h as i32).max(1) as u32;

            if is_gles3 {
                self.update_shader_state(surf_w, surf_h)?;
            }
            let mut present_tex_id = self.textures[idx];
            if is_gles3 {
                if let Ok(Some(shader_tex)) = self.render_shader_pass(idx, swapchain) {
                    present_tex_id = shader_tex;
                }
            }

            self.frame_count = self.frame_count.wrapping_add(1);

            // Blit pass
            unsafe {
                self.glow_ctx.viewport(0, 0, surf_w as i32, surf_h as i32);
                self.glow_ctx.clear_color(0.0, 0.0, 0.0, 1.0);
                self.glow_ctx.clear(glow::COLOR_BUFFER_BIT);
                self.glow_ctx.disable(glow::CULL_FACE);
                self.glow_ctx.disable(glow::BLEND);
                self.glow_ctx.disable(glow::DEPTH_TEST);
                self.glow_ctx.use_program(self.blit_program);
                self.glow_ctx.active_texture(glow::TEXTURE0);
                let tex = NonZeroU32::new(present_tex_id).map(glow::NativeTexture);
                self.glow_ctx.bind_texture(glow::TEXTURE_2D, tex);
                if let Some(vao) = self.quad_vao {
                    self.glow_ctx.bind_vertex_array(Some(vao));
                } else {
                    self.glow_ctx.bind_buffer(glow::ARRAY_BUFFER, self.quad_vbo);
                    let stride = 4 * 4;
                    self.glow_ctx.enable_vertex_attrib_array(0);
                    self.glow_ctx
                        .vertex_attrib_pointer_f32(0, 2, glow::FLOAT, false, stride, 0);
                    self.glow_ctx.enable_vertex_attrib_array(1);
                    self.glow_ctx.vertex_attrib_pointer_f32(
                        1,
                        2,
                        glow::FLOAT,
                        false,
                        stride,
                        2 * 4,
                    );
                }
                self.glow_ctx.draw_arrays(glow::TRIANGLE_STRIP, 0, 4);
                self.glow_ctx.bind_texture(glow::TEXTURE_2D, None);
            }

            // SAFETY: swapping current surface.
            if unsafe { eglSwapBuffers(self.egl.dpy, self.egl.surf) } == EGL_FALSE {
                tracing::error!("eglSwapBuffers failed");
                break;
            }
            self.wait_for_gpu();
        }

        self.wait_for_gpu();
        // SAFETY: gl resources managed by drop but explicit cleanup for some.
        unsafe {
            glDeleteTextures(2, self.textures.as_ptr());
            for img in self.images.iter_mut() {
                if *img != EGL_NO_IMAGE_KHR {
                    (self.egl_procs.egl_destroy_image)(self.egl.dpy, *img);
                }
            }
            if let Some(tex) = self.shader_output_tex {
                self.glow_ctx.delete_texture(tex);
            }
            if let Some(vao) = self.quad_vao {
                self.glow_ctx.delete_vertex_array(vao);
            }
            if let Some(vbo) = self.quad_vbo {
                self.glow_ctx.delete_buffer(vbo);
            }
            if let Some(prog) = self.blit_program {
                self.glow_ctx.delete_program(prog);
            }
        }

        Ok(())
    }

    fn wait_for_gpu(&self) {
        if let Some((egl_create_sync, egl_destroy_sync, egl_client_wait_sync)) =
            self.egl_procs.fence_sync
        {
            // SAFETY: EGL sync creation.
            let sync =
                unsafe { egl_create_sync(self.egl.dpy, EGL_SYNC_FENCE_KHR, [EGL_NONE].as_ptr()) };
            if sync != EGL_NO_SYNC_KHR {
                // SAFETY: EGL sync wait.
                let _ = unsafe {
                    egl_client_wait_sync(
                        self.egl.dpy,
                        sync,
                        EGL_SYNC_FLUSH_COMMANDS_BIT_KHR,
                        EGL_FOREVER_KHR,
                    )
                };
                // SAFETY: EGL sync destruction.
                let _ = unsafe { egl_destroy_sync(self.egl.dpy, sync) };
                return;
            }
        }
        // SAFETY: Fallback glFinish.
        unsafe { glFinish() };
    }

    fn update_shader_state(&mut self, surf_w: u32, surf_h: u32) -> Result<(), String> {
        let cfg = android_shader_snapshot();
        if cfg.generation != self.shader_seen_generation {
            let mut final_result: Option<Result<ShaderParameters, String>> = None;

            if !cfg.enabled || cfg.preset_path.is_none() {
                // Disabling or clearing doesn't require background parsing
                self.shader_seen_generation = cfg.generation;
                ANDROID_SHADER_SESSION.store(None);
                final_result = Some(Ok(ShaderParameters {
                    path: String::new(),
                    parameters: Vec::new(),
                }));
            } else {
                // Check if background parsing is complete for this generation
                let pending_guard = PENDING_SHADER_DATA.load();
                if let Some(pending) = &*pending_guard {
                    if pending.generation == cfg.generation {
                        // IO and Preprocessing done in background, now do GL resource creation
                        self.shader_seen_generation = cfg.generation;
                        ANDROID_SHADER_SESSION.store(None);

                        let shader_chain_options = LibrashaderFilterChainOptions {
                            glsl_version: 0,
                            use_dsa: false,
                            force_no_mipmaps: false,
                            disable_cache: true,
                        };

                        // We take the preset out of pending by clearing it if we are the ones who use it?
                        // Actually, it's simpler to just use it and let it be replaced by next request.
                        // But we should probably clear it to avoid using it again (though generation check protects us).
                        let path = cfg.preset_path.unwrap();
                        match super::chain::load_from_parsed_preset(
                            &self.glow_ctx,
                            pending.preset.clone(),
                            &shader_chain_options,
                        ) {
                            Ok(chain) => {
                                tracing::info!("Android shader chain loaded from {}", path);
                                let mut api_parameters = Vec::new();
                                for meta in pending.parameters.iter() {
                                    api_parameters.push(ShaderParameter {
                                        name: meta.id.to_string(),
                                        description: meta.description.clone(),
                                        initial: meta.initial,
                                        current: meta.initial,
                                        minimum: meta.minimum,
                                        maximum: meta.maximum,
                                        step: meta.step,
                                    });
                                }

                                let parameters = ShaderParameters {
                                    path: path.clone(),
                                    parameters: api_parameters,
                                };

                                ANDROID_SHADER_SESSION.store(Some(Arc::new(ShaderSession {
                                    chain: Mutex::new(Some(chain)),
                                    parameters: pending.parameters.clone(),
                                    path,
                                })));
                                final_result = Some(Ok(parameters));
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Failed to load GL shader chain from preset: {}",
                                    e
                                );
                                final_result = Some(Err(e));
                            }
                        }
                    } else if pending.generation > cfg.generation {
                        // Should not happen normally, but if it does, we must have missed a generation.
                        // We reset and wait for a matching or newer one.
                        self.shader_seen_generation = cfg.generation;
                    } else {
                        // Background parsing still in progress for current or older generation.
                        // Just wait for next loop/wake.
                        return Ok(());
                    }
                } else {
                    // Background parsing hasn't stored anything yet.
                    // Just wait for next loop/wake.
                    return Ok(());
                }
            }

            // If we have a final result (success or definitive failure), fulfill pending async requests
            if let Some(res) = final_result {
                let mut channels = super::session::RELOAD_CHANNELS.lock();
                while let Some(tx) = channels.pop_front() {
                    let _ = tx.send(res.clone());
                }
            }
        }

        let has_chain = ANDROID_SHADER_SESSION
            .load()
            .as_ref()
            .map_or(false, |s| s.chain.lock().is_some());

        if has_chain {
            if self.shader_output_size.width != surf_w || self.shader_output_size.height != surf_h {
                self.shader_output_size = LibrashaderSize {
                    width: surf_w,
                    height: surf_h,
                };
                // SAFETY: texture recreation using glow.
                unsafe {
                    if let Some(tex) = self.shader_output_tex.take() {
                        self.glow_ctx.delete_texture(tex);
                    }
                    let tex = self.glow_ctx.create_texture()?;
                    self.glow_ctx.bind_texture(glow::TEXTURE_2D, Some(tex));
                    self.glow_ctx.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_MIN_FILTER,
                        glow::NEAREST as i32,
                    );
                    self.glow_ctx.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_MAG_FILTER,
                        glow::NEAREST as i32,
                    );
                    self.glow_ctx.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_WRAP_S,
                        GL_CLAMP_TO_EDGE as i32,
                    );
                    self.glow_ctx.tex_parameter_i32(
                        glow::TEXTURE_2D,
                        glow::TEXTURE_WRAP_T,
                        GL_CLAMP_TO_EDGE as i32,
                    );
                    self.glow_ctx.tex_image_2d(
                        glow::TEXTURE_2D,
                        0,
                        GL_RGBA8 as i32,
                        surf_w as i32,
                        surf_h as i32,
                        0,
                        GL_RGBA,
                        GL_UNSIGNED_BYTE,
                        glow::PixelUnpackData::Slice(None),
                    );
                    self.shader_output_tex = Some(tex);
                }
            }
        }
        Ok(())
    }

    fn render_shader_pass(
        &mut self,
        idx: usize,
        swapchain: &'static AhbSwapchain,
    ) -> Result<Option<u32>, String> {
        let session_guard = ANDROID_SHADER_SESSION.load();
        if let (Some(session), Some(out_tex)) = (session_guard.as_ref(), self.shader_output_tex) {
            let mut chain_guard = session.chain.lock();
            if let Some(chain) = chain_guard.as_mut() {
                let in_tex = NonZeroU32::new(self.textures[idx]).map(glow::NativeTexture);
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
                    size: self.shader_output_size,
                };
                let viewport = LibrashaderViewport::new_render_target_sized_origin(&output, None)
                    .map_err(|e| format!("failed to create viewport: {:?}", e))?;
                let frame_options = LibrashaderFrameOptions {
                    frames_per_second: 60.0,
                    frametime_delta: 17,
                    ..Default::default()
                };

                // SAFETY: librashader pass.
                if let Ok(_) = super::chain::render_shader_frame(
                    chain,
                    &input,
                    &viewport,
                    self.frame_count,
                    &frame_options,
                ) {
                    return Ok(Some(out_tex.0.get()));
                }
            }
        }
        Ok(None)
    }
}
