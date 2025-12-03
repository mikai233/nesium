use eframe::egui;
use egui::{Color32, Context as EguiContext, Vec2, ViewportBuilder, ViewportId};
use gilrs::Button as GilrsButton;
use nesium_core::controller::Button;

use super::{
    NesiumApp, TextId, controller,
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
                .with_title(self.t(TextId::MenuWindowTools))
                .with_inner_size([360.0, 260.0]);
            ctx.show_viewport_immediate(ViewportId::from_hash_of("tools"), builder, |ctx, _| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_tools = false;
                    return;
                }
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading(self.t(TextId::ToolsHeading));
                    ui.label(self.t(TextId::ToolsPlaceholder));
                });
            });
        }

        if self.show_palette {
            let builder = ViewportBuilder::default()
                .with_title(self.t(TextId::MenuWindowPalette))
                .with_inner_size([280.0, 240.0]);
            ctx.show_viewport_immediate(ViewportId::from_hash_of("palette"), builder, |ctx, _| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_palette = false;
                    return;
                }
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading(self.t(TextId::PaletteHeading));
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
                .with_title(self.t(TextId::MenuWindowInput))
                .with_inner_size([420.0, 300.0]);
            ctx.show_viewport_immediate(ViewportId::from_hash_of("input"), builder, |ctx, _| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_input = false;
                    return;
                }
                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.heading(self.t(TextId::InputHeading));

                    egui::ScrollArea::vertical()
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.label(self.t(TextId::InputControllerPortsLabel));
                                for port in 0..4 {
                                    let label = match self.language() {
                                        super::Language::English => {
                                            format!("Port {}", port + 1)
                                        }
                                        super::Language::ChineseSimplified => {
                                            format!("端口 {}", port + 1)
                                        }
                                    };
                                    ui.selectable_value(&mut self.active_input_port, port, label);
                                }
                            });

                            ui.separator();
                            let port = self.active_input_port.min(3);

                            ui.horizontal(|ui| {
                                let language = self.language();
                                let device_label = match language {
                                    super::Language::English => {
                                        format!("Port {} device:", port + 1)
                                    }
                                    super::Language::ChineseSimplified => {
                                        format!("端口 {} 设备:", port + 1)
                                    }
                                };
                                let keyboard_label = self.t(TextId::InputDeviceKeyboard);
                                let disabled_label = self.t(TextId::InputDeviceDisabled);
                                let no_gamepads_label = self.t(TextId::InputNoGamepads);
                                let gamepad_unavailable_label =
                                    self.t(TextId::InputGamepadUnavailable);

                                ui.label(device_label);
                                let dev = &mut self.controller_devices[port];
                                ui.selectable_value(
                                    dev,
                                    ControllerDevice::Keyboard,
                                    keyboard_label,
                                );
                                if let Some(manager) = &self.gamepads {
                                    let gamepads = manager.gamepads();
                                    for (id, name) in &gamepads {
                                        let label = match language {
                                            super::Language::English => {
                                                format!("Gamepad: {}", name)
                                            }
                                            super::Language::ChineseSimplified => {
                                                format!("手柄: {}", name)
                                            }
                                        };
                                        ui.selectable_value(
                                            dev,
                                            ControllerDevice::Gamepad(*id),
                                            label,
                                        );
                                    }
                                    if gamepads.is_empty() {
                                        ui.label(no_gamepads_label);
                                    }
                                } else {
                                    ui.label(gamepad_unavailable_label);
                                }
                                ui.selectable_value(
                                    dev,
                                    ControllerDevice::Disabled,
                                    disabled_label,
                                );
                            });

                            if port >= 2 {
                                ui.colored_label(
                                    Color32::DARK_GRAY,
                                    self.t(TextId::InputPort34Notice),
                                );
                            }

                            ui.separator();
                            ui.horizontal(|ui| {
                                let preset_label = self.t(TextId::InputPresetLabel);
                                let nes_label = self.t(TextId::InputPresetNesStandard);
                                let fight_label = self.t(TextId::InputPresetFightStick);
                                let arcade_label = self.t(TextId::InputPresetArcadeLayout);

                                ui.label(preset_label);
                                let mut preset_value = self.controller_presets[port];
                                egui::ComboBox::from_id_salt(format!("input_preset_combo_{port}"))
                                    .selected_text(match preset_value {
                                        InputPreset::NesStandard => nes_label,
                                        InputPreset::FightStick => fight_label,
                                        InputPreset::ArcadeLayout => arcade_label,
                                    })
                                    .show_ui(ui, |ui| {
                                        if ui
                                            .selectable_value(
                                                &mut preset_value,
                                                InputPreset::NesStandard,
                                                nes_label,
                                            )
                                            .clicked()
                                        {
                                            self.controllers[port]
                                                .apply_preset(InputPreset::NesStandard);
                                        }
                                        if ui
                                            .selectable_value(
                                                &mut preset_value,
                                                InputPreset::FightStick,
                                                fight_label,
                                            )
                                            .clicked()
                                        {
                                            self.controllers[port]
                                                .apply_preset(InputPreset::FightStick);
                                        }
                                        if ui
                                            .selectable_value(
                                                &mut preset_value,
                                                InputPreset::ArcadeLayout,
                                                arcade_label,
                                            )
                                            .clicked()
                                        {
                                            self.controllers[port]
                                                .apply_preset(InputPreset::ArcadeLayout);
                                        }
                                    });
                                self.controller_presets[port] = preset_value;
                            });

                            ui.separator();
                            ui.label(self.t(TextId::InputKeyboardMappingTitle));
                            ui.small(self.t(TextId::InputKeyboardMappingHelp));

                            ui.separator();
                            egui::Grid::new("input_mapping_grid")
                                .num_columns(4)
                                .spacing([12.0, 4.0])
                                .striped(true)
                                .show(ui, |ui| {
                                    let header_category = self.t(TextId::InputGridHeaderCategory);
                                    let header_button = self.t(TextId::InputGridHeaderButton);
                                    let header_current = self.t(TextId::InputGridHeaderCurrentKey);
                                    let header_action = self.t(TextId::InputGridHeaderAction);
                                    let prompt_label = self.t(TextId::InputPromptPressAnyKey);
                                    let not_bound_label = self.t(TextId::InputNotBound);
                                    let cancel_label = self.t(TextId::InputCancelButton);
                                    let bind_label = self.t(TextId::InputBindButton);

                                    ui.strong(header_category);
                                    ui.strong(header_button);
                                    ui.strong(header_current);
                                    ui.strong(header_action);
                                    ui.end_row();

                                    let mapping_rows: &[(TextId, Button)] = &[
                                        (TextId::InputCategoryDirection, Button::Up),
                                        (TextId::InputCategoryDirection, Button::Down),
                                        (TextId::InputCategoryDirection, Button::Left),
                                        (TextId::InputCategoryDirection, Button::Right),
                                        (TextId::InputCategoryAction, Button::A),
                                        (TextId::InputCategoryAction, Button::B),
                                        (TextId::InputCategorySystem, Button::Select),
                                        (TextId::InputCategorySystem, Button::Start),
                                    ];

                                    for (category_id, button) in mapping_rows {
                                        let category_label = self.t(*category_id);
                                        let name = controller::format_button_name(*button);
                                        let ctrl = &mut self.controllers[port];
                                        let bound = ctrl.binding_for(*button);
                                        let is_capturing = ctrl.capture_target() == Some(*button);

                                        ui.label(category_label);
                                        ui.label(name);

                                        if is_capturing {
                                            ui.colored_label(Color32::LIGHT_YELLOW, prompt_label);
                                        } else if let Some(key) = bound {
                                            ui.monospace(format!("{key:?}"));
                                        } else {
                                            ui.colored_label(Color32::DARK_GRAY, not_bound_label);
                                        }

                                        let button_label = if is_capturing {
                                            cancel_label
                                        } else {
                                            bind_label
                                        };
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
                                ui.label(self.t(TextId::InputCurrentlyPressedLabel));
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
                            ui.collapsing(self.t(TextId::InputGamepadMappingSection), |ui| {
                                ui.label(self.t(TextId::InputGamepadMappingTitle));
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
                                        let header_category =
                                            self.t(TextId::InputGamepadGridHeaderCategory);
                                        let header_button =
                                            self.t(TextId::InputGamepadGridHeaderButton);
                                        let header_gamepad_button =
                                            self.t(TextId::InputGamepadGridHeaderGamepadButton);
                                        let not_bound_label = self.t(TextId::InputNotBound);

                                        ui.strong(header_category);
                                        ui.strong(header_button);
                                        ui.strong(header_gamepad_button);
                                        ui.end_row();

                                        let mapping_rows: &[(TextId, Button)] = &[
                                            (TextId::InputCategoryDirection, Button::Up),
                                            (TextId::InputCategoryDirection, Button::Down),
                                            (TextId::InputCategoryDirection, Button::Left),
                                            (TextId::InputCategoryDirection, Button::Right),
                                            (TextId::InputCategoryAction, Button::A),
                                            (TextId::InputCategoryAction, Button::B),
                                            (TextId::InputCategorySystem, Button::Select),
                                            (TextId::InputCategorySystem, Button::Start),
                                        ];

                                        for (category_id, button) in mapping_rows {
                                            let category_label = self.t(*category_id);
                                            let name = controller::format_button_name(*button);
                                            let ctrl = &mut self.controllers[port];
                                            let mut binding = ctrl.gamepad_binding_for(*button);

                                            ui.label(category_label);
                                            ui.label(name);

                                            let current_label = binding
                                                .map(|b| format!("{b:?}"))
                                                .unwrap_or_else(|| not_bound_label.to_string());

                                            egui::ComboBox::from_id_salt(format!(
                                                "gp_map_{port}_{name}"
                                            ))
                                            .selected_text(current_label)
                                            .show_ui(
                                                ui,
                                                |ui| {
                                                    if ui
                                                        .selectable_label(
                                                            binding.is_none(),
                                                            not_bound_label,
                                                        )
                                                        .clicked()
                                                    {
                                                        binding = None;
                                                    }
                                                    for btn in all_buttons {
                                                        let label = format!("{btn:?}");
                                                        let selected = binding == Some(*btn);
                                                        if ui
                                                            .selectable_label(selected, label)
                                                            .clicked()
                                                        {
                                                            binding = Some(*btn);
                                                        }
                                                    }
                                                },
                                            );

                                            ctrl.set_gamepad_binding(*button, binding);
                                            ui.end_row();
                                        }
                                    });

                                let restore_label = self.t(TextId::InputRestoreDefaults);
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        if ui.button(restore_label).clicked() {
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
                .with_title(self.t(TextId::MenuWindowAudio))
                .with_inner_size([320.0, 320.0]);
            ctx.show_viewport_immediate(ViewportId::from_hash_of("audio"), builder, |ctx, _| {
                if ctx.input(|i| i.viewport().close_requested()) {
                    self.show_audio = false;
                    return;
                }
                egui::CentralPanel::default().show(ctx, |ui| {
                    let heading = self.t(TextId::AudioHeading);
                    let master_label = self.t(TextId::AudioMasterVolumeLabel);
                    let bg_label = self.t(TextId::AudioBgFastBehaviorLabel);
                    let mute_bg_label = self.t(TextId::AudioMuteInBackground);
                    let reduce_bg_label = self.t(TextId::AudioReduceInBackground);
                    let reduce_ff_label = self.t(TextId::AudioReduceInFastForward);
                    let reduce_amount_label = self.t(TextId::AudioReduceAmount);
                    let reverb_section_label = self.t(TextId::AudioReverbSection);
                    let reverb_enable_label = self.t(TextId::AudioEnableReverb);
                    let reverb_strength_label = self.t(TextId::AudioReverbStrength);
                    let reverb_delay_label = self.t(TextId::AudioReverbDelayMs);
                    let crossfeed_section_label = self.t(TextId::AudioCrossfeedSection);
                    let crossfeed_enable_label = self.t(TextId::AudioEnableCrossfeed);
                    let crossfeed_ratio_label = self.t(TextId::AudioCrossfeedRatio);
                    let eq_section_label = self.t(TextId::AudioEqSection);
                    let eq_enable_label = self.t(TextId::AudioEnableEq);
                    let eq_gain_label = self.t(TextId::AudioEqGlobalGain);

                    ui.heading(heading);
                    ui.separator();

                    let cfg = &mut self.audio_cfg;

                    // Master volume
                    let mut vol_percent = (cfg.master_volume * 100.0).clamp(0.0, 100.0);
                    ui.horizontal(|ui| {
                        ui.label(master_label);
                        if ui
                            .add(egui::Slider::new(&mut vol_percent, 0.0..=100.0).suffix("%"))
                            .changed()
                        {
                            cfg.master_volume = (vol_percent / 100.0).clamp(0.0, 1.0);
                        }
                    });

                    ui.separator();
                    ui.label(bg_label);
                    ui.checkbox(&mut cfg.mute_in_background, mute_bg_label);
                    ui.checkbox(&mut cfg.reduce_in_background, reduce_bg_label);
                    ui.checkbox(&mut cfg.reduce_in_fast_forward, reduce_ff_label);
                    let mut red_percent = (cfg.volume_reduction * 100.0).clamp(0.0, 100.0);
                    if ui
                        .add(
                            egui::Slider::new(&mut red_percent, 0.0..=100.0)
                                .suffix("%")
                                .text(reduce_amount_label),
                        )
                        .changed()
                    {
                        cfg.volume_reduction = (red_percent / 100.0).clamp(0.0, 1.0);
                    }

                    ui.separator();
                    ui.collapsing(reverb_section_label, |ui| {
                        ui.checkbox(&mut cfg.reverb_enabled, reverb_enable_label);
                        if cfg.reverb_enabled {
                            ui.add(
                                egui::Slider::new(&mut cfg.reverb_strength, 0.0..=1.0)
                                    .text(reverb_strength_label),
                            );
                            ui.add(
                                egui::Slider::new(&mut cfg.reverb_delay_ms, 1.0..=250.0)
                                    .text(reverb_delay_label),
                            );
                        }
                    });

                    ui.collapsing(crossfeed_section_label, |ui| {
                        ui.checkbox(&mut cfg.crossfeed_enabled, crossfeed_enable_label);
                        if cfg.crossfeed_enabled {
                            ui.add(
                                egui::Slider::new(&mut cfg.crossfeed_ratio, 0.0..=1.0)
                                    .text(crossfeed_ratio_label),
                            );
                        }
                    });

                    ui.collapsing(eq_section_label, |ui| {
                        ui.checkbox(&mut cfg.enable_equalizer, eq_enable_label);
                        if cfg.enable_equalizer {
                            // 统一使用单一增益控制 20 个频段。
                            let mut gain_db = cfg.eq_band_gains[0];
                            if ui
                                .add(
                                    egui::Slider::new(&mut gain_db, -12.0..=12.0)
                                        .text(eq_gain_label),
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
