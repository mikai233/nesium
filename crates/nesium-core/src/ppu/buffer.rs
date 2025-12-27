/// Double-buffered PPU framebuffer used by the NES core.
///
/// This module provides a simple front/back framebuffer with two modes:
/// - index mode: stores raw palette indices for debugging or PPU inspection
/// - color mode: stores packed RGB/RGBA pixels ready to be consumed by a frontend (SDL, libretro, Flutter, etc.)
use crate::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH, palette::Color};
use core::{ffi::c_void, fmt};
use std::{
    ptr::NonNull,
    slice,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

pub type FrameReadyCallback =
    extern "C" fn(buffer_index: u32, width: u32, height: u32, pitch: u32, user_data: *mut c_void);

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
    fn call(&self, buffer_index: usize, pitch: usize) {
        debug_assert!(buffer_index < 2);
        (self.cb)(
            buffer_index as u32,
            SCREEN_WIDTH as u32,
            SCREEN_HEIGHT as u32,
            pitch as u32,
            self.user_data,
        );
    }
}

/// Describes how a logical RGB color is packed into the underlying byte buffer.
///
/// The format controls both the number of bytes per pixel and the channel ordering
/// when writing color values into the framebuffer.
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
/// Internally this maintains two planes:
/// - the **back** plane is written to by the PPU
/// - the **front** plane is exposed for rendering by the frontend
///
/// The `mode` controls whether the planes store palette indices or packed colors.
#[derive(Debug, Clone)]
pub struct FrameBuffer {
    /// Index of the **back/write** plane.
    active_index: usize,
    storage: FrameBufferStorage,
    mode: BufferMode,
    frame_ready_hook: Option<FrameReadyHook>,
}

/// Backing storage for the framebuffer planes.
///
/// - `Owned` keeps the planes inside the NES core.
/// - `External` writes directly into caller-provided memory. This is useful when
///   the core runs on a dedicated thread and the frontend owns the pixel buffers.
#[derive(Debug, Clone)]
enum FrameBufferStorage {
    Owned([Box<[u8]>; 2]),
    /// Externally owned double buffers shared with the frontend.
    ///
    /// The PPU writes to the **back** plane (`active_index`). At end-of-frame, `swap()`
    /// publishes the new **front** plane index to the handle.
    External(Arc<ExternalFrameHandle>),
}

/// Shared external framebuffer planes + published front index.
///
/// This is the **simple** model (no ACK / no waiting):
/// - Core thread writes into the back plane and calls `swap()` at end-of-frame.
/// - `swap()` publishes the new front index and flips the back plane.
/// - Frontend reads the current front plane via `front_ptr()`/`front_slice()`.
///
/// # Safety contract
/// The creator must ensure:
/// - both planes are valid writable regions of length `len` for the lifetime of all Arc clones
/// - planes do not overlap
#[derive(Debug)]
pub struct ExternalFrameHandle {
    planes: [NonNull<u8>; 2],
    len: usize,

    /// Pixel format used when the framebuffer is in color mode.
    ///
    /// `None` means the framebuffer is in index mode (1 byte per pixel palette indices).
    color_format: Option<ColorFormat>,

    front_index: AtomicUsize,
    /// Monotonic frame counter incremented each time the core presents a new front buffer.
    frame_seq: AtomicUsize,
    /// Which plane the frontend is currently copying from (0/1), or 2 when idle.
    reading_plane: AtomicUsize,
}

unsafe impl Send for ExternalFrameHandle {}
unsafe impl Sync for ExternalFrameHandle {}

impl ExternalFrameHandle {
    #[inline]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the active color format when the framebuffer is in color mode.
    ///
    /// When `None`, the framebuffer is in index mode (1 byte per pixel).
    #[inline]
    pub fn color_format(&self) -> Option<ColorFormat> {
        self.color_format
    }

    /// Returns the number of bytes per pixel for the current mode.
    ///
    /// - Index mode: 1 byte per pixel
    /// - Color mode: `format.bytes_per_pixel()`
    #[inline]
    pub fn bytes_per_pixel(&self) -> usize {
        self.color_format.map(|f| f.bytes_per_pixel()).unwrap_or(1)
    }

