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
    collections::VecDeque,
    os::raw::{c_uint, c_void},
    path::PathBuf,
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicPtr, Ordering},
        mpsc::{self, Sender},
    },
    thread,
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use cpal::{
    FromSample, Sample, SampleFormat, SizedSample,
    traits::{DeviceTrait, HostTrait, StreamTrait},
};

use nesium_core::{
    Nes,
    audio::CPU_CLOCK_NTSC,
    controller::Button as CoreButton,
    ppu::{SCREEN_HEIGHT, SCREEN_WIDTH, buffer::ColorFormat},
};

pub const FRAME_WIDTH: usize = SCREEN_WIDTH;
pub const FRAME_HEIGHT: usize = SCREEN_HEIGHT;
pub const BYTES_PER_PIXEL: usize = 4; // BGRA8888

/// CPU clock rate (NTSC).
const CPU_HZ: f64 = CPU_CLOCK_NTSC;
/// PPU clock rate (3x CPU).
const PPU_HZ: f64 = CPU_HZ * 3.0;
/// PPU dots (cycles) per frame.
const DOTS_PER_FRAME: f64 = 341.0 * 262.0;
/// Nominal NTSC frame duration in seconds (~60.10 Hz).
const FRAME_DURATION_SECS: f64 = DOTS_PER_FRAME / PPU_HZ;

/// Thin audio output wrapper that feeds PCM samples from the emulator into
/// cpal's default output stream.
struct NesAudioPlayer {
    buffer: Arc<Mutex<VecDeque<f32>>>,
    sample_rate: u32,
    max_queue: usize,
    _stream: cpal::Stream,
}

impl NesAudioPlayer {
    fn new() -> Result<Self> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .context("no default output device")?;

        let config = device.default_output_config()?;
        let sample_rate = config.sample_rate().0;

        let buffer = Arc::new(Mutex::new(VecDeque::with_capacity(
            (sample_rate / 5) as usize,
        )));
        // Allow ~0.3s of queued audio to avoid underruns; avoid aggressive dropping that skews pitch.
        let max_queue = (sample_rate as f32 * 0.3).ceil() as usize;
        let stream = match config.sample_format() {
            SampleFormat::F32 => {
                Self::build_stream::<f32>(&device, &config.into(), buffer.clone())?
            }
            SampleFormat::I16 => {
                Self::build_stream::<i16>(&device, &config.into(), buffer.clone())?
            }
            SampleFormat::U16 => {
                Self::build_stream::<u16>(&device, &config.into(), buffer.clone())?
            }
            other => anyhow::bail!("unsupported sample format {other:?}"),
        };

        stream.play()?;

        Ok(Self {
            buffer,
            sample_rate,
            max_queue,
            _stream: stream,
        })
    }

    fn build_stream<T>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        buffer: Arc<Mutex<VecDeque<f32>>>,
    ) -> Result<cpal::Stream>
    where
        T: Sample + SizedSample + FromSample<f32>,
    {
        let channels = config.channels as usize;
        let err_fn = |err| eprintln!("Audio stream error: {err}");
        let stream = device.build_output_stream(
            config,
            move |data: &mut [T], _| {
                for frame in data.chunks_mut(channels) {
                    let sample = {
                        let mut guard = buffer.lock().unwrap();
                        guard.pop_front().unwrap_or(0.0)
                    };
                    let converted: T = sample.to_sample::<T>();
                    frame.iter_mut().for_each(|out| *out = converted);
                }
            },
            err_fn,
            None,
        )?;
        Ok(stream)
    }

    /// Pushes a batch of mono samples into the output buffer.
    fn push_samples(&self, samples: &[f32]) {
        if samples.is_empty() {
            return;
        }
        if let Ok(mut guard) = self.buffer.lock() {
            for &raw in samples {
                let scaled = (raw * 0.9).clamp(-1.0, 1.0);
                guard.push_back(scaled);
            }
            if guard.len() > self.max_queue {
                let drop_count = guard.len() - self.max_queue;
                for _ in 0..drop_count {
                    guard.pop_front();
                }
            }
        }
    }

    /// Clears any queued samples (useful when resetting the emulator).
    fn clear(&self) {
        if let Ok(mut guard) = self.buffer.lock() {
            guard.clear();
        }
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

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
    let frame_time = Duration::from_secs_f64(FRAME_DURATION_SECS);
    let mut nes: Option<Nes> = None;

    // Best-effort audio initialization; if the host has no default output
    // device, the emulator will still run but remain silent.
    let audio: Option<NesAudioPlayer> = NesAudioPlayer::new().ok();
    let runtime_sample_rate = audio.as_ref().map(|a| a.sample_rate()).unwrap_or(48_000);

    // Simple fade-in after load/reset to reduce initial pops: ramp from 0 to
    // full volume over the first ~50ms of audio.
    let fade_total_samples: u32 = if runtime_sample_rate > 0 {
        (runtime_sample_rate / 20).max(1) // ~50ms
    } else {
        0
    };
    let mut fade_cursor: u32 = 0;

    let mut next_frame_deadline = Instant::now();

    loop {
        while let Ok(msg) = rx.try_recv() {
            match msg {
                ControlMessage::LoadRom(path) => match load_nes(&path, runtime_sample_rate) {
                    Ok(new_nes) => {
                        let buf_ptr = new_nes.render_buffer().as_ptr() as *mut u8;
                        FRONT_PTR.store(buf_ptr, Ordering::Release);
                        nes = Some(new_nes);
                        // Reset frame pacing so we don't try to "catch up"
                        // for the time spent before the ROM was loaded.
                        next_frame_deadline = Instant::now() + frame_time;

                        // Reset audio queue and timing so the new ROM starts
                        // from a clean state.
                        if let Some(a) = &audio {
                            a.clear();
                        }
                        fade_cursor = 0;
                    }
                    Err(err) => eprintln!("Failed to load ROM {path:?}: {err}"),
                },
                ControlMessage::Reset => {
                    if let Some(n) = nes.as_mut() {
                        n.reset();
                    }
                    if let Some(a) = &audio {
                        a.clear();
                    }
                    fade_cursor = 0;
                    // After a reset, also restart the frame scheduler from
                    // "now" so we don't fast-forward to catch up.
                    next_frame_deadline = Instant::now() + frame_time;
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

        // Fixed-step scheduler similar to nesium-egui: run up to a small
        // number of frames to catch up when behind, otherwise wait until the
        // next frame deadline.
        let now = Instant::now();
        if now < next_frame_deadline {
            thread::sleep(next_frame_deadline - now);
            continue;
        }

        let mut frames_run: u32 = 0;
        while Instant::now() >= next_frame_deadline && frames_run < 3 {
            match &audio {
                Some(audio) => {
                    let mut samples = Vec::new();
                    nes.run_frame_with_audio(&mut samples);

                    // Apply a short fade-in after load/reset to soften any
                    // initial transients.
                    if fade_total_samples > 0 && fade_cursor < fade_total_samples {
                        for s in &mut samples {
                            if fade_cursor >= fade_total_samples {
                                break;
                            }
                            let gain = fade_cursor as f32 / fade_total_samples as f32;
                            *s *= gain;
                            fade_cursor += 1;
                        }
                    }

                    if !samples.is_empty() {
                        audio.push_samples(&samples);
                    }
                }
                None => nes.run_frame(),
            }

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

            next_frame_deadline += frame_time;
            frames_run += 1;
        }
    }
}

fn load_nes(path: &PathBuf, sample_rate: u32) -> Result<Nes, String> {
    let mut nes = Nes::new_with_sample_rate(ColorFormat::Bgra8888, sample_rate);
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
