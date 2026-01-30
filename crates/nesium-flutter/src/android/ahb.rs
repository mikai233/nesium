use parking_lot::{Condvar, Mutex};
use std::ffi::c_void;
use std::os::raw::c_int;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Duration;

use super::renderer::rust_renderer_wake;
use super::{RUST_RENDERER_RUNNING, is_fast_forwarding};

#[repr(C)]
pub struct AHardwareBuffer {
    _private: [u8; 0],
}

#[repr(C)]
#[allow(non_camel_case_types)]
pub struct AHardwareBuffer_Desc {
    pub width: u32,
    pub height: u32,
    pub layers: u32,
    pub format: u32,
    pub usage: u64,
    pub stride: u32,
    pub rfu0: u32,
    pub rfu1: u64,
}

#[repr(C)]
pub struct ARect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

const AHARDWAREBUFFER_FORMAT_R8G8B8A8_UNORM: u32 = 1;
const AHARDWAREBUFFER_USAGE_CPU_WRITE_OFTEN: u64 = 0x30;
const AHARDWAREBUFFER_USAGE_GPU_SAMPLED_IMAGE: u64 = 0x100;

#[link(name = "android")]
unsafe extern "C" {
    fn AHardwareBuffer_allocate(
        desc: *const AHardwareBuffer_Desc,
        out: *mut *mut AHardwareBuffer,
    ) -> c_int;
    fn AHardwareBuffer_release(buffer: *mut AHardwareBuffer);
    fn AHardwareBuffer_describe(buffer: *const AHardwareBuffer, out: *mut AHardwareBuffer_Desc);
    fn AHardwareBuffer_lock(
        buffer: *mut AHardwareBuffer,
        usage: u64,
        fence: c_int,
        rect: *const ARect,
        out_virtual_address: *mut *mut c_void,
    ) -> c_int;
    fn AHardwareBuffer_unlock(buffer: *mut AHardwareBuffer, fence: *mut c_int) -> c_int;
}

pub struct AhbSwapchain {
    sync_mu: Mutex<AhbState>,
    sync_cv: Condvar,
    generation: AtomicU32,
}

struct AhbState {
    buffers: [*mut AHardwareBuffer; 2],
    width: u32,
    height: u32,
    pitch_bytes: usize,
    fallback_planes: [Box<[u8]>; 2],
    gpu_busy: [bool; 2],
    cpu_locked: [bool; 2],
    cpu_locked_ahb: [bool; 2],
    resizing: bool,
    retired_buffers: Vec<[*mut AHardwareBuffer; 2]>,
}

// SAFETY: The swapchain buffers are stable native handles; access is coordinated via internal
// atomics/mutexes and the Android NDK AHardwareBuffer APIs are thread-safe.
unsafe impl Send for AhbSwapchain {}
unsafe impl Sync for AhbSwapchain {}

impl AhbSwapchain {
    pub fn new(width: u32, height: u32) -> Result<Self, String> {
        let (buffers, pitch_bytes, fallback_planes) = allocate_buffers(width, height)?;

        Ok(Self {
            sync_mu: Mutex::new(AhbState {
                buffers,
                width,
                height,
                pitch_bytes,
                fallback_planes,
                gpu_busy: [false; 2],
                cpu_locked: [false; 2],
                cpu_locked_ahb: [false; 2],
                resizing: false,
                retired_buffers: Vec::new(),
            }),
            sync_cv: Condvar::new(),
            generation: AtomicU32::new(0),
        })
    }

    pub fn pitch_bytes(&self) -> usize {
        let mut state = self.sync_mu.lock();
        while state.resizing {
            self.sync_cv.wait(&mut state);
        }
        state.pitch_bytes
    }

    pub fn buffer(&self, idx: usize) -> *mut AHardwareBuffer {
        let mut state = self.sync_mu.lock();
        while state.resizing {
            self.sync_cv.wait(&mut state);
        }
        state.buffers[idx]
    }

    pub fn width(&self) -> u32 {
        let mut state = self.sync_mu.lock();
        while state.resizing {
            self.sync_cv.wait(&mut state);
        }
        state.width
    }

    pub fn height(&self) -> u32 {
        let mut state = self.sync_mu.lock();
        while state.resizing {
            self.sync_cv.wait(&mut state);
        }
        state.height
    }

    pub fn generation(&self) -> u32 {
        self.generation.load(Ordering::Acquire)
    }

