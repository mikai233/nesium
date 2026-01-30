use crate::api::video::ShaderParameters;
use crate::senders::shader::emit_shader_parameters_update;

use super::session::{SHADER_SESSION, ShaderSession};
use librashader::presets::{
    ShaderFeatures as LibrashaderShaderFeatures, ShaderPreset, get_parameter_meta,
};
use librashader::runtime::Viewport;
use librashader::runtime::d3d11::FrameOptions as LibrashaderFrameOptions;
use librashader::runtime::d3d11::{
    FilterChain as LibrashaderFilterChain, FilterChainOptions as LibrashaderFilterChainOptions,
};
use parking_lot::Mutex;
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
) -> bool {
    tracing::info!(
        "Reloading shader chain (path={}, device changed=...)",
        effective_path
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

        let mut parameters = Vec::new();
        let res = (|| {
            let preset = ShaderPreset::try_parse(effective_path, features)
                .map_err(|e| format!("{:?}", e))?;

            if let Ok(meta) = get_parameter_meta(&preset) {
                for p in meta {
                    parameters.push(p.clone());
                }
            }

            LibrashaderFilterChain::load_from_preset(preset, &*device, Some(&options))
                .map_err(|e| format!("{:?}", e))
        })();

        match res {
            Ok(chain) => {
                tracing::info!("Windows shader chain loaded from {}", effective_path);

                // Emit update to Flutter immediately
                let api_parameters = parameters
                    .iter()
                    .map(|meta| crate::api::video::ShaderParameter {
                        name: meta.id.to_string(),
                        description: meta.description.clone(),
                        initial: meta.initial,
                        current: meta.initial,
                        minimum: meta.minimum,
                        maximum: meta.maximum,
                        step: meta.step,
                    })
                    .collect();

                emit_shader_parameters_update(ShaderParameters {
                    path: effective_path.to_string(),
                    parameters: api_parameters,
                });

                SHADER_SESSION.store(Some(Arc::new(ShaderSession {
                    chain: Mutex::new(Some(chain)),
                    generation,
                    device_addr: device_ptr as usize,
                    parameters,
                    path: effective_path.to_string(),
                })));
                true
            }
            Err(e) => {
                tracing::error!(
                    "Failed to load Windows shader preset ({}): {:?}",
                    effective_path,
                    e
                );

                emit_shader_parameters_update(ShaderParameters {
                    path: effective_path.to_string(),
                    parameters: Vec::new(),
                });

                SHADER_SESSION.store(Some(Arc::new(ShaderSession {
                    chain: Mutex::new(None),
                    generation,
                    device_addr: device_ptr as usize,
                    parameters: Vec::new(),
                    path: effective_path.to_string(),
                })));
                false
            }
        }
    };

    load_result
}
