use parking_lot::Mutex;
use std::ffi::c_void;
use std::sync::{OnceLock, atomic::Ordering};
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

static WINDOWS_SHADER_CONFIG: OnceLock<Mutex<WindowsShaderConfig>> = OnceLock::new();

fn windows_shader_config() -> &'static Mutex<WindowsShaderConfig> {
    WINDOWS_SHADER_CONFIG.get_or_init(|| {
        Mutex::new(WindowsShaderConfig {
            enabled: false,
            preset_path: None,
            generation: 1,
        })
    })
}

fn windows_shader_snapshot() -> WindowsShaderConfig {
    windows_shader_config().lock().clone()
}

fn get_passthrough_preset() -> std::path::PathBuf {
    let temp = std::env::temp_dir();
    let slangp = temp.join("nesium_passthrough.slangp");
    let slang = temp.join("passthrough.slang");

    // Rationale: This passthrough shader is required by the pipeline to handle
    // HiDPI scaling and to ensure consistent rendering when no user shader is active.
    //
    // Note: We now use a pure BGRA pipeline (Core -> Staging -> Librashader -> Output),
    // thanks to upstream librashader support. An identity pass is still useful
    // for handling the scaling aspect via the robust shader chain.
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
    let mut cfg = windows_shader_config().lock();
    if cfg.enabled == enabled {
        return;
    }
    cfg.enabled = enabled;
    cfg.generation = cfg.generation.wrapping_add(1);
}

pub(crate) fn windows_set_shader_preset_path(path: Option<String>) {
    let mut cfg = windows_shader_config().lock();
    if cfg.preset_path == path {
        return;
    }
    cfg.preset_path = path;
    cfg.generation = cfg.generation.wrapping_add(1);
}

/// Raw device/context pointers passed from C++.
/// Must be updatable (device/context can change after resize/recreate).
#[derive(Copy, Clone)]
struct D3D11DeviceContext {
    device: *mut c_void,
    context: *mut c_void,
}

unsafe impl Send for D3D11DeviceContext {}
unsafe impl Sync for D3D11DeviceContext {}

/// Shared D3D11 context. Using Mutex instead of OnceLock to allow
/// context updates during device recreation or resize events.
static D3D11_CONTEXT: Mutex<Option<D3D11DeviceContext>> = Mutex::new(None);

#[unsafe(no_mangle)]
pub extern "C" fn nesium_set_d3d11_device(device: *mut c_void, context: *mut c_void) {
    if device.is_null() || context.is_null() {
        log::error!(
            "nesium_set_d3d11_device called with null ptr(s): device={:p}, context={:p}",
            device,
            context
        );
        *D3D11_CONTEXT.lock() = None;
        return;
    }

    *D3D11_CONTEXT.lock() = Some(D3D11DeviceContext { device, context });
    log::info!(
        "D3D11 device/context updated: device={:p}, context={:p}",
        device,
        context
    );
}

struct ShaderState {
    chain: Option<LibrashaderFilterChain>,
    generation: u64,
}

static SHADER_STATE: Mutex<Option<ShaderState>> = Mutex::new(None);
static FRAME_COUNT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

fn hresult_from_windows_error(e: &windows::core::Error) -> HRESULT {
    e.code().into()
}

