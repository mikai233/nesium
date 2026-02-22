/// Double-buffered PPU framebuffer used by the NES core.
///
/// This module implements "Route A" refactoring:
/// - The canonical representation is palette indices (1 byte per pixel, 256x240).
/// - Packed pixel buffers (RGBA, RGB565, etc.) are DERIVED from indices only at presentation.
/// - The PPU writes raw indices; conversion happens once per frame.
use crate::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH, palette::Color};
use core::{ffi::c_void, fmt};
use std::{
    slice,
    sync::{
        Arc,
        atomic::{AtomicPtr, AtomicUsize, Ordering},
    },
};

mod post_process;
pub use post_process::{NearestPostProcessor, SourceFrame, TargetFrameMut, VideoPostProcessor};

pub const SCREEN_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

pub type FrameReadyCallback =
    extern "C" fn(buffer_index: u32, width: u32, height: u32, pitch: u32, user_data: *mut c_void);

pub type SwapchainLockCallback =
    extern "C" fn(buffer_index: u32, pitch_out: *mut u32, user_data: *mut c_void) -> *mut u8;
pub type SwapchainUnlockCallback = extern "C" fn(buffer_index: u32, user_data: *mut c_void);

#[derive(Clone, Copy)]
struct FrameReadyHook {
    cb: FrameReadyCallback,
    user_data: *mut c_void,
}

// SAFETY: `FrameReadyHook` only carries opaque pointers that are never dereferenced by the core.
// The embedder is responsible for ensuring the callback and `user_data` remain valid on the NES thread.
unsafe impl Send for FrameReadyHook {}

impl fmt::Debug for FrameReadyHook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FrameReadyHook")
            .field("cb", &(self.cb as usize))
            .field("user_data", &self.user_data)
            .finish()
    }
}

impl FrameReadyHook {
    #[inline]
    fn call(&self, buffer_index: usize, width: usize, height: usize, pitch: usize) {
        debug_assert!(buffer_index < 2);
        (self.cb)(
            buffer_index as u32,
            width as u32,
            height as u32,
            pitch as u32,
            self.user_data,
        );
    }
}

/// Describes how a logical RGB color is packed into the underlying byte buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorFormat {
    /// 16-bit packed RGB using 5 bits per channel (0RRRRRGGGGGBBBBB).
    Rgb555,
    /// 16-bit packed RGB using 5/6/5 bits (RRRRRGGGGGGBBBBB).
    Rgb565,
    /// Packed 24-bit RGB, 3 bytes per pixel in R, G, B order.
    Rgb888,
    /// Packed 32-bit RGBA, 4 bytes per pixel in R, G, B, A order.
    Rgba8888,
    /// Packed 32-bit BGRA, 4 bytes per pixel in B, G, R, A order.
    Bgra8888,
    /// Packed 32-bit ARGB, 4 bytes per pixel in A, R, G, B order.
    Argb8888,
}

impl ColorFormat {
    /// Returns the number of bytes used to represent a single pixel in this format.
    #[inline]
    pub const fn bytes_per_pixel(self) -> usize {
        match self {
            ColorFormat::Rgb555 | ColorFormat::Rgb565 => 2,
            ColorFormat::Rgb888 => 3,
            ColorFormat::Rgba8888 | ColorFormat::Bgra8888 | ColorFormat::Argb8888 => 4,
        }
    }
}

/// A double-buffered framebuffer for the NES PPU.
///
/// Always maintains two internal index planes. Packed pixels are derived on demand.
#[derive(Debug, Clone)]
struct CanonicalFrameStore {
    /// Index of the **back/write** index plane (0 or 1).
    active_index: usize,
    /// Canonical index planes (1 byte per pixel).
    index_planes: [Box<[u8]>; 2],
    /// Per-pixel emphasis bits (0..7) captured at render time.
    emphasis_planes: [Box<[u8]>; 2],
}

#[derive(Debug)]
struct PostProcessPipeline {
    /// Format used for packed pixel derivation.
    color_format: ColorFormat,
    /// Output packed buffer width in pixels.
    output_width: usize,
    /// Output packed buffer height in pixels.
    output_height: usize,
    /// Packed output post-processor (scaler/filter chain).
    post_processor: Box<dyn VideoPostProcessor>,
}

#[derive(Debug)]
struct PresentBackend {
    /// Destination for packed pixel output.
    storage: FrameBufferStorage,
    frame_ready_hook: Option<FrameReadyHook>,
}

/// A double-buffered framebuffer for the NES PPU.
#[derive(Debug)]
pub struct FrameBuffer {
    canonical: CanonicalFrameStore,
    pipeline: PostProcessPipeline,
    backend: PresentBackend,
}