    /// Published **front** plane index.
    #[inline]
    pub fn front_index(&self) -> usize {
        self.front_index.load(Ordering::Acquire)
    }

    /// Monotonic frame sequence number.
    ///
    /// This value is incremented each time the core thread calls [`present`].
    /// Frontends can use it to detect new frames without comparing buffers.
    #[inline]
    pub fn frame_seq(&self) -> usize {
        self.frame_seq.load(Ordering::Acquire)
    }

    /// Returns the current **front** plane as an immutable slice.
    #[inline]
    pub fn front_slice(&self) -> &[u8] {
        let idx = self.front_index();
        unsafe { slice::from_raw_parts(self.planes[idx].as_ptr() as *const u8, self.len) }
    }

    /// Returns a raw pointer to the **front** plane and its length.
    #[inline]
    pub fn front_ptr(&self) -> (*const u8, usize) {
        let idx = self.front_index();
        (self.planes[idx].as_ptr() as *const u8, self.len)
    }

    /// Publish `index` as the new **front** plane.
    ///
    /// This also bumps the [`frame_seq`] so the frontend can observe that a new
    /// frame has been presented.
    #[inline]
    pub fn present(&self, index: usize) {
        debug_assert!(index < 2);
        self.front_index.store(index, Ordering::Release);
        // Increment after publishing the front index.
        self.frame_seq.fetch_add(1, Ordering::Release);
    }

    #[inline]
    pub fn plane_slice(&self, index: usize) -> &[u8] {
        debug_assert!(index < 2);
        unsafe { slice::from_raw_parts(self.planes[index].as_ptr() as *const u8, self.len) }
    }

    /// Returns a raw writable pointer to the given plane.
    ///
    /// # Safety
    /// The caller must ensure there is no concurrent aliasing access to this plane
    /// (e.g. frontend reading while core writes), otherwise this can cause UB/data races.
    #[inline]
    fn plane_ptr_mut(&self, index: usize) -> *mut u8 {
        debug_assert!(index < 2);
        self.planes[index].as_ptr()
    }

    const NOT_READING: usize = 2;

    /// Begin a frontend copy of the current front plane.
    ///
    /// Returns the stable front index to copy from. The caller must call
    /// [`end_front_copy`] after the copy completes.
    #[inline]
    pub fn begin_front_copy(&self) -> usize {
        // Ensure we mark the same plane that is currently published as front.
        loop {
            let idx = self.front_index();
            self.reading_plane.store(idx, Ordering::Release);
            // If front changed concurrently, drop the marker and retry.
            if self.front_index() == idx {
                return idx;
            }
            self.reading_plane
                .store(Self::NOT_READING, Ordering::Release);
        }
    }

    /// End a frontend copy started with [`begin_front_copy`].
    #[inline]
    pub fn end_front_copy(&self) {
        self.reading_plane
            .store(Self::NOT_READING, Ordering::Release);
    }

    #[inline]
    fn wait_until_not_reading(&self, index: usize) {
        // This is a short critical window (frontend memcpy). Spin briefly, then yield.
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
}

/// Selects how framebuffer data is stored.
///
/// `Index` mode stores one byte per pixel as a palette index.
/// `Color` mode stores packed RGB/RGBA pixels according to the chosen `ColorFormat`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BufferMode {
    /// Palette index buffer (1 byte per pixel).
    #[default]
    Index,
    /// Packed color buffer using a concrete `ColorFormat`.
    Color { format: ColorFormat },
}

impl FrameBuffer {
    /// Creates a new `FrameBuffer` with the given mode and raw buffer length.
    ///
    /// This is a low-level constructor. Prefer the `new_*` convenience constructors
    /// when you want a framebuffer sized to the NES screen.
    pub fn new(mode: BufferMode, len: usize) -> Self {
        if let BufferMode::Color { format } = &mode {
            let expected = SCREEN_WIDTH * SCREEN_HEIGHT * format.bytes_per_pixel();
            debug_assert!(
                len == expected,
                "FrameBuffer len ({len}) does not match expected pixel buffer size ({expected}) for {:?}",
                format
            );
        }

        Self {
            active_index: 0,
            storage: FrameBufferStorage::Owned([
                vec![0; len].into_boxed_slice(),
                vec![0; len].into_boxed_slice(),
            ]),
            mode,
            frame_ready_hook: None,
        }
    }

