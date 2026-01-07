//! Demo libretro core built with `libretro-bridge`.
//!
//! This crate exposes a tiny illustrative core that renders a moving color
//! gradient and emits a simple sine wave so that the `libretro-bridge`
//! integration can be validated inside RetroArch.

use libretro_bridge::{
    Frame, GameGeometry, GameInfo, LibretroCore, LoadGameError, RuntimeHandles, SystemAvInfo,
    SystemInfo, SystemTiming, export_libretro_core,
};
use nesium_core::{
    Nes,
    cartridge::load_cartridge,
    controller::Button,
    ppu::{SCREEN_HEIGHT, SCREEN_WIDTH, buffer::ColorFormat},
    reset_kind::ResetKind,
};

use libretro_bridge::raw::{
    RETRO_DEVICE_ID_JOYPAD_A, RETRO_DEVICE_ID_JOYPAD_B, RETRO_DEVICE_ID_JOYPAD_DOWN,
    RETRO_DEVICE_ID_JOYPAD_LEFT, RETRO_DEVICE_ID_JOYPAD_RIGHT, RETRO_DEVICE_ID_JOYPAD_SELECT,
    RETRO_DEVICE_ID_JOYPAD_START, RETRO_DEVICE_ID_JOYPAD_UP, RETRO_DEVICE_JOYPAD,
};

const WIDTH: u32 = SCREEN_WIDTH as u32;
const HEIGHT: u32 = SCREEN_HEIGHT as u32;
const SAMPLE_RATE: f64 = 44_100.0;
// NES NTSC framerate (~60.0988 Hz). Keep in sync with runtime frame pacing.
const FPS: f64 = 1_000_000_000.0 / 16_639_263.0;
const COLOR_FORMAT: ColorFormat = ColorFormat::Rgb555;

struct NesiumCore {
    nes: Nes,
}

impl NesiumCore {
    fn new() -> Self {
        Self {
            nes: Nes::builder()
                .format(COLOR_FORMAT)
                .sample_rate(SAMPLE_RATE as u32)
                .build(),
        }
    }

    /// Polls libretro input and updates the NES controller state.
    fn update_input(&mut self, runtime: &mut RuntimeHandles) {
        if let Some(input) = runtime.input() {
            // Libretro convention: poll once per frame before querying state.
            input.poll();

            // Handle up to two players: port 0 (player 1) and port 1 (player 2).
            for port in 0..=1 {
                let is_pressed = |id| input.state(port, RETRO_DEVICE_JOYPAD, 0, id) != 0;
                let pad = port as usize;

                self.nes
                    .set_button(pad, Button::A, is_pressed(RETRO_DEVICE_ID_JOYPAD_A));
                self.nes
                    .set_button(pad, Button::B, is_pressed(RETRO_DEVICE_ID_JOYPAD_B));
                self.nes.set_button(
                    pad,
                    Button::Select,
                    is_pressed(RETRO_DEVICE_ID_JOYPAD_SELECT),
                );
                self.nes
                    .set_button(pad, Button::Start, is_pressed(RETRO_DEVICE_ID_JOYPAD_START));
                self.nes
                    .set_button(pad, Button::Up, is_pressed(RETRO_DEVICE_ID_JOYPAD_UP));
                self.nes
                    .set_button(pad, Button::Down, is_pressed(RETRO_DEVICE_ID_JOYPAD_DOWN));
                self.nes
                    .set_button(pad, Button::Left, is_pressed(RETRO_DEVICE_ID_JOYPAD_LEFT));
                self.nes
                    .set_button(pad, Button::Right, is_pressed(RETRO_DEVICE_ID_JOYPAD_RIGHT));
            }
        }
    }

    fn render_audio(&mut self) -> Vec<[i16; 2]> {
        let samples = self.nes.run_frame(true);
        let mut frames = Vec::new();
        for chunk in samples.chunks(2) {
            let l = *chunk.first().unwrap_or(&0.0);
            let r = *chunk.get(1).unwrap_or(&l);
            let l_i16 = (l.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            let r_i16 = (r.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
            frames.push([l_i16, r_i16]);
        }
        frames
    }
}

impl LibretroCore for NesiumCore {
    fn construct() -> Self
    where
        Self: Sized,
    {
        Self::new()
    }

    fn system_info() -> SystemInfo {
        SystemInfo::new("Nesium Core", env!("CARGO_PKG_VERSION")).with_extensions("bin|nes|rom")
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
                fps: FPS,
                sample_rate: SAMPLE_RATE,
            },
        }
    }

    fn run(&mut self, runtime: &mut RuntimeHandles) {
        // Update controller state from libretro input before running a frame.
        self.update_input(runtime);

        let audio_frames = self.render_audio();
        if let Some(video) = runtime.video() {
            let pitch = WIDTH as usize * COLOR_FORMAT.bytes_per_pixel();
            video.submit(Frame::from_pixels(
                self.nes.render_buffer(),
                WIDTH,
                HEIGHT,
                pitch,
            ));
        }

        if !audio_frames.is_empty() {
            runtime.audio().push_frames(&audio_frames);
        }
    }

    fn load_game(&mut self, game: &GameInfo<'_>) -> Result<(), LoadGameError> {
        match game.data {
            Some(data) => {
                let cartridge =
                    load_cartridge(data).map_err(|e| LoadGameError::Message(e.to_string()))?;
                self.nes.insert_cartridge(cartridge);
                Ok(())
            }
            None => Err(LoadGameError::MissingContent),
        }
    }

    fn unload_game(&mut self) {
        self.nes.power_off();
    }

    fn reset(&mut self) {
        self.nes.reset(ResetKind::Soft);
    }
}

export_libretro_core!(NesiumCore);
