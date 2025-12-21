use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
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
use egui::{
    ColorImage, Context as EguiContext, TextureHandle, TextureOptions, ViewportId, Visuals,
};
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
    turbo_frames_per_toggle: u8,
    gamepads_available: bool,
    gamepads: Vec<(GamepadId, String)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(usize)]
pub(super) enum AppViewport {
    Debugger = 0,
    Tools = 1,
    Palette = 2,
    Input = 3,
    Audio = 4,
    About = 5,
}

impl AppViewport {
    pub(super) const ALL: [Self; 6] = [
        Self::Debugger,
        Self::Tools,
        Self::Palette,
        Self::Input,
        Self::Audio,
        Self::About,
    ];

    pub(super) fn id(self) -> ViewportId {
        match self {
            Self::Debugger => ViewportId::from_hash_of("debugger"),
            Self::Tools => ViewportId::from_hash_of("tools"),
            Self::Palette => ViewportId::from_hash_of("palette"),
            Self::Input => ViewportId::from_hash_of("input"),
            Self::Audio => ViewportId::from_hash_of("audio"),
            Self::About => ViewportId::from_hash_of("about"),
        }
    }
}

const VIEWPORT_COUNT: usize = AppViewport::ALL.len();

pub(super) struct Viewports {
    open: [bool; VIEWPORT_COUNT],
    close_requested: [Arc<AtomicBool>; VIEWPORT_COUNT],
}

impl Viewports {
    pub(super) fn new() -> Self {
        Self {
            open: [false; VIEWPORT_COUNT],
            close_requested: std::array::from_fn(|_| Arc::new(AtomicBool::new(false))),
        }
    }

    pub(super) fn is_open(&self, viewport: AppViewport) -> bool {
        self.open[viewport as usize]
    }

    #[cfg(windows)]
    pub(super) fn any_open(&self) -> bool {
        self.open.iter().any(|&open| open)
    }

    pub(super) fn open_mut(&mut self, viewport: AppViewport) -> &mut bool {
        &mut self.open[viewport as usize]
    }

    pub(super) fn set_open(&mut self, viewport: AppViewport, open: bool) {
        self.open[viewport as usize] = open;
    }

    pub(super) fn close_flag(&self, viewport: AppViewport) -> Arc<AtomicBool> {
        Arc::clone(&self.close_requested[viewport as usize])
    }
}

pub struct NesiumApp {
    _video_backing: VideoBackingStore,
    runtime_handle: RuntimeHandle,
    _runtime: Runtime,
    frame_texture: Option<TextureHandle>,
    frame_image: Option<Arc<ColorImage>>,
    last_frame_seq: u64,
    last_pad_masks: [u8; 4],
    last_turbo_masks: [u8; 4],
    rom_path: Option<PathBuf>,
    paused: bool,
    error_dialog: Option<String>,
    error_dialog_close_requested: Arc<AtomicBool>,
    ui_state: Arc<Mutex<UiState>>,
    viewports: Viewports,
    fps: f32,
    fps_accum_frames: u32,
    fps_last_update: Instant,
    gamepads: Option<GamepadManager>,
}

