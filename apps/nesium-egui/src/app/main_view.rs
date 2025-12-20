use eframe::egui;
use egui::{Color32, Context as EguiContext, Vec2};
use nesium_core::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};

use super::{NesiumApp, TextId};

impl NesiumApp {
    pub(super) fn draw_main_view(&mut self, ctx: &EguiContext) {
        let fullscreen = ctx.input(|i| i.viewport().fullscreen).unwrap_or(false);

        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(ctx.style().as_ref()).inner_margin(0))
            .show(ctx, |ui| {
                if !fullscreen {
                    egui::Frame::NONE.inner_margin(8).show(ui, |ui| {
                        if let Some(status) = &self.status_line {
                            ui.label(status);
                        } else if let Some(path) = &self.rom_path {
                            let text = match self.language() {
                                super::Language::English => {
                                    format!("Loaded: {}", path.display())
                                }
                                super::Language::ChineseSimplified => {
                                    format!("已加载：{}", path.display())
                                }
                            };
                            ui.label(text);
                        } else {
                            ui.label(self.t(TextId::MainNoRom));
                        }

                        ui.separator();
                    });
                }

                let canvas_size = ui.available_size();
                let (rect, _) = ui.allocate_exact_size(canvas_size, egui::Sense::hover());
                ui.painter().rect_filled(rect, 0.0, Color32::BLACK);

                let base = Vec2::new(SCREEN_WIDTH as f32, SCREEN_HEIGHT as f32);
                let desired_inset: f32 = 12.0;
                let max_inset = (rect.size().min_elem() * 0.5).max(0.0);
                let inset = desired_inset.min(max_inset);
                let inner_rect = rect.shrink(inset);

                if let Some(tex) = &self.frame_texture {
                    let desired_size = if self.pixel_perfect_scaling() {
                        // Pixel-perfect scaling must be computed in physical pixels.
                        // If we only floor the scale in egui "points", fractional DPI scaling
                        // (e.g. 125%) can still produce non-integer pixel mapping and shimmer.
                        let ppp = ctx.pixels_per_point();
                        let available_px = inner_rect.size() * ppp;
                        let scale_px = (available_px.x / base.x)
                            .min(available_px.y / base.y)
                            .floor()
                            .max(1.0);
                        Vec2::new(base.x * scale_px / ppp, base.y * scale_px / ppp)
                    } else {
                        let available = inner_rect.size();
                        let scale = (available.x / base.x).min(available.y / base.y).max(1.0);
                        base * scale
                    };

                    let mut image_rect =
                        egui::Rect::from_center_size(inner_rect.center(), desired_size);
                    if self.pixel_perfect_scaling() {
                        let ppp = ctx.pixels_per_point();
                        let min_x = (image_rect.min.x * ppp).round() / ppp;
                        let min_y = (image_rect.min.y * ppp).round() / ppp;
                        image_rect =
                            egui::Rect::from_min_size(egui::pos2(min_x, min_y), desired_size);
                    }

                    ui.painter().image(
                        tex.id(),
                        image_rect,
                        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }
            });
    }
}
