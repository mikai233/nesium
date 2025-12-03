use eframe::egui;
use egui::{Color32, Context as EguiContext, Vec2, ViewportBuilder, ViewportId};
use gilrs::Button as GilrsButton;
use nesium_core::controller::Button;

use super::{
    NesiumApp, controller,
    controller::{ControllerDevice, InputPreset},
};

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
                .with_inner_size([420.0, 300.0]);
            ctx.show_viewport_immediate(ViewportId::from_hash_of("input"), builder, |ctx, _| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_input = false;
                    return;
                }
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading("输入配置");

                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label("控制器端口:");
                                for port in 0..4 {
                                    let label = format!("端口 {}", port + 1);
                                    ui.selectable_value(&mut self.active_input_port, port, label);
                                }
                            });

                            ui.separator();
                            let port = self.active_input_port.min(3);

                            ui.horizontal(|ui| {
                                ui.label(format!("端口 {} 设备:", port + 1));
                                let dev = &mut self.controller_devices[port];
                                ui.selectable_value(dev, ControllerDevice::Keyboard, "键盘");
                                if let Some(manager) = &self.gamepads {
                                    let gamepads = manager.gamepads();
                                    for (id, name) in &gamepads {
                                        let label = format!("手柄: {}", name);
                                        ui.selectable_value(
                                            dev,
                                            ControllerDevice::Gamepad(*id),
                                            label,
                                        );
                                    }
                                    if gamepads.is_empty() {
                                        ui.label("无手柄连接");
                                    }
                                } else {
                                    ui.label("手柄不可用");
                                }
                                ui.selectable_value(dev, ControllerDevice::Disabled, "禁用");
                            });

                            if port >= 2 {
                                ui.colored_label(
                                    Color32::DARK_GRAY,
                                    "注意：端口 3 和 4 当前尚未接入 NES 核心，仅用于预配置映射。",
                                );
                            }

                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label("预设:");
                                let preset = &mut self.controller_presets[port];
                                egui::ComboBox::from_id_salt(format!("input_preset_combo_{port}"))
                                    .selected_text(match preset {
                                        InputPreset::NesStandard => "NES 标准手柄",
                                        InputPreset::FightStick => "Fight Stick",
                                        InputPreset::ArcadeLayout => "Arcade Layout",
                                    })
                                    .show_ui(ui, |ui| {
                                        if ui
                                            .selectable_value(
                                                preset,
                                                InputPreset::NesStandard,
                                                "NES 标准手柄",
                                            )
                                            .clicked()
                                        {
                                            self.controllers[port]
                                                .apply_preset(InputPreset::NesStandard);
                                        }
                                        if ui
                                            .selectable_value(
                                                preset,
                                                InputPreset::FightStick,
                                                "Fight Stick",
                                            )
                                            .clicked()
                                        {
                                            self.controllers[port]
                                                .apply_preset(InputPreset::FightStick);
                                        }
                                        if ui
                                            .selectable_value(
                                                preset,
                                                InputPreset::ArcadeLayout,
                                                "Arcade Layout",
                                            )
                                            .clicked()
                                        {
                                            self.controllers[port]
                                                .apply_preset(InputPreset::ArcadeLayout);
                                        }
                                    });
                            });

                            ui.separator();
                            ui.label("键盘映射 → NES 手柄");
                            ui.small("点击“绑定”后按一个键，Esc 清除绑定；右下角“恢复默认”可还原出厂配置。");

                            ui.separator();
                            egui::Grid::new("input_mapping_grid")
                                .num_columns(4)
                                .spacing([12.0, 4.0])
                                .striped(true)
                                .show(ui, |ui| {
                                    ui.strong("类别");
                                    ui.strong("按钮");
                                    ui.strong("当前键位");
                                    ui.strong("操作");
                                    ui.end_row();

                                    let mapping_rows: &[(&str, Button)] = &[
                                        ("方向", Button::Up),
                                        ("方向", Button::Down),
                                        ("方向", Button::Left),
                                        ("方向", Button::Right),
                                        ("动作", Button::A),
                                        ("动作", Button::B),
                                        ("系统", Button::Select),
                                        ("系统", Button::Start),
                                    ];

                                    for (category, button) in mapping_rows {
                                        let name = controller::format_button_name(*button);
                                        let ctrl = &mut self.controllers[port];
                                        let bound = ctrl.binding_for(*button);
                                        let is_capturing =
                                            ctrl.capture_target() == Some(*button);

                                        ui.label(*category);
                                        ui.label(name);

                                        if is_capturing {
                                            ui.colored_label(
                                                Color32::LIGHT_YELLOW,
                                                "按任意键...",
                                            );
                                        } else if let Some(key) = bound {
                                            ui.monospace(format!("{key:?}"));
                                        } else {
                                            ui.colored_label(
                                                Color32::DARK_GRAY,
                                                "未绑定",
                                            );
                                        }

                                        let button_label =
                                            if is_capturing { "取消" } else { "绑定" };
                                        if ui.button(button_label).clicked() {
                                            if is_capturing {
                                                ctrl.clear_capture();
                                            } else {
                                                ctrl.begin_capture(*button);
                                            }
                                        }

                                        ui.end_row();
                                    }
                                });

                            ui.horizontal(|ui| {
                                ui.label("当前按下的按钮:");
                                ui.horizontal_wrapped(|ui| {
                                    let ctrl = &self.controllers[port];
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
                                        if ctrl.is_pressed(button) {
                                            ui.colored_label(
                                                Color32::LIGHT_GREEN,
                                                controller::format_button_name(button),
                                            );
                                        }
                                    }
                                });
                            });

                            ui.separator();
                            ui.collapsing("手柄映射", |ui| {
                                ui.label("NES 按钮 → 手柄按键");
                                let all_buttons: &[GilrsButton] = &[
                                    GilrsButton::South,
                                    GilrsButton::East,
                                    GilrsButton::West,
                                    GilrsButton::North,
                                    GilrsButton::LeftTrigger,
                                    GilrsButton::RightTrigger,
                                    GilrsButton::LeftTrigger2,
                                    GilrsButton::RightTrigger2,
                                    GilrsButton::DPadUp,
                                    GilrsButton::DPadDown,
                                    GilrsButton::DPadLeft,
                                    GilrsButton::DPadRight,
                                    GilrsButton::Start,
                                    GilrsButton::Select,
                                ];

                                egui::Grid::new("gamepad_mapping_grid")
                                    .num_columns(3)
                                    .spacing([12.0, 4.0])
                                    .striped(true)
                                    .show(ui, |ui| {
                                        ui.strong("类别");
                                        ui.strong("按钮");
                                        ui.strong("手柄按键");
                                        ui.end_row();

                                        let mapping_rows: &[(&str, Button)] = &[
                                            ("方向", Button::Up),
                                            ("方向", Button::Down),
                                            ("方向", Button::Left),
                                            ("方向", Button::Right),
                                            ("动作", Button::A),
                                            ("动作", Button::B),
                                            ("系统", Button::Select),
                                            ("系统", Button::Start),
                                        ];

                                        for (category, button) in mapping_rows {
                                            let name = controller::format_button_name(*button);
                                            let ctrl = &mut self.controllers[port];
                                            let mut binding =
                                                ctrl.gamepad_binding_for(*button);

                                            ui.label(*category);
                                            ui.label(name);

                                            let current_label = binding
                                                .map(|b| format!("{b:?}"))
                                                .unwrap_or_else(|| "未绑定".to_string());

                                            egui::ComboBox::from_id_salt(format!(
                                                "gp_map_{port}_{name}"
                                            ))
                                            .selected_text(current_label)
                                            .show_ui(ui, |ui| {
                                                if ui
                                                    .selectable_label(
                                                        binding.is_none(),
                                                        "未绑定",
                                                    )
                                                    .clicked()
                                                {
                                                    binding = None;
                                                }
                                                for btn in all_buttons {
                                                    let label = format!("{btn:?}");
                                                    let selected =
                                                        binding == Some(*btn);
                                                    if ui
                                                        .selectable_label(
                                                            selected,
                                                            label,
                                                        )
                                                        .clicked()
                                                    {
                                                        binding = Some(*btn);
                                                    }
                                                }
                                            });

                                            ctrl.set_gamepad_binding(*button, binding);
                                            ui.end_row();
                                        }
                                    });

                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.button("恢复默认").clicked() {
                                            self.controllers[port] =
                                                controller::ControllerInput::new_with_defaults();
                                        }
                                    },
                                );
                            });
                        });
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
