//! Auxiliary texture management for debugger views (Tilemap, Pattern, etc.)
//!
//! This module provides a simple double-buffered texture system completely
//! separate from the main NES screen rendering pipeline.

use std::{
    collections::HashMap,
    os::raw::c_uint,
    sync::{
        Mutex, OnceLock,
        atomic::{AtomicU8, Ordering},
    },
};

/// A double-buffered RGBA texture for auxiliary views.
pub struct AuxTexture {
    pub width: u32,
    pub height: u32,
    planes: [Box<[u8]>; 2],
    /// Index of the buffer currently being written to (0 or 1).
    write_idx: AtomicU8,
    /// Index of the buffer that contains the latest complete frame (0 or 1).
    ready_idx: AtomicU8,
}

impl AuxTexture {
    /// Creates a new auxiliary texture with the given dimensions.
    /// Pixel format is BGRA8888 (matching macOS CVPixelBuffer).
    pub fn new(width: u32, height: u32) -> Self {
        let len = (width as usize) * (height as usize) * 4;
        Self {
            width,
            height,
            planes: [
                vec![0u8; len].into_boxed_slice(),
                vec![0u8; len].into_boxed_slice(),
            ],
            write_idx: AtomicU8::new(0),
            ready_idx: AtomicU8::new(0),
        }
    }

    /// Returns a mutable slice to the back buffer for writing.
    /// After writing, call `commit()` to swap buffers.
    pub fn back_buffer_mut(&mut self) -> &mut [u8] {
        let idx = self.write_idx.load(Ordering::Acquire) as usize;
        &mut self.planes[idx]
    }

    /// Commits the current back buffer, making it the new front buffer.
    pub fn commit(&self) {
        let idx = self.write_idx.load(Ordering::Acquire);
        self.ready_idx.store(idx, Ordering::Release);
        // Swap write index for next frame.
        self.write_idx.store(1 - idx, Ordering::Release);
    }

    /// Copies the front buffer to the destination.
    /// Returns the number of bytes copied.
    pub fn copy_front_to(&self, dst: &mut [u8], dst_pitch: usize) -> usize {
        let idx = self.ready_idx.load(Ordering::Acquire) as usize;
        let src = &self.planes[idx];
        let src_pitch = (self.width as usize) * 4;
        let height = self.height as usize;

        let mut copied = 0;
        for y in 0..height {
            let src_start = y * src_pitch;
            let dst_start = y * dst_pitch;
            let row_len = src_pitch.min(dst_pitch);
            if dst_start + row_len > dst.len() || src_start + row_len > src.len() {
                break;
            }
            dst[dst_start..dst_start + row_len]
                .copy_from_slice(&src[src_start..src_start + row_len]);
            copied += row_len;
        }
        copied
    }

    /// Returns the byte length of one plane.
    #[inline]
    pub fn plane_len(&self) -> usize {
        (self.width as usize) * (self.height as usize) * 4
    }
}

// ============================================================================
// Global Registry
// ============================================================================

static AUX_TEXTURES: OnceLock<Mutex<HashMap<u32, AuxTexture>>> = OnceLock::new();

fn registry() -> &'static Mutex<HashMap<u32, AuxTexture>> {
    AUX_TEXTURES.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Creates an auxiliary texture with the given ID and dimensions.
/// If a texture with the same ID already exists, it is replaced.
pub fn aux_create(id: u32, width: u32, height: u32) {
    let tex = AuxTexture::new(width, height);
    registry().lock().unwrap().insert(id, tex);
}

/// Destroys the auxiliary texture with the given ID.
pub fn aux_destroy(id: u32) {
    registry().lock().unwrap().remove(&id);
}

/// Updates the auxiliary texture by copying RGBA data into its back buffer,
/// then commits to make it the front buffer.
///
/// Returns `true` on success.
pub fn aux_update(id: u32, rgba: &[u8]) -> bool {
    let mut guard = registry().lock().unwrap();
    if let Some(tex) = guard.get_mut(&id) {
        let expected_len = tex.plane_len();
        if rgba.len() >= expected_len {
            tex.back_buffer_mut()[..expected_len].copy_from_slice(&rgba[..expected_len]);
            tex.commit();
            return true;
        }
    }
    false
}

/// Copies the front buffer of the auxiliary texture to `dst`.
///
/// Returns the number of bytes copied, or 0 if the texture does not exist.
pub fn aux_copy(id: u32, dst: &mut [u8], dst_pitch: usize) -> usize {
    let guard = registry().lock().unwrap();
    if let Some(tex) = guard.get(&id) {
        tex.copy_front_to(dst, dst_pitch)
    } else {
        0
    }
}

// ============================================================================
// C ABI
// ============================================================================

/// Creates an auxiliary texture.
///
/// # Safety
/// This function is safe to call from C.
#[unsafe(no_mangle)]
pub extern "C" fn nesium_aux_create(id: c_uint, width: c_uint, height: c_uint) {
    aux_create(id, width, height);
}

/// Destroys an auxiliary texture.
///
/// # Safety
/// This function is safe to call from C.
#[unsafe(no_mangle)]
pub extern "C" fn nesium_aux_destroy(id: c_uint) {
    aux_destroy(id);
}

/// Updates the auxiliary texture with new RGBA data.
///
/// # Safety
/// - `rgba` must point to at least `len` readable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nesium_aux_update(id: c_uint, rgba: *const u8, len: usize) -> bool {
    if rgba.is_null() {
        return false;
    }
    let slice = unsafe { std::slice::from_raw_parts(rgba, len) };
    aux_update(id, slice)
}

/// Copies the front buffer of the auxiliary texture to `dst`.
///
/// # Safety
/// - `dst` must point to at least `dst_pitch * height` writable bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nesium_aux_copy(
    id: c_uint,
    dst: *mut u8,
    dst_pitch: c_uint,
    dst_height: c_uint,
) -> usize {
    if dst.is_null() {
        return 0;
    }
    let total = (dst_pitch as usize) * (dst_height as usize);
    let slice = unsafe { std::slice::from_raw_parts_mut(dst, total) };
    aux_copy(id, slice, dst_pitch as usize)
}
