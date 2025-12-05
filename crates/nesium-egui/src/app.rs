use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

mod controller;
mod dialogs;
mod fonts;
mod gamepad;
mod i18n;
mod main_view;
mod menu;
mod viewports;

use anyhow::{Context, Result};
use eframe::egui;
use egui::{ColorImage, Context as EguiContext, TextureHandle, TextureOptions, Visuals};
use nesium_audio::NesAudioPlayer;
use nesium_core::{
    CpuSnapshot, Nes,
    audio::bus::AudioBusConfig,
    ppu::{SCREEN_HEIGHT, SCREEN_WIDTH, buffer::ColorFormat},
};

use self::{
    controller::{ControllerDevice, ControllerInput, InputPreset},
    fonts::install_cjk_font,
    gamepad::GamepadManager,
    i18n::{I18n, Language, TextId},
};

const TARGET_FRAME: Duration = Duration::from_nanos(16_683_000); // ~59.94 Hz

pub struct AppConfig {
    pub rom_path: Option<PathBuf>,
    pub start_pc: Option<u16>,
}

pub struct NesiumApp {
    nes: Nes,
    frame_texture: Option<TextureHandle>,
    rom_path: Option<PathBuf>,
    start_pc: Option<u16>,
    audio: Option<NesAudioPlayer>,
    paused: bool,
    status_line: Option<String>,
    i18n: I18n,
    fps: f32,
    fps_accum_frames: u32,
    fps_last_update: Instant,
    show_debugger: bool,
    show_tools: bool,
    show_palette: bool,
    show_input: bool,
    show_audio: bool,
    show_about: bool,
    audio_cfg: AudioBusConfig,
    recording: bool,
    record_buffer: Vec<f32>,
    record_sample_rate: u32,
    record_path: Option<PathBuf>,
    controllers: [ControllerInput; 4],
    controller_devices: [ControllerDevice; 4],
    controller_presets: [InputPreset; 4],
    active_input_port: usize,
    gamepads: Option<GamepadManager>,
    next_frame_deadline: Option<Instant>,
}

impl NesiumApp {
    pub fn new(cc: &eframe::CreationContext<'_>, config: AppConfig) -> Self {
        cc.egui_ctx.set_visuals(Visuals::light());
        install_cjk_font(&cc.egui_ctx);

        let mut status_line = None;
        let audio = match NesAudioPlayer::new() {
            Ok(player) => Some(player),
            Err(err) => {
                status_line = Some(format!("Audio init failed: {err}"));
                tracing::warn!("Audio init failed: {err}");
                None
            }
        };
        let sample_rate = audio.as_ref().map(|a| a.sample_rate()).unwrap_or(48_000);
        let mut nes = Nes::new_with_sample_rate(ColorFormat::Rgba8888, sample_rate);

        let audio_cfg = AudioBusConfig::default();
        nes.set_audio_bus_config(audio_cfg);

        let mut app = Self {
            nes,
            frame_texture: None,
            rom_path: None,
            start_pc: config.start_pc,
            audio,
            paused: false,
            status_line,
            i18n: I18n::new(Language::ChineseSimplified),
            fps: 0.0,
            fps_accum_frames: 0,
            fps_last_update: Instant::now(),
            show_debugger: false,
            show_tools: false,
            show_palette: false,
            show_input: false,
            show_audio: false,
            show_about: false,
            audio_cfg,
            recording: false,
            record_buffer: Vec::new(),
            record_sample_rate: sample_rate,
            record_path: None,
            controllers: std::array::from_fn(|_| ControllerInput::new_with_defaults()),
            controller_devices: [
                ControllerDevice::Keyboard,
                ControllerDevice::Keyboard,
                ControllerDevice::Disabled,
                ControllerDevice::Disabled,
            ],
            controller_presets: [InputPreset::NesStandard; 4],
            active_input_port: 0,
            gamepads: GamepadManager::new(),
            next_frame_deadline: None,
        };

        if let Some(path) = config.rom_path
            && let Err(err) = app.load_rom(&path) {
                app.status_line = Some(match app.language() {
                    Language::English => format!("Failed to load ROM: {err}"),
                    Language::ChineseSimplified => format!("加载 ROM 失败: {err}"),
                });
            }

        app
    }

    /// Runs one video frame while emitting audio samples at the host sample rate.
    fn run_frame_with_audio(&mut self) {
        match &mut self.audio {
            Some(audio) => {
                let mut samples = Vec::new();
                self.nes.run_frame_with_audio(&mut samples);
                if self.recording && !samples.is_empty() {
                    self.record_buffer.extend_from_slice(&samples);
                }
                if !samples.is_empty() {
                    audio.push_samples(&samples);
                }
            }
            None => self.nes.run_frame(),
        }
    }

    fn has_rom(&self) -> bool {
        self.rom_path.is_some()
    }

