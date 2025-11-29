//! nesium-flutter
//!
//! This crate provides a small bridge layer between Rust and the Flutter/macOS
//! runner. For now, it does **not** integrate the real NES core – instead it
//! runs a background thread that continuously renders a BGRA8888 fractal
//! into a pair of frame buffers.
//!
//! The goal is to prove out the data flow:
//!   Rust thread -> shared frame buffers -> Swift copies into CVPixelBuffer ->
//!   Flutter Texture widget displays the result.
//!
//! Once this is stable, the gradient renderer can be replaced by the real NES
//! PPU output without changing the Swift/Flutter integration surface.
use std::os::raw::{c_uint, c_void};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

/// Logical framebuffer width used for the fractal demo.
pub const FRAME_WIDTH: usize = 256;
/// Logical framebuffer height used for the fractal demo.
pub const FRAME_HEIGHT: usize = 240;
/// Bytes per pixel for the demo output (BGRA8888).
pub const BYTES_PER_PIXEL: usize = 4;

/// Frames per second target for the demo renderer.
const TARGET_FPS: f32 = 60.0;

/// C ABI callback type used by Swift/macOS.
///
/// The Rust side invokes this after finishing a frame, passing:
/// - `buffer_index`: which of the two internal buffers (0 or 1) now contains
///   the freshly rendered frame
/// - `width`, `height`: logical dimensions in pixels
/// - `pitch`: number of bytes per row in the internal buffer
/// - `user_data`: opaque pointer provided by the caller when registering the
///   callback; typically used to recover a Swift object instance.
pub type FrameReadyCallback = extern "C" fn(
    buffer_index: c_uint,
    width: c_uint,
    height: c_uint,
    pitch: c_uint,
    user_data: *mut c_void,
);

/// Internal framebuffer state.
///
/// For now we keep two buffers (double buffering) so that we can keep a stable
/// snapshot for the caller to copy from, while the next frame is rendered
/// into the other buffer.
struct FrameState {
    buffers: [Vec<u8>; 2],
    active: usize,
    width: usize,
    height: usize,
}

impl FrameState {
    fn new(width: usize, height: usize) -> Self {
        let len = width * height * BYTES_PER_PIXEL;
        Self {
            buffers: [vec![0; len], vec![0; len]],
            active: 0,
            width,
            height,
        }
    }

    /// Returns the index of the buffer that is safe to treat as "back" for the
    /// next frame.
    fn back_index(&self) -> usize {
        self.active ^ 1
    }

    fn buffer_len(&self) -> usize {
        self.width * self.height * BYTES_PER_PIXEL
    }

    fn pitch(&self) -> usize {
        self.width * BYTES_PER_PIXEL
    }
}

/// Global runtime instance used by the demo.
///
/// For the Flutter/macOS app we only ever expect a single runtime instance,
/// so a global singleton is sufficient. We keep it behind an `Arc` so that
/// the render thread and callers can safely share ownership.
static RUNTIME: OnceLock<Arc<Runtime>> = OnceLock::new();

struct Runtime {
    state: Mutex<FrameState>,
    callback: Mutex<Option<(FrameReadyCallback, *mut c_void)>>,
}

// SAFETY: `Runtime` only contains `Mutex<FrameState>` and a callback pointer
// that is never touched from Rust except to pass it back into the user-provided
// callback. We never dereference `user_data` on the Rust side, so it is safe
// to treat `Runtime` as `Send` and `Sync` for the purposes of the render
// thread and global singleton.
unsafe impl Send for Runtime {}
unsafe impl Sync for Runtime {}

impl Runtime {
    fn new() -> Self {
        Self {
            state: Mutex::new(FrameState::new(FRAME_WIDTH, FRAME_HEIGHT)),
            callback: Mutex::new(None),
        }
    }

    /// Spawns the background thread that continuously renders the demo
    /// fractal into the internal frame buffers.
    ///
    /// The thread runs until the process exits; for the initial demo we do
    /// not support stopping/restarting the runtime.
    fn spawn(self: &Arc<Self>) {
        let rt = Arc::clone(self);
        thread::spawn(move || rt.run_loop());
    }