fn log_hresult_context(prefix: &str, hr: HRESULT) {
    if hr == E_INVALIDARG {
        log::error!("{}: HRESULT=0x{:08X} (E_INVALIDARG)", prefix, hr.0 as u32);
    } else {
        log::error!("{}: HRESULT=0x{:08X}", prefix, hr.0 as u32);
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
        log::error!("{} resource has no owning device", label);
        return Err(E_INVALIDARG);
    };

    let owning_ptr = owning_dev.as_raw() as *mut c_void;
    if owning_ptr != expected_device_ptr {
        log::error!(
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
            log::error!(
                "nesium_apply_shader: null texture ptr(s): input={:p}, output={:p}",
                input_tex,
                output_tex
            );
            return false;
        }
        if src_width == 0 || src_height == 0 || dst_width == 0 || dst_height == 0 {
            log::error!(
                "nesium_apply_shader: invalid sizes (src={}x{}, dst={}x{})",
                src_width,
                src_height,
                dst_width,
                dst_height
            );
            return false;
        }

        let Some(d3d11) = *D3D11_CONTEXT.lock() else {
            log::error!("D3D11 context not set");
            return false;
        };

        let mut state_lock = SHADER_STATE.lock();

        // Reload if generation changed or shader not loaded
        let needs_reload = match &*state_lock {
            Some(state) => state.generation != cfg.generation,
            None => true,
        };

        if needs_reload {
            log::info!("Reloading shader chain (path={})", effective_path);
            let features = LibrashaderShaderFeatures::ORIGINAL_ASPECT_UNIFORMS
                | LibrashaderShaderFeatures::FRAMETIME_UNIFORMS;

            let options = LibrashaderFilterChainOptions {
                force_no_mipmaps: true,
                disable_cache: true,
                ..Default::default()
            };

            unsafe {
                let device: std::mem::ManuallyDrop<ID3D11Device> =
                    std::mem::ManuallyDrop::new(std::mem::transmute_copy(&d3d11.device));

                match LibrashaderFilterChain::load_from_path(
                    &effective_path,
                    features,
                    &*device,
                    Some(&options),
                ) {
                    Ok(chain) => {
                        log::info!("Windows shader chain loaded from {}", effective_path);
                        *state_lock = Some(ShaderState {
                            chain: Some(chain),
                            generation: cfg.generation,
                        });
                    }
                    Err(e) => {
                        log::error!(
                            "Failed to load Windows shader preset ({}): {:?}",
                            effective_path,
                            e
                        );
                        // Save the failure state for this generation to prevent lag
                        *state_lock = Some(ShaderState {
                            chain: None,
                            generation: cfg.generation,
                        });
                    }
                }
            }
        }

        let Some(state) = state_lock.as_mut() else {
            return false;
        };
        let Some(chain) = state.chain.as_mut() else {
            return false;
        };

        unsafe {
            // Again, wrap in ManuallyDrop to avoid Release() on drop.
            let device: std::mem::ManuallyDrop<ID3D11Device> =
                std::mem::ManuallyDrop::new(std::mem::transmute_copy(&d3d11.device));
            let context: std::mem::ManuallyDrop<ID3D11DeviceContext> =
                std::mem::ManuallyDrop::new(std::mem::transmute_copy(&d3d11.context));

            let input_tex_resource: std::mem::ManuallyDrop<ID3D11Resource> =
                std::mem::ManuallyDrop::new(std::mem::transmute_copy(&input_tex));
            let output_tex_resource: std::mem::ManuallyDrop<ID3D11Resource> =
                std::mem::ManuallyDrop::new(std::mem::transmute_copy(&output_tex));

            // log_tex2d_desc("input", &*input_tex_resource);
            // log_tex2d_desc("output", &*output_tex_resource);

            // --- New: resource/device consistency checks (high-signal for 0x80070057) ---
            if let Err(hr) = validate_resource_device("input", &*input_tex_resource, d3d11.device) {
                log_hresult_context("Input resource/device validation failed", hr);
                return false;
            }
            if let Err(hr) = validate_resource_device("output", &*output_tex_resource, d3d11.device)
            {
                log_hresult_context("Output resource/device validation failed", hr);
                return false;
            }

            // --- Create SRV (with fallback to inferred desc on E_INVALIDARG) ---
            let mut srv: Option<ID3D11ShaderResourceView> = None;

            let mut srv_desc = D3D11_SHADER_RESOURCE_VIEW_DESC::default();
            // Upstream librashader now supports BGRA on Windows (fixed 0x80070057).
            // We use a pure BGRA pipeline for best performance/simplicity.
            srv_desc.Format = DXGI_FORMAT_B8G8R8A8_UNORM;
            srv_desc.ViewDimension = D3D11_SRV_DIMENSION_TEXTURE2D;
            srv_desc.Anonymous.Texture2D = D3D11_TEX2D_SRV {
                MipLevels: 1,
                MostDetailedMip: 0,
            };

            if let Err(e) = device.CreateShaderResourceView(
                &*input_tex_resource,
                Some(&srv_desc),
                Some(&mut srv),
            ) {
                let hr = hresult_from_windows_error(&e);
                log::error!("Failed to create SRV (explicit desc): {:?}", e);

                // Fallback: let D3D infer format/desc (often fixes typeless/sRGB cases).
                if hr == E_INVALIDARG {
                    log::warn!(
                        "Retry CreateShaderResourceView with inferred desc (None) due to E_INVALIDARG"
                    );
                    srv = None;
                    if let Err(e2) =
                        device.CreateShaderResourceView(&*input_tex_resource, None, Some(&mut srv))
                    {
                        log::error!("Failed to create SRV (inferred desc): {:?}", e2);
                        return false;
                    }
                } else {
                    return false;
                }
            }

            let Some(srv) = srv else {
                log::error!("SRV created but is None");
                return false;
            };

            // --- Create RTV (with fallback to inferred desc on E_INVALIDARG) ---
            let mut rtv: Option<ID3D11RenderTargetView> = None;

            let mut rtv_desc = D3D11_RENDER_TARGET_VIEW_DESC::default();
            rtv_desc.Format = DXGI_FORMAT_B8G8R8A8_UNORM; // Shader output (GPU shared) is BGRA
            rtv_desc.ViewDimension = D3D11_RTV_DIMENSION_TEXTURE2D;
            rtv_desc.Anonymous.Texture2D = D3D11_TEX2D_RTV { MipSlice: 0 };

            if let Err(e) = device.CreateRenderTargetView(
                &*output_tex_resource,
                Some(&rtv_desc),
                Some(&mut rtv),
            ) {
                let hr = hresult_from_windows_error(&e);
                log::error!("Failed to create RTV (explicit desc): {:?}", e);

                // Fallback: inferred desc
                if hr == E_INVALIDARG {
                    log::warn!(
                        "Retry CreateRenderTargetView with inferred desc (None) due to E_INVALIDARG"
                    );
                    rtv = None;
                    if let Err(e2) =
                        device.CreateRenderTargetView(&*output_tex_resource, None, Some(&mut rtv))
                    {
                        log::error!("Failed to create RTV (inferred desc): {:?}", e2);
                        return false;
                    }
                } else {
                    return false;
                }
            }

            let Some(rtv) = rtv else {
                log::error!("RTV created but is None");
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
                Some(&*context),
                &srv,
                &viewport,
                frame_count,
                Some(&frame_options),
            ) {
                Ok(_) => true,
                Err(e) => {
                    log::error!(
                        "Windows shader frame failed: {:?} (src={}x{}, dst={}x{}) device={:p} context={:p} input={:p} output={:p}",
                        e,
                        src_width,
                        src_height,
                        dst_width,
                        dst_height,
                        d3d11.device,
                        d3d11.context,
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
            log::error!("Panic in nesium_apply_shader: {:?}", e);
            false
        }
    }
}