    /// Creates a new framebuffer backed by externally provided double buffers.
    ///
    /// This allows the PPU to write directly into frontend-owned memory.
    ///
    /// Returns both:
    /// - the `FrameBuffer` intended to live inside the NES thread
    /// - an `Arc<ExternalFrameHandle>` intended to be held by the frontend thread
    ///
    /// # Safety
    /// - `plane0` and `plane1` must point to writable regions of length `len`
    /// - both buffers must remain valid for the lifetime of the `FrameBuffer` and any `Arc` clones
    /// - the two buffers must not overlap
    pub unsafe fn new_external(
        mode: BufferMode,
        len: usize,
        plane0: *mut u8,
        plane1: *mut u8,
    ) -> (Self, Arc<ExternalFrameHandle>) {
        if let BufferMode::Color { format } = &mode {
            let expected = SCREEN_WIDTH * SCREEN_HEIGHT * format.bytes_per_pixel();
            debug_assert!(
                len == expected,
                "FrameBuffer len ({len}) does not match expected pixel buffer size ({expected}) for {:?}",
                format
            );
        }

        let planes = [
            NonNull::new(plane0).expect("plane0 must not be null"),
            NonNull::new(plane1).expect("plane1 must not be null"),
        ];

        let color_format = match mode {
            BufferMode::Index => None,
            BufferMode::Color { format } => Some(format),
        };

        // Publish plane 0 as the initial front buffer.
        // The PPU will start writing into plane 1 (back).
        let handle = Arc::new(ExternalFrameHandle {
            planes,
            len,
            color_format,
            front_index: AtomicUsize::new(0),
            frame_seq: AtomicUsize::new(0),
            reading_plane: AtomicUsize::new(ExternalFrameHandle::NOT_READING),
        });

        let fb = Self {
            // active_index is the **back** (write) plane
            active_index: 1,
            storage: FrameBufferStorage::External(Arc::clone(&handle)),
            mode,
            frame_ready_hook: None,
        };

        (fb, handle)
    }

    /// Creates a new index-mode framebuffer sized to the NES screen.
    ///
    /// Each pixel is stored as a single palette index byte.
    pub fn new_index() -> Self {
        Self::new(BufferMode::Index, SCREEN_WIDTH * SCREEN_HEIGHT)
    }

    /// Creates a new color framebuffer with the given format,
    /// sized to the NES screen.
    pub fn new_color(format: ColorFormat) -> Self {
        let len = SCREEN_WIDTH * SCREEN_HEIGHT * format.bytes_per_pixel();
        Self::new(BufferMode::Color { format }, len)
    }

    /// Creates a new 16-bit RGB555 framebuffer.
    pub fn new_rgb555() -> Self {
        Self::new_color(ColorFormat::Rgb555)
    }

    /// Creates a new 16-bit RGB565 framebuffer.
    pub fn new_rgb565() -> Self {
        Self::new_color(ColorFormat::Rgb565)
    }

    /// Creates a new 24-bit RGB888 framebuffer.
    pub fn new_rgb888() -> Self {
        Self::new_color(ColorFormat::Rgb888)
    }

    /// Creates a new 32-bit RGBA8888 framebuffer.
    pub fn new_rgba8888() -> Self {
        Self::new_color(ColorFormat::Rgba8888)
    }

    /// Creates a new 32-bit BGRA8888 framebuffer.
    pub fn new_bgra8888() -> Self {
        Self::new_color(ColorFormat::Bgra8888)
    }

    /// Creates a new 32-bit ARGB8888 framebuffer.
    pub fn new_argb8888() -> Self {
        Self::new_color(ColorFormat::Argb8888)
    }

