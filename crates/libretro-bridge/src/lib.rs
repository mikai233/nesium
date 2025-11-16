#![doc = include_str!("../README.md")]

mod core;
mod runtime;
mod state;

#[doc(hidden)]
pub mod __private {
    pub use crate::state::CoreInstance;
}

pub use crate::core::{
    GameGeometry, GameInfo, LibretroCore, LoadGameError, MemoryRegion, SerializeError,
    SystemAvInfo, SystemInfo, SystemTiming,
};
pub use crate::runtime::{Audio, Environment, Frame, Input, RuntimeHandles, Video};

/// Raw bindings for `libretro.h`. Generated via `bindgen` in `build.rs`.
#[allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
pub mod raw {
    include!(concat!(env!("OUT_DIR"), "/libretro_bindings.rs"));
}

pub use raw::*;

/// Exports the provided [`LibretroCore`](crate::LibretroCore) implementation as
/// a libretro entry point table.
///
/// ```no_run
/// use libretro_bridge::{export_libretro_core, LibretroCore};
///
/// struct MyCore;
///
/// impl LibretroCore for MyCore {
///     # fn construct() -> Self where Self: Sized { Self }
///     # fn system_info() -> libretro_bridge::SystemInfo {
///     #     libretro_bridge::SystemInfo::new("MyCore", "0.1")
///     # }
///     # fn run(&mut self, _rt: &mut libretro_bridge::RuntimeHandles) {}
///     # fn system_av_info(&mut self) -> libretro_bridge::SystemAvInfo {
///     #     unimplemented!()
///     # }
///     # fn load_game(&mut self, _: &libretro_bridge::GameInfo<'_>) -> Result<(), libretro_bridge::LoadGameError> {
///     #     Ok(())
///     # }
///     # fn unload_game(&mut self) {}
/// }
///
/// export_libretro_core!(MyCore);
/// ```
#[macro_export]
macro_rules! export_libretro_core {
    ($core:ty) => {
        const _: () = {
            fn __libretro_bridge_state() -> &'static $crate::__private::CoreInstance<$core> {
                static STATE: ::once_cell::sync::Lazy<$crate::__private::CoreInstance<$core>> =
                    ::once_cell::sync::Lazy::new(|| $crate::__private::CoreInstance::<$core>::new());
                &STATE
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn retro_set_environment(
                cb: $crate::raw::retro_environment_t,
            ) {
                __libretro_bridge_state().set_environment(cb);
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn retro_set_video_refresh(
                cb: $crate::raw::retro_video_refresh_t,
            ) {
                __libretro_bridge_state().set_video_refresh(cb);
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn retro_set_audio_sample(
                cb: $crate::raw::retro_audio_sample_t,
            ) {
                __libretro_bridge_state().set_audio_sample(cb);
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn retro_set_audio_sample_batch(
                cb: $crate::raw::retro_audio_sample_batch_t,
            ) {
                __libretro_bridge_state().set_audio_batch(cb);
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn retro_set_input_poll(
                cb: $crate::raw::retro_input_poll_t,
            ) {
                __libretro_bridge_state().set_input_poll(cb);
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn retro_set_input_state(
                cb: $crate::raw::retro_input_state_t,
            ) {
                __libretro_bridge_state().set_input_state(cb);
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn retro_init() {
                __libretro_bridge_state().init();
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn retro_deinit() {
                __libretro_bridge_state().deinit();
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn retro_api_version() -> u32 {
                __libretro_bridge_state().api_version()
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn retro_get_system_info(
                info: *mut $crate::raw::retro_system_info,
            ) {
                unsafe { __libretro_bridge_state().system_info(info) };
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn retro_get_system_av_info(
                info: *mut $crate::raw::retro_system_av_info,
            ) {
                unsafe { __libretro_bridge_state().system_av_info(info) };
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn retro_set_controller_port_device(
                port: u32,
                device: u32,
            ) {
                __libretro_bridge_state().set_controller_port_device(port, device);
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn retro_reset() {
                __libretro_bridge_state().reset();
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn retro_run() {
                __libretro_bridge_state().run();
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn retro_serialize_size() -> usize {
                __libretro_bridge_state().serialize_size()
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn retro_serialize(
                data: *mut ::std::ffi::c_void,
                len: usize,
            ) -> bool {
                unsafe { __libretro_bridge_state().serialize(data, len) }
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn retro_unserialize(
                data: *const ::std::ffi::c_void,
                len: usize,
            ) -> bool {
                unsafe { __libretro_bridge_state().unserialize(data, len) }
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn retro_cheat_reset() {
                __libretro_bridge_state().cheat_reset();
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn retro_cheat_set(
                index: u32,
                enabled: bool,
                code: *const ::std::os::raw::c_char,
            ) {
                unsafe { __libretro_bridge_state().cheat_set(index, enabled, code) };
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn retro_load_game(
                game: *const $crate::raw::retro_game_info,
            ) -> bool {
                unsafe { __libretro_bridge_state().load_game(game) }
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn retro_load_game_special(
                game_type: u32,
                info: *const $crate::raw::retro_game_info,
                num_info: usize,
            ) -> bool {
                unsafe { __libretro_bridge_state().load_game_special(game_type, info, num_info) }
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn retro_unload_game() {
                __libretro_bridge_state().unload_game();
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn retro_get_region() -> u32 {
                __libretro_bridge_state().region()
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn retro_get_memory_data(id: u32) -> *mut ::std::ffi::c_void {
                __libretro_bridge_state().memory_data(id)
            }

            #[unsafe(no_mangle)]
            pub unsafe extern "C" fn retro_get_memory_size(id: u32) -> usize {
                __libretro_bridge_state().memory_size(id)
            }
        };
    };
}

#[cfg(test)]
mod tests;
