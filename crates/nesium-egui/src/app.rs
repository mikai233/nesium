use std::{
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use eframe::egui;
use egui::{
    Color32, ColorImage, Context as EguiContext, FontData, FontDefinitions, FontFamily, MenuBar,
    TextureHandle, TextureOptions, Vec2, ViewportBuilder, ViewportId, Visuals,
};
use nesium_core::{
    CpuSnapshot, Nes,
    controller::Button,
    ppu::{SCREEN_HEIGHT, SCREEN_WIDTH, buffer::ColorFormat, palette::PaletteKind},
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
    paused: bool,
    status_line: Option<String>,
    show_debugger: bool,
    show_tools: bool,
    show_palette: bool,
    show_input: bool,
    controller: ControllerInput,
    next_frame_deadline: Option<Instant>,
}

impl NesiumApp {
    pub fn new(cc: &eframe::CreationContext<'_>, config: AppConfig) -> Self {
        cc.egui_ctx.set_visuals(Visuals::light());
        install_cjk_font(&cc.egui_ctx);

        let mut nes = Nes::new(ColorFormat::Rgba8888);
        nes.ppu
            .framebuffer
            .set_palette(PaletteKind::RawLinear.palette());

        let mut app = Self {
            nes,
            frame_texture: None,
            rom_path: None,
            start_pc: config.start_pc,
            paused: false,
            status_line: None,
            show_debugger: false,
            show_tools: false,
            show_palette: false,
            show_input: false,
            controller: ControllerInput::default(),
            next_frame_deadline: None,
        };

        if let Some(path) = config.rom_path {
            if let Err(err) = app.load_rom(&path) {
                app.status_line = Some(format!("加载 ROM 失败: {err}"));
            }
        }

        app
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
        self.status_line = Some(format!("已加载 {}", path.display()));
        Ok(())
    }

    fn reset(&mut self) {
        self.nes.reset();
        self.paused = false;
        self.status_line = Some("已重置主机".to_string());
        self.controller.release_all(&mut self.nes);
    }

    fn eject(&mut self) {
        self.nes.eject_cartridge();
        self.rom_path = None;
        self.status_line = Some("已弹出卡带".to_string());
        self.controller.release_all(&mut self.nes);
    }

    fn update_frame_texture(&mut self, ctx: &EguiContext) {
        let frame = self.nes.render_buffer();
        if frame.is_empty() {
            return;
        }

        let image = ColorImage::from_rgba_unmultiplied(
            [SCREEN_WIDTH as usize, SCREEN_HEIGHT as usize],
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

    fn draw_menu(&mut self, ctx: &EguiContext) -> Option<AppCommand> {
        let mut cmd = AppCommand::default();
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ui.menu_button("文件", |ui| {
                    if ui.button("加载 ROM…").clicked() {
                        if let Some(path) = pick_file_dialog() {
                            cmd.load_rom = Some(path);
                        }
                        ui.close();
                    }
                    if ui
                        .add_enabled(self.has_rom(), egui::Button::new("重置"))
                        .clicked()
                    {
                        cmd.reset = true;
                        ui.close();
                    }
                    if ui
                        .add_enabled(self.has_rom(), egui::Button::new("弹出"))
                        .clicked()
                    {
                        cmd.eject = true;
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("退出").clicked() {
                        cmd.quit = true;
                        ui.close();
                    }
                });

                ui.menu_button("仿真", |ui| {
                    if ui
                        .add_enabled(
                            self.has_rom(),
                            egui::Button::new(if self.paused { "继续" } else { "暂停" }),
                        )
                        .clicked()
                    {
                        cmd.toggle_pause = true;
                        ui.close();
                    }
                    if ui
                        .add_enabled(self.has_rom(), egui::Button::new("重置"))
                        .clicked()
                    {
                        cmd.reset = true;
                        ui.close();
                    }
                });

                ui.menu_button("窗口", |ui| {
                    ui.toggle_value(&mut self.show_debugger, "Debugger");
                    ui.toggle_value(&mut self.show_tools, "Tools");
                    ui.toggle_value(&mut self.show_palette, "Palette");
                    ui.toggle_value(&mut self.show_input, "Input");
                });

                ui.menu_button("帮助", |ui| {
                    ui.label("Mesen2 风格，eframe + egui 前端");
                    ui.label("拖拽 .nes/.fds 或使用 文件 → 加载 ROM");
                });
            });
        });

        if let Some(mut command) = cmd.load_rom.take() {
            return Some(AppCommand {
                load_rom: Some(std::mem::take(&mut command)),
                ..cmd
            });
        }
        Some(cmd)
    }

    fn draw_main_view(&mut self, ctx: &EguiContext) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(status) = &self.status_line {
                ui.label(status);
            } else if let Some(path) = &self.rom_path {
                ui.label(format!("已加载：{}", path.display()));
            } else {
                ui.label("未加载 ROM");
            }

            ui.separator();

            egui::Frame::canvas(ui.style()).show(ui, |ui| {
                ui.set_min_size(Vec2::new(
                    SCREEN_WIDTH as f32 * 2.0,
                    SCREEN_HEIGHT as f32 * 2.0,
                ));
                ui.centered_and_justified(|ui| {
                    if let Some(tex) = &self.frame_texture {
                        let available = ui.available_size();
                        let base = Vec2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32);
                        let scale = (available.x / base.x).min(available.y / base.y).max(1.0);
                        let desired = base * scale;
                        ui.add(egui::Image::from_texture(tex).fit_to_exact_size(desired));
                    } else {
                        ui.colored_label(Color32::DARK_GRAY, "等待首帧…");
                    }
                });
            });
        });
    }

    fn show_viewports(&mut self, ctx: &EguiContext) {
        if self.show_debugger {
            let builder = ViewportBuilder::default()
                .with_title("Debugger")
                .with_inner_size([420.0, 320.0]);
            ctx.show_viewport_immediate(ViewportId::from_hash_of("debugger"), builder, |ctx, _| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_debugger = false;
                    return;
                }
                egui::CentralPanel::default().show(ctx, |ui| {
                    let snapshot = self.nes.cpu_snapshot();
                    ui.heading("CPU Snapshot");
                    ui.monospace(format!(
                        "PC:{:04X}  A:{:02X}  X:{:02X}  Y:{:02X}  P:{:02X}  S:{:02X}",
                        snapshot.pc, snapshot.a, snapshot.x, snapshot.y, snapshot.p, snapshot.s
                    ));
                    ui.separator();
                    ui.label(format!("PPU Frame: {}", self.nes.ppu.frame_count()));
                    ui.label(format!("Dot Counter: {}", self.nes.dot_counter()));
                });
            });
        }

        if self.show_tools {
            let builder = ViewportBuilder::default()
                .with_title("Tools")
                .with_inner_size([360.0, 260.0]);
            ctx.show_viewport_immediate(ViewportId::from_hash_of("tools"), builder, |ctx, _| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_tools = false;
                    return;
                }
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("工具箱");
                    ui.label("在此添加保存状态、断点等工具逻辑。");
                });
            });
        }

        if self.show_palette {
            let builder = ViewportBuilder::default()
                .with_title("Palette")
                .with_inner_size([280.0, 240.0]);
            ctx.show_viewport_immediate(ViewportId::from_hash_of("palette"), builder, |ctx, _| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_palette = false;
                    return;
                }
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("当前调色板 (前 16 项)");
                    let palette = self.nes.palette().as_colors();
                    for (idx, color) in palette.iter().take(16).enumerate() {
                        let swatch = egui::Color32::from_rgb(color.r, color.g, color.b);
                        ui.horizontal(|ui| {
                            let (rect, _) =
                                ui.allocate_exact_size(Vec2::splat(18.0), egui::Sense::hover());
                            ui.painter().rect_filled(rect, 2.0, swatch);
                            ui.label(format!(
                                "{idx:02}: #{:02X}{:02X}{:02X}",
                                color.r, color.g, color.b
                            ));
                        });
                    }
                });
            });
        }

        if self.show_input {
            let builder = ViewportBuilder::default()
                .with_title("Input")
                .with_inner_size([260.0, 220.0]);
            ctx.show_viewport_immediate(ViewportId::from_hash_of("input"), builder, |ctx, _| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_input = false;
                    return;
                }
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("输入状态");
                    ui.label("键盘 -> 手柄 1");
                    ui.separator();
                    for button in [
                        Button::Up,
                        Button::Down,
                        Button::Left,
                        Button::Right,
                        Button::A,
                        Button::B,
                        Button::Select,
                        Button::Start,
                    ] {
                        let active = self.controller.is_pressed(button);
                        let label = format!(
                            "{:>6}: {}",
                            format_button_name(button),
                            if active { "ON" } else { "off" }
                        );
                        if active {
                            ui.colored_label(Color32::GREEN, label);
                        } else {
                            ui.label(label);
                        }
                    }
                });
            });
        }
    }
}

