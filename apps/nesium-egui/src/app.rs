use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
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

use anyhow::Result;
use eframe::egui;
use egui::{ColorImage, Context as EguiContext, TextureHandle, TextureOptions, Visuals};
use gilrs::GamepadId;
use nesium_core::{
    audio::bus::AudioBusConfig,
    ppu::buffer::ColorFormat,
    ppu::{SCREEN_HEIGHT, SCREEN_WIDTH},
    reset_kind::ResetKind,
};
use nesium_runtime::{AudioMode, Runtime, RuntimeConfig, RuntimeEvent, RuntimeHandle, VideoConfig};

struct VideoBackingStore {
    _plane0: Box<[u8]>,
    _plane1: Box<[u8]>,
}

use self::{
    controller::{ControllerDevice, ControllerInput, InputPreset},
    fonts::install_cjk_font,
    gamepad::GamepadManager,
    i18n::{I18n, Language, TextId},
};

pub struct AppConfig {
    pub rom_path: Option<PathBuf>,
}

pub(super) struct UiState {
    i18n: I18n,
    audio_cfg: AudioBusConfig,
    controllers: [ControllerInput; 4],
    controller_devices: [ControllerDevice; 4],
    controller_presets: [InputPreset; 4],
    active_input_port: usize,
    pixel_perfect_scaling: bool,
    gamepads_available: bool,
    gamepads: Vec<(GamepadId, String)>,
}

pub struct NesiumApp {
    video_backing: VideoBackingStore,
    runtime_handle: RuntimeHandle,
    runtime: Runtime,
    frame_texture: Option<TextureHandle>,
    frame_image: Option<Arc<ColorImage>>,
    last_frame_seq: u64,
    rom_path: Option<PathBuf>,
    paused: bool,
    status_line: Option<String>,
    ui_state: Arc<Mutex<UiState>>,
    fps: f32,
    fps_accum_frames: u32,
    fps_last_update: Instant,
    show_debugger: bool,
    show_tools: bool,
    show_palette: bool,
    show_input: bool,
    show_audio: bool,
    show_about: bool,
    gamepads: Option<GamepadManager>,
}

impl NesiumApp {
    pub fn new(cc: &eframe::CreationContext<'_>, config: AppConfig) -> Self {
        cc.egui_ctx.set_visuals(Visuals::light());
        install_cjk_font(&cc.egui_ctx);

        let len = SCREEN_WIDTH * SCREEN_HEIGHT * 4;
        let plane0 = vec![0u8; len].into_boxed_slice();
        let plane1 = vec![0u8; len].into_boxed_slice();

        let mut video_backing = VideoBackingStore {
            _plane0: plane0,
            _plane1: plane1,
        };

        // SAFETY: `video_backing` keeps the two planes alive for the lifetime of the app.
        // The planes do not overlap and are sized to the NES framebuffer.
        let runtime = Runtime::start(RuntimeConfig {
            video: VideoConfig {
                color_format: ColorFormat::Rgba8888,
                plane0: video_backing._plane0.as_mut_ptr(),
                plane1: video_backing._plane1.as_mut_ptr(),
            },
            audio: AudioMode::Auto,
        })
        .expect("failed to start nesium runtime");
        let runtime_handle = runtime.handle();

        let gamepads = GamepadManager::new();
        let gamepad_snapshot = gamepads.as_ref().map(|m| m.gamepads()).unwrap_or_default();

        let ui_state = Arc::new(Mutex::new(UiState {
            i18n: I18n::new(Language::ChineseSimplified),
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
            pixel_perfect_scaling: false,
            gamepads_available: gamepads.is_some(),
            gamepads: gamepad_snapshot,
        }));

        let mut app = Self {
            video_backing,
            runtime_handle,
            runtime,
            frame_texture: None,
            frame_image: None,
            last_frame_seq: 0,
            rom_path: None,
            paused: false,
            status_line: None,
            ui_state,
            fps: 0.0,
            fps_accum_frames: 0,
            fps_last_update: Instant::now(),
            show_debugger: false,
            show_tools: false,
            show_palette: false,
            show_input: false,
            show_audio: false,
            show_about: false,
            gamepads,
        };

        if let Some(path) = config.rom_path
            && let Err(err) = app.load_rom(&path)
        {
            app.status_line = Some(match app.language() {
                Language::English => format!("Failed to load ROM: {err}"),
                Language::ChineseSimplified => format!("加载 ROM 失败: {err}"),
            });
        }

        app
    }

    fn has_rom(&self) -> bool {
        self.rom_path.is_some()
    }

