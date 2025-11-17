#![allow(dead_code)]

use crate::{
    core::{GameInfo, LibretroCore, MemoryRegion, SerializeError, SystemInfo},
    raw,
    runtime::{CallbackSet, RuntimeHandles},
};
use std::{
    ffi::{CStr, CString, c_char, c_void},
    ptr, slice,
    sync::Mutex,
};

#[doc(hidden)]
pub struct CoreInstance<T: LibretroCore> {
    callbacks: Mutex<CallbackSet>,
    core: Mutex<Option<T>>,
    system_info: SystemInfoCache,
}

impl<T: LibretroCore> CoreInstance<T> {
    pub fn new() -> Self {
        Self {
            callbacks: Mutex::new(CallbackSet::default()),
            core: Mutex::new(None),
            system_info: SystemInfoCache::from_info(T::system_info()),
        }
    }

    pub fn set_environment(&self, cb: raw::retro_environment_t) {
        self.callbacks.lock().unwrap().set_environment(cb);
    }

    pub fn set_video_refresh(&self, cb: raw::retro_video_refresh_t) {
        self.callbacks.lock().unwrap().set_video(cb);
    }

    pub fn set_audio_sample(&self, cb: raw::retro_audio_sample_t) {
        self.callbacks.lock().unwrap().set_audio_sample(cb);
    }

    pub fn set_audio_batch(&self, cb: raw::retro_audio_sample_batch_t) {
        self.callbacks.lock().unwrap().set_audio_batch(cb);
    }

    pub fn set_input_poll(&self, cb: raw::retro_input_poll_t) {
        self.callbacks.lock().unwrap().set_input_poll(cb);
    }

    pub fn set_input_state(&self, cb: raw::retro_input_state_t) {
        self.callbacks.lock().unwrap().set_input_state(cb);
    }

    pub fn init(&self) {
        let mut guard = self.core.lock().unwrap();
        assert!(
            guard.is_none(),
            "retro_init called while the core is still active"
        );

        let mut core = T::construct();
        core.init();
        *guard = Some(core);
    }

    pub fn deinit(&self) {
        let mut guard = self.core.lock().unwrap();
        if let Some(mut core) = guard.take() {
            core.deinit();
        }
    }

    pub fn api_version(&self) -> u32 {
        T::api_version()
    }

    /// Writes cached [`SystemInfo`](crate::SystemInfo) into the raw structure.
    ///
    /// # Safety
    /// The `info` pointer must be valid for writes and properly aligned.
    pub unsafe fn system_info(&self, info: *mut raw::retro_system_info) {
        if info.is_null() {
            return;
        }

        unsafe {
            self.system_info.write_into(&mut *info);
        }
    }

    /// Writes [`SystemAvInfo`](crate::SystemAvInfo) into the raw structure.
    ///
    /// # Safety
    /// The `info` pointer must be valid for writes and properly aligned.
    pub unsafe fn system_av_info(&self, info: *mut raw::retro_system_av_info) {
        if info.is_null() {
            return;
        }

        let av_info = self.with_core_mut(|core| core.system_av_info());
        unsafe {
            *info = av_info.to_raw();
        }
    }

    pub fn reset(&self) {
        self.with_core_mut(|core| core.reset());
    }

    pub fn run(&self) {
        let callbacks = *self.callbacks.lock().unwrap();
        let mut runtime = RuntimeHandles::new(callbacks);
        self.with_core_mut(|core| core.run(&mut runtime));
    }

    /// Loads a single piece of content.
    ///
    /// # Safety
    /// The pointer passed in must reference a valid `retro_game_info`.
    pub unsafe fn load_game(&self, info: *const raw::retro_game_info) -> bool {
        let game = unsafe { GameInfo::from_ptr(info) };
        self.with_core_mut(|core| core.load_game(&game)).is_ok()
    }

    /// Loads multi-part or special content.
    ///
    /// # Safety
    /// The pointer passed in must reference an array of `retro_game_info`
    /// structures with `count` elements.
    pub unsafe fn load_game_special(
        &self,
        ty: u32,
        info: *const raw::retro_game_info,
        count: usize,
    ) -> bool {
        let games = unsafe {
            if info.is_null() || count == 0 {
                Vec::new()
            } else {
                GameInfo::from_slice(slice::from_raw_parts(info, count))
            }
        };

        self.with_core_mut(|core| core.load_game_special(ty, &games))
            .is_ok()
    }

    pub fn unload_game(&self) {
        self.with_core_mut(|core| core.unload_game());
    }