impl eframe::App for NesiumApp {
    fn update(&mut self, ctx: &EguiContext, _: &mut eframe::Frame) {
        // Drive UI every loop; emulator step pacing is handled below.
        ctx.request_repaint();

        let keyboard_blocked = ctx.wants_keyboard_input();
        self.controller
            .sync_from_input(ctx, &mut self.nes, keyboard_blocked);

        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        if let Some(path) = dropped.iter().filter_map(|f| f.path.clone()).last() {
            let _ = self.load_rom(&path);
        }

        // Fixed-step schedule: run frames when deadline passes; allow small catch-up.
        let now = Instant::now();
        let mut deadline = self
            .next_frame_deadline
            .unwrap_or_else(|| now + TARGET_FRAME);
        let mut run_count = 0u32;
        while now >= deadline && run_count < 3 {
            if self.has_rom() && !self.paused {
                self.nes.run_frame();
            }
            deadline += TARGET_FRAME;
            run_count += 1;
        }
        self.next_frame_deadline = Some(deadline);

        if let Some(next) = self.next_frame_deadline {
            let wait = next.saturating_duration_since(Instant::now());
            ctx.request_repaint_after(wait.min(TARGET_FRAME));
        }
        self.update_frame_texture(ctx);

        if let Some(cmd) = self.draw_menu(ctx) {
            if let Some(path) = cmd.load_rom {
                match self.load_rom(&path) {
                    Ok(_) => {}
                    Err(err) => self.status_line = Some(format!("加载失败: {err}")),
                }
            }
            if cmd.reset {
                self.reset();
            }
            if cmd.eject {
                self.eject();
            }
            if cmd.toggle_pause {
                self.paused = !self.paused;
                self.status_line = Some(if self.paused {
                    "已暂停".to_string()
                } else {
                    "已继续".to_string()
                });
            }
            if cmd.quit {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        }

        self.draw_main_view(ctx);
        self.show_viewports(ctx);
    }
}

#[derive(Default)]
struct AppCommand {
    load_rom: Option<PathBuf>,
    reset: bool,
    eject: bool,
    toggle_pause: bool,
    quit: bool,
}

#[derive(Default)]
struct ControllerInput {
    pressed: Vec<Button>,
}

impl ControllerInput {
    fn sync_from_input(&mut self, ctx: &EguiContext, nes: &mut Nes, keyboard_blocked: bool) {
        let keys = ctx.input(|i| i.keys_down.clone());
        let mut desired: Vec<Button> = Vec::new();

        if !keyboard_blocked {
            for key in keys {
                if let Some(button) = map_key(key) {
                    if !desired.contains(&button) {
                        desired.push(button);
                    }
                }
            }
        }

        // Release all, then re-apply desired. Simple and keeps in sync.
        for button in self.pressed.drain(..) {
            nes.set_button(0, button, false);
        }
        for &button in &desired {
            nes.set_button(0, button, true);
        }
        self.pressed = desired;
    }