    fn load_rom(&mut self, path: &Path) -> Result<()> {
        self.runtime_handle
            .load_rom(path.to_path_buf())
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        self.rom_path = Some(path.to_path_buf());
        self.paused = false;
        self.runtime_handle.set_paused(false);
        self.last_frame_seq = self.runtime_handle.frame_seq();
        self.fps = 0.0;
        self.fps_accum_frames = 0;
        self.fps_last_update = Instant::now();

        // Note: Status line will be updated by RuntimeEvent::StatusInfo from thread
        Ok(())
    }

    fn reset(&mut self) {
        let _ = self.runtime_handle.reset(ResetKind::Soft);
        self.paused = false;
        self.runtime_handle.set_paused(false);
        self.last_frame_seq = self.runtime_handle.frame_seq();
        // Reset local input state
        if let Ok(mut ui_state) = self.ui_state.lock() {
            for ctrl in &mut ui_state.controllers {
                ctrl.release_all();
            }
        }
    }

    fn eject(&mut self) {
        let _ = self.runtime_handle.eject();
        self.rom_path = None;
        self.last_frame_seq = self.runtime_handle.frame_seq();
        if let Ok(mut ui_state) = self.ui_state.lock() {
            for ctrl in &mut ui_state.controllers {
                ctrl.release_all();
            }
        }
        self.fps = 0.0;
        self.fps_accum_frames = 0;
    }

    fn update_frame_texture(&mut self, ctx: &EguiContext) {
        let handle = self.runtime_handle.frame_handle();

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
            // `Color32` is `[r, g, b, a]` in memory; our runtime outputs RGBA8888 with a=255.
            // Copying raw bytes avoids per-pixel conversion overhead and improves frame pacing.
            let dst_bytes = unsafe {
                std::slice::from_raw_parts_mut(
                    image.pixels.as_mut_ptr() as *mut u8,
                    image.pixels.len() * 4,
                )
            };
            dst_bytes.copy_from_slice(slice);
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
        self.ui_state
            .lock()
            .ok()
            .map(|s| s.i18n.text(id))
            .unwrap_or("")
    }

    fn language(&self) -> Language {
        self.ui_state
            .lock()
            .ok()
            .map(|s| s.i18n.language())
            .unwrap_or(Language::English)
    }

    fn set_language(&mut self, language: Language) {
        if let Ok(mut s) = self.ui_state.lock() {
            s.i18n.set_language(language);
        }
    }

    fn pixel_perfect_scaling(&self) -> bool {
        self.ui_state
            .lock()
            .ok()
            .map(|s| s.pixel_perfect_scaling)
            .unwrap_or(false)
    }
}

impl eframe::App for NesiumApp {
    fn update(&mut self, ctx: &EguiContext, _: &mut eframe::Frame) {
        // Avoid an unbounded repaint loop: throttle to ~60Hz while the emulator runs.
        // This reduces CPU contention and improves frame pacing on some platforms.
        if self.has_rom() && !self.paused {
            ctx.request_repaint_after(Duration::from_micros(16_666));
        }

        // 1. Process Events from Runtime thread
        while let Some(event) = self.runtime_handle.try_recv_event() {
            match event {
                RuntimeEvent::StatusInfo(msg) => {
                    self.status_line = Some(msg);
                }
                RuntimeEvent::Error(msg) => {
                    self.status_line = Some(format!("Error: {}", msg));
                }
            }
        }

        let current_seq = self.runtime_handle.frame_seq();
        let new_frames = current_seq.saturating_sub(self.last_frame_seq);
        if new_frames > 0 {
            self.last_frame_seq = current_seq;
            self.update_frame_texture(ctx);
            self.fps_accum_frames = self
                .fps_accum_frames
                .saturating_add((new_frames.min(u32::MAX as u64)) as u32);
        }

        // 2. Poll Gamepads
        let gamepad_snapshot = if let Some(manager) = &mut self.gamepads {
            manager.poll();
            manager.gamepads()
        } else {
            Vec::new()
        };
        if let Ok(mut ui_state) = self.ui_state.lock() {
            ui_state.gamepads_available = self.gamepads.is_some();
            ui_state.gamepads = gamepad_snapshot;
        }

        // 3. Process Input
        let keyboard_busy = ctx.wants_keyboard_input();
        let mut pad_masks = [0u8; 4];
        if let Ok(mut ui_state) = self.ui_state.lock() {
            for port in 0..4 {
                let device = ui_state.controller_devices[port];
                let ctrl = &mut ui_state.controllers[port];
                match device {
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
                pad_masks[port] = ctrl.pressed_mask();
            }
        }

        // Always publish input state via atomics (no control channel, low latency).
        for port in 0..4 {
            self.runtime_handle.set_pad_mask(port, pad_masks[port]);
        }

        // 4. Handle Drag & Drop
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        if let Some(path) = dropped.iter().filter_map(|f| f.path.clone()).next_back() {
            let _ = self.load_rom(&path);
        }

        // 5. Update FPS
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