    pub fn resize(&self, width: u32, height: u32) -> Result<(), String> {
        if width == 0 || height == 0 {
            return Err("invalid output size".to_string());
        }

        let (old_buffers, should_retire) = {
            let mut state = self.sync_mu.lock();
            if state.width == width && state.height == height {
                return Ok(());
            }

            state.resizing = true;
            self.sync_cv.notify_all();

            while state.cpu_locked.iter().any(|&b| b) || state.gpu_busy.iter().any(|&b| b) {
                self.sync_cv.wait(&mut state);
            }

            (state.buffers, RUST_RENDERER_RUNNING.load(Ordering::Acquire))
        };

        let (new_buffers, pitch_bytes, fallback_planes) = match allocate_buffers(width, height) {
            Ok(v) => v,
            Err(e) => {
                let mut state = self.sync_mu.lock();
                state.resizing = false;
                self.sync_cv.notify_all();
                return Err(e);
            }
        };

        let to_release = {
            let mut state = self.sync_mu.lock();
            if should_retire {
                state.retired_buffers.push(old_buffers);
            }
            state.buffers = new_buffers;
            state.width = width;
            state.height = height;
            state.pitch_bytes = pitch_bytes;
            state.fallback_planes = fallback_planes;
            state.resizing = false;
            self.generation.fetch_add(1, Ordering::AcqRel);
            self.sync_cv.notify_all();
            if should_retire {
                None
            } else {
                Some(old_buffers)
            }
        };

        if let Some(buffers) = to_release {
            unsafe { release_buffers(buffers) };
        }

        rust_renderer_wake();
        Ok(())
    }

    pub fn take_retired_buffers(&self) -> Vec<[*mut AHardwareBuffer; 2]> {
        let mut state = self.sync_mu.lock();
        std::mem::take(&mut state.retired_buffers)
    }

    pub fn wait_gpu_idle(&self, idx: usize) {
        let mut state = self.sync_mu.lock();
        while state.resizing || state.gpu_busy[idx] {
            self.sync_cv.wait(&mut state);
        }
    }

    pub fn set_gpu_busy(&self, idx: usize, busy: bool) {
        let mut state = self.sync_mu.lock();
        if busy {
            while state.resizing {
                self.sync_cv.wait(&mut state);
            }
            state.gpu_busy[idx] = true;
            return;
        }

        // Clearing busy must never block on `resizing`, otherwise we can deadlock:
        // resize waits for `gpu_busy=false`, while the renderer waits to clear it.
        state.gpu_busy[idx] = false;
        self.sync_cv.notify_all();
    }

    pub fn lock_plane(&self, idx: usize) -> *mut u8 {
        let (buffer, fallback_ptr) = {
            let mut state = self.sync_mu.lock();
            // During fast-forward, skip GPU sync wait to allow faster emulation.
            // Frames may be overwritten before the GPU renders them (dropped frames).
            let skip_gpu_wait = is_fast_forwarding();
            while state.resizing || (!skip_gpu_wait && state.gpu_busy[idx]) {
                self.sync_cv.wait(&mut state);
            }
            state.cpu_locked[idx] = true;
            state.cpu_locked_ahb[idx] = true;
            (
                state.buffers[idx],
                state.fallback_planes[idx].as_ptr() as *mut u8,
            )
        };

        let mut out: *mut c_void;
        let mut last_err: c_int = 0;
        for attempt in 0..6u32 {
            out = std::ptr::null_mut();
            let res = unsafe {
                AHardwareBuffer_lock(
                    buffer,
                    AHARDWAREBUFFER_USAGE_CPU_WRITE_OFTEN,
                    -1,
                    std::ptr::null(),
                    &mut out as *mut _,
                )
            };
            if res == 0 && !out.is_null() {
                return out as *mut u8;
            }
            last_err = res;

            // Short backoff to tolerate transient failures; avoid spinning too aggressively.
            let backoff_ms = (1u64 << attempt).min(16);
            std::thread::sleep(Duration::from_millis(backoff_ms));
        }

        tracing::error!(
            "AHardwareBuffer_lock failed for idx={idx} (err={last_err}); falling back to dummy buffer"
        );
        let mut state = self.sync_mu.lock();
        state.cpu_locked_ahb[idx] = false;
        fallback_ptr
    }