    /// Main render loop: renders frames into the back buffer, flips them to
    /// become the active/front buffer, and notifies the registered callback.
    ///
    /// The heavy fractal rendering is performed *outside* of the framebuffer
    /// mutex to avoid blocking the UI thread when it copies frames via
    /// `copy_frame_into`.
    fn run_loop(&self) {
        let frame_time = Duration::from_secs_f32(1.0 / TARGET_FPS);
        let mut t: f32 = 0.0;

        loop {
            let start = Instant::now();

            // 1) Grab metadata and a raw pointer to the back buffer while
            //    holding the mutex for a very short time.
            //
            // We *do not* perform any heavy rendering work while the mutex
            // is held; that would block the UI thread when it tries to lock
            // the same state in `copy_frame_into`.
            let (back_index, width, height, pitch, buf_ptr, buf_len) = {
                let mut state = self.state.lock().unwrap();
                let back = state.back_index();
                let width = state.width;
                let height = state.height;
                let pitch = state.pitch();

                let buf = &mut state.buffers[back];
                let len = buf.len();
                let ptr = buf.as_mut_ptr();

                (
                    back as c_uint,
                    width as c_uint,
                    height as c_uint,
                    pitch as c_uint,
                    ptr,
                    len,
                )
            };

            // 2) Perform the fractal rendering outside of the mutex. We render
            //    into the "back" buffer using the raw pointer obtained above.
            //
            // Safety:
            // - The underlying Vec for this buffer is never reallocated after
            //   initialization (we do not change its capacity or length).
            // - Only the render thread ever writes to the back buffer; readers
            //   always copy from the current "front" buffer as indicated by
            //   `state.active`, which we only flip *after* rendering is done.
            unsafe {
                let slice = std::slice::from_raw_parts_mut(buf_ptr, buf_len);
                render_fractal_into(
                    slice,
                    width as usize,
                    height as usize,
                    pitch as usize,
                    t,
                );
            }

            // 3) Now that the back buffer contains a complete frame, flip it
            //    to become the active/front buffer under the mutex. This is a
            //    very short critical section.
            {
                let mut state = self.state.lock().unwrap();
                state.active = back_index as usize;
            }

            // 4) Notify the caller, if a callback was registered. The buffer
            //    index we pass here is the one we just flipped to "front".
            if let Some((cb, user_data)) = *self.callback.lock().unwrap() {
                cb(back_index, width, height, pitch, user_data);
            }

            // 5) Simple frame pacing for ~60 FPS. This is intentionally rough;
            //    precise timing will be handled by the real NES core later.
            let elapsed = start.elapsed();
            if elapsed < frame_time {
                thread::sleep(frame_time - elapsed);
            }

            t += 1.0 / TARGET_FPS;
        }
    }

    fn set_callback(&self, cb: Option<FrameReadyCallback>, user_data: *mut c_void) {
        let mut guard = self.callback.lock().unwrap();
        *guard = cb.map(|f| (f, user_data));
    }

    /// Copies the contents of the selected buffer into the caller-provided
    /// destination.
    ///
    /// This avoids exposing raw pointers to the internal storage across the
    /// FFI boundary and keeps all concurrent access synchronized under the
    /// internal mutex.
    fn copy_frame_into(
        &self,
        buffer_index: usize,
        dst: *mut u8,
        dst_pitch: usize,
        dst_height: usize,
    ) {
        let state = self.state.lock().unwrap();
        if buffer_index >= state.buffers.len() {
            return;
        }

        let src = &state.buffers[buffer_index];

        let height = state.height.min(dst_height);
        let src_pitch = state.pitch();

        if dst.is_null() {
            return;
        }

        // Safety: the caller promises that `dst` points to a buffer of at
        // least `dst_pitch * dst_height` bytes.
        let dst_slice = unsafe { std::slice::from_raw_parts_mut(dst, dst_pitch * dst_height) };

        for y in 0..height {
            let src_off = y * src_pitch;
            let dst_off = y * dst_pitch;

            let src_row = &src[src_off..src_off + src_pitch];
            let dst_row = &mut dst_slice[dst_off..dst_off + src_pitch.min(dst_pitch)];

            dst_row.copy_from_slice(&src_row[..dst_row.len()]);
        }
    }
}

