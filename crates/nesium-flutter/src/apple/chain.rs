use super::session::{SHADER_SESSION, ShaderSession};
use librashader::presets::{ShaderFeatures as LibrashaderShaderFeatures, ShaderPreset};
use librashader::runtime::mtl::{
    FilterChain as LibrashaderFilterChain, FilterChainOptions as LibrashaderFilterChainOptions,
    FrameOptions as LibrashaderFrameOptions,
};
use librashader::runtime::{
    FilterChainParameters as _, Size as LibrashaderSize, Viewport as LibrashaderViewport,
};
use parking_lot::Mutex;
use std::collections::HashMap;
use std::ffi::c_void;
use std::sync::Arc;

pub(crate) unsafe fn render_shader_frame(
    chain: &mut LibrashaderFilterChain,
    input_tex_ptr: *mut c_void,
    output_tex_ptr: *mut c_void,
    command_buffer_ptr: *mut c_void,
    viewport_width: u32,
    viewport_height: u32,
    frame_count: usize,
) -> bool {
    // Leverage type inference for viewport
    let viewport = LibrashaderViewport {
        x: 0.0,
        y: 0.0,
        mvp: None,
        // SAFETY: output_tex_ptr is valid id<MTLTexture>
        output: unsafe { std::mem::transmute(output_tex_ptr) },
        size: LibrashaderSize {
            width: viewport_width,
            height: viewport_height,
        },
    };

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
}

pub(crate) fn reload_shader_chain(
    effective_path: String,
    device_ptr: *mut c_void,
    command_queue_ptr: *mut c_void,
    generation: u64,
    preparsed_preset: Option<ShaderPreset>,
    parameters: HashMap<String, f32>,
) {
    let device_addr = device_ptr as usize;
    let command_queue_addr = command_queue_ptr as usize;

    std::thread::spawn(move || {
        let device_ptr = device_addr as *mut c_void;
        let command_queue_ptr = command_queue_addr as *mut c_void;

        tracing::info!(
            "Reloading Apple Metal shader chain (async, path={}, generation={}, device={:?}, queue={:?})",
            effective_path,
            generation,
            device_ptr,
            command_queue_ptr
        );
        let features = LibrashaderShaderFeatures::ORIGINAL_ASPECT_UNIFORMS
            | LibrashaderShaderFeatures::FRAMETIME_UNIFORMS;

        let options = LibrashaderFilterChainOptions {
            force_no_mipmaps: true,
            ..Default::default()
        };

        let load_result = (|| {
            let preset = if let Some(p) = preparsed_preset {
                // Use already parsed preset
                p.clone()
            } else {
                ShaderPreset::try_parse(&effective_path, features)
                    .map_err(|e| format!("{:?}", e))?
            };

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
                tracing::info!(
                    "Apple shader chain loaded from {} (device={:?}, queue={:?})",
                    effective_path,
                    device_ptr,
                    command_queue_ptr
                );

                // Apply parameter overrides
                for (name, value) in &parameters {
                    chain.parameters().set_parameter_value(name, *value);
                }

                SHADER_SESSION.store(Some(Arc::new(ShaderSession {
                    chain: Mutex::new(Some(chain)),
                    generation,
                    device_addr: device_ptr as usize,
                })));
            }
            Err(e) => {
                tracing::error!(
                    "Failed to load Apple shader preset ({}): {:?}",
                    effective_path,
                    e
                );

                SHADER_SESSION.store(Some(Arc::new(ShaderSession {
                    chain: Mutex::new(None),
                    generation,
                    device_addr: device_ptr as usize,
                })));
            }
        };
    });
}
