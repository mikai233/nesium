mod chain;
mod passthrough;
pub mod session;
mod utils;

use std::ffi::c_void;
use std::mem::{ManuallyDrop, transmute_copy};
use std::sync::atomic::Ordering;

use librashader::runtime::Size as LibrashaderSize;
use librashader::runtime::Viewport as LibrashaderViewport;
use windows::Win32::Graphics::Direct3D11::{ID3D11Device, ID3D11DeviceContext, ID3D11Resource};

use chain::{reload_shader_chain, render_shader_frame};
use passthrough::get_passthrough_preset;
use session::{FRAME_COUNT, windows_shader_snapshot};
use utils::{log_hresult_context, validate_resource_device};

// Re-export state functions and state for api/video.rs
pub use session::{SHADER_SESSION, windows_set_shader_config, windows_set_shader_preset_path};

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

        if input_tex.is_null() || output_tex.is_null() || device.is_null() || context.is_null() {
            tracing::error!("nesium_apply_shader: null ptr(s)");
            return false;
        }

        if src_width == 0 || src_height == 0 || dst_width == 0 || dst_height == 0 {
            tracing::error!("nesium_apply_shader: invalid sizes");
            return false;
        }

        let current_session = SHADER_SESSION.load();

        session::LAST_DEVICE_ADDR.store(device as usize, Ordering::Release);

        let needs_reload_gen = match &*current_session {
            Some(session) => session.generation != cfg.generation,
            None => true,
        };
        let needs_reload_device = match &*current_session {
            Some(session) => session.device_addr != device as usize,
            None => false,
        };

        let loading_gen = session::LOADING_GENERATION.load(Ordering::Acquire);
        if needs_reload_device
            || (needs_reload_gen
                && loading_gen != cfg.generation
                && session::LOADING_GENERATION
                    .compare_exchange(
                        loading_gen,
                        cfg.generation,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    )
                    .is_ok())
        {
            reload_shader_chain(&effective_path, device, cfg.generation);
        }

        // Re-load session after potential reload
        let current_session = SHADER_SESSION.load();
        let Some(session) = &*current_session else {
            return false;
        };

        // Lock the internal chain for rendering
        let mut chain = session.chain.lock();
        let Some(chain) = chain.as_mut() else {
            return false;
        };

        // SAFETY: The provided pointers are assumed to be valid D3D11 pointers.
        // We use ManuallyDrop to prevent the interfaces from being released.
        let (device_ref, context_ref, input_res, output_res) = unsafe {
            (
                ManuallyDrop::new(transmute_copy::<_, ID3D11Device>(&device)),
                ManuallyDrop::new(transmute_copy::<_, ID3D11DeviceContext>(&context)),
                ManuallyDrop::new(transmute_copy::<_, ID3D11Resource>(&input_tex)),
                ManuallyDrop::new(transmute_copy::<_, ID3D11Resource>(&output_tex)),
            )
        };

        // Validate resource/device consistency
        // SAFETY: input_res and output_res are valid references to D3D11 resources.
        unsafe {
            if let Err(hr) = validate_resource_device("input", &input_res, device) {
                log_hresult_context("Input resource/device validation failed", hr);
                return false;
            }
            if let Err(hr) = validate_resource_device("output", &output_res, device) {
                log_hresult_context("Output resource/device validation failed", hr);
                return false;
            }
        }

        // Create Views
        let srv = match utils::create_srv(&device_ref, &input_res) {
            Ok(s) => s,
            Err(hr) => {
                log_hresult_context("Failed to create SRV", hr);
                return false;
            }
        };

        let rtv = match utils::create_rtv(&device_ref, &output_res) {
            Ok(r) => r,
            Err(hr) => {
                log_hresult_context("Failed to create RTV", hr);
                return false;
            }
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

        // SAFETY: chain, context_ref, srv, and viewport are valid.
        unsafe { render_shader_frame(chain, &context_ref, &srv, &viewport, frame_count) }
    });

    match result {
        Ok(val) => val,
        Err(e) => {
            tracing::error!("Panic in nesium_apply_shader: {:?}", e);
            false
        }
    }
}
