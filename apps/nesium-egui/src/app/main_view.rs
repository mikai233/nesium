use eframe::egui;
use egui::{Color32, Context as EguiContext, Vec2};
use nesium_core::ppu::{SCREEN_HEIGHT, SCREEN_WIDTH};

use super::NesiumApp;

const CURSOR_HIDE_DELAY: std::time::Duration = std::time::Duration::from_secs(2);

impl NesiumApp {
    pub(super) fn draw_main_view(&mut self, ctx: &EguiContext) {
        egui::CentralPanel::default()
            .frame(egui::Frame::central_panel(ctx.style().as_ref()).inner_margin(0))
            .show(ctx, |ui| {
                let canvas_size = ui.available_size();
                let (rect, response) = ui.allocate_exact_size(canvas_size, egui::Sense::hover());
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

                    // Auto-hide cursor when hovering over the game image without moving.
                    let pointer_over_image = response.hovered()
                        && ctx
                            .input(|i| i.pointer.hover_pos())
                            .is_some_and(|p| image_rect.contains(p));

                    if pointer_over_image {
                        let pointer_delta = ctx.input(|i| i.pointer.delta());
                        if pointer_delta != Vec2::ZERO {
                            self.cursor_last_activity = std::time::Instant::now();
                            self.cursor_hidden = false;
                        } else if !self.cursor_hidden
                            && self.cursor_last_activity.elapsed() >= CURSOR_HIDE_DELAY
                        {
                            self.cursor_hidden = true;
                        }
                    } else {
                        self.cursor_hidden = false;
                        self.cursor_last_activity = std::time::Instant::now();
                    }

                    if self.cursor_hidden {
                        ctx.output_mut(|o| o.cursor_icon = egui::CursorIcon::None);
                    }
                }
            });
    }
}
