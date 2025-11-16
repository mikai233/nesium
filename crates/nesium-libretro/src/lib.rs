//! Demo libretro core built with `libretro-bridge`.
//!
//! This crate exposes a tiny illustrative core that renders a moving color
//! gradient and emits a simple sine wave so that the `libretro-bridge`
//! integration can be validated inside RetroArch.

use std::f32::consts::TAU;

use libretro_bridge::{
    Frame, GameGeometry, GameInfo, LibretroCore, LoadGameError, RuntimeHandles, SystemAvInfo,
    SystemInfo, SystemTiming, export_libretro_core,
};

const WIDTH: u32 = 320;
const HEIGHT: u32 = 240;
const SAMPLE_RATE: f64 = 44_100.0;
const AUDIO_FRAMES: usize = (SAMPLE_RATE as usize) / 60;

struct DemoCore {
    frame: u64,
    pixels: Vec<u16>,
    tone_phase: f32,
}

impl DemoCore {
    fn new() -> Self {
        Self {
            frame: 0,
            pixels: vec![0; (WIDTH * HEIGHT) as usize],
            tone_phase: 0.0,
        }
    }

    fn render_pattern(&mut self) {
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let idx = (y * WIDTH + x) as usize;
                let phase = self.frame as u32;
                let r = (((x + phase) & 0x1F) as u16) << 10;
                let g = (((y + (phase >> 1)) & 0x1F) as u16) << 5;
                let b = (((x ^ y) + phase) & 0x1F) as u16;
                self.pixels[idx] = r | g | b;
            }
        }
        self.frame = self.frame.wrapping_add(1);
    }

    fn pixel_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.pixels.as_ptr() as *const u8,
                self.pixels.len() * std::mem::size_of::<u16>(),
            )
        }
    }

    fn generate_audio(&mut self) -> [[i16; 2]; AUDIO_FRAMES] {
        let mut frames = [[0i16; 2]; AUDIO_FRAMES];
        let step = (220.0f32 / SAMPLE_RATE as f32) * TAU;
        for sample in &mut frames {
            let value = (self.tone_phase).sin();
            let amplitude = (value * 5_000.0) as i16;
            sample[0] = amplitude;
            sample[1] = amplitude;
            self.tone_phase = (self.tone_phase + step) % TAU;
        }
        frames
    }
}

impl LibretroCore for DemoCore {
    fn construct() -> Self
    where
        Self: Sized,
    {
        Self::new()
    }

    fn system_info() -> SystemInfo {
        SystemInfo::new("Nesium Demo Core", env!("CARGO_PKG_VERSION"))
            .with_extensions("bin|nes|rom")
    }

    fn system_av_info(&mut self) -> SystemAvInfo {
        SystemAvInfo {
            geometry: GameGeometry {
                base_width: WIDTH,
                base_height: HEIGHT,
                max_width: WIDTH,
                max_height: HEIGHT,
                aspect_ratio: WIDTH as f32 / HEIGHT as f32,
            },
            timing: SystemTiming {
                fps: 60.0,
                sample_rate: SAMPLE_RATE,
            },
        }
    }

    fn run(&mut self, runtime: &mut RuntimeHandles) {
        self.render_pattern();
        if let Some(video) = runtime.video() {
            let pitch = (WIDTH as usize * std::mem::size_of::<u16>()) as usize;
            video.submit(Frame::from_pixels(self.pixel_bytes(), WIDTH, HEIGHT, pitch));
        }

        let audio_frames = self.generate_audio();
        runtime.audio().push_frames(&audio_frames);
    }

    fn load_game(&mut self, game: &GameInfo<'_>) -> Result<(), LoadGameError> {
        if game.data.is_none() && game.path.is_none() {
            // Allow RetroArch to boot even with "No content".
            return Ok(());
        }
        Ok(())
    }

    fn unload_game(&mut self) {
        self.frame = 0;
        self.tone_phase = 0.0;
        self.pixels.fill(0);
    }
}

export_libretro_core!(DemoCore);
