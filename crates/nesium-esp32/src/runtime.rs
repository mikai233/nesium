//! Small runtime wrapper that connects the `nesium-core` NES emulator
//! to platform-specific I/O on ESP32.
//!
//! Design goals:
//! - keep the NES core platform-agnostic;
//! - describe display / audio / input via small traits;
//! - let board-specific crates implement those traits without modifying
//!   the emulator internals.

use anyhow::{anyhow, Result};
use nesium_core::{
    Nes,
    cartridge,
    ppu::{buffer::ColorFormat, SCREEN_HEIGHT, SCREEN_WIDTH},
};

/// Abstraction for a frame output backend (LCD, framebuffer, etc.).
///
/// - One frame is `SCREEN_WIDTH x SCREEN_HEIGHT` pixels;
/// - Pixel format is RGB565, little-endian, row-major;
/// - Typical implementations:
///   - SPI LCDs (e.g. ILI9341, ST7789) that accept full-frame writes;
///   - memory-mapped LCDs where you memcpy into VRAM;
///   - GUI libraries (LVGL, etc.) where you blit into an image widget.
pub trait FrameSink {
    /// Present a fully rendered RGB565 frame buffer.
    fn present_frame(&mut self, frame_rgb565_le: &[u8]);
}

/// Abstraction for an audio output backend.
///
/// The NES core exposes stereo PCM at a host sample rate:
/// - sample format: `f32` in `[-1.0, 1.0]`;
/// - channel layout: interleaved stereo `LRLRLR...`;
/// - sample rate: chosen by the implementation (e.g. 44100 or 48000 Hz).
pub trait AudioSink {
    /// Target device sample rate in Hz.
    ///
    /// The NES core configures its internal resampler to match this rate so
    /// that `push_samples` always receives host-rate audio.
    fn sample_rate(&self) -> u32;

    /// Push a batch of interleaved stereo samples.
    ///
    /// - `samples.len()` is always even;
    /// - indices 0/1 hold the first frame (L/R), 2/3 the second, etc;
    /// - implementations should avoid blocking (e.g. write into a ring buffer
    ///   that an I2S/DAC task consumes).
    fn push_samples(&mut self, samples: &[f32]);
}

/// Abstraction for an input source (controller / buttons).
///
/// Typical implementations:
/// - read GPIO / I2C keyboard / Bluetooth gamepad state;
/// - call `nes.set_button(pad, Button, pressed)` to update the NES controllers.
pub trait InputSource {
    /// Poll input devices once per frame and update controller state.
    fn poll_input(&mut self, nes: &mut Nes);
}

/// Frame backend that discards all video output.
///
/// Useful when you want to validate core behavior without a display.
pub struct NullFrameSink;

impl NullFrameSink {
    pub fn new() -> Self {
        Self
    }
}

impl FrameSink for NullFrameSink {
    fn present_frame(&mut self, _frame_rgb565_le: &[u8]) {
        // Intentionally no-op: no display attached.
    }
}

/// Audio backend that drops all samples.
///
/// Once the NES core is stable you can replace this with an I2S/DAC backend.
pub struct NullAudioSink {
    sample_rate: u32,
}

impl NullAudioSink {
    /// Create a new null audio backend.
    ///
    /// `sample_rate` defines the host sample rate used by the NES core
    /// (typically 44100 or 48000 Hz).
    pub fn new(sample_rate: u32) -> Self {
        Self { sample_rate }
    }
}

impl AudioSink for NullAudioSink {
    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn push_samples(&mut self, _samples: &[f32]) {
        // Intentionally drop all audio data.
    }
}

/// Input backend that never reports any pressed buttons.
///
/// Useful for bring-up and for non-interactive test ROMs.
pub struct NullInputSource;

impl NullInputSource {
    pub fn new() -> Self {
        Self
    }
}

impl InputSource for NullInputSource {
    fn poll_input(&mut self, _nes: &mut Nes) {
        // Intentionally no-op: no input wired up.
    }
}

/// Runtime wrapper that wires the NES core to display/audio/input traits.
///
/// This type is completely agnostic of the underlying hardware. It only
/// handles:
/// - constructing the NES core;
/// - loading a static ROM image as a cartridge;
/// - stepping the emulator one frame at a time and forwarding results.
pub struct NesRuntime<D, A, I>
where
    D: FrameSink,
    A: AudioSink,
    I: InputSource,
{
    nes: Nes,
    display: D,
    audio: A,
    input: I,
    /// Reusable audio buffer to avoid allocating every frame.
    audio_buffer: Vec<f32>,
}

impl<D, A, I> NesRuntime<D, A, I>
where
    D: FrameSink,
    A: AudioSink,
    I: InputSource,
{
    /// Create a runtime from a statically embedded `.nes` ROM image.
    ///
    /// - `display`: display backend (LCD / LVGL / framebuffer, etc.);
    /// - `audio`: audio backend (I2S / DAC, etc.);
    /// - `input`: input backend (buttons / gamepad, etc.);
    /// - `rom_image`: complete `.nes` file (header + PRG + CHR), typically from `include_bytes!`.
    pub fn from_static_rom(
        display: D,
        audio: A,
        input: I,
        rom_image: &'static [u8],
    ) -> Result<Self> {
        // Ask the audio backend for the host sample rate.
        let sample_rate = audio.sample_rate();
        let mut nes = Nes::new_with_sample_rate(ColorFormat::Rgb565, sample_rate);

        // Load the embedded ROM as a cartridge using the static-slice loader.
        let cart = cartridge::load_cartridge(rom_image)
            .map_err(|err| anyhow!("failed to load embedded ROM: {err}"))?;
        nes.insert_cartridge(cart);

        Ok(Self {
            nes,
            display,
            audio,
            input,
            audio_buffer: Vec::new(),
        })
    }

    /// Execute one frame of emulation:
    /// 1) poll input devices and update controller state;
    /// 2) run CPU/PPU/APU for one frame and collect audio;
    /// 3) forward audio and video to the backends.
    ///
    /// Frame pacing (for example, locking to 60 Hz) is the caller's responsibility.
    pub fn step_frame(&mut self) {
        // 1) Read inputs and update controllers.
        self.input.poll_input(&mut self.nes);

        // 2) Run one frame and gather audio samples.
        self.audio_buffer.clear();
        self.nes.run_frame_with_audio(&mut self.audio_buffer);

        if !self.audio_buffer.is_empty() {
            self.audio.push_samples(&self.audio_buffer);
        }

        // 3) Fetch the current framebuffer (RGB565) and present it.
        let frame = self.nes.render_buffer();
        if !frame.is_empty() {
            debug_assert_eq!(
                frame.len(),
                SCREEN_WIDTH * SCREEN_HEIGHT * ColorFormat::Rgb565.bytes_per_pixel()
            );
            self.display.present_frame(frame);
        }
    }

    /// Expose a mutable reference to the NES core for debugging or custom control.
    pub fn nes_mut(&mut self) -> &mut Nes {
        &mut self.nes
    }
}
