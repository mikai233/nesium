use parking_lot::Mutex;
use std::ffi::c_void;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicUsize, Ordering};

use librashader::presets::ShaderFeatures as LibrashaderShaderFeatures;
use librashader::runtime::Size as LibrashaderSize;
use librashader::runtime::Viewport as LibrashaderViewport;
use librashader::runtime::mtl::{
    FilterChain as LibrashaderFilterChain, FilterChainOptions as LibrashaderFilterChainOptions,
    FrameOptions as LibrashaderFrameOptions,
};

#[derive(Debug, Clone)]
struct AppleShaderConfig {
    enabled: bool,
    preset_path: Option<String>,
    generation: u64,
}

static APPLE_SHADER_CONFIG: OnceLock<Mutex<AppleShaderConfig>> = OnceLock::new();

fn apple_shader_config() -> &'static Mutex<AppleShaderConfig> {
    APPLE_SHADER_CONFIG.get_or_init(|| {
        Mutex::new(AppleShaderConfig {
            enabled: false,
            preset_path: None,
            generation: 1,
        })
    })
}

fn apple_shader_snapshot() -> AppleShaderConfig {
    apple_shader_config().lock().clone()
}

fn get_passthrough_preset() -> std::path::PathBuf {
    let temp = std::env::temp_dir();
    let slangp = temp.join("nesium_passthrough.slangp");
    let slang = temp.join("passthrough.slang");

    // Use a passthrough shader to copy/scale the texture when no user shader is active.
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

    // Write the preset pointing to the shader file with a sanitized path
    let slang_path = slang.to_string_lossy().replace("\\", "/");
    let _ = std::fs::write(
        &slangp,
        format!("shaders = 1\nshader0 = \"{}\"\n", slang_path),
    );
    slangp
}

pub(crate) fn apple_set_shader_enabled(enabled: bool) {
    let mut cfg = apple_shader_config().lock();
    if cfg.enabled == enabled {
        return;
    }
    cfg.enabled = enabled;
    cfg.generation = cfg.generation.wrapping_add(1);
}

pub(crate) fn apple_set_shader_preset_path(path: Option<String>) {
    let mut cfg = apple_shader_config().lock();
    if cfg.preset_path == path {
        return;
    }
    cfg.preset_path = path;
    cfg.generation = cfg.generation.wrapping_add(1);
}

pub(crate) struct ShaderState {
    pub(crate) chain: Option<LibrashaderFilterChain>,
    pub(crate) generation: u64,
    pub(crate) device_addr: usize,
    pub(crate) parameters:
        std::collections::HashMap<String, librashader::preprocess::ShaderParameter>,
    pub(crate) path: String,
}

// SAFETY:
// `ShaderState` contains `LibrashaderFilterChain` which wraps Metal objects.
// Metal objects (MTLDevice, etc.) are intrinsically thread-safe.
// We are storing this in a global Mutex, so we specifically need `Send`.
// `Sync` is implemented for completeness, though likely not strictly needed
// if only accessed through `Mutex`.
unsafe impl Send for ShaderState {}
unsafe impl Sync for ShaderState {}