    fn load_rom(&mut self, path: &Path) -> Result<()> {
        self.nes
            .load_cartridge_from_file(path)
            .with_context(|| format!("loading ROM {}", path.display()))?;

        if let Some(pc) = self.start_pc {
            let snapshot = CpuSnapshot {
                pc,
                a: 0,
                x: 0,
                y: 0,
                s: 0xFD,
                p: 0x24,
            };
            self.nes.set_cpu_snapshot(snapshot);
        }

        self.rom_path = Some(path.to_path_buf());
        self.paused = false;
        self.fps = 0.0;
        self.fps_accum_frames = 0;
        self.fps_last_update = Instant::now();
        self.status_line = Some(match self.language() {
            Language::English => format!("Loaded {}", path.display()),
            Language::ChineseSimplified => format!("已加载 {}", path.display()),
        });
        // Reset the frame scheduler so we don't try to "catch up" for the
        // time spent before the ROM was loaded, which would otherwise cause
        // a brief period of fast-forward.
        self.next_frame_deadline = Some(Instant::now() + TARGET_FRAME);
        Ok(())
    }

    fn reset(&mut self) {
        self.nes.reset();
        self.paused = false;
        self.status_line = Some(self.t(TextId::StatusReset).to_string());
        for ctrl in &mut self.controllers {
            ctrl.release_all(&mut self.nes);
        }
        if let Some(audio) = &self.audio {
            audio.clear();
        }
        // After a reset, restart the frame scheduler from "now" to avoid a
        // burst of catch-up frames.
        self.next_frame_deadline = Some(Instant::now() + TARGET_FRAME);
    }

    fn eject(&mut self) {
        self.nes.eject_cartridge();
        self.rom_path = None;
        self.status_line = Some(self.t(TextId::StatusEject).to_string());
        for ctrl in &mut self.controllers {
            ctrl.release_all(&mut self.nes);
        }
        if let Some(audio) = &self.audio {
            audio.clear();
        }
        // When ejecting, reset the frame scheduler as well.
        self.next_frame_deadline = Some(Instant::now() + TARGET_FRAME);
        self.fps = 0.0;
        self.fps_accum_frames = 0;
    }

    fn update_frame_texture(&mut self, ctx: &EguiContext) {
        let frame = self.nes.render_buffer();
        if frame.is_empty() {
            return;
        }

        let image = ColorImage::from_rgba_unmultiplied(
            [SCREEN_WIDTH, SCREEN_HEIGHT],
            frame,
        );

        match &mut self.frame_texture {
            Some(tex) => tex.set(image, TextureOptions::NEAREST),
            None => {
                self.frame_texture =
                    Some(ctx.load_texture("framebuffer", image, TextureOptions::NEAREST));
            }
        }
    }

    fn t(&self, id: TextId) -> &'static str {
        self.i18n.text(id)
    }

    fn language(&self) -> Language {
        self.i18n.language()
    }

    fn set_language(&mut self, language: Language) {
        self.i18n.set_language(language);
    }
}

impl eframe::App for NesiumApp {
    fn update(&mut self, ctx: &EguiContext, _: &mut eframe::Frame) {
        // Drive UI every loop; emulator step pacing is handled below.
        ctx.request_repaint();

        // Keep gamepad state fresh.
        if let Some(manager) = &mut self.gamepads {
            manager.poll();
        }

        let keyboard_busy = ctx.wants_keyboard_input();
        for (port, ctrl) in self.controllers.iter_mut().enumerate() {
            match self.controller_devices[port] {
                ControllerDevice::Keyboard => {
                    let blocked = keyboard_busy;
                    ctrl.sync_from_input(ctx, &mut self.nes, port, blocked);
                }
                ControllerDevice::Gamepad(id) => {
                    if let Some(manager) = &self.gamepads {
                        ctrl.sync_from_gamepad(&mut self.nes, port, manager, id);
                    } else {
                        ctrl.release_all(&mut self.nes);
                    }
                }
                ControllerDevice::Disabled => {
                    ctrl.release_all(&mut self.nes);
                }
            }
        }

        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        if let Some(path) = dropped.iter().filter_map(|f| f.path.clone()).next_back() {
            let _ = self.load_rom(&path);
        }

        // Fixed-step schedule: run frames when deadline passes; allow small catch-up.
        let now = Instant::now();
        let mut deadline = self
            .next_frame_deadline
            .unwrap_or_else(|| now + TARGET_FRAME);
        let mut run_count = 0u32;
        let mut frames_run = 0u32;
        while now >= deadline && run_count < 3 {
            if self.has_rom() && !self.paused {
                self.run_frame_with_audio();
                frames_run += 1;
            }
            deadline += TARGET_FRAME;
            run_count += 1;
        }
        self.next_frame_deadline = Some(deadline);

        // Update FPS based on how many emulation frames we actually ran.
        if frames_run > 0 {
            self.fps_accum_frames = self.fps_accum_frames.saturating_add(frames_run);
        }
        let elapsed = self.fps_last_update.elapsed();
        if elapsed >= Duration::from_secs(1) {
            if elapsed.as_secs_f32() > 0.0 {
                self.fps = self.fps_accum_frames as f32 / elapsed.as_secs_f32();
            } else {
                self.fps = 0.0;
            }
            self.fps_accum_frames = 0;
            self.fps_last_update = now;
        }

        if let Some(next) = self.next_frame_deadline {
            let wait = next.saturating_duration_since(Instant::now());
            ctx.request_repaint_after(wait.min(TARGET_FRAME));
        }
        self.update_frame_texture(ctx);

        if let Some(cmd) = self.draw_menu(ctx) {
            self.handle_app_command(ctx, cmd);
        }

        self.draw_main_view(ctx);
        self.show_viewports(ctx);
    }
}