/// Backing storage for the derived packed pixel planes.
#[derive(Debug)]
enum FrameBufferStorage {
    /// Internal double-buffered packed pixels + a shared handle for frontend copies.
    Owned {
        planes: [Box<[u8]>; 2],
        handle: Arc<ExternalFrameHandle>,
    },
    /// Swapchain-backed framebuffer where the core obtains writable planes via callbacks.
    Swapchain(SwapchainFrameBuffer),
}

impl Clone for FrameBufferStorage {
    fn clone(&self) -> Self {
        match self {
            Self::Owned { planes, handle } => {
                let width = handle.width();
                let height = handle.height();
                let pitch_bytes = handle.pitch_bytes();
                let front_index = handle.front_index();
                let new_planes = [planes[0].clone(), planes[1].clone()];
                let p0 = new_planes[0].as_ptr() as *mut u8;
                let p1 = new_planes[1].as_ptr() as *mut u8;
                let new_handle = Arc::new(ExternalFrameHandle::new(
                    handle.color_format(),
                    width,
                    height,
                    pitch_bytes,
                    p0,
                    p1,
                    front_index,
                ));
                Self::Owned {
                    planes: new_planes,
                    handle: new_handle,
                }
            }
            Self::Swapchain(_) => {
                panic!("cloning a swapchain-backed FrameBuffer is not supported")
            }
        }
    }
}

impl Clone for FrameBuffer {
    fn clone(&self) -> Self {
        Self {
            canonical: self.canonical.clone(),
            pipeline: self.pipeline.clone(),
            backend: self.backend.clone(),
        }
    }
}

impl Clone for PostProcessPipeline {
    fn clone(&self) -> Self {
        Self {
            color_format: self.color_format,
            output_width: self.output_width,
            output_height: self.output_height,
            post_processor: dyn_clone::clone_box(&*self.post_processor),
        }
    }
}

impl Clone for PresentBackend {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            frame_ready_hook: self.frame_ready_hook,
        }
    }
}

/// Shared external framebuffer planes + published front index.
#[derive(Debug)]
pub struct ExternalFrameHandle {
    planes: [AtomicPtr<u8>; 2],
    len: AtomicUsize,
    pitch_bytes: AtomicUsize,
    width: AtomicUsize,
    height: AtomicUsize,
    color_format: ColorFormat,
    front_index: AtomicUsize,
    frame_seq: AtomicUsize,
    reading_plane: AtomicUsize,
}

unsafe impl Send for ExternalFrameHandle {}
unsafe impl Sync for ExternalFrameHandle {}

impl ExternalFrameHandle {
    const RESIZING: usize = 2;
    #[inline]
    pub fn len(&self) -> usize {
        self.len.load(Ordering::Acquire)
    }

    #[inline]
    pub fn pitch_bytes(&self) -> usize {
        self.pitch_bytes.load(Ordering::Acquire)
    }

    #[inline]
    pub fn width(&self) -> usize {
        self.width.load(Ordering::Acquire)
    }

    #[inline]
    pub fn height(&self) -> usize {
        self.height.load(Ordering::Acquire)
    }

    #[inline]
    pub fn color_format(&self) -> ColorFormat {
        self.color_format
    }

    #[inline]
    pub fn bytes_per_pixel(&self) -> usize {
        self.color_format.bytes_per_pixel()
    }

    #[inline]
    pub fn front_index(&self) -> usize {
        loop {
            let idx = self.front_index.load(Ordering::Acquire);
            if idx < 2 {
                return idx;
            }
            std::hint::spin_loop();
        }
    }

    #[inline]
    pub fn frame_seq(&self) -> usize {
        self.frame_seq.load(Ordering::Acquire)
    }

    /// Returns the current **front** plane as an immutable slice.
    #[inline]
    pub fn front_slice(&self) -> &[u8] {
        let idx = self.front_index();
        self.plane_slice(idx)
    }

    /// Publish `index` as the new **front** plane.
    #[inline]
    pub fn present(&self, index: usize) {
        debug_assert!(index < 2);
        self.front_index.store(index, Ordering::Release);
        self.frame_seq.fetch_add(1, Ordering::Release);
    }

    #[inline]
    pub fn plane_slice(&self, index: usize) -> &[u8] {
        debug_assert!(index < 2);
        let len = self.len();
        if len == 0 {
            return &[];
        }
        let ptr = self.planes[index].load(Ordering::Acquire);
        if ptr.is_null() {
            return &[];
        }
        unsafe { slice::from_raw_parts(ptr, len) }
    }

    #[inline]
    fn plane_ptr_mut(&self, index: usize) -> *mut u8 {
        debug_assert!(index < 2);
        self.planes[index].load(Ordering::Acquire)
    }

    const NOT_READING: usize = 3;

    #[inline]
    pub fn begin_front_copy(&self) -> usize {
        loop {
            let idx = self.front_index.load(Ordering::Acquire);
            if idx >= 2 {
                std::hint::spin_loop();
                continue;
            }
            self.reading_plane.store(idx, Ordering::Release);
            if self.front_index.load(Ordering::Acquire) == idx {
                return idx;
            }
            self.reading_plane
                .store(Self::NOT_READING, Ordering::Release);
        }
    }