/// Render a time-varying Mandelbrot fractal into the given buffer.
///
/// The buffer is expected to be `height * pitch` bytes large and encoded in
/// BGRA8888 format. We render a Mandelbrot zoom focused on an interesting
/// region, with a continuous exponential zoom factor driven by `t` to create a
/// “falling into the set” feeling over a long period of time.
fn render_fractal_into(buf: &mut [u8], width: usize, height: usize, pitch: usize, t: f32) {
    let bytes_per_row = pitch;
    debug_assert!(buf.len() >= height * bytes_per_row);

    // Lower iteration count to keep things fast even in release.
    let max_iter: u32 = 64;

    // Focus on a well-known detailed area (near Seahorse Valley).
    // These are classic Mandelbrot coordinates.
    let cx_center: f32 = -0.7436439;
    let cy_center: f32 = 0.1318259;

    // Continuous exponential zoom with no abrupt reset, to create a smoother
    // “falling into the set” feeling over a long period of time. You can
    // tweak `zoom_speed` to control how quickly the zoom progresses.
    let zoom_speed: f32 = 0.50;
    let zoom_factor = (2.0_f32).powf(t * zoom_speed);

    let base_scale = 2.5 / (width.min(height) as f32);
    let scale = base_scale / zoom_factor;

    for y in 0..height {
        let row_off = y * bytes_per_row;
        let row = &mut buf[row_off..row_off + bytes_per_row];

        // Map y into the imaginary axis around the chosen center.
        let imag0 = (y as f32 - height as f32 / 2.0) * scale + cy_center;

        for x in 0..width {
            // Map x into the real axis around the chosen center.
            let real0 = (x as f32 - width as f32 / 2.0) * scale + cx_center;

            let mut zx = 0.0_f32;
            let mut zy = 0.0_f32;
            let mut iter = 0;

            // Classic Mandelbrot iteration: z_{n+1} = z_n^2 + c.
            while zx * zx + zy * zy <= 4.0 && iter < max_iter {
                let zx2 = zx * zx - zy * zy + real0;
                let zy2 = 2.0 * zx * zy + imag0;
                zx = zx2;
                zy = zy2;
                iter += 1;
            }

            let offset = x * BYTES_PER_PIXEL;

            if iter == max_iter {
                // Likely inside the set: render as dark.
                row[offset + 0] = 0;
                row[offset + 1] = 0;
                row[offset + 2] = 0;
                row[offset + 3] = 0xFF;
            } else {
                // Simple (non-smooth) coloring based only on the normalized
                // iteration count. This avoids expensive log/sin operations
                // and keeps the inner loop very cheap.
                let norm = iter as f32 / max_iter as f32;

                // Cheap "gradient" palette:
                // - red grows linearly
                // - green grows quadratically
                // - blue fades out
                let r_f = norm;
                let g_f = (norm * norm).min(1.0);
                let b_f = (1.0 - norm).max(0.0);

                let r_byte = (r_f * 255.0).clamp(0.0, 255.0) as u8;
                let g_byte = (g_f * 255.0).clamp(0.0, 255.0) as u8;
                let b_byte = (b_f * 255.0).clamp(0.0, 255.0) as u8;

                // BGRA8888 in memory (matches kCVPixelFormatType_32BGRA).
                row[offset + 0] = b_byte;
                row[offset + 1] = g_byte;
                row[offset + 2] = r_byte;
                row[offset + 3] = 0xFF;
            }
        }
    }
}

/// Ensures that the global runtime exists and its render thread is running.
fn ensure_runtime() -> Arc<Runtime> {
    let arc = RUNTIME.get_or_init(|| {
        let rt = Arc::new(Runtime::new());
        rt.spawn();
        rt
    });
    Arc::clone(arc)
}

// === C ABI exposed to Swift/macOS ==========================================

/// Starts the demo runtime if it is not already running.
///
/// This is intended to be called from Dart via flutter_rust_bridge, or from
/// Swift early in the app's lifecycle. Subsequent calls are cheap no-ops.
#[unsafe(no_mangle)]
pub extern "C" fn nesium_runtime_start() {
    let _ = ensure_runtime();
}

/// Registers (or clears) the frame-ready callback.
///
/// Swift should call this once during initialization, passing an
/// `@convention(c)` function pointer and an opaque `user_data` pointer that
/// can later be used to recover the owning Swift object.
#[unsafe(no_mangle)]
pub extern "C" fn nesium_set_frame_ready_callback(
    cb: Option<FrameReadyCallback>,
    user_data: *mut c_void,
) {
    let rt = ensure_runtime();
    rt.set_callback(cb, user_data);
}

/// Copies the contents of the specified internal buffer into the caller's
/// destination buffer.
///
/// This is typically called from Swift/macOS in response to a frame-ready
/// callback:
///
/// 1. Lock the destination CVPixelBuffer.
/// 2. Obtain its base address and bytes-per-row.
/// 3. Call this function with the buffer index and destination info.
/// 4. Unlock the CVPixelBuffer and notify Flutter that a new frame is
///    available.
#[unsafe(no_mangle)]
pub extern "C" fn nesium_copy_frame(
    buffer_index: c_uint,
    dst: *mut u8,
    dst_pitch: c_uint,
    dst_height: c_uint,
) {
    let rt = ensure_runtime();
    rt.copy_frame_into(
        buffer_index as usize,
        dst,
        dst_pitch as usize,
        dst_height as usize,
    );
}