pub(crate) static SHADER_STATE: Mutex<Option<ShaderState>> = Mutex::new(None);
static FRAME_COUNT: AtomicUsize = AtomicUsize::new(0);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nesium_apply_shader_metal(
    device_ptr: *mut c_void,
    command_queue_ptr: *mut c_void,
    command_buffer_ptr: *mut c_void,
    input_tex_ptr: *mut c_void,
    output_tex_ptr: *mut c_void,
    src_width: u32,
    src_height: u32,
    dst_width: u32,
    dst_height: u32,
) -> bool {
    let result = std::panic::catch_unwind(|| {
        let cfg = apple_shader_snapshot();
        let effective_path = if cfg.enabled && cfg.preset_path.is_some() {
            cfg.preset_path.clone().unwrap()
        } else {
            get_passthrough_preset().to_string_lossy().to_string()
        };

        if input_tex_ptr.is_null() || output_tex_ptr.is_null() {
            return false;
        }
        if src_width == 0 || src_height == 0 || dst_width == 0 || dst_height == 0 {
            return false;
        }
        if device_ptr.is_null() || command_queue_ptr.is_null() || command_buffer_ptr.is_null() {
            return false;
        }

        let mut state_lock = SHADER_STATE.lock();

        // Reload if generation changed OR device changed OR shader not loaded
        let needs_reload = match &*state_lock {
            Some(state) => {
                state.generation != cfg.generation || state.device_addr != device_ptr as usize
            }
            None => true,
        };

        if needs_reload {
            tracing::info!(
                "Reloading Apple Metal shader chain (path={})",
                effective_path
            );
            let features = LibrashaderShaderFeatures::ORIGINAL_ASPECT_UNIFORMS
                | LibrashaderShaderFeatures::FRAMETIME_UNIFORMS;

            let options = LibrashaderFilterChainOptions {
                force_no_mipmaps: true,
                ..Default::default()
            };

            let mut parameters = std::collections::HashMap::new();
            let load_result = (|| {
                let preset =
                    librashader::presets::ShaderPreset::try_parse(&effective_path, features)
                        .map_err(|e| format!("{:?}", e))?;

                if let Ok(meta) = librashader::presets::get_parameter_meta(&preset) {
                    for p in meta {
                        parameters.insert(p.id.to_string(), p);
                    }
                }

                unsafe {
                    LibrashaderFilterChain::load_from_preset(
                        preset,
                        // SAFETY: Trust that command_queue_ptr is valid id<MTLCommandQueue> compatible with librashader
                        std::mem::transmute(command_queue_ptr),
                        Some(&options),
                    )
                }
                .map_err(|e| format!("{:?}", e))
            })();

            match load_result {
                Ok(chain) => {
                    tracing::info!("Apple shader chain loaded from {}", effective_path);

                    // Emit update to Flutter immediately using local data
                    let mut api_parameters = std::collections::HashMap::new();
                    for (name, meta) in parameters.iter() {
                        api_parameters.insert(
                            name.clone(),
                            crate::api::video::ShaderParameter {
                                name: name.clone(),
                                description: meta.description.clone(),
                                initial: meta.initial,
                                current: meta.initial,
                                minimum: meta.minimum,
                                maximum: meta.maximum,
                                step: meta.step,
                            },
                        );
                    }
                    crate::senders::shader::emit_shader_parameters_update(
                        crate::api::video::ShaderParameters {
                            path: effective_path.clone(),
                            parameters: api_parameters,
                        },
                    );

                    *state_lock = Some(ShaderState {
                        chain: Some(chain),
                        generation: cfg.generation,
                        device_addr: device_ptr as usize,
                        parameters,
                        path: effective_path,
                    });
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to load Apple shader preset ({}): {:?}",
                        effective_path,
                        e
                    );

                    crate::senders::shader::emit_shader_parameters_update(
                        crate::api::video::ShaderParameters {
                            path: effective_path.clone(),
                            parameters: std::collections::HashMap::new(),
                        },
                    );

                    *state_lock = Some(ShaderState {
                        chain: None,
                        generation: cfg.generation,
                        device_addr: device_ptr as usize,
                        parameters: std::collections::HashMap::new(),
                        path: effective_path,
                    });
                }
            }
        }

        let Some(chain) = state_lock.as_mut().and_then(|s| s.chain.as_mut()) else {
            return false;
        };

        // Leverage type inference for viewport
        let viewport = LibrashaderViewport {
            x: 0.0,
            y: 0.0,
            mvp: None,
            // SAFETY: output_tex_ptr is valid id<MTLTexture>
            output: unsafe { std::mem::transmute(output_tex_ptr) },
            size: LibrashaderSize {
                width: dst_width,
                height: dst_height,
            },
        };

        let frame_count = FRAME_COUNT.fetch_add(1, Ordering::Relaxed);
        let frame_options = LibrashaderFrameOptions {
            frames_per_second: 60.0,
            frametime_delta: 16, // approx 1/60s
            ..Default::default()
        };

        match unsafe {
            chain.frame(
                // SAFETY: input_tex_ptr is valid id<MTLTexture>
                std::mem::transmute(input_tex_ptr),
                &viewport,
                // SAFETY: command_buffer_ptr is valid id<MTLCommandBuffer>
                std::mem::transmute(command_buffer_ptr),
                frame_count,
                Some(&frame_options),
            )
        } {
            Ok(_) => true,
            Err(e) => {
                tracing::error!("Apple shader frame failed: {:?}", e);
                false
            }
        }
    });

    match result {
        Ok(val) => val,
        Err(e) => {
            tracing::error!("Panic in nesium_apply_shader_metal: {:?}", e);
            false
        }
    }
}