    #[inline]
    pub fn end_front_copy(&self) {
        self.reading_plane
            .store(Self::NOT_READING, Ordering::Release);
    }

    #[inline]
    fn wait_until_not_reading(&self, index: usize) {
        let mut spins = 0u32;
        while self.reading_plane.load(Ordering::Acquire) == index {
            std::hint::spin_loop();
            spins += 1;
            if spins >= 128 {
                spins = 0;
                std::thread::yield_now();
            }
        }
    }

    fn begin_resize(&self) {
        self.front_index.store(Self::RESIZING, Ordering::Release);
    }

    fn end_resize(&self, front_index: usize) {
        debug_assert!(front_index < 2);
        self.front_index.store(front_index, Ordering::Release);
        self.frame_seq.fetch_add(1, Ordering::Release);
    }

    fn wait_until_not_reading_any(&self) {
        let mut spins = 0u32;
        while self.reading_plane.load(Ordering::Acquire) != Self::NOT_READING {
            std::hint::spin_loop();
            spins += 1;
            if spins >= 128 {
                spins = 0;
                std::thread::yield_now();
            }
        }
    }

    fn update_buffers(
        &self,
        plane0: *mut u8,
        plane1: *mut u8,
        width: usize,
        height: usize,
        front_index: usize,
    ) {
        debug_assert!(front_index < 2);
        let pitch_bytes = width * self.bytes_per_pixel();
        let len = pitch_bytes * height;

        self.begin_resize();
        self.wait_until_not_reading_any();

        self.len.store(0, Ordering::Release);
        self.width.store(width, Ordering::Release);
        self.height.store(height, Ordering::Release);
        self.pitch_bytes.store(pitch_bytes, Ordering::Release);
        self.planes[0].store(plane0, Ordering::Release);
        self.planes[1].store(plane1, Ordering::Release);
        self.len.store(len, Ordering::Release);

        self.end_resize(front_index);
    }

    fn new(
        color_format: ColorFormat,
        width: usize,
        height: usize,
        pitch_bytes: usize,
        plane0: *mut u8,
        plane1: *mut u8,
        front_index: usize,
    ) -> Self {
        debug_assert!(front_index < 2);
        let len = pitch_bytes * height;
        Self {
            planes: [AtomicPtr::new(plane0), AtomicPtr::new(plane1)],
            len: AtomicUsize::new(len),
            pitch_bytes: AtomicUsize::new(pitch_bytes),
            width: AtomicUsize::new(width),
            height: AtomicUsize::new(height),
            color_format,
            front_index: AtomicUsize::new(front_index),
            frame_seq: AtomicUsize::new(0),
            reading_plane: AtomicUsize::new(Self::NOT_READING),
        }
    }
}

#[derive(Clone, Copy)]
struct SwapchainHook {
    lock: SwapchainLockCallback,
    unlock: SwapchainUnlockCallback,
    user_data: *mut c_void,
}

unsafe impl Send for SwapchainHook {}

impl fmt::Debug for SwapchainHook {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SwapchainHook")
            .field("lock", &(self.lock as usize))
            .field("unlock", &(self.unlock as usize))
            .field("user_data", &self.user_data)
            .finish()
    }
}

#[derive(Debug)]
struct SwapchainFrameBuffer {
    hook: SwapchainHook,
    ptr: [*mut u8; 2],
    pitch_bytes: [usize; 2],
    locked: [bool; 2],
}

unsafe impl Send for SwapchainFrameBuffer {}

impl SwapchainFrameBuffer {
    fn new(
        lock: SwapchainLockCallback,
        unlock: SwapchainUnlockCallback,
        user_data: *mut c_void,
    ) -> Self {
        Self {
            hook: SwapchainHook {
                lock,
                unlock,
                user_data,
            },
            ptr: [std::ptr::null_mut(); 2],
            pitch_bytes: [0, 0],
            locked: [false, false],
        }
    }

    fn lock(&mut self, index: usize) -> (*mut u8, usize) {
        debug_assert!(index < 2);
        if self.locked[index] {
            return (self.ptr[index], self.pitch_bytes[index]);
        }
        let mut pitch = 0u32;
        let ptr = (self.hook.lock)(index as u32, &mut pitch as *mut u32, self.hook.user_data);
        assert!(!ptr.is_null());
        self.ptr[index] = ptr;
        self.pitch_bytes[index] = pitch as usize;
        self.locked[index] = true;
        (ptr, pitch as usize)
    }

    fn unlock(&mut self, index: usize) {
        debug_assert!(index < 2);
        if !self.locked[index] {
            return;
        }
        (self.hook.unlock)(index as u32, self.hook.user_data);
        self.ptr[index] = std::ptr::null_mut();
        self.pitch_bytes[index] = 0;
        self.locked[index] = false;
    }
}