    /// Returns a read-only view of the **front** plane for rendering.
    ///
    /// The returned slice is interpreted according to the current `BufferMode`:
    /// - `Index`: 1 byte per pixel containing a palette index
    /// - `Color`: packed pixels in the selected `ColorFormat`
    pub fn render(&self) -> &[u8] {
        self.plane_slice(1 - self.active_index)
    }

    /// Returns a read-only view of the given plane by index.
    ///
    /// This is a low-level accessor intended for bridge layers (e.g. FFI to
    /// libretro/Flutter) that need to expose the raw backing storage. For
    /// normal PPU usage prefer [`render`] and [`write`].
    #[inline]
    pub fn plane(&self, index: usize) -> &[u8] {
        self.plane_slice(index)
    }

    /// Returns the number of bytes per scanline (pitch) for the current mode.
    ///
    /// In `Index` mode this is simply `SCREEN_WIDTH` (1 byte per pixel).
    /// In `Color` mode it is `SCREEN_WIDTH * format.bytes_per_pixel()`.
    #[inline]
    pub fn pitch(&self) -> usize {
        match &self.mode {
            BufferMode::Index => SCREEN_WIDTH,
            BufferMode::Color { format, .. } => SCREEN_WIDTH * format.bytes_per_pixel(),
        }
    }

    pub fn set_frame_ready_callback(
        &mut self,
        cb: Option<FrameReadyCallback>,
        user_data: *mut c_void,
    ) {
        self.frame_ready_hook = cb.map(|cb| FrameReadyHook { cb, user_data });
    }

    /// Returns the index of the current **back** (write) plane.
    ///
    /// For external storage, the published **front** index is managed by the
    /// [`ExternalFrameHandle`].
    #[inline]
    pub fn active_plane_index(&self) -> usize {
        self.active_index
    }

    /// Returns a mutable view of the **back** plane for PPU writes.
    ///
    /// The frontend should read from [`render`], which exposes the **front** plane.
    /// After the PPU finishes a frame, call [`swap`] to present the back plane.
    pub fn write(&mut self) -> &mut [u8] {
        self.plane_slice_mut(self.active_index)
    }

    /// Presents the back plane as the new front plane.
    ///
    /// After calling this:
    /// - the previously written (back) plane becomes the render source
    /// - the previously rendered (front) plane becomes the new back plane and is cleared
    pub fn swap(&mut self) {
        match &self.storage {
            FrameBufferStorage::Owned(_) => {
                let finished_back = self.active_index;
                self.active_index = 1 - self.active_index;
                if let Some(hook) = self.frame_ready_hook {
                    hook.call(finished_back, self.pitch());
                }
                self.write().fill(0);
            }
            FrameBufferStorage::External(handle) => {
                // Publish the plane we just finished writing as the new front buffer.
                let finished_back = self.active_index;
                handle.present(finished_back);

                // Switch to the other plane for the next frame.
                self.active_index = 1 - self.active_index;

                if let Some(hook) = self.frame_ready_hook {
                    hook.call(finished_back, self.pitch());
                }

                // The new back plane was previously the front plane. If the frontend is
                // still copying it, wait for the copy to finish before clearing/writing.
                handle.wait_until_not_reading(self.active_index);
                self.write().fill(0);
            }
        }
    }

    /// Clears both planes to zero.
    ///
    /// This is useful when resetting the PPU or when you need a fully blank frame.
    pub fn clear(&mut self) {
        match &mut self.storage {
            FrameBufferStorage::Owned(planes) => {
                for plane in planes {
                    plane.fill(0);
                }
            }
            FrameBufferStorage::External(handle) => {
                for i in 0..2 {
                    handle.wait_until_not_reading(i);
                    unsafe {
                        slice::from_raw_parts_mut(handle.plane_ptr_mut(i), handle.len()).fill(0)
                    };
                }
            }
        }
    }

    /// Returns `true` if the framebuffer is currently configured in index mode.
    ///
    /// In index mode each pixel is stored as a single palette index byte
    /// (`0..=63`) instead of a packed RGB/RGBA color.
    #[inline]
    pub fn is_index_mode(&self) -> bool {
        matches!(self.mode, BufferMode::Index)
    }