    pub fn serialize_size(&self) -> usize {
        self.with_core(|core| core.serialize_size())
    }

    /// Serializes the core into the provided buffer.
    ///
    /// # Safety
    /// The `data` pointer must be valid for `len` mutable bytes.
    pub unsafe fn serialize(&self, data: *mut c_void, len: usize) -> bool {
        let buffer = unsafe {
            if len == 0 {
                &mut []
            } else if data.is_null() {
                return false;
            } else {
                slice::from_raw_parts_mut(data as *mut u8, len)
            }
        };

        match self.with_core_mut(|core| core.serialize(buffer)) {
            Ok(_) => true,
            Err(SerializeError::BufferTooSmall { .. }) => false,
            Err(SerializeError::Unsupported) => false,
            Err(SerializeError::Message(_)) => false,
        }
    }

    /// Restores the core state from the provided buffer.
    ///
    /// # Safety
    /// The `data` pointer must be valid for `len` readable bytes.
    pub unsafe fn unserialize(&self, data: *const c_void, len: usize) -> bool {
        if len == 0 {
            return true;
        }

        if data.is_null() {
            return false;
        }

        let slice = unsafe { slice::from_raw_parts(data as *const u8, len) };
        self.with_core_mut(|core| core.unserialize(slice)).is_ok()
    }

    pub fn cheat_reset(&self) {
        self.with_core_mut(|core| core.cheat_reset());
    }

    /// Forwards a cheat code to the active core.
    ///
    /// # Safety
    /// The `code` pointer must either be null or a null-terminated string.
    pub unsafe fn cheat_set(&self, index: u32, enabled: bool, code: *const c_char) {
        let code = c_string(code).unwrap_or_default();
        self.with_core_mut(|core| core.cheat_set(index, enabled, &code));
    }

    pub fn set_controller_port_device(&self, port: u32, device: u32) {
        self.with_core_mut(|core| core.set_controller_port_device(port, device));
    }

    pub fn region(&self) -> u32 {
        self.with_core(|core| core.region())
    }

    pub fn memory_data(&self, id: u32) -> *mut c_void {
        self.with_core_mut(|core| {
            core.get_memory_data(id)
                .map(MemoryRegion::into_raw)
                .map(|(ptr, _)| ptr)
                .unwrap_or(ptr::null_mut())
        })
    }

    pub fn memory_size(&self, id: u32) -> usize {
        self.with_core_mut(|core| {
            core.get_memory_data(id)
                .map(MemoryRegion::into_raw)
                .map(|(_, len)| len)
                .unwrap_or(0)
        })
    }

    fn with_core<R>(&self, f: impl FnOnce(&T) -> R) -> R {
        let guard = self.core.lock().unwrap();
        let core = guard
            .as_ref()
            .expect("libretro core has not been initialized");
        f(core)
    }

    fn with_core_mut<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut guard = self.core.lock().unwrap();
        let core = guard
            .as_mut()
            .expect("libretro core has not been initialized");
        f(core)
    }
}

impl<T: LibretroCore> Default for CoreInstance<T> {
    fn default() -> Self {
        Self::new()
    }
}

struct SystemInfoCache {
    library_name: CString,
    library_version: CString,
    valid_extensions: Option<CString>,
    need_fullpath: bool,
    block_extract: bool,
}

impl SystemInfoCache {
    fn from_info(info: SystemInfo) -> Self {
        Self {
            library_name: CString::new(info.library_name).expect("library name contains NUL bytes"),
            library_version: CString::new(info.library_version)
                .expect("library version contains NUL bytes"),
            valid_extensions: info
                .valid_extensions
                .map(|ext| CString::new(ext).expect("extensions contain NUL bytes")),
            need_fullpath: info.need_fullpath,
            block_extract: info.block_extract,
        }
    }

    unsafe fn write_into(&self, info: &mut raw::retro_system_info) {
        info.library_name = self.library_name.as_ptr();
        info.library_version = self.library_version.as_ptr();
        info.valid_extensions = self
            .valid_extensions
            .as_ref()
            .map(|ext| ext.as_ptr())
            .unwrap_or(ptr::null());
        info.need_fullpath = self.need_fullpath;
        info.block_extract = self.block_extract;
    }
}

fn c_string(ptr: *const c_char) -> Option<String> {
    if ptr.is_null() {
        return None;
    }

    unsafe { CStr::from_ptr(ptr) }
        .to_str()
        .ok()
        .map(|s| s.to_owned())
}