impl FrameBuffer {
    /// Creates a new `FrameBuffer` with internal storage.
    pub fn new(color_format: ColorFormat) -> Self {
        let bpp = color_format.bytes_per_pixel();
        let output_width = SCREEN_WIDTH;
        let output_height = SCREEN_HEIGHT;
        let pitch_bytes = output_width * bpp;
        let len = pitch_bytes * output_height;
        let active_index = 0usize;
        let front_index = 1 - active_index;

        let planes = [
            vec![0; len].into_boxed_slice(),
            vec![0; len].into_boxed_slice(),
        ];
        let p0 = planes[0].as_ptr() as *mut u8;
        let p1 = planes[1].as_ptr() as *mut u8;
        let handle = Arc::new(ExternalFrameHandle::new(
            color_format,
            output_width,
            output_height,
            pitch_bytes,
            p0,
            p1,
            front_index,
        ));

        Self {
            canonical: CanonicalFrameStore {
                active_index,
                index_planes: [
                    vec![0u8; SCREEN_SIZE].into_boxed_slice(),
                    vec![0u8; SCREEN_SIZE].into_boxed_slice(),
                ],
                emphasis_planes: [
                    vec![0u8; SCREEN_SIZE].into_boxed_slice(),
                    vec![0u8; SCREEN_SIZE].into_boxed_slice(),
                ],
            },
            pipeline: PostProcessPipeline {
                color_format,
                output_width,
                output_height,
                post_processor: Box::new(NearestPostProcessor::default()),
            },
            backend: PresentBackend {
                storage: FrameBufferStorage::Owned { planes, handle },
                frame_ready_hook: None,
            },
        }
    }

    /// Creates a new swapchain-backed framebuffer.
    pub fn new_swapchain(
        color_format: ColorFormat,
        lock: SwapchainLockCallback,
        unlock: SwapchainUnlockCallback,
        user_data: *mut c_void,
    ) -> Self {
        Self {
            canonical: CanonicalFrameStore {
                active_index: 1,
                index_planes: [
                    vec![0u8; SCREEN_SIZE].into_boxed_slice(),
                    vec![0u8; SCREEN_SIZE].into_boxed_slice(),
                ],
                emphasis_planes: [
                    vec![0u8; SCREEN_SIZE].into_boxed_slice(),
                    vec![0u8; SCREEN_SIZE].into_boxed_slice(),
                ],
            },
            pipeline: PostProcessPipeline {
                color_format,
                output_width: SCREEN_WIDTH,
                output_height: SCREEN_HEIGHT,
                post_processor: Box::new(NearestPostProcessor::default()),
            },
            backend: PresentBackend {
                storage: FrameBufferStorage::Swapchain(SwapchainFrameBuffer::new(
                    lock, unlock, user_data,
                )),
                frame_ready_hook: None,
            },
        }
    }

    /// Primary entry point for presenting a completed frame.
    ///
    /// This converts the active index plane into packed pixels and performs the swap.
    pub(crate) fn present(&mut self, palette: &[Color; 64]) {
        let finished_back = self.canonical.active_index;
        let format = self.pipeline.color_format;
        let indices = &self.canonical.index_planes[finished_back];
        let emphasis = &self.canonical.emphasis_planes[finished_back];
        let out_w = self.pipeline.output_width;
        let out_h = self.pipeline.output_height;
        let row_bytes = out_w * format.bytes_per_pixel();

        let dst_pitch = match &mut self.backend.storage {
            FrameBufferStorage::Owned { .. } => row_bytes,
            FrameBufferStorage::Swapchain(s) => s.lock(finished_back).1,
        };

        if dst_pitch < row_bytes {
            if let FrameBufferStorage::Swapchain(s) = &mut self.backend.storage {
                s.unlock(finished_back);
            }
            return;
        }
        let source = SourceFrame::new(indices, emphasis, SCREEN_WIDTH, SCREEN_HEIGHT);

        // Convert indices to packed pixels for the entire frame.
        match &mut self.backend.storage {
            FrameBufferStorage::Owned { planes, .. } => {
                self.pipeline.post_processor.process(
                    source,
                    palette,
                    TargetFrameMut::new(
                        &mut planes[finished_back],
                        dst_pitch,
                        out_w,
                        out_h,
                        format,
                    ),
                );
            }
            FrameBufferStorage::Swapchain(s) => {
                let (ptr, pitch) = s.lock(finished_back);
                let dst = unsafe { slice::from_raw_parts_mut(ptr, pitch * out_h) };
                self.pipeline.post_processor.process(
                    source,
                    palette,
                    TargetFrameMut::new(dst, pitch, out_w, out_h, format),
                );
            }
        }

        // Handle presentation and index plane flipping.
        match &mut self.backend.storage {
            FrameBufferStorage::Owned { handle, .. } => {
                handle.present(finished_back);
                if let Some(hook) = self.backend.frame_ready_hook {
                    hook.call(finished_back, out_w, out_h, dst_pitch);
                }
                self.canonical.active_index = 1 - self.canonical.active_index;
                handle.wait_until_not_reading(self.canonical.active_index);
            }
            FrameBufferStorage::Swapchain(s) => {
                s.unlock(finished_back);
                if let Some(hook) = self.backend.frame_ready_hook {
                    hook.call(finished_back, out_w, out_h, dst_pitch);
                }
                self.canonical.active_index = 1 - self.canonical.active_index;
            }
        }

        // Clear the new back index plane for the next frame.
        self.canonical.index_planes[self.canonical.active_index].fill(0);
        self.canonical.emphasis_planes[self.canonical.active_index].fill(0);
    }

