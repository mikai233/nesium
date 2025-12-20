use std::path::PathBuf;

use eframe::egui;
use egui::{Context as EguiContext, MenuBar, TextWrapMode};

use crate::emulator_thread::Command;

use super::{Language, NesiumApp, TextId, dialogs::pick_file_dialog};

#[derive(Default)]
pub(super) struct AppCommand {
    pub load_rom: Option<PathBuf>,
    pub reset: bool,
    pub eject: bool,
    pub toggle_pause: bool,
    pub quit: bool,
}

impl NesiumApp {
    pub(super) fn draw_menu(&mut self, ctx: &EguiContext) -> Option<AppCommand> {
        let mut cmd = AppCommand::default();
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            MenuBar::new().ui(ui, |ui| {
                ui.menu_button(self.t(TextId::MenuFile), |ui| {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);

                    if ui.button(self.t(TextId::MenuFileLoadRom)).clicked() {
                        if let Some(path) = pick_file_dialog() {
                            cmd.load_rom = Some(path);
                        }
                        ui.close();
                    }
                    if ui
                        .add_enabled(
                            self.rom_path.is_some(),
                            egui::Button::new(self.t(TextId::MenuFileReset)),
                        )
                        .clicked()
                    {
                        cmd.reset = true;
                        ui.close();
                    }
                    if ui
                        .add_enabled(
                            self.rom_path.is_some(),
                            egui::Button::new(self.t(TextId::MenuFileEject)),
                        )
                        .clicked()
                    {
                        cmd.eject = true;
                        ui.close();
                    }
                    ui.separator();
                    if ui.button(self.t(TextId::MenuFileQuit)).clicked() {
                        cmd.quit = true;
                        ui.close();
                    }
                });

                ui.menu_button(self.t(TextId::MenuEmulation), |ui| {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);

                    if ui
                        .add_enabled(
                            self.rom_path.is_some(),
                            egui::Button::new(if self.paused {
                                self.t(TextId::MenuEmulationResume)
                            } else {
                                self.t(TextId::MenuEmulationPause)
                            }),
                        )
                        .clicked()
                    {
                        cmd.toggle_pause = true;
                        ui.close();
                    }
                    if ui
                        .add_enabled(
                            self.rom_path.is_some(),
                            egui::Button::new(self.t(TextId::MenuFileReset)),
                        )
                        .clicked()
                    {
                        cmd.reset = true;
                        ui.close();
                    }
                    if ui
                        .add_enabled(
                            self.rom_path.is_some(),
                            egui::Button::new(self.t(TextId::MenuFileEject)),
                        )
                        .clicked()
                    {
                        cmd.eject = true;
                        ui.close();
                    }
                });

                ui.menu_button(self.t(TextId::MenuWindow), |ui| {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);

                    let dbg_label = self.t(TextId::MenuWindowDebugger);
                    let tools_label = self.t(TextId::MenuWindowTools);
                    let palette_label = self.t(TextId::MenuWindowPalette);
                    let input_label = self.t(TextId::MenuWindowInput);
                    let audio_label = self.t(TextId::MenuWindowAudio);

                    ui.toggle_value(&mut self.show_debugger, dbg_label);
                    ui.toggle_value(&mut self.show_tools, tools_label);
                    ui.toggle_value(&mut self.show_palette, palette_label);
                    ui.toggle_value(&mut self.show_input, input_label);
                    ui.toggle_value(&mut self.show_audio, audio_label);
                });

                ui.menu_button(self.t(TextId::MenuLanguage), |ui| {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);

                    for language in Language::ALL {
                        let selected = self.language() == language;
                        let label = language.label();
                        if ui.selectable_label(selected, label).clicked() {
                            self.set_language(language);
                            ui.close();
                        }
                    }
                });

                ui.menu_button(self.t(TextId::MenuHelp), |ui| {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);

                    if ui.button(self.t(TextId::MenuHelpAbout)).clicked() {
                        self.show_about = true;
                        ui.close();
                    }
                    ui.separator();
                    ui.label(self.t(TextId::MenuHelpLine1));
                    ui.label(self.t(TextId::MenuHelpLine2));
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.has_rom() && !self.paused && self.fps > 0.0 {
                        ui.label(format!("FPS: {:.1}", self.fps));
                    } else {
                        ui.label("FPS: --");
                    }
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
                Err(err) => {
                    self.status_line = Some(match self.language() {
                        Language::English => format!("Load failed: {err}"),
                        Language::ChineseSimplified => format!("加载失败: {err}"),
                    });
                }
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
            self.emulator.send(Command::SetPaused(self.paused));
            self.status_line = Some(if self.paused {
                self.t(TextId::StatusPaused).to_string()
            } else {
                self.t(TextId::StatusResumed).to_string()
            });
        }
        if cmd.quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}
