use std::path::PathBuf;

use eframe::egui;
use egui::{Context as EguiContext, MenuBar, TextWrapMode};

use super::{AppViewport, Language, NesiumApp, TextId, dialogs::pick_file_dialog};

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
        let fullscreen = ctx.input(|i| i.viewport().fullscreen).unwrap_or(false);
        if fullscreen {
            let any_popup_open = egui::Popup::is_any_open(ctx);
            let base_height = ctx.style().spacing.interact_size.y
                + 2.0 * ctx.style().spacing.item_spacing.y
                + 2.0;
            let reveal_height = base_height + 24.0;
            let hover_at_top = ctx
                .input(|i| i.pointer.hover_pos())
                .is_some_and(|p| p.y <= reveal_height);

            if !hover_at_top && !any_popup_open {
                return None;
            }

            let content_rect = ctx.content_rect();
            egui::Area::new(egui::Id::new("menu_bar_overlay"))
                .order(egui::Order::Foreground)
                .movable(false)
                .interactable(true)
                .fixed_pos(content_rect.left_top())
                .show(ctx, |ui| {
                    ui.set_min_width(content_rect.width());
                    egui::Frame::NONE
                        .fill(ui.visuals().panel_fill)
                        .shadow(ui.visuals().popup_shadow)
                        .inner_margin(egui::Margin::symmetric(8, 4))
                        .show(ui, |ui| {
                            MenuBar::new().ui(ui, |ui| self.menu_contents(ui, &mut cmd));
                        });
                });
        } else {
            egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
                MenuBar::new().ui(ui, |ui| self.menu_contents(ui, &mut cmd));
            });
        }

        if let Some(mut command) = cmd.load_rom.take() {
            return Some(AppCommand {
                load_rom: Some(std::mem::take(&mut command)),
                ..cmd
            });
        }
        Some(cmd)
    }

    fn menu_contents(&mut self, ui: &mut egui::Ui, cmd: &mut AppCommand) {
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

            const WINDOW_TOGGLES: &[(AppViewport, TextId)] = &[
                (AppViewport::Debugger, TextId::MenuWindowDebugger),
                (AppViewport::Tools, TextId::MenuWindowTools),
                (AppViewport::Palette, TextId::MenuWindowPalette),
                (AppViewport::Input, TextId::MenuWindowInput),
                (AppViewport::Audio, TextId::MenuWindowAudio),
            ];

            for (viewport, label_id) in WINDOW_TOGGLES {
                let label = self.t(*label_id);
                ui.toggle_value(self.viewports.open_mut(*viewport), label);
            }
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
                self.viewports.set_open(AppViewport::About, true);
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
    }

    pub(super) fn handle_app_command(&mut self, ctx: &EguiContext, cmd: AppCommand) {
        if let Some(path) = cmd.load_rom {
            match self.load_rom(&path) {
                Ok(_) => {}
                Err(err) => {
                    self.error_dialog = Some(match self.language() {
                        Language::English => format!("Load failed:\n{err}"),
                        Language::ChineseSimplified => format!("加载失败：\n{err}"),
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
            self.runtime_handle.set_paused(self.paused);
        }
        if cmd.quit {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
}