    /// Rebuilds the current front packed buffer from the current front index plane.
    ///
    /// Useful after a rewind restore to ensure the display matches the restored state.
    pub(crate) fn rebuild_packed(&mut self, palette: &[Color; 64]) {
        let front_idx = 1 - self.canonical.active_index;
        let indices = &self.canonical.index_planes[front_idx];
        let emphasis = &self.canonical.emphasis_planes[front_idx];
        let format = self.pipeline.color_format;
        let out_w = self.pipeline.output_width;
        let out_h = self.pipeline.output_height;
        let row_bytes = out_w * format.bytes_per_pixel();

        let dst_pitch = match &mut self.backend.storage {
            FrameBufferStorage::Owned { planes: _, handle } => {
                // Avoid writing into a plane while the frontend is copying it.
                handle.wait_until_not_reading(front_idx);
                row_bytes
            }
            FrameBufferStorage::Swapchain(s) => s.lock(front_idx).1,
        };

        if dst_pitch < row_bytes {
            if let FrameBufferStorage::Swapchain(s) = &mut self.backend.storage {
                s.unlock(front_idx);
            }
            return;
        }
        let source = SourceFrame::new(indices, emphasis, SCREEN_WIDTH, SCREEN_HEIGHT);

        match &mut self.backend.storage {
            FrameBufferStorage::Owned { planes, .. } => {
                self.pipeline.post_processor.process(
                    source,
                    palette,
                    TargetFrameMut::new(&mut planes[front_idx], dst_pitch, out_w, out_h, format),
                );
            }
            FrameBufferStorage::Swapchain(s) => {
                let (ptr, pitch) = s.lock(front_idx);
                let dst = unsafe { slice::from_raw_parts_mut(ptr, pitch * out_h) };
                self.pipeline.post_processor.process(
                    source,
                    palette,
                    TargetFrameMut::new(dst, pitch, out_w, out_h, format),
                );
                s.unlock(front_idx);
            }
        }

        // Owned storage does not need explicit unlock.
    }

    /// Writes a single pixel at `(x, y)` using a palette index.
    #[inline]
    pub(crate) fn write_index(&mut self, x: usize, y: usize, index: u8) {
        self.write_index_with_emphasis(x, y, index, 0);
    }

    /// Writes a single pixel at `(x, y)` using palette index and emphasis bits.
    #[inline]
    pub(crate) fn write_index_with_emphasis(
        &mut self,
        x: usize,
        y: usize,
        index: u8,
        emphasis: u8,
    ) {
        let pos = y * SCREEN_WIDTH + x;
        self.canonical.index_planes[self.canonical.active_index][pos] = index;
        self.canonical.emphasis_planes[self.canonical.active_index][pos] = emphasis & 0x07;
    }

    /// Applies grayscale/emphasis bit masks over an inclusive linear pixel range
    /// on the current back buffer.
    #[inline]
    pub(crate) fn apply_color_mask_range(
        &mut self,
        start: usize,
        end: usize,
        grayscale: bool,
        emphasis: u8,
    ) {
        if start > end || end >= SCREEN_SIZE {
            return;
        }
        let mask = if grayscale { 0x30 } else { 0x3F };
        let emphasis = emphasis & 0x07;
        let indices = &mut self.canonical.index_planes[self.canonical.active_index];
        let em = &mut self.canonical.emphasis_planes[self.canonical.active_index];
        for pos in start..=end {
            indices[pos] &= mask;
            em[pos] = emphasis;
        }
    }

    /// Returns the current front packed pixel plane when directly accessible.
    ///
    /// For `Owned` storage, this returns a tightly packed slice with length
    /// `output_width * output_height * bytes_per_pixel`.
    ///
    /// For `Swapchain` storage, returns `None` because plane memory is only valid
    /// while locked by the swapchain callbacks.
    pub fn try_render_packed(&self) -> Option<&[u8]> {
        let front_idx = 1 - self.canonical.active_index;
        match &self.backend.storage {
            FrameBufferStorage::Owned { planes, .. } => Some(&planes[front_idx]),
            FrameBufferStorage::Swapchain(_) => None,
        }
    }

