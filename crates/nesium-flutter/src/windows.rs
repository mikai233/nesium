use arc_swap::ArcSwapOption;
use librashader::preprocess::ShaderParameter;
use parking_lot::Mutex;

use std::ffi::c_void;
use std::mem::{ManuallyDrop, transmute_copy};
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, atomic::Ordering};
use windows::Win32::Foundation::E_INVALIDARG;
use windows::Win32::Graphics::Direct3D::D3D11_SRV_DIMENSION_TEXTURE2D;
use windows::core::{HRESULT, Interface};

use librashader::presets::ShaderFeatures as LibrashaderShaderFeatures;
use librashader::runtime::Size as LibrashaderSize;
use librashader::runtime::Viewport as LibrashaderViewport;
use librashader::runtime::d3d11::{
    FilterChain as LibrashaderFilterChain, FilterChainOptions as LibrashaderFilterChainOptions,
    FrameOptions as LibrashaderFrameOptions,
};
use windows::Win32::Graphics::Direct3D11::{
    D3D11_RENDER_TARGET_VIEW_DESC, D3D11_RTV_DIMENSION_TEXTURE2D, D3D11_SHADER_RESOURCE_VIEW_DESC,
    D3D11_TEX2D_RTV, D3D11_TEX2D_SRV, ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView,
    ID3D11Resource, ID3D11ShaderResourceView,
};
use windows::Win32::Graphics::Dxgi::Common::{
    DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_FORMAT_R8G8B8A8_UNORM,
};

#[derive(Debug, Clone)]
struct WindowsShaderConfig {
    enabled: bool,
    preset_path: Option<String>,
    generation: u64,
}

static WINDOWS_SHADER_CONFIG: ArcSwapOption<WindowsShaderConfig> = ArcSwapOption::const_empty();

