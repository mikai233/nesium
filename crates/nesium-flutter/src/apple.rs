mod chain;
mod passthrough;
pub mod session;

use std::ffi::c_void;
use std::sync::atomic::Ordering;

use chain::{reload_shader_chain, render_shader_frame};
use passthrough::get_passthrough_preset;
use session::{FRAME_COUNT, SHADER_SESSION, apple_shader_snapshot};

// Re-export state functions and state for api/video.rs
pub use session::{apple_set_shader_enabled, apple_set_shader_preset_path};

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

        let current_session = SHADER_SESSION.load();

        // Reload if generation changed OR device changed OR shader not loaded
        let needs_reload = match &*current_session {
            Some(session) => {
                session.generation != cfg.generation || session.device_addr != device_ptr as usize
            }
            None => true,
        };

        if needs_reload {
            reload_shader_chain(
                &effective_path,
                device_ptr,
                command_queue_ptr,
                cfg.generation,
            );
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

        let frame_count = FRAME_COUNT.fetch_add(1, Ordering::Relaxed);

        // SAFETY: Pointers are checked for nullity above.
        // We trust the caller (Flutter/iOS/macOS shell) to provide valid Metal objects.
        unsafe {
            render_shader_frame(
                chain,
                input_tex_ptr,
                output_tex_ptr,
                command_buffer_ptr,
                dst_width,
                dst_height,
                frame_count,
            )
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