    /// Returns a read-only view of the **front** index plane.
    pub(crate) fn render_index(&self) -> &[u8] {
        &self.canonical.index_planes[1 - self.canonical.active_index]
    }

    /// Returns a read-only view of the **front** emphasis plane (`0..=7` per pixel).
    pub(crate) fn render_emphasis(&self) -> &[u8] {
        &self.canonical.emphasis_planes[1 - self.canonical.active_index]
    }

    /// Copies the current front index plane into `dst`.
    ///
    /// The destination buffer must be exactly `SCREEN_WIDTH * SCREEN_HEIGHT` bytes.
    ///
    /// This is intended for features such as rewind: the index plane is the canonical
    /// representation of the frame and is significantly smaller than packed pixels.
    pub(crate) fn copy_render_index_buffer(&self, dst: &mut [u8]) {
        assert!(
            dst.len() == SCREEN_SIZE,
            "dst must be SCREEN_WIDTH * SCREEN_HEIGHT bytes"
        );
        dst.copy_from_slice(self.render_index());
    }

    /// Copies the current front emphasis plane into `dst`.
    ///
    /// The destination buffer must be exactly `SCREEN_WIDTH * SCREEN_HEIGHT` bytes.
    pub(crate) fn copy_render_emphasis_buffer(&self, dst: &mut [u8]) {
        assert!(
            dst.len() == SCREEN_SIZE,
            "dst must be SCREEN_WIDTH * SCREEN_HEIGHT bytes"
        );
        dst.copy_from_slice(self.render_emphasis());
    }

    /// Copies the current front packed pixel buffer into `dst`.
    ///
    /// This method always writes a tightly packed image, with no per-row padding.
    /// The required length is `output_width * output_height * bytes_per_pixel`.
    ///
    /// For backends that expose a padded stride (pitch), this method copies each
    /// scanline while skipping the padding bytes.
    pub fn copy_render_buffer(&mut self, dst: &mut [u8]) {
        let front_idx = 1 - self.canonical.active_index;
        let bpp = self.pipeline.color_format.bytes_per_pixel();
        let row_len = self.pipeline.output_width * bpp;
        let expected = row_len * self.pipeline.output_height;
        assert!(
            dst.len() == expected,
            "dst must be output_width * output_height * bytes_per_pixel bytes"
        );

        match &mut self.backend.storage {
            FrameBufferStorage::Owned { planes, .. } => {
                dst.copy_from_slice(&planes[front_idx]);
            }
            FrameBufferStorage::Swapchain(s) => {
                let (ptr, pitch) = s.lock(front_idx);
                debug_assert!(pitch >= row_len);
                let src =
                    unsafe { slice::from_raw_parts(ptr, pitch * self.pipeline.output_height) };
                for y in 0..self.pipeline.output_height {
                    let src_off = y * pitch;
                    let dst_off = y * row_len;
                    dst[dst_off..dst_off + row_len]
                        .copy_from_slice(&src[src_off..src_off + row_len]);
                }
                s.unlock(front_idx);
            }
        }
    }

    /// Returns a read-only view of the given index plane.
    #[inline]
    pub(crate) fn index_plane(&self, index: usize) -> &[u8] {
        debug_assert!(index < 2);
        &self.canonical.index_planes[index]
    }

    /// Returns the number of bytes per scanline (pitch) for the packed output.
    #[inline]
    pub fn pitch(&self) -> usize {
        match &self.backend.storage {
            FrameBufferStorage::Owned { .. } => {
                self.pipeline.output_width * self.pipeline.color_format.bytes_per_pixel()
            }
            FrameBufferStorage::Swapchain(s) => {
                // If not locked, we return the baseline pitch.
                let front_idx = 1 - self.canonical.active_index;
                if s.locked[front_idx] {
                    s.pitch_bytes[front_idx]
                } else {
                    self.pipeline.output_width * self.pipeline.color_format.bytes_per_pixel()
                }
            }
        }
    }

    pub fn set_frame_ready_callback(
        &mut self,
        cb: Option<FrameReadyCallback>,
        user_data: *mut c_void,
    ) {
        self.backend.frame_ready_hook = cb.map(|cb| FrameReadyHook { cb, user_data });
    }

    /// Returns the external frame handle if this framebuffer exposes one.
    #[inline]
    pub fn external_frame_handle(&self) -> Option<&Arc<ExternalFrameHandle>> {
        match &self.backend.storage {
            FrameBufferStorage::Owned { handle, .. } => Some(handle),
            FrameBufferStorage::Swapchain(_) => None,
        }
    }

    #[inline]
    pub(crate) fn active_plane_index(&self) -> usize {
        self.canonical.active_index
    }

    /// Returns a mutable view of the **back** index plane for PPU writes.
    pub(crate) fn write(&mut self) -> &mut [u8] {
        &mut self.canonical.index_planes[self.canonical.active_index]
    }

