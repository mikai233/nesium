use glow::HasContext;
use parking_lot::Mutex;
use std::num::NonZeroU32;
use std::sync::{Arc, OnceLock};

use librashader::presets::ShaderFeatures as LibrashaderShaderFeatures;
use librashader::runtime::Size as LibrashaderSize;
use librashader::runtime::Viewport as LibrashaderViewport;
use librashader::runtime::gl::{
    FilterChain as LibrashaderFilterChain, FilterChainOptions as LibrashaderFilterChainOptions,
    FrameOptions as LibrashaderFrameOptions, GLImage as LibrashaderGlImage,
};

#[derive(Debug, Clone)]
struct LinuxShaderConfig {
    enabled: bool,
    preset_path: Option<String>,
    generation: u64,
}

static LINUX_SHADER_CONFIG: OnceLock<Mutex<LinuxShaderConfig>> = OnceLock::new();

fn linux_shader_config() -> &'static Mutex<LinuxShaderConfig> {
    LINUX_SHADER_CONFIG.get_or_init(|| {
        Mutex::new(LinuxShaderConfig {
            enabled: false,
            preset_path: None,
            generation: 1,
        })
    })
}

fn linux_shader_snapshot() -> LinuxShaderConfig {
    linux_shader_config().lock().clone()
}

pub(crate) fn linux_set_shader_enabled(enabled: bool) {
    let mut cfg = linux_shader_config().lock();
    if cfg.enabled == enabled {
        return;
    }
    cfg.enabled = enabled;
    cfg.generation = cfg.generation.wrapping_add(1);
}

pub(crate) fn linux_set_shader_preset_path(path: Option<String>) {
    let mut cfg = linux_shader_config().lock();
    if cfg.preset_path == path {
        return;
    }
    cfg.preset_path = path;
    cfg.generation = cfg.generation.wrapping_add(1);
}

struct ShaderState {
    chain: Option<LibrashaderFilterChain>,
    generation: u64,
}

static SHADER_STATE: Mutex<Option<ShaderState>> = Mutex::new(None);
static GLOW_CONTEXT: OnceLock<Arc<glow::Context>> = OnceLock::new();

fn glow_context() -> Arc<glow::Context> {
    GLOW_CONTEXT
        .get_or_init(|| {
            Arc::new(unsafe {
                glow::Context::from_loader_function(|s| {
                    let c_name = std::ffi::CString::new(s).unwrap();
                    libc::dlsym(libc::RTLD_DEFAULT, c_name.as_ptr()) as *const _
                })
            })
        })
        .clone()
}

fn get_passthrough_preset() -> std::path::PathBuf {
    let temp = std::env::temp_dir();
    let slangp = temp.join("nesium_passthrough_linux.slangp");
    let slang = temp.join("passthrough_linux.slang");

    let _ = std::fs::write(
        &slang,
        r#"#version 450

#pragma stage vertex
layout(location = 0) in vec4 Position;
layout(location = 1) in vec2 TexCoord;
layout(location = 0) out vec2 vTexCoord;

layout(set = 0, binding = 0, std140) uniform UBO {
    mat4 MVP;
} global;

void main() {
    gl_Position = global.MVP * Position;
    vTexCoord = TexCoord;
}

#pragma stage fragment
layout(location = 0) in vec2 vTexCoord;
layout(location = 0) out vec4 FragColor;
layout(set = 0, binding = 2) uniform sampler2D Source;

void main() {
    FragColor = texture(Source, vTexCoord);
}
"#,
    );

    let slang_path = slang.to_string_lossy();
    let _ = std::fs::write(
        &slangp,
        format!("shaders = 1\nshader0 = \"{}\"\n", slang_path),
    );
    slangp
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nesium_linux_apply_shader(
    input_tex: u32,
    output_tex: u32,
    src_width: u32,
    src_height: u32,
    dst_width: u32,
    dst_height: u32,
    frame_count: u64,
) -> bool {
    let result = std::panic::catch_unwind(|| {
        let cfg = linux_shader_snapshot();

        // If disabled, we return false so the C++ side can fallback to source texture.
        if !cfg.enabled {
            return false;
        }

        let effective_path = if cfg.preset_path.is_some() {
            cfg.preset_path.clone().unwrap()
        } else {
            get_passthrough_preset().to_string_lossy().to_string()
        };

        let mut state_lock = SHADER_STATE.lock();

        // Initialize glow context on demand.
        let glow_ctx = glow_context();

        if state_lock
            .as_ref()
            .map_or(true, |s| s.generation != cfg.generation)
        {
            tracing::info!("Reloading Linux shader chain: {}", effective_path);
            let features = LibrashaderShaderFeatures::ORIGINAL_ASPECT_UNIFORMS
                | LibrashaderShaderFeatures::FRAMETIME_UNIFORMS;

            let options = LibrashaderFilterChainOptions {
                force_no_mipmaps: true,
                disable_cache: true,
                ..Default::default()
            };

            match LibrashaderFilterChain::load_from_path(
                &effective_path,
                features,
                Arc::clone(&glow_ctx),
                Some(&options),
            ) {
                Ok(chain) => {
                    tracing::info!("Linux shader chain loaded from {}", effective_path);
                    *state_lock = Some(ShaderState {
                        chain: Some(chain),
                        generation: cfg.generation,
                    });
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to load Linux shader preset ({}): {:?}",
                        effective_path,
                        e
                    );
                    *state_lock = Some(ShaderState {
                        chain: None,
                        generation: cfg.generation,
                    });
                }
            }
        }

        let Some(state) = state_lock.as_mut() else {
            return false;
        };
        let Some(chain) = state.chain.as_mut() else {
            return false;
        };

        let input = LibrashaderGlImage {
            handle: NonZeroU32::new(input_tex).map(glow::NativeTexture),
            format: glow::RGBA8,
            size: LibrashaderSize {
                width: src_width,
                height: src_height,
            },
        };

        let output = LibrashaderGlImage {
            handle: NonZeroU32::new(output_tex).map(glow::NativeTexture),
            format: glow::RGBA8,
            size: LibrashaderSize {
                width: dst_width,
                height: dst_height,
            },
        };

        let Some(viewport) =
            LibrashaderViewport::new_render_target_sized_origin(&output, None).ok()
        else {
            return false;
        };

        let frame_options = LibrashaderFrameOptions {
            frames_per_second: 60.0,
            frametime_delta: 17,
            ..Default::default()
        };

        let prev_fbo = unsafe { glow_ctx.get_parameter_i32(glow::FRAMEBUFFER_BINDING) as u32 };
        let prev_scissor_enabled = unsafe { glow_ctx.is_enabled(glow::SCISSOR_TEST) };

        let res = unsafe {
            chain.frame(
                &input,
                &viewport,
                frame_count as usize,
                Some(&frame_options),
            )
        };

        unsafe {
            glow_ctx.bind_framebuffer(
                glow::FRAMEBUFFER,
                NonZeroU32::new(prev_fbo).map(glow::NativeFramebuffer),
            );
            if prev_scissor_enabled {
                glow_ctx.enable(glow::SCISSOR_TEST);
            } else {
                glow_ctx.disable(glow::SCISSOR_TEST);
            }
        }

        match res {
            Ok(_) => true,
            Err(e) => {
                tracing::error!("Linux shader frame failed: {:?}", e);
                false
            }
        }
    });

    match result {
        Ok(val) => val,
        Err(e) => {
            tracing::error!("Panic in nesium_linux_apply_shader: {:?}", e);
            false
        }
    }
}