    /// Writes a single pixel at `(x, y)` using a palette index.
    ///
    /// This helper is only valid when the framebuffer is in `Index` mode.
    pub fn write_index(&mut self, x: usize, y: usize, index: u8) {
        match &mut self.mode {
            BufferMode::Index => {
                let idx = y * SCREEN_WIDTH + x;
                self.write()[idx] = index;
            }
            BufferMode::Color { .. } => {
                panic!("write_index called on color framebuffer");
            }
        }
    }

    /// Writes a single pixel at `(x, y)` using an RGB triplet.
    ///
    /// This helper is only valid when the framebuffer is in `Color` mode and
    /// encodes the color into the underlying buffer according to the active
    /// `ColorFormat`.
    pub fn write_color(&mut self, x: usize, y: usize, color: Color) {
        match self.mode {
            BufferMode::Index => panic!("write_color called on index framebuffer"),
            BufferMode::Color { format } => {
                let buffer = self.plane_slice_mut(self.active_index);
                let bpp = format.bytes_per_pixel();
                let idx = (y * SCREEN_WIDTH + x) * bpp;
                debug_assert!(idx + bpp <= buffer.len());

                match format {
                    ColorFormat::Rgb555 => {
                        // 5 bits per channel: use high bits of 8-bit channels
                        let r5 = (color.r as u16) >> 3;
                        let g5 = (color.g as u16) >> 3;
                        let b5 = (color.b as u16) >> 3;
                        let packed = (r5 << 10) | (g5 << 5) | b5;
                        buffer[idx] = (packed & 0xFF) as u8;
                        buffer[idx + 1] = (packed >> 8) as u8;
                    }
                    ColorFormat::Rgb565 => {
                        let r5 = (color.r as u16) >> 3;
                        let g6 = (color.g as u16) >> 2;
                        let b5 = (color.b as u16) >> 3;
                        let packed = (r5 << 11) | (g6 << 5) | b5;
                        buffer[idx] = (packed & 0xFF) as u8;
                        buffer[idx + 1] = (packed >> 8) as u8;
                    }
                    ColorFormat::Rgb888 => {
                        // 8 bits per channel, 3 bytes: R, G, B
                        buffer[idx] = color.r;
                        buffer[idx + 1] = color.g;
                        buffer[idx + 2] = color.b;
                    }
                    ColorFormat::Rgba8888 => {
                        // 8 bits per channel, 4 bytes: R, G, B, A
                        buffer[idx] = color.r;
                        buffer[idx + 1] = color.g;
                        buffer[idx + 2] = color.b;
                        buffer[idx + 3] = 0xFF; // opaque alpha
                    }
                    ColorFormat::Bgra8888 => {
                        // 8 bits per channel, 4 bytes: B, G, R, A
                        buffer[idx] = color.b;
                        buffer[idx + 1] = color.g;
                        buffer[idx + 2] = color.r;
                        buffer[idx + 3] = 0xFF; // opaque alpha
                    }
                    ColorFormat::Argb8888 => {
                        // 8 bits per channel, 4 bytes: A, R, G, B
                        buffer[idx] = 0xFF; // opaque alpha
                        buffer[idx + 1] = color.r;
                        buffer[idx + 2] = color.g;
                        buffer[idx + 3] = color.b;
                    }
                }
            }
        }
    }

    #[inline]
    fn plane_slice(&self, index: usize) -> &[u8] {
        match &self.storage {
            FrameBufferStorage::Owned(planes) => &planes[index],
            FrameBufferStorage::External(handle) => handle.plane_slice(index),
        }
    }

    #[inline]
    fn plane_slice_mut(&mut self, index: usize) -> &mut [u8] {
        match &mut self.storage {
            FrameBufferStorage::Owned(planes) => &mut planes[index],
            FrameBufferStorage::External(handle) => unsafe {
                slice::from_raw_parts_mut(handle.plane_ptr_mut(index), handle.len())
            },
        }
    }
}

impl Default for FrameBuffer {
    fn default() -> Self {
        Self::new(BufferMode::Index, SCREEN_WIDTH * SCREEN_HEIGHT)
    }
}
