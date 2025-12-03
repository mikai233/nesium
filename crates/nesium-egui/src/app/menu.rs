use std::path::PathBuf;

use eframe::egui;
use egui::{Context as EguiContext, MenuBar};

use super::{
    NesiumApp,
    dialogs::{pick_file_dialog, save_wav_dialog, write_wav},
};

#[derive(Default)]
pub(super) struct AppCommand {
    pub load_rom: Option<PathBuf>,
    pub reset: bool,
    pub eject: bool,
    pub toggle_pause: bool,
    pub start_record: bool,
    pub stop_record: bool,
    pub quit: bool,
}

impl NesiumApp {
    pub(super) fn draw_menu(&mut self, ctx: &EguiContext) -> Option<AppCommand> {
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
                        .add_enabled(self.rom_path.is_some(), egui::Button::new("重置"))
                        .clicked()
                    {
                        cmd.reset = true;
                        ui.close();
                    }
                    if ui
                        .add_enabled(self.rom_path.is_some(), egui::Button::new("弹出"))
                        .clicked()
                    {
                        cmd.eject = true;
                        ui.close();
                    }
                    ui.separator();
                    let rec_label = if self.recording {
                        "停止录制 WAV"
                    } else {
                        "开始录制 WAV…"
                    };
                    if ui
                        .add_enabled(self.rom_path.is_some(), egui::Button::new(rec_label))
                        .clicked()
                    {
                        if self.recording {
                            cmd.stop_record = true;
                        } else {
                            cmd.start_record = true;
                        }
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
                            self.rom_path.is_some(),
                            egui::Button::new(if self.paused { "继续" } else { "暂停" }),
                        )
                        .clicked()
                    {
                        cmd.toggle_pause = true;
                        ui.close();
                    }
                    if ui
                        .add_enabled(self.rom_path.is_some(), egui::Button::new("重置"))
                        .clicked()
                    {
                        cmd.reset = true;
                        ui.close();
                    }
                    if ui
                        .add_enabled(self.rom_path.is_some(), egui::Button::new("弹出"))
                        .clicked()
                    {
                        cmd.eject = true;
                        ui.close();
                    }
                });

                ui.menu_button("窗口", |ui| {
                    ui.toggle_value(&mut self.show_debugger, "Debugger");
                    ui.toggle_value(&mut self.show_tools, "Tools");
                    ui.toggle_value(&mut self.show_palette, "Palette");
                    ui.toggle_value(&mut self.show_input, "Input");
                    ui.toggle_value(&mut self.show_audio, "Audio");
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

    pub(super) fn handle_app_command(&mut self, ctx: &EguiContext, cmd: AppCommand) {
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
        if cmd.start_record {
            if let Some(path) = save_wav_dialog() {
                self.record_buffer.clear();
                self.record_sample_rate = self
                    .audio
                    .as_ref()
                    .map(|a| a.sample_rate())
                    .unwrap_or_else(|| self.nes.audio_sample_rate());
                self.record_path = Some(path.clone());
                self.recording = true;
                self.status_line = Some(format!("开始录制音频到 {}", path.display()));
            }
        }
        if cmd.stop_record {
            if self.recording {
                self.recording = false;
                if let Some(path) = self.record_path.take() {
                    match write_wav(&path, self.record_sample_rate, &self.record_buffer) {
                        Ok(()) => {
                            self.status_line = Some(format!("已保存录音到 {}", path.display()));
                        }
                        Err(err) => {
                            self.status_line = Some(format!("保存录音失败: {err}"));
                        }
                    }
                    self.record_buffer.clear();
                }
            }
        }
        if cmd.quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}
