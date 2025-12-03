mod runtime;

use std::time::{Duration, Instant};

use anyhow::Result;
use esp_idf_sys as _; // Ensure esp-idf startup patches are linked in

use crate::runtime::{NesRuntime, NullAudioSink, NullFrameSink, NullInputSource};

/// Embedded test ROM used to validate the NES core on ESP32 without a filesystem.
///
/// This defaults to `nestest.nes` from the `nes-test-roms` bundle. You can replace
/// this with your own game ROM, for example:
///
/// ```ignore
/// static ROM_IMAGE: &[u8] = include_bytes!("/spiffs/smb1.nes");
/// ```
static ROM_IMAGE: &[u8] =
    include_bytes!("../../nesium-core/vendor/nes-test-roms/other/nestest.nes");

/// Target frame duration (~59.94 Hz).
const TARGET_FRAME: Duration = Duration::from_nanos(16_683_000);

fn main() {
    // For most esp-idf Rust projects this call is required: it links in
    // a set of startup patches (e.g. PSRAM configuration) so the firmware
    // can boot correctly.
    esp_idf_sys::link_patches();

    if let Err(err) = run() {
        eprintln!("NES runtime failed to start: {err}");
    }
}

fn run() -> Result<()> {
    // 1) Prepare default I/O backends.
    //
    // The initial version only wires up "null" backends: no display, no audio,
    // no input. This lets you validate that the NES core runs reliably on
    // ESP32 before integrating real hardware.
    //
    // To hook up real hardware, replace:
    // - `NullFrameSink` with your SPI/LCD implementation,
    // - `NullAudioSink` with an I2S/DAC audio backend,
    // - `NullInputSource` with GPIO / gamepad input.
    let display = NullFrameSink::new();
    let audio = NullAudioSink::new(48_000);
    let input = NullInputSource::new();

    // 2) Create the NES runtime and load the embedded ROM.
    let mut nes_runtime = NesRuntime::from_static_rom(display, audio, input, ROM_IMAGE)?;

    // 3) Main loop: advance the emulator at ~60 Hz.
    //
    // Here we use `Instant + thread::sleep` for a simple frame scheduler:
    // in the esp-idf `std` environment `thread::sleep` is backed by FreeRTOS.
    // If you migrate to a multi-task design, you can move `step_frame` into
    // its own task and drive it from there.
    let mut next_frame_deadline = Instant::now();
    loop {
        nes_runtime.step_frame();

        next_frame_deadline += TARGET_FRAME;
        let now = Instant::now();
        if next_frame_deadline > now {
            let sleep = next_frame_deadline - now;
            std::thread::sleep(sleep);
        } else {
            // If emulation falls behind, drop the delay and resync to "now".
            next_frame_deadline = now;
        }
    }
}