    fn release_all(&mut self, nes: &mut Nes) {
        for button in self.pressed.drain(..) {
            nes.set_button(0, button, false);
        }
    }

    fn is_pressed(&self, button: Button) -> bool {
        self.pressed.iter().any(|b| *b == button)
    }
}

fn map_key(key: egui::Key) -> Option<Button> {
    match key {
        egui::Key::Z => Some(Button::A),
        egui::Key::X => Some(Button::B),
        egui::Key::Enter => Some(Button::Start),
        egui::Key::Space | egui::Key::C => Some(Button::Select),
        egui::Key::ArrowUp => Some(Button::Up),
        egui::Key::ArrowDown => Some(Button::Down),
        egui::Key::ArrowLeft => Some(Button::Left),
        egui::Key::ArrowRight => Some(Button::Right),
        _ => None,
    }
}

fn format_button_name(button: Button) -> &'static str {
    match button {
        Button::A => "A",
        Button::B => "B",
        Button::Select => "Select",
        Button::Start => "Start",
        Button::Up => "Up",
        Button::Down => "Down",
        Button::Left => "Left",
        Button::Right => "Right",
    }
}

fn pick_file_dialog() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("NES ROM", &["nes", "fds"])
        .pick_file()
}

fn install_cjk_font(ctx: &EguiContext) {
    let mut db = fontdb::Database::new();
    db.load_system_fonts();

    let target_chars = ['你', '汉', '测', '试'];
    let mut picked: Option<Vec<u8>> = None;

    for face in db.faces() {
        let has_all = db.with_face_data(face.id, |data, idx| {
            let face = match ttf_parser::Face::parse(data, idx) {
                Ok(f) => f,
                Err(_) => return false,
            };
            target_chars
                .iter()
                .all(|ch| face.glyph_index(*ch).is_some())
        });
        if has_all == Some(true) {
            if let Some(bytes) = db.with_face_data(face.id, |data, _| data.to_vec()) {
                picked = Some(bytes);
                break;
            }
        }
    }

    if let Some(data) = picked {
        let mut fonts = FontDefinitions::default();
        fonts
            .font_data
            .insert("ui_cjk".to_string(), FontData::from_owned(data).into());
        fonts
            .families
            .entry(FontFamily::Proportional)
            .or_default()
            .insert(0, "ui_cjk".to_string());
        fonts
            .families
            .entry(FontFamily::Monospace)
            .or_default()
            .push("ui_cjk".to_string());
        ctx.set_fonts(fonts);
    }
}
