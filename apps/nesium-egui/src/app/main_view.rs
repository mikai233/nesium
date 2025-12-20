use eframe::egui;
use egui::{Color32, Context as EguiContext, Vec2};
use nesium_core::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};

use super::{NesiumApp, TextId};

impl NesiumApp {
    pub(super) fn draw_main_view(&mut self, ctx: &EguiContext) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(status) = &self.status_line {
                ui.label(status);
            } else if let Some(path) = &self.rom_path {
                let text = match self.language() {
                    super::Language::English => format!("Loaded: {}", path.display()),
                    super::Language::ChineseSimplified => {
                        format!("已加载：{}", path.display())
                    }
                };
                ui.label(text);
            } else {
                ui.label(self.t(TextId::MainNoRom));
            }

            ui.separator();

            // Black canvas behind the game texture for better immersion.
            egui::Frame::canvas(ui.style())
                .fill(Color32::BLACK)
                .show(ui, |ui| {
                    ui.set_min_size(Vec2::new(
                        SCREEN_WIDTH as f32 * 2.0,
                        SCREEN_HEIGHT as f32 * 2.0,
                    ));
                    ui.centered_and_justified(|ui| {
                        if let Some(tex) = &self.frame_texture {
                            let available = ui.available_size();
                            let base = Vec2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32);
                            let mut scale = (available.x / base.x).min(available.y / base.y);
                            if self.pixel_perfect_scaling {
                                // Nearest + non-integer scaling makes 1px scroll steps look uneven.
                                scale = scale.floor();
                            }
                            let scale = scale.max(1.0);
                            let desired = base * scale;
                            ui.add(egui::Image::from_texture(tex).fit_to_exact_size(desired));
                        } else {
                            ui.colored_label(
                                Color32::DARK_GRAY,
                                self.t(TextId::MainWaitingFirstFrame),
                            );
                        }
                    });
                });
        });
    }
}