fn windows_shader_snapshot() -> WindowsShaderConfig {
    let guard = WINDOWS_SHADER_CONFIG.load();
    if let Some(arc) = &*guard {
        (**arc).clone()
    } else {
        WindowsShaderConfig {
            enabled: false,
            preset_path: None,
            generation: 1,
        }
    }
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

pub(crate) fn windows_set_shader_enabled(enabled: bool) {
    WINDOWS_SHADER_CONFIG.rcu(|old| {
        let mut new = old
            .as_ref()
            .map(|a| (**a).clone())
            .unwrap_or(WindowsShaderConfig {
                enabled: false,
                preset_path: None,
                generation: 1,
            });

        if new.enabled == enabled {
            old.clone()
        } else {
            new.enabled = enabled;
            new.generation = new.generation.wrapping_add(1);
            Some(Arc::new(new))
        }
    });
}

pub(crate) fn windows_set_shader_preset_path(path: Option<String>) {
    WINDOWS_SHADER_CONFIG.rcu(|old| {
        let mut new = old
            .as_ref()
            .map(|a| (**a).clone())
            .unwrap_or(WindowsShaderConfig {
                enabled: false,
                preset_path: None,
                generation: 1,
            });

        if new.preset_path == path {
            old.clone()
        } else {
            new.preset_path = path.clone();
            new.generation = new.generation.wrapping_add(1);
            Some(Arc::new(new))
        }
    });
}

pub(crate) struct ShaderState {
    // Chain needs to be mutable for frame() calls, but ShaderState is held in ArcSwap (Arc)
    pub(crate) chain: Mutex<Option<LibrashaderFilterChain>>,
    pub(crate) generation: u64,
    // Store device address to detect if the underlying D3D11 device has changed
    // (e.g. backend switch or recreation).
    pub(crate) device_addr: usize,
    pub(crate) parameters: Vec<ShaderParameter>,
    pub(crate) path: String,
}

pub(crate) static SHADER_STATE: ArcSwapOption<ShaderState> = ArcSwapOption::const_empty();
static FRAME_COUNT: AtomicUsize = AtomicUsize::new(0);

fn hresult_from_windows_error(e: &windows::core::Error) -> HRESULT {
    e.code().into()
}

fn log_hresult_context(prefix: &str, hr: HRESULT) {
    if hr == E_INVALIDARG {
        tracing::error!("{}: HRESULT=0x{:08X} (E_INVALIDARG)", prefix, hr.0 as u32);
    } else {
        tracing::error!("{}: HRESULT=0x{:08X}", prefix, hr.0 as u32);
    }
}

/// Validate that a resource belongs to the same device pointer we were given.
/// Mixing resources from different devices is a very common cause of E_INVALIDARG.
unsafe fn validate_resource_device(
    label: &str,
    res: &ID3D11Resource,
    expected_device_ptr: *mut c_void,
) -> Result<(), HRESULT> {
    let owning = unsafe { res.GetDevice() };

    let Ok(owning_dev) = owning else {
        tracing::error!("{} resource has no owning device", label);
        return Err(E_INVALIDARG);
    };

    let owning_ptr = owning_dev.as_raw() as *mut c_void;
    if owning_ptr != expected_device_ptr {
        tracing::error!(
            "{} resource device mismatch: owning={:p}, expected={:p}",
            label,
            owning_ptr,
            expected_device_ptr
        );
        return Err(E_INVALIDARG);
    }

    Ok(())
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nesium_apply_shader(
    device: *mut c_void,
    context: *mut c_void,
    input_tex: *mut c_void,
    output_tex: *mut c_void,
    src_width: u32,
    src_height: u32,
    dst_width: u32,
    dst_height: u32,
) -> bool {
    let result = std::panic::catch_unwind(|| {
        let cfg = windows_shader_snapshot();
        let effective_path = if cfg.enabled && cfg.preset_path.is_some() {
            cfg.preset_path.clone().unwrap()
        } else {
            get_passthrough_preset().to_string_lossy().to_string()
        };

        if input_tex.is_null() || output_tex.is_null() {
            tracing::error!(
                "nesium_apply_shader: null texture ptr(s): input={:p}, output={:p}",
                input_tex,
                output_tex
            );
            return false;
        }
        if src_width == 0 || src_height == 0 || dst_width == 0 || dst_height == 0 {
            tracing::error!(
                "nesium_apply_shader: invalid sizes (src={}x{}, dst={}x{})",
                src_width,
                src_height,
                dst_width,
                dst_height
            );
            return false;
        }

        if device.is_null() || context.is_null() {
            tracing::error!(
                "nesium_apply_shader: null device/context ptr(s): device={:p}, context={:p}",
                device,
                context
            );
            return false;
        }

        let current_state = SHADER_STATE.load();

        // Reload if generation changed OR device changed OR shader not loaded
        let needs_reload = match &*current_state {
            Some(state) => {
                state.generation != cfg.generation || state.device_addr != device as usize
            }
            None => true,
        };

        if needs_reload {
            tracing::info!(
                "Reloading shader chain (path={}, device changed={})",
                effective_path,
                match &*current_state {
                    Some(state) => state.device_addr != device as usize,
                    None => true,
                }
            );

            let features = LibrashaderShaderFeatures::ORIGINAL_ASPECT_UNIFORMS
                | LibrashaderShaderFeatures::FRAMETIME_UNIFORMS;

            let options = LibrashaderFilterChainOptions {
                force_no_mipmaps: true,
                disable_cache: true,
                ..Default::default()
            };

            unsafe {
                let device_ptr = device;
                let device: ManuallyDrop<ID3D11Device> = ManuallyDrop::new(transmute_copy(&device));

                let mut parameters = Vec::new();
                let load_result = (|| {
                    let preset =
                        librashader::presets::ShaderPreset::try_parse(&effective_path, features)
                            .map_err(|e| format!("{:?}", e))?;

                    if let Ok(meta) = librashader::presets::get_parameter_meta(&preset) {
                        for p in meta {
                            parameters.push(p.clone());
                        }
                    }

                    LibrashaderFilterChain::load_from_preset(preset, &*device, Some(&options))
                        .map_err(|e| format!("{:?}", e))
                })();

                match load_result {
                    Ok(chain) => {
                        tracing::info!("Windows shader chain loaded from {}", effective_path);

                        // Emit update to Flutter immediately using local data
                        let mut api_parameters = Vec::new();
                        for meta in parameters.iter() {
                            let name = &meta.id;
                            api_parameters.push(crate::api::video::ShaderParameter {
                                name: name.to_string(),
                                description: meta.description.clone(),
                                initial: meta.initial,
                                current: meta.initial,
                                minimum: meta.minimum,
                                maximum: meta.maximum,
                                step: meta.step,
                            });
                        }
                        crate::senders::shader::emit_shader_parameters_update(
                            crate::api::video::ShaderParameters {
                                path: effective_path.clone(),
                                parameters: api_parameters,
                            },
                        );

                        SHADER_STATE.store(Some(Arc::new(ShaderState {
                            chain: Mutex::new(Some(chain)),
                            generation: cfg.generation,
                            device_addr: device_ptr as usize,
                            parameters,
                            path: effective_path,
                        })));
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to load Windows shader preset ({}): {:?}",
                            effective_path,
                            e
                        );

                        crate::senders::shader::emit_shader_parameters_update(
                            crate::api::video::ShaderParameters {
                                path: effective_path.clone(),
                                parameters: Vec::new(),
                            },
                        );

                        SHADER_STATE.store(Some(Arc::new(ShaderState {
                            chain: Mutex::new(None),
                            generation: cfg.generation,
                            device_addr: device_ptr as usize,
                            parameters: Vec::new(),
                            path: effective_path,
                        })));
                    }
                }
            }
        }

        // Re-load state after potential reload
        let current_state = SHADER_STATE.load();
        let Some(state) = &*current_state else {
            return false;
        };

        // Lock the internal chain for rendering
        let mut chain = state.chain.lock();
        let Some(chain) = chain.as_mut() else {
            return false;
        };

        unsafe {
            // Again, wrap in ManuallyDrop to avoid Release() on drop.
            let device_ref: ManuallyDrop<ID3D11Device> = ManuallyDrop::new(transmute_copy(&device));
            let context_ref: ManuallyDrop<ID3D11DeviceContext> =
                ManuallyDrop::new(transmute_copy(&context));

            let input_tex_resource: ManuallyDrop<ID3D11Resource> =
                ManuallyDrop::new(transmute_copy(&input_tex));
            let output_tex_resource: ManuallyDrop<ID3D11Resource> =
                ManuallyDrop::new(transmute_copy(&output_tex));

            // --- New: resource/device consistency checks (high-signal for 0x80070057) ---
            if let Err(hr) = validate_resource_device("input", &*input_tex_resource, device) {
                log_hresult_context("Input resource/device validation failed", hr);
                return false;
            }
            if let Err(hr) = validate_resource_device("output", &*output_tex_resource, device) {
                log_hresult_context("Output resource/device validation failed", hr);
                return false;
            }

            // --- Create SRV (with fallback to inferred desc on E_INVALIDARG) ---
            let mut srv: Option<ID3D11ShaderResourceView> = None;

            let mut srv_desc = D3D11_SHADER_RESOURCE_VIEW_DESC::default();
            // Official librashader 0.10.1 D3D11 runtime only supports RGBA input.
            // We use a GPU swizzle (Compute Shader) in C++ to bridge the BGRA output
            // of the core to the RGBA required here.
            srv_desc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
            srv_desc.ViewDimension = D3D11_SRV_DIMENSION_TEXTURE2D;
            srv_desc.Anonymous.Texture2D = D3D11_TEX2D_SRV {
                MipLevels: 1,
                MostDetailedMip: 0,
            };

            if let Err(e) = device_ref.CreateShaderResourceView(
                &*input_tex_resource,
                Some(&srv_desc),
                Some(&mut srv),
            ) {
                let hr = hresult_from_windows_error(&e);
                tracing::error!("Failed to create SRV (explicit desc): {:?}", e);

                // Fallback: let D3D infer format/desc (often fixes typeless/sRGB cases).
                if hr == E_INVALIDARG {
                    tracing::warn!(
                        "Retry CreateShaderResourceView with inferred desc (None) due to E_INVALIDARG"
                    );
                    srv = None;
                    if let Err(e2) = device_ref.CreateShaderResourceView(
                        &*input_tex_resource,
                        None,
                        Some(&mut srv),
                    ) {
                        tracing::error!("Failed to create SRV (inferred desc): {:?}", e2);
                        return false;
                    }
                } else {
                    return false;
                }
            }

            let Some(srv) = srv else {
                tracing::error!("SRV created but is None");
                return false;
            };

            // --- Create RTV (with fallback to inferred desc on E_INVALIDARG) ---
            let mut rtv: Option<ID3D11RenderTargetView> = None;

            let mut rtv_desc = D3D11_RENDER_TARGET_VIEW_DESC::default();
            rtv_desc.Format = DXGI_FORMAT_B8G8R8A8_UNORM; // Shader output (GPU shared) is BGRA
            rtv_desc.ViewDimension = D3D11_RTV_DIMENSION_TEXTURE2D;
            rtv_desc.Anonymous.Texture2D = D3D11_TEX2D_RTV { MipSlice: 0 };

            if let Err(e) = device_ref.CreateRenderTargetView(
                &*output_tex_resource,
                Some(&rtv_desc),
                Some(&mut rtv),
            ) {
                let hr = hresult_from_windows_error(&e);
                tracing::error!("Failed to create RTV (explicit desc): {:?}", e);

                // Fallback: inferred desc
                if hr == E_INVALIDARG {
                    tracing::warn!(
                        "Retry CreateRenderTargetView with inferred desc (None) due to E_INVALIDARG"
                    );
                    rtv = None;
                    if let Err(e2) = device_ref.CreateRenderTargetView(
                        &*output_tex_resource,
                        None,
                        Some(&mut rtv),
                    ) {
                        tracing::error!("Failed to create RTV (inferred desc): {:?}", e2);
                        return false;
                    }
                } else {
                    return false;
                }
            }

            let Some(rtv) = rtv else {
                tracing::error!("RTV created but is None");
                return false;
            };

            let viewport = LibrashaderViewport {
                x: 0.0,
                y: 0.0,
                mvp: None,
                output: &rtv,
                size: LibrashaderSize {
                    width: dst_width,
                    height: dst_height,
                },
            };

            let frame_count = FRAME_COUNT.fetch_add(1, Ordering::Relaxed);

            let frame_options = LibrashaderFrameOptions {
                frames_per_second: 60.0,
                frametime_delta: 17,
                ..Default::default()
            };

            match chain.frame(
                Some(&*context_ref),
                &srv,
                &viewport,
                frame_count,
                Some(&frame_options),
            ) {
                Ok(_) => true,
                Err(e) => {
                    tracing::error!(
                        "Windows shader frame failed: {:?} (src={}x{}, dst={}x{}) device={:p} context={:p} input={:p} output={:p}",
                        e,
                        src_width,
                        src_height,
                        dst_width,
                        dst_height,
                        device,
                        context,
                        input_tex,
                        output_tex
                    );
                    false
                }
            }
        }
    });

    match result {
        Ok(val) => val,
        Err(e) => {
            tracing::error!("Panic in nesium_apply_shader: {:?}", e);
            false
        }
    }
}
