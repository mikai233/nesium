use super::session::{SHADER_SESSION, ShaderSession};
use librashader::presets::{ShaderFeatures as LibrashaderShaderFeatures, ShaderPreset};
use librashader::runtime::FilterChainParameters as _;
use librashader::runtime::Viewport;
use librashader::runtime::d3d11::FrameOptions as LibrashaderFrameOptions;
use librashader::runtime::d3d11::{
    FilterChain as LibrashaderFilterChain, FilterChainOptions as LibrashaderFilterChainOptions,
};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::mem::{ManuallyDrop, transmute_copy};
use std::sync::Arc;
use windows::Win32::Graphics::Direct3D11::{
    ID3D11Device, ID3D11DeviceContext, ID3D11RenderTargetView, ID3D11ShaderResourceView,
};

pub(crate) unsafe fn render_shader_frame(
    chain: &mut LibrashaderFilterChain,
    context: &ID3D11DeviceContext,
    srv: &ID3D11ShaderResourceView,
    viewport: &Viewport<&ID3D11RenderTargetView>,
    frame_count: usize,
) -> bool {
    let frame_options = LibrashaderFrameOptions {
        frames_per_second: 60.0,
        frametime_delta: 17,
        ..Default::default()
    };

    // SAFETY: The caller ensure chain, context, srv, and viewport are valid and compatible.
    match unsafe {
        chain.frame(
            Some(context),
            srv,
            viewport,
            frame_count,
            Some(&frame_options),
        )
    } {
        Ok(_) => true,
        Err(e) => {
            tracing::error!("Windows shader frame failed: {:?}", e);
            false
        }
    }
}

pub(crate) fn reload_shader_chain(
    effective_path: &str,
    device_ptr: *mut std::ffi::c_void,
    generation: u64,
    preparsed_preset: Option<ShaderPreset>,
    parameters: HashMap<String, f32>,
) {
    let path = effective_path.to_string();
    let device_addr = device_ptr as usize;

    std::thread::spawn(move || {
        let device_ptr = device_addr as *mut std::ffi::c_void;

        tracing::info!(
            "Reloading Windows D3D11 shader chain (async, path={}, generation={}, device={:?})",
            path,
            generation,
            device_ptr
        );

        let features = LibrashaderShaderFeatures::ORIGINAL_ASPECT_UNIFORMS
            | LibrashaderShaderFeatures::FRAMETIME_UNIFORMS;

        let options = LibrashaderFilterChainOptions {
            force_no_mipmaps: true,
            disable_cache: true,
            ..Default::default()
        };

        // SAFETY: The device_ptr is provided by the platform (Flutter/Win32) and must be a valid ID3D11Device.
        // We wrap it in ManuallyDrop to prevent it from being released when the local variable goes out of scope.
        let load_result = unsafe {
            let device: ManuallyDrop<ID3D11Device> = ManuallyDrop::new(transmute_copy(&device_ptr));

            let res = (|| {
                let preset = if let Some(p) = preparsed_preset {
                    p.clone()
                } else {
                    ShaderPreset::try_parse(&path, features).map_err(|e| format!("{:?}", e))?
                };

                LibrashaderFilterChain::load_from_preset(preset, &*device, Some(&options))
                    .map_err(|e| format!("{:?}", e))
            })();
            res
        };

        match load_result {
            Ok(chain) => {
                tracing::info!(
                    "Windows shader chain loaded from {} (device={:?})",
                    path,
                    device_ptr
                );

                // Apply parameter overrides.
                // We combine the captured parameters with the latest global config to prevent race conditions during loading.
                let mut final_parameters = parameters;
                if let Some(cfg) = crate::windows::WINDOWS_SHADER_CONFIG.load().as_ref() {
                    if cfg.generation == generation {
                        for (name, value) in &cfg.parameters {
                            final_parameters.insert(name.clone(), *value);
                        }
                    }
                }

                for (name, value) in &final_parameters {
                    chain.parameters().set_parameter_value(name, *value);
                }

                SHADER_SESSION.store(Some(Arc::new(ShaderSession {
                    chain: Mutex::new(Some(chain)),
                    generation,
                    device_addr: device_addr,
                })));
            }
            Err(e) => {
                tracing::error!("Failed to load Windows shader preset ({}): {:?}", path, e);

                SHADER_SESSION.store(Some(Arc::new(ShaderSession {
                    chain: Mutex::new(None),
                    generation,
                    device_addr: device_addr,
                })));
            }
        };
    });
}
