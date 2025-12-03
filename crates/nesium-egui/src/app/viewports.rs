use eframe::egui;
use egui::{Color32, Context as EguiContext, Vec2, ViewportBuilder, ViewportId};
use nesium_core::controller::Button;

use super::{NesiumApp, controller};

impl NesiumApp {
    pub(super) fn show_viewports(&mut self, ctx: &EguiContext) {
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
                            controller::format_button_name(button),
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

        if self.show_audio {
            let builder = ViewportBuilder::default()
                .with_title("Audio")
                .with_inner_size([320.0, 320.0]);
            ctx.show_viewport_immediate(ViewportId::from_hash_of("audio"), builder, |ctx, _| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_audio = false;
                    return;
                }
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("音频设置");
                    ui.separator();

                    let cfg = &mut self.audio_cfg;

                    // Master volume
                    let mut vol_percent = (cfg.master_volume * 100.0).clamp(0.0, 100.0);
                    ui.horizontal(|ui| {
                        ui.label("主音量");
                        if ui
                            .add(egui::Slider::new(&mut vol_percent, 0.0..=100.0).suffix("%"))
                            .changed()
                        {
                            cfg.master_volume = (vol_percent / 100.0).clamp(0.0, 1.0);
                        }
                    });

                    ui.separator();
                    ui.label("后台 / 快进行为");
                    ui.checkbox(&mut cfg.mute_in_background, "后台静音");
                    ui.checkbox(&mut cfg.reduce_in_background, "后台降低音量");
                    ui.checkbox(&mut cfg.reduce_in_fast_forward, "快进时降低音量");
                    let mut red_percent = (cfg.volume_reduction * 100.0).clamp(0.0, 100.0);
                    if ui
                        .add(
                            egui::Slider::new(&mut red_percent, 0.0..=100.0)
                                .suffix("%")
                                .text("降低幅度"),
                        )
                        .changed()
                    {
                        cfg.volume_reduction = (red_percent / 100.0).clamp(0.0, 1.0);
                    }

                    ui.separator();
                    ui.collapsing("混响 (Reverb)", |ui| {
                        ui.checkbox(&mut cfg.reverb_enabled, "启用混响");
                        if cfg.reverb_enabled {
                            ui.add(
                                egui::Slider::new(&mut cfg.reverb_strength, 0.0..=1.0).text("强度"),
                            );
                            ui.add(
                                egui::Slider::new(&mut cfg.reverb_delay_ms, 1.0..=250.0)
                                    .text("延迟 (ms)"),
                            );
                        }
                    });

                    ui.collapsing("串音 (Crossfeed)", |ui| {
                        ui.checkbox(&mut cfg.crossfeed_enabled, "启用串音");
                        if cfg.crossfeed_enabled {
                            ui.add(
                                egui::Slider::new(&mut cfg.crossfeed_ratio, 0.0..=1.0).text("比率"),
                            );
                        }
                    });

                    ui.collapsing("均衡器 (EQ)", |ui| {
                        ui.checkbox(&mut cfg.enable_equalizer, "启用 EQ");
                        if cfg.enable_equalizer {
                            // 统一使用单一增益控制 20 个频段。
                            let mut gain_db = cfg.eq_band_gains[0];
                            if ui
                                .add(
                                    egui::Slider::new(&mut gain_db, -12.0..=12.0)
                                        .text("全局增益 (dB)"),
                                )
                                .changed()
                            {
                                for g in cfg.eq_band_gains.iter_mut() {
                                    *g = gain_db;
                                }
                            }
                        }
                    });

                    // Apply configuration to the core each frame where it might change.
                    self.nes.set_audio_bus_config(*cfg);
                });
            });
        }
    }
}