impl NesiumApp {
    pub fn new(cc: &eframe::CreationContext<'_>, config: AppConfig) -> Self {
        cc.egui_ctx.set_visuals(Visuals::light());
        let has_cjk_font = install_cjk_font(&cc.egui_ctx);

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

        runtime_handle.set_turbo_frames_per_toggle(2);

        let ui_state = Arc::new(Mutex::new(UiState {
            i18n: if has_cjk_font {
                I18n::new(Language::ChineseSimplified)
            } else {
                I18n::new(Language::English)
            },
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
            turbo_frames_per_toggle: 2,
            gamepads_available: gamepads.is_some(),
            gamepads: gamepad_snapshot,
        }));

        let mut app = Self {
            _video_backing: video_backing,
            runtime_handle,
            _runtime: runtime,
            frame_texture: None,
            frame_image: None,
            last_frame_seq: 0,
            last_pad_masks: [0u8; 4],
            last_turbo_masks: [0u8; 4],
            rom_path: None,
            paused: false,
            error_dialog: None,
            error_dialog_close_requested: Arc::new(AtomicBool::new(false)),
            ui_state,
            viewports: Viewports::new(),
            fps: 0.0,
            fps_accum_frames: 0,
            fps_last_update: Instant::now(),
            gamepads,
        };

        if let Some(path) = config.rom_path
            && let Err(err) = app.load_rom(&path)
        {
            app.error_dialog = Some(match app.language() {
                Language::English => format!("Failed to load ROM:\n{err}"),
                Language::ChineseSimplified => format!("加载 ROM 失败：\n{err}"),
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

        Ok(())
    }

    fn reset(&mut self) {
        let _ = self.runtime_handle.reset(ResetKind::Soft);
        self.paused = false;
        self.runtime_handle.set_paused(false);
        self.last_frame_seq = self.runtime_handle.frame_seq();
        self.last_pad_masks = [0u8; 4];
        self.last_turbo_masks = [0u8; 4];
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
        self.last_pad_masks = [0u8; 4];
        self.last_turbo_masks = [0u8; 4];
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

    fn show_error_dialog(&mut self, ctx: &EguiContext) {
        let error_viewport_id = egui::ViewportId::from_hash_of("error_dialog");

        if self
            .error_dialog_close_requested
            .swap(false, Ordering::Relaxed)
            || ctx.viewport_for(error_viewport_id, |v| v.input.viewport().close_requested())
        {
            self.error_dialog = None;
            return;
        }

        let Some(message) = self.error_dialog.as_ref() else {
            return;
        };

        let title = match self.language() {
            Language::English => "Error",
            Language::ChineseSimplified => "错误",
        };
        let ok_label = match self.language() {
            Language::English => "OK",
            Language::ChineseSimplified => "确定",
        };
        let copy_label = match self.language() {
            Language::English => "Copy",
            Language::ChineseSimplified => "复制",
        };

        let builder = egui::ViewportBuilder::default()
            .with_title(title)
            .with_inner_size([420.0, 160.0])
            .with_resizable(false)
            .with_minimize_button(false)
            .with_maximize_button(false)
            .with_taskbar(false)
            .with_always_on_top();

        let message = message.clone();
        let close_flag = Arc::clone(&self.error_dialog_close_requested);
        ctx.show_viewport_deferred(error_viewport_id, builder, move |ctx, class| {
            // Keep the dialog visuals consistent with the main window.
            ctx.set_visuals(Visuals::light());

            let mut close_requested =
                ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));

            match class {
                egui::ViewportClass::Embedded => {
                    egui::Window::new(title)
                        .collapsible(false)
                        .resizable(false)
                        .show(ctx, |ui| {
                            ui.label(message.as_str());
                            ui.add_space(10.0);
                            ui.horizontal(|ui| {
                                if ui.button(copy_label).clicked() {
                                    ui.output_mut(|o| {
                                        o.commands.push(eframe::egui::OutputCommand::CopyText(
                                            message.clone(),
                                        ));
                                    });
                                }
                                ui.with_layout(
                                    eframe::egui::Layout::right_to_left(
                                        eframe::egui::Align::Center,
                                    ),
                                    |ui| {
                                        if ui.button(ok_label).clicked() {
                                            close_requested = true;
                                        }
                                    },
                                );
                            });
                        });
                }
                _ => {
                    let content_margin_lr = 18;
                    let content_margin_top = 12;
                    let buttons_margin_bottom = 10;
                    let panel_fill = ctx.style().visuals.panel_fill;

                    egui::TopBottomPanel::bottom("error_dialog_buttons")
                        .frame(
                            egui::Frame::NONE
                                .fill(panel_fill)
                                .inner_margin(egui::Margin {
                                    left: content_margin_lr,
                                    right: content_margin_lr,
                                    top: 6,
                                    bottom: buttons_margin_bottom,
                                }),
                        )
                        .show(ctx, |ui| {
                            ui.horizontal(|ui| {
                                if ui.button(copy_label).clicked() {
                                    ui.output_mut(|o| {
                                        o.commands.push(eframe::egui::OutputCommand::CopyText(
                                            message.clone(),
                                        ));
                                    });
                                }
                                ui.with_layout(
                                    eframe::egui::Layout::right_to_left(
                                        eframe::egui::Align::Center,
                                    ),
                                    |ui| {
                                        if ui.button(ok_label).clicked() {
                                            close_requested = true;
                                        }
                                    },
                                );
                            });
                        });

                    egui::CentralPanel::default()
                        .frame(
                            egui::Frame::NONE
                                .fill(panel_fill)
                                .inner_margin(egui::Margin {
                                    left: content_margin_lr,
                                    right: content_margin_lr,
                                    top: content_margin_top,
                                    bottom: 0,
                                }),
                        )
                        .show(ctx, |ui| {
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    ui.label(message.as_str());
                                });
                        });
                }
            }

            if close_requested {
                close_flag.store(true, Ordering::Relaxed);
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    }
}

impl eframe::App for NesiumApp {
    fn update(&mut self, ctx: &EguiContext, _: &mut eframe::Frame) {
        // Avoid an unbounded repaint loop: throttle to ~60Hz while the emulator runs.
        // This reduces CPU contention and improves frame pacing on some platforms.
        if self.has_rom() && !self.paused {
            ctx.request_repaint_after(Duration::from_micros(16_666));
        }

        // Windows: keep the root viewport responsive while auxiliary viewports are open.
        // This also helps `show_viewport_immediate` (which couples parent/child repaint).
        #[cfg(windows)]
        if self.viewports.any_open() {
            ctx.request_repaint_after(Duration::from_micros(16_666));
        }

        // 1. Process Events from Runtime thread
        while let Some(event) = self.runtime_handle.try_recv_event() {
            match event {
                RuntimeEvent::StatusInfo(msg) => {
                    tracing::info!("{msg}");
                }
                RuntimeEvent::Error(msg) => {
                    tracing::error!("{msg}");
                    self.error_dialog = Some(msg);
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
        if let Ok(mut ui_state) = self.ui_state.try_lock() {
            ui_state.gamepads_available = self.gamepads.is_some();
            ui_state.gamepads = gamepad_snapshot;
        }

        // 3. Process Input
        let keyboard_busy = ctx.wants_keyboard_input();
        let mut pad_masks = [0u8; 4];
        let mut turbo_masks = [0u8; 4];
        if let Ok(mut ui_state) = self.ui_state.try_lock() {
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
                turbo_masks[port] = ctrl.turbo_mask();
            }
            self.last_pad_masks = pad_masks;
            self.last_turbo_masks = turbo_masks;
        } else {
            pad_masks = self.last_pad_masks;
            turbo_masks = self.last_turbo_masks;
        }

        // Always publish input state via atomics (no control channel, low latency).
        for port in 0..4 {
            self.runtime_handle.set_pad_mask(port, pad_masks[port]);
            self.runtime_handle.set_turbo_mask(port, turbo_masks[port]);
        }

        // 4. Handle Drag & Drop
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        if let Some(path) = dropped.iter().filter_map(|f| f.path.clone()).next_back() {
            if let Err(err) = self.load_rom(&path) {
                self.error_dialog = Some(match self.language() {
                    Language::English => format!("Load failed:\n{err}"),
                    Language::ChineseSimplified => format!("加载失败：\n{err}"),
                });
            }
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
        self.show_error_dialog(ctx);
    }
}
