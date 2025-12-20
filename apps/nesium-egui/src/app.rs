use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

pub mod controller;
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
use nesium_core::{
    audio::bus::AudioBusConfig,
    ppu::{SCREEN_HEIGHT, SCREEN_WIDTH},
    reset_kind::ResetKind,
};

use crate::emulator_thread::{Command, EmulatorThread, Event};

use self::{
    controller::{ControllerDevice, ControllerInput, InputPreset},
    fonts::install_cjk_font,
    gamepad::GamepadManager,
    i18n::{I18n, Language, TextId},
};

pub struct AppConfig {
    pub rom_path: Option<PathBuf>,
}

pub struct NesiumApp {
    emulator: EmulatorThread,
    frame_texture: Option<TextureHandle>,
    frame_image: Option<Arc<ColorImage>>,
    rom_path: Option<PathBuf>,
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
    controllers: [ControllerInput; 4],
    controller_devices: [ControllerDevice; 4],
    controller_presets: [InputPreset; 4],
    active_input_port: usize,
    gamepads: Option<GamepadManager>,
    pixel_perfect_scaling: bool,
}

impl NesiumApp {
    pub fn new(cc: &eframe::CreationContext<'_>, config: AppConfig) -> Self {
        cc.egui_ctx.set_visuals(Visuals::light());
        install_cjk_font(&cc.egui_ctx);

        let emulator = EmulatorThread::new();

        let mut app = Self {
            emulator,
            frame_texture: None,
            frame_image: None,
            rom_path: None,
            paused: false,
            status_line: None,
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
            audio_cfg: AudioBusConfig::default(),
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
            pixel_perfect_scaling: false,
        };

        if let Some(path) = config.rom_path {
            if let Err(err) = app.load_rom(&path) {
                app.status_line = Some(match app.language() {
                    Language::English => format!("Failed to load ROM: {err}"),
                    Language::ChineseSimplified => format!("加载 ROM 失败: {err}"),
                });
            }
        }

        app
    }

    fn has_rom(&self) -> bool {
        self.rom_path.is_some()
    }

    fn load_rom(&mut self, path: &Path) -> Result<()> {
        self.emulator.send(Command::LoadRom(path.to_path_buf()));
        self.rom_path = Some(path.to_path_buf());
        self.paused = false;
        self.fps = 0.0;
        self.fps_accum_frames = 0;
        self.fps_last_update = Instant::now();

        // Note: Status line will be updated by Event::StatusInfo from thread
        Ok(())
    }

    fn reset(&mut self) {
        self.emulator.send(Command::Reset(ResetKind::Soft));
        self.paused = false;
        // Reset local input state
        for ctrl in &mut self.controllers {
            ctrl.release_all();
        }
    }

    fn eject(&mut self) {
        self.emulator.send(Command::Eject);
        self.rom_path = None;
        for ctrl in &mut self.controllers {
            ctrl.release_all();
        }
        self.fps = 0.0;
        self.fps_accum_frames = 0;
    }

    fn update_frame_texture(&mut self, ctx: &EguiContext) {
        let handle = &self.emulator.frame_handle;

        let idx = handle.begin_front_copy();
        let slice = handle.plane_slice(idx);

        // Avoid per-frame allocations: keep a `ColorImage` buffer around and update it in-place.
        let image = self.frame_image.get_or_insert_with(|| {
            Arc::new(ColorImage::filled(
                [SCREEN_WIDTH, SCREEN_HEIGHT],
                egui::Color32::BLACK,
            ))
        });
        {
            let image = Arc::make_mut(image);
            debug_assert_eq!(image.size, [SCREEN_WIDTH, SCREEN_HEIGHT]);
            debug_assert_eq!(slice.len(), SCREEN_WIDTH * SCREEN_HEIGHT * 4);
            for (dst, src) in image.pixels.iter_mut().zip(slice.chunks_exact(4)) {
                *dst = egui::Color32::from_rgb(src[0], src[1], src[2]);
            }
        }

        match &mut self.frame_texture {
            Some(tex) => tex.set(image.clone(), TextureOptions::NEAREST),
            None => {
                self.frame_texture =
                    Some(ctx.load_texture("framebuffer", image.clone(), TextureOptions::NEAREST));
            }
        }

        handle.end_front_copy();
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
        // Avoid an unbounded repaint loop: throttle to ~60Hz while the emulator runs.
        // This reduces CPU contention and improves frame pacing on some platforms.
        if self.has_rom() && !self.paused {
            ctx.request_repaint_after(Duration::from_micros(16_666));
        }

        // 1. Process Events from Emulator Thread
        let mut frames_received = 0;
        while let Some(event) = self.emulator.try_recv() {
            match event {
                Event::FrameReady => {
                    // We only need to update the texture once per UI frame, even if multiple
                    // emu frames arrived (we just take the latest state from shared memory).
                    // But we count them for FPS.
                    frames_received += 1;
                }
                Event::StatusInfo(msg) => {
                    self.status_line = Some(msg);
                }
                Event::Error(msg) => {
                    self.status_line = Some(format!("Error: {}", msg));
                }
            }
        }

        if frames_received > 0 {
            self.update_frame_texture(ctx);
        }

        // 2. Poll Gamepads
        if let Some(manager) = &mut self.gamepads {
            manager.poll();
        }

        // 3. Process Input
        let keyboard_busy = ctx.wants_keyboard_input();
        let mut input_changed = false;
        for (port, ctrl) in self.controllers.iter_mut().enumerate() {
            // Check previous state logic if needed, but for now we just resend.
            // Optimization: compare hash or dirty flag?
            // Actually, sending every frame is fine for local channel.
            match self.controller_devices[port] {
                ControllerDevice::Keyboard => {
                    let blocked = keyboard_busy;
                    ctrl.sync_from_input(ctx, port, blocked);
                }
                ControllerDevice::Gamepad(id) => {
                    if let Some(manager) = &self.gamepads {
                        ctrl.sync_from_gamepad(port, manager, id);
                    } else {
                        ctrl.release_all();
                    }
                }
                ControllerDevice::Disabled => {
                    ctrl.release_all();
                }
            }
        }

        // Always send input state to emulator
        // Cloning 4 ControllerInputs is cheap (Vec<Button> is small)
        self.emulator
            .send(Command::UpdateInput(self.controllers.clone()));

        // 4. Handle Drag & Drop
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        if let Some(path) = dropped.iter().filter_map(|f| f.path.clone()).next_back() {
            let _ = self.load_rom(&path);
        }

        // 5. Update FPS
        if frames_received > 0 {
            self.fps_accum_frames += frames_received;
        }
        let now = Instant::now();
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

        // 6. Draw UI
        if let Some(cmd) = self.draw_menu(ctx) {
            self.handle_app_command(ctx, cmd);
        }

        self.draw_main_view(ctx);
        self.show_viewports(ctx);
    }
}
