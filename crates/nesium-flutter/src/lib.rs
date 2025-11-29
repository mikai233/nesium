//! nesium-flutter
//!
//! Bridge between the NES core and the Flutter/macOS runner.
//! - Flutter (via FRB) starts the runtime and issues control commands.
//! - The runtime owns a dedicated NES thread that renders frames into a
//!   double-buffered BGRA8888 framebuffer.
//! - The platform layer registers a frame-ready callback, copies the latest
//!   buffer into a CVPixelBuffer, and marks the Flutter texture as dirty.

pub mod api;
mod frb_generated; /* AUTO INJECTED BY flutter_rust_bridge. This line may not be accurate, and you can change it according to your needs. */
use std::{
    os::raw::{c_uint, c_void},
    path::PathBuf,
    sync::{
        OnceLock,
        atomic::{AtomicPtr, Ordering},
        mpsc::{self, Sender},
    },
    thread,
    time::{Duration, Instant},
};

use nesium_core::{
    Nes,
    controller::Button as CoreButton,
    ppu::{SCREEN_HEIGHT, SCREEN_WIDTH, buffer::ColorFormat},
};

pub const FRAME_WIDTH: usize = SCREEN_WIDTH;
pub const FRAME_HEIGHT: usize = SCREEN_HEIGHT;
pub const BYTES_PER_PIXEL: usize = 4; // BGRA8888

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PadButton {
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}

impl From<PadButton> for CoreButton {
    fn from(value: PadButton) -> Self {
        match value {
            PadButton::A => CoreButton::A,
            PadButton::B => CoreButton::B,
            PadButton::Select => CoreButton::Select,
            PadButton::Start => CoreButton::Start,
            PadButton::Up => CoreButton::Up,
            PadButton::Down => CoreButton::Down,
            PadButton::Left => CoreButton::Left,
            PadButton::Right => CoreButton::Right,
        }
    }
}

/// C ABI callback type used by Swift/macOS.
pub type FrameReadyCallback = extern "C" fn(
    buffer_index: c_uint,
    width: c_uint,
    height: c_uint,
    pitch: c_uint,
    user_data: *mut c_void,
);

enum ControlMessage {
    LoadRom(PathBuf),
    Reset,
    SetCallback(Option<FrameReadyCallback>, *mut c_void),
    SetButton {
        pad: u8,
        button: PadButton,
        pressed: bool,
    },
}

// SAFETY: raw pointers and function pointers are forwarded to the NES thread without dereferencing
// on the sending thread; the receiver owns and uses them.
unsafe impl Send for ControlMessage {}

// Control channel used by FRB and C ABI calls to send work to the NES thread.
static CONTROL_TX: OnceLock<Sender<ControlMessage>> = OnceLock::new();

// Pointer to the current front buffer. Backed by the NES thread's owned buffers.
static FRONT_PTR: AtomicPtr<u8> = AtomicPtr::new(std::ptr::null_mut());

fn start_thread_if_needed() -> Sender<ControlMessage> {
    CONTROL_TX
        .get_or_init(|| {
            let (tx, rx) = mpsc::channel();
            thread::spawn(move || nes_thread(rx));
            tx
        })
        .clone()
}

