mod chain;
mod passthrough;
pub mod session;

use std::ffi::c_void;
use std::sync::atomic::Ordering;

use chain::render_shader_frame;
use session::{FRAME_COUNT, SHADER_SESSION};

// Re-export state functions and state for api/video.rs
pub use session::{apple_set_shader_config, apple_set_shader_preset_path};

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
        if input_tex_ptr.is_null() || output_tex_ptr.is_null() {
            return false;
        }
        if src_width == 0 || src_height == 0 || dst_width == 0 || dst_height == 0 {
            return false;
        }
        if device_ptr.is_null() || command_queue_ptr.is_null() || command_buffer_ptr.is_null() {
            return false;
        }

        session::LAST_DEVICE_ADDR.store(device_ptr as usize, Ordering::Release);
        session::try_trigger_reload(device_ptr, command_queue_ptr);

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