    /// Clears both index planes and any accessible packed planes.
    pub fn clear(&mut self) {
        for plane in &mut self.canonical.index_planes {
            plane.fill(0);
        }
        for plane in &mut self.canonical.emphasis_planes {
            plane.fill(0);
        }
        match &mut self.backend.storage {
            FrameBufferStorage::Owned { planes, handle } => {
                for (i, plane) in planes.iter_mut().enumerate() {
                    handle.wait_until_not_reading(i);
                    plane.fill(0);
                }
            }
            FrameBufferStorage::Swapchain(s) => {
                for i in 0..2 {
                    let (ptr, pitch) = s.lock(i);
                    unsafe {
                        slice::from_raw_parts_mut(ptr, pitch * self.pipeline.output_height).fill(0)
                    };
                    s.unlock(i);
                }
            }
        }
    }

    /// Clears both buffers to black and notifies the frontend.
    ///
    /// Unlike `present()`, this method does NOT re-render using the palette.
    /// The packed buffers are filled with 0 bytes directly (black/transparent),
    /// and the frame-ready callback is invoked so the frontend updates its texture.
    pub(crate) fn clear_and_present(&mut self) {
        self.clear();

        // Swap to the other plane so the cleared plane becomes front.
        let cleared_plane = self.canonical.active_index;
        self.canonical.active_index = 1 - self.canonical.active_index;

        let pitch = match &mut self.backend.storage {
            FrameBufferStorage::Owned { handle, .. } => {
                handle.present(cleared_plane);
                handle.wait_until_not_reading(self.canonical.active_index);
                self.pipeline.output_width * self.pipeline.color_format.bytes_per_pixel()
            }
            FrameBufferStorage::Swapchain(s) => {
                // For swapchain, we need to lock/unlock to signal the frontend.
                let (_, pitch) = s.lock(cleared_plane);
                s.unlock(cleared_plane);
                pitch
            }
        };

        if let Some(hook) = self.backend.frame_ready_hook {
            hook.call(
                cleared_plane,
                self.pipeline.output_width,
                self.pipeline.output_height,
                pitch,
            );
        }
    }

    /// Update the packed output resolution and scaling mode.
    ///
    /// This does not change the canonical index resolution (always 256×240).
    /// For `Owned` storage, backing buffers are reallocated and the shared
    /// [`ExternalFrameHandle`] is updated in-place so frontends do not need to
    /// fetch a new handle.
    pub fn set_output_config(&mut self, width: usize, height: usize) {
        assert!(width > 0 && height > 0, "output size must be non-zero");
        self.pipeline.output_width = width;
        self.pipeline.output_height = height;

        let bpp = self.pipeline.color_format.bytes_per_pixel();
        let pitch_bytes = width * bpp;
        let len = pitch_bytes * height;
        let front_index = 1 - self.canonical.active_index;

        match &mut self.backend.storage {
            FrameBufferStorage::Owned { planes, handle } => {
                if planes[0].len() != len || planes[1].len() != len {
                    let mut new_planes = [
                        vec![0u8; len].into_boxed_slice(),
                        vec![0u8; len].into_boxed_slice(),
                    ];
                    let p0 = new_planes[0].as_mut_ptr();
                    let p1 = new_planes[1].as_mut_ptr();
                    handle.update_buffers(p0, p1, width, height, front_index);
                    *planes = new_planes;
                } else {
                    let p0 = planes[0].as_mut_ptr();
                    let p1 = planes[1].as_mut_ptr();
                    handle.update_buffers(p0, p1, width, height, front_index);
                }
            }
            FrameBufferStorage::Swapchain(_) => {
                // Swapchain backends must honor the new `width`/`height` in their lock callback.
            }
        }
    }

    /// Replaces the post-processor used to derive packed output frames.
    ///
    /// This is intended for Rust-side integrations that want to inject a filter/upscaler
    /// implementation at runtime.
    pub fn set_post_processor(&mut self, processor: Box<dyn VideoPostProcessor>) {
        self.pipeline.post_processor = processor;
    }

    #[inline]
    pub fn color_format(&self) -> ColorFormat {
        self.pipeline.color_format
    }

    /// Change the color format at runtime.
    ///
    /// # Panics
    /// Panics if the new format has a different `bytes_per_pixel` than the current format.
    pub fn set_color_format(&mut self, format: ColorFormat) {
        let current_bpp = self.pipeline.color_format.bytes_per_pixel();
        let new_bpp = format.bytes_per_pixel();
        assert_eq!(
            current_bpp, new_bpp,
            "Cannot change color format: bytes_per_pixel mismatch ({} != {})",
            current_bpp, new_bpp
        );
        self.pipeline.color_format = format;
    }
}