fn nes_thread(rx: std::sync::mpsc::Receiver<ControlMessage>) {
    // We hold the NES inside the thread and expose a raw pointer to its
    // render buffer. The buffer lives as long as the thread, so the pointer
    // remains valid across frames.
    let mut callback: Option<(FrameReadyCallback, *mut c_void)> = None;

    let frame_time = Duration::from_secs_f32(1.0 / 60.0);
    let mut nes: Option<Nes> = None;

    loop {
        while let Ok(msg) = rx.try_recv() {
            match msg {
                ControlMessage::LoadRom(path) => match load_nes(&path) {
                    Ok(new_nes) => {
                        let buf_ptr = new_nes.render_buffer().as_ptr() as *mut u8;
                        FRONT_PTR.store(buf_ptr, Ordering::Release);
                        nes = Some(new_nes);
                    }
                    Err(err) => eprintln!("Failed to load ROM {path:?}: {err}"),
                },
                ControlMessage::Reset => {
                    if let Some(n) = nes.as_mut() {
                        n.reset();
                    }
                }
                ControlMessage::SetCallback(cb, user_data) => {
                    callback = cb.map(|f| (f, user_data));
                }
                ControlMessage::SetButton {
                    pad,
                    button,
                    pressed,
                } => {
                    if let Some(n) = nes.as_mut() {
                        n.set_button(pad as usize, CoreButton::from(button), pressed);
                    }
                }
            }
        }

        let Some(nes) = nes.as_mut() else {
            thread::sleep(Duration::from_millis(10));
            continue;
        };

        let frame_start = Instant::now();
        nes.run_frame();

        let buffer = nes.render_buffer();
        FRONT_PTR.store(buffer.as_ptr() as *mut u8, Ordering::Release);

        if let Some((cb, user_data)) = callback {
            cb(
                0, // buffer index is always 0 since we expose the render buffer directly
                FRAME_WIDTH as c_uint,
                FRAME_HEIGHT as c_uint,
                (FRAME_WIDTH * BYTES_PER_PIXEL) as c_uint,
                user_data,
            );
        }

        let elapsed = frame_start.elapsed();
        if elapsed < frame_time {
            thread::sleep(frame_time - elapsed);
        }
    }
}

fn load_nes(path: &PathBuf) -> Result<Nes, String> {
    let mut nes = Nes::new(ColorFormat::Bgra8888);
    nes.load_cartridge_from_file(path)
        .map_err(|e| e.to_string())?;
    Ok(nes)
}

pub(crate) fn send_command(cmd: ControlMessage) -> Result<(), String> {
    let tx = start_thread_if_needed();
    tx.send(cmd).map_err(|e| e.to_string())
}

// === C ABI exposed to Swift/macOS =========================================

#[unsafe(no_mangle)]
pub extern "C" fn nesium_runtime_start() {
    let _ = start_thread_if_needed();
}

#[unsafe(no_mangle)]
pub extern "C" fn nesium_set_frame_ready_callback(
    cb: Option<FrameReadyCallback>,
    user_data: *mut c_void,
) {
    let _ = send_command(ControlMessage::SetCallback(cb, user_data));
}

#[unsafe(no_mangle)]
pub extern "C" fn nesium_copy_frame(
    _buffer_index: c_uint,
    dst: *mut u8,
    dst_pitch: c_uint,
    dst_height: c_uint,
) {
    if dst.is_null() {
        return;
    }

    let src_ptr = FRONT_PTR.load(Ordering::Acquire);
    if src_ptr.is_null() {
        return;
    }

    let height = FRAME_HEIGHT.min(dst_height as usize);
    let src_pitch = FRAME_WIDTH * BYTES_PER_PIXEL;
    let dst_pitch = dst_pitch as usize;

    // Safety: caller guarantees `dst` points to `dst_pitch * dst_height` bytes.
    let dst_slice = unsafe {
        std::slice::from_raw_parts_mut(
            dst,
            dst_pitch
                .saturating_mul(dst_height as usize)
                .min(src_pitch * FRAME_HEIGHT),
        )
    };
    let src_slice =
        unsafe { std::slice::from_raw_parts(src_ptr as *const u8, src_pitch * FRAME_HEIGHT) };

    for y in 0..height {
        let src_off = y * src_pitch;
        let dst_off = y * dst_pitch;
        let src_row = &src_slice[src_off..src_off + src_pitch];
        let dst_row = &mut dst_slice[dst_off..dst_off + src_pitch.min(dst_pitch)];
        dst_row.copy_from_slice(&src_row[..dst_row.len()]);
    }
}