    pub fn unlock_plane(&self, idx: usize) {
        let (buffer, should_unlock) = {
            let state = self.sync_mu.lock();
            if !state.cpu_locked[idx] {
                return;
            }
            (state.buffers[idx], state.cpu_locked_ahb[idx])
        };

        if should_unlock {
            let res = unsafe { AHardwareBuffer_unlock(buffer, std::ptr::null_mut()) };
            if res != 0 {
                tracing::error!("AHardwareBuffer_unlock failed: {res}");
            }
        }

        let mut state = self.sync_mu.lock();
        state.cpu_locked[idx] = false;
        state.cpu_locked_ahb[idx] = false;
        self.sync_cv.notify_all();
    }
}

pub extern "C" fn ahb_lock_plane(
    buffer_index: u32,
    pitch_out: *mut u32,
    user_data: *mut c_void,
) -> *mut u8 {
    // SAFETY: user_data is a stable pointer to AhbSwapchain.
    let swapchain = unsafe { &*(user_data as *const AhbSwapchain) };
    let ptr = swapchain.lock_plane(buffer_index as usize);
    unsafe {
        *pitch_out = swapchain.pitch_bytes() as u32;
    }
    ptr
}

pub extern "C" fn ahb_unlock_plane(buffer_index: u32, user_data: *mut c_void) {
    // SAFETY: user_data is a stable pointer to AhbSwapchain.
    let swapchain = unsafe { &*(user_data as *const AhbSwapchain) };
    swapchain.unlock_plane(buffer_index as usize);
}

impl Drop for AhbSwapchain {
    fn drop(&mut self) {
        let state = self.sync_mu.get_mut();
        unsafe {
            release_buffers(state.buffers);
            for retired in state.retired_buffers.drain(..) {
                release_buffers(retired);
            }
        }
    }
}

pub fn allocate_buffers(
    width: u32,
    height: u32,
) -> Result<([*mut AHardwareBuffer; 2], usize, [Box<[u8]>; 2]), String> {
    let mut buffers: [*mut AHardwareBuffer; 2] = [std::ptr::null_mut(), std::ptr::null_mut()];
    let desc = AHardwareBuffer_Desc {
        width,
        height,
        layers: 1,
        format: AHARDWAREBUFFER_FORMAT_R8G8B8A8_UNORM,
        usage: AHARDWAREBUFFER_USAGE_CPU_WRITE_OFTEN | AHARDWAREBUFFER_USAGE_GPU_SAMPLED_IMAGE,
        stride: 0,
        rfu0: 0,
        rfu1: 0,
    };

    for slot in &mut buffers {
        let mut out: *mut AHardwareBuffer = std::ptr::null_mut();
        let res = unsafe { AHardwareBuffer_allocate(&desc as *const _, &mut out as *mut _) };
        if res != 0 || out.is_null() {
            unsafe { release_buffers(buffers) };
            return Err(format!("AHardwareBuffer_allocate failed: {res}"));
        }
        *slot = out;
    }

    let mut described = AHardwareBuffer_Desc {
        width: 0,
        height: 0,
        layers: 0,
        format: 0,
        usage: 0,
        stride: 0,
        rfu0: 0,
        rfu1: 0,
    };
    unsafe { AHardwareBuffer_describe(buffers[0] as *const _, &mut described as *mut _) };
    let stride_pixels = described.stride.max(width);
    let pitch_bytes = stride_pixels as usize * 4;
    let fallback_len = pitch_bytes * height as usize;
    let fallback_planes = [
        vec![0u8; fallback_len].into_boxed_slice(),
        vec![0u8; fallback_len].into_boxed_slice(),
    ];

    Ok((buffers, pitch_bytes, fallback_planes))
}

pub unsafe fn release_buffers(buffers: [*mut AHardwareBuffer; 2]) {
    for b in buffers {
        if !b.is_null() {
            unsafe { AHardwareBuffer_release(b) };
        }
    }
}

pub struct GpuBusyGuard {
    pub swapchain: &'static AhbSwapchain,
    pub idx: u32,
}

impl GpuBusyGuard {
    pub fn new(swapchain: &'static AhbSwapchain, idx: u32) -> Self {
        swapchain.set_gpu_busy(idx as usize, true);
        Self { swapchain, idx }
    }
}

impl Drop for GpuBusyGuard {
    fn drop(&mut self) {
        self.swapchain.set_gpu_busy(self.idx as usize, false);
    }
}