#[inline]
unsafe fn pack_pixel(color: Color, dst: *mut u8, format: ColorFormat) {
    unsafe {
        match format {
            ColorFormat::Rgb555 => {
                let r5 = (color.r as u16) >> 3;
                let g5 = (color.g as u16) >> 3;
                let b5 = (color.b as u16) >> 3;
                let packed = (r5 << 10) | (g5 << 5) | b5;
                let bytes = packed.to_le_bytes();
                *dst = bytes[0];
                *dst.add(1) = bytes[1];
            }
            ColorFormat::Rgb565 => {
                let r5 = (color.r as u16) >> 3;
                let g6 = (color.g as u16) >> 2;
                let b5 = (color.b as u16) >> 3;
                let packed = (r5 << 11) | (g6 << 5) | b5;
                let bytes = packed.to_le_bytes();
                *dst = bytes[0];
                *dst.add(1) = bytes[1];
            }
            ColorFormat::Rgb888 => {
                *dst = color.r;
                *dst.add(1) = color.g;
                *dst.add(2) = color.b;
            }
            ColorFormat::Rgba8888 => {
                *dst = color.r;
                *dst.add(1) = color.g;
                *dst.add(2) = color.b;
                *dst.add(3) = 0xFF;
            }
            ColorFormat::Bgra8888 => {
                *dst = color.b;
                *dst.add(1) = color.g;
                *dst.add(2) = color.r;
                *dst.add(3) = 0xFF;
            }
            ColorFormat::Argb8888 => {
                *dst = 0xFF;
                *dst.add(1) = color.r;
                *dst.add(2) = color.g;
                *dst.add(3) = color.b;
            }
        }
    }
}

pub(super) fn apply_emphasis(color: Color, color_index: u8, emphasis: u8) -> Color {
    let emphasis = emphasis & 0x07;
    if emphasis == 0 || (color_index & 0x0F) > 0x0D {
        return color;
    }

    // Match Mesen2 default filter behavior: emphasis de-intensifies the
    // non-selected channels by 0.84 and leaves selected channels unchanged.
    let mut r = color.r as f64;
    let mut g = color.g as f64;
    let mut b = color.b as f64;
    if (emphasis & 0x01) != 0 {
        g *= 0.84;
        b *= 0.84;
    }
    if (emphasis & 0x02) != 0 {
        r *= 0.84;
        b *= 0.84;
    }
    if (emphasis & 0x04) != 0 {
        r *= 0.84;
        g *= 0.84;
    }

    Color::new(
        r.clamp(0.0, 255.0) as u8,
        g.clamp(0.0, 255.0) as u8,
        b.clamp(0.0, 255.0) as u8,
    )
}

/// Helper to pack a single line of indices into a destination buffer.
pub unsafe fn pack_line(
    indices: &[u8],
    emphasis: &[u8],
    dst: *mut u8,
    format: ColorFormat,
    palette: &[Color; 64],
) {
    debug_assert_eq!(indices.len(), emphasis.len());
    let bpp = format.bytes_per_pixel();
    for (x, &idx) in indices.iter().enumerate() {
        let color = apply_emphasis(palette[(idx & 0x3F) as usize], idx, emphasis[x]);
        unsafe {
            let p = dst.add(x * bpp);
            match format {
                ColorFormat::Rgb555 => {
                    let r5 = (color.r as u16) >> 3;
                    let g5 = (color.g as u16) >> 3;
                    let b5 = (color.b as u16) >> 3;
                    let packed = (r5 << 10) | (g5 << 5) | b5;
                    let bytes = packed.to_le_bytes();
                    *p = bytes[0];
                    *p.add(1) = bytes[1];
                }
                ColorFormat::Rgb565 => {
                    let r5 = (color.r as u16) >> 3;
                    let g6 = (color.g as u16) >> 2;
                    let b5 = (color.b as u16) >> 3;
                    let packed = (r5 << 11) | (g6 << 5) | b5;
                    let bytes = packed.to_le_bytes();
                    *p = bytes[0];
                    *p.add(1) = bytes[1];
                }
                ColorFormat::Rgb888 => {
                    *p = color.r;
                    *p.add(1) = color.g;
                    *p.add(2) = color.b;
                }
                ColorFormat::Rgba8888 => {
                    *p = color.r;
                    *p.add(1) = color.g;
                    *p.add(2) = color.b;
                    *p.add(3) = 0xFF;
                }
                ColorFormat::Bgra8888 => {
                    *p = color.b;
                    *p.add(1) = color.g;
                    *p.add(2) = color.r;
                    *p.add(3) = 0xFF;
                }
                ColorFormat::Argb8888 => {
                    *p = 0xFF;
                    *p.add(1) = color.r;
                    *p.add(2) = color.g;
                    *p.add(3) = color.b;
                }
            }
        }
    }
}

impl Default for FrameBuffer {
    fn default() -> Self {
        Self::new(ColorFormat::Rgba8888)
    }
}
