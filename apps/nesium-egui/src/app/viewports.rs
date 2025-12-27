use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use eframe::egui;
use egui::{Color32, Context as EguiContext, ViewportBuilder, ViewportClass, ViewportId};
use gilrs::Button as GilrsButton;
use nesium_core::controller::Button;

use super::{
    AppViewport, NesiumApp, TextId, controller,
    controller::{ControllerDevice, InputAction, InputPreset},
};

fn consume_close_requests(
    ctx: &EguiContext,
    id: ViewportId,
    open: &mut bool,
    close_flag: &Arc<AtomicBool>,
) {
    if !*open {
        return;
    }

    let close_requested = close_flag.swap(false, Ordering::Relaxed)
        || ctx.viewport_for(id, |v| v.input.viewport().close_requested());
    if close_requested {
        *open = false;
        ctx.send_viewport_cmd_to(id, egui::ViewportCommand::Close);
    }
}

#[cfg(windows)]
fn show_viewport_with_close(
    ctx: &EguiContext,
    id: ViewportId,
    builder: ViewportBuilder,
    close_flag: Arc<AtomicBool>,
    mut draw: impl FnMut(&EguiContext, ViewportClass) -> bool,
) {
    ctx.show_viewport_immediate(id, builder, |ctx, class| {
        let mut close_requested = ctx.input(|i| i.viewport().close_requested());
        close_requested |= draw(ctx, class);
        if close_requested {
            close_flag.store(true, Ordering::Relaxed);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    });
}

#[cfg(not(windows))]
fn show_viewport_with_close(
    ctx: &EguiContext,
    id: ViewportId,
    builder: ViewportBuilder,
    close_flag: Arc<AtomicBool>,
    draw: impl Fn(&EguiContext, ViewportClass) -> bool + Send + Sync + 'static,
) {
    ctx.show_viewport_deferred(id, builder, move |ctx, class| {
        let mut close_requested = ctx.input(|i| i.viewport().close_requested());
        close_requested |= draw(ctx, class);
        if close_requested {
            close_flag.store(true, Ordering::Relaxed);
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    });
}

fn show_viewport_content(
    ctx: &EguiContext,
    class: ViewportClass,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui),
) -> bool {
    match class {
        ViewportClass::Embedded => {
            let mut open = true;
            egui::Window::new(title)
                .open(&mut open)
                .show(ctx, add_contents);
            !open
        }
        _ => {
            egui::CentralPanel::default().show(ctx, add_contents);
            false
        }
    }
}

impl NesiumApp {
    pub(super) fn show_viewports(&mut self, ctx: &EguiContext) {
        for viewport in AppViewport::ALL {
            let id = viewport.id();
            let close_flag = self.viewports.close_flag(viewport);
            consume_close_requests(ctx, id, self.viewports.open_mut(viewport), &close_flag);
        }

        if self.viewports.is_open(AppViewport::Debugger) {
            let builder = ViewportBuilder::default()
                .with_title("Debugger")
                .with_inner_size([420.0, 320.0]);
            show_viewport_with_close(
                ctx,
                AppViewport::Debugger.id(),
                builder,
                self.viewports.close_flag(AppViewport::Debugger),
                move |ctx, class| {
                    show_viewport_content(ctx, class, "Debugger", |ui| {
                        ui.heading("CPU Snapshot");
                        ui.label("Debugger is currently unavailable in multi-threaded mode.");
                        ui.label(
                            "Full debugger implementation via state snapshots is coming soon.",
                        );
                    })
                },
            );
        }

        if self.viewports.is_open(AppViewport::Tools) {
            let builder = ViewportBuilder::default()
                .with_title(self.t(TextId::MenuWindowTools))
                .with_inner_size([360.0, 260.0]);
            let ui_state_arc = Arc::clone(&self.ui_state);
            let runtime_handle = self.runtime_handle.clone();
            let close_flag = self.viewports.close_flag(AppViewport::Tools);
            show_viewport_with_close(
                ctx,
                AppViewport::Tools.id(),
                builder,
                close_flag,
                move |ctx, class| {
                    // Snapshot state
                    let (
                        mut pixel_perfect,
                        mut integer_fps,
                        title,
                        pixel_label,
                        fps_label,
                        fps_hint,
                        heading,
                        placeholder,
                    ) = {
                        let state = ui_state_arc.lock().unwrap();
                        let lang = state.i18n.language();
                        (
                            state.pixel_perfect_scaling,
                            state.integer_fps_mode,
                            state.i18n.text(TextId::MenuWindowTools),
                            match lang {
                                super::Language::English => "Pixel-perfect scaling (integer)",
                                super::Language::ChineseSimplified => "像素完美缩放（整数倍）",
                            },
                            match lang {
                                super::Language::English => "Integer FPS mode (60Hz, NTSC)",
                                super::Language::ChineseSimplified => "整数 FPS 模式（60Hz，NTSC）",
                            },
                            match lang {
                                super::Language::English => "PAL (50Hz) will be added later.",
                                super::Language::ChineseSimplified => "PAL（50Hz）后续再支持。",
                            },
                            state.i18n.text(TextId::ToolsHeading),
                            state.i18n.text(TextId::ToolsPlaceholder),
                        )
                    };

                    let mut changed = false;
                    let mut fps_changed = false;
                    let close_requested = show_viewport_content(ctx, class, title, |ui| {
                        ui.heading(heading);
                        if ui.checkbox(&mut pixel_perfect, pixel_label).changed() {
                            changed = true;
                        }
                        if ui.checkbox(&mut integer_fps, fps_label).changed() {
                            fps_changed = true;
                        }
                        ui.label(fps_hint);
                        ui.add_space(6.0);
                        ui.label(placeholder);
                    });

                    if changed {
                        ui_state_arc.lock().unwrap().pixel_perfect_scaling = pixel_perfect;
                    }
                    if fps_changed {
                        ui_state_arc.lock().unwrap().integer_fps_mode = integer_fps;
                        let _ = runtime_handle.set_integer_fps_target(if integer_fps {
                            Some(60)
                        } else {
                            None
                        });
                    }
                    close_requested
                },
            );
        }

        if self.viewports.is_open(AppViewport::About) {
            let builder = ViewportBuilder::default()
                .with_title(self.t(TextId::AboutWindowTitle))
                .with_inner_size([520.0, 420.0]);
            let ui_state = Arc::clone(&self.ui_state);
            let close_flag = self.viewports.close_flag(AppViewport::About);
            show_viewport_with_close(
                ctx,
                AppViewport::About.id(),
                builder,
                close_flag,
                move |ctx, class| {
                    // Snapshot state
                    let (title, lang, lead, intro, comp_heading, comp_hint) = {
                        let state = ui_state.lock().unwrap();
                        (
                            state.i18n.text(TextId::AboutWindowTitle),
                            state.i18n.language(),
                            state.i18n.text(TextId::AboutLead),
                            state.i18n.text(TextId::AboutIntro),
                            state.i18n.text(TextId::AboutComponentsHeading),
                            state.i18n.text(TextId::AboutComponentsHint),
                        )
                    };

                    let close_requested = show_viewport_content(ctx, class, title, |ui| {
                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.heading(title);
                                ui.label(lead);
                                ui.add_space(4.0);
                                ui.label(intro);
                                ui.add_space(8.0);
                                let repo_label = match lang {
                                    super::Language::English => "GitHub:",
                                    super::Language::ChineseSimplified => "GitHub：",
                                };
                                ui.horizontal_wrapped(|ui| {
                                    ui.label(repo_label);
                                    ui.hyperlink_to(
                                        "mikai233/nesium",
                                        "https://github.com/mikai233/nesium",
                                    );
                                });
                                ui.separator();
                                ui.heading(comp_heading);
                                ui.label(comp_hint);
                                ui.add_space(6.0);

                                struct ComponentInfo {
                                    name: &'static str,
                                    desc_en: &'static str,
                                    desc_zh: &'static str,
                                    url: &'static str,
                                }

                                const COMPONENTS: [ComponentInfo; 4] = [
                                    ComponentInfo {
                                        name: "eframe / egui",
                                        desc_en: "Rust-native UI + windowing shell",
                                        desc_zh: "Rust 原生 UI 与窗口框架",
                                        url: "https://github.com/emilk/egui",
                                    },
                                    ComponentInfo {
                                        name: "gilrs",
                                        desc_en: "Gamepad input layer",
                                        desc_zh: "手柄输入层",
                                        url: "https://crates.io/crates/gilrs",
                                    },
                                    ComponentInfo {
                                        name: "cpal",
                                        desc_en: "Native audio I/O backend",
                                        desc_zh: "本地音频 I/O 后端",
                                        url: "https://crates.io/crates/cpal",
                                    },
                                    ComponentInfo {
                                        name: "rfd",
                                        desc_en: "Native file dialogs",
                                        desc_zh: "原生文件对话框",
                                        url: "https://crates.io/crates/rfd",
                                    },
                                ];

                                egui::Grid::new("about_components_grid")
                                    .num_columns(2)
                                    .spacing([12.0, 8.0])
                                    .striped(true)
                                    .show(ui, |ui| {
                                        for component in COMPONENTS {
                                            let desc = match lang {
                                                super::Language::English => component.desc_en,
                                                super::Language::ChineseSimplified => {
                                                    component.desc_zh
                                                }
                                            };
                                            ui.hyperlink_to(component.name, component.url);
                                            ui.label(desc);
                                            ui.end_row();
                                        }
                                    });
                            });
                    });
                    close_requested
                },
            );
        }

        if self.viewports.is_open(AppViewport::Palette) {
            let builder = ViewportBuilder::default()
                .with_title(self.t(TextId::MenuWindowPalette))
                .with_inner_size([280.0, 240.0]);
            let ui_state = Arc::clone(&self.ui_state);
            let close_flag = self.viewports.close_flag(AppViewport::Palette);
            show_viewport_with_close(
                ctx,
                AppViewport::Palette.id(),
                builder,
                close_flag,
                move |ctx, class| {
                    let (title, heading) = {
                        let ui_state = ui_state.lock().unwrap();
                        (
                            ui_state.i18n.text(TextId::MenuWindowPalette),
                            ui_state.i18n.text(TextId::PaletteHeading),
                        )
                    };
                    let close_requested = show_viewport_content(ctx, class, title, |ui| {
                        ui.heading(heading);
                        ui.label("Palette viewer is currently unavailable.");
                    });
                    close_requested
                },
            );
        }

        if self.viewports.is_open(AppViewport::Input) {
            let builder = ViewportBuilder::default()
                .with_title(self.t(TextId::MenuWindowInput))
                .with_inner_size([420.0, 300.0]);
            let ui_state = Arc::clone(&self.ui_state);
            let runtime_handle = self.runtime_handle.clone();
            let close_flag = self.viewports.close_flag(AppViewport::Input);
            show_viewport_with_close(
                ctx,
                AppViewport::Input.id(),
                builder,
                close_flag,
                move |ctx, class| {
                    let mut ui_state = ui_state.lock().unwrap();
                    let title = ui_state.i18n.text(TextId::MenuWindowInput);

                    // Update key-capture in this viewport too, otherwise key events won't be seen
                    // when the Input window is focused.
                    let port_for_capture = ui_state.active_input_port.min(3);
                    if matches!(
                        ui_state.controller_devices[port_for_capture],
                        ControllerDevice::Keyboard
                    ) {
                        ui_state.controllers[port_for_capture].sync_from_input(
                            ctx,
                            port_for_capture,
                            true,
                        );
                    }

                    let close_requested = show_viewport_content(ctx, class, title, |ui| {
                        ui.heading(ui_state.i18n.text(TextId::InputHeading));

                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.collapsing(
                                    ui_state.i18n.text(TextId::InputTurboSection),
                                    |ui| {
                                        let mut frames = ui_state.turbo_frames_per_toggle.max(1);
                                        let label = ui_state.i18n.text(TextId::InputTurboRateLabel);
                                        if ui
                                            .add(
                                                egui::Slider::new(&mut frames, 1u8..=10u8)
                                                    .text(label),
                                            )
                                            .changed()
                                        {
                                            ui_state.turbo_frames_per_toggle = frames;
                                            runtime_handle.set_turbo_frames_per_toggle(frames);
                                        }
                                        ui.small(ui_state.i18n.text(TextId::InputTurboHelp));
                                    },
                                );

                                ui.horizontal(|ui| {
                                    ui.label(ui_state.i18n.text(TextId::InputControllerPortsLabel));
                                    for port in 0..4 {
                                        let label = match ui_state.i18n.language() {
                                            super::Language::English => {
                                                format!("Port {}", port + 1)
                                            }
                                            super::Language::ChineseSimplified => {
                                                format!("端口 {}", port + 1)
                                            }
                                        };
                                        ui.selectable_value(
                                            &mut ui_state.active_input_port,
                                            port,
                                            label,
                                        );
                                    }
                                });

                                ui.separator();
                                let port = ui_state.active_input_port.min(3);

                                ui.horizontal(|ui| {
                                    let language = ui_state.i18n.language();
                                    let device_label = match language {
                                        super::Language::English => {
                                            format!("Port {} device:", port + 1)
                                        }
                                        super::Language::ChineseSimplified => {
                                            format!("端口 {} 设备:", port + 1)
                                        }
                                    };
                                    let keyboard_label =
                                        ui_state.i18n.text(TextId::InputDeviceKeyboard);
                                    let disabled_label =
                                        ui_state.i18n.text(TextId::InputDeviceDisabled);
                                    let no_gamepads_label =
                                        ui_state.i18n.text(TextId::InputNoGamepads);
                                    let gamepad_unavailable_label =
                                        ui_state.i18n.text(TextId::InputGamepadUnavailable);

                                    ui.label(device_label);

                                    let gamepads_available = ui_state.gamepads_available;
                                    let gamepads = ui_state.gamepads.clone();

                                    ui.selectable_value(
                                        &mut ui_state.controller_devices[port],
                                        ControllerDevice::Keyboard,
                                        keyboard_label,
                                    );

                                    if gamepads_available {
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
                                                &mut ui_state.controller_devices[port],
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
                                        &mut ui_state.controller_devices[port],
                                        ControllerDevice::Disabled,
                                        disabled_label,
                                    );
                                });

                                if port >= 2 {
                                    ui.colored_label(
                                        Color32::DARK_GRAY,
                                        ui_state.i18n.text(TextId::InputPort34Notice),
                                    );
                                }

                                ui.separator();
                                ui.horizontal(|ui| {
                                    let preset_label = ui_state.i18n.text(TextId::InputPresetLabel);
                                    let nes_label =
                                        ui_state.i18n.text(TextId::InputPresetNesStandard);
                                    let fight_label =
                                        ui_state.i18n.text(TextId::InputPresetFightStick);
                                    let arcade_label =
                                        ui_state.i18n.text(TextId::InputPresetArcadeLayout);

                                    ui.label(preset_label);
                                    let mut preset_value = ui_state.controller_presets[port];
                                    egui::ComboBox::from_id_salt(format!(
                                        "input_preset_combo_{port}"
                                    ))
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
                                            ui_state.controllers[port]
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
                                            ui_state.controllers[port]
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
                                            ui_state.controllers[port]
                                                .apply_preset(InputPreset::ArcadeLayout);
                                        }
                                    });
                                    ui_state.controller_presets[port] = preset_value;
                                });

                                ui.separator();
                                ui.label(ui_state.i18n.text(TextId::InputKeyboardMappingTitle));
                                ui.small(ui_state.i18n.text(TextId::InputKeyboardMappingHelp));

                                ui.separator();
                                egui::Grid::new("input_mapping_grid")
                                    .num_columns(4)
                                    .spacing([12.0, 4.0])
                                    .striped(true)
                                    .show(ui, |ui| {
                                        let header_category =
                                            ui_state.i18n.text(TextId::InputGridHeaderCategory);
                                        let header_button =
                                            ui_state.i18n.text(TextId::InputGridHeaderButton);
                                        let header_current =
                                            ui_state.i18n.text(TextId::InputGridHeaderCurrentKey);
                                        let header_action =
                                            ui_state.i18n.text(TextId::InputGridHeaderAction);
                                        let prompt_label =
                                            ui_state.i18n.text(TextId::InputPromptPressAnyKey);
                                        let not_bound_label =
                                            ui_state.i18n.text(TextId::InputNotBound);
                                        let cancel_label =
                                            ui_state.i18n.text(TextId::InputCancelButton);
                                        let bind_label =
                                            ui_state.i18n.text(TextId::InputBindButton);

                                        ui.strong(header_category);
                                        ui.strong(header_button);
                                        ui.strong(header_current);
                                        ui.strong(header_action);
                                        ui.end_row();

                                        let mapping_rows: &[(TextId, InputAction)] = &[
                                            (
                                                TextId::InputCategoryDirection,
                                                InputAction::Button(Button::Up),
                                            ),
                                            (
                                                TextId::InputCategoryDirection,
                                                InputAction::Button(Button::Down),
                                            ),
                                            (
                                                TextId::InputCategoryDirection,
                                                InputAction::Button(Button::Left),
                                            ),
                                            (
                                                TextId::InputCategoryDirection,
                                                InputAction::Button(Button::Right),
                                            ),
                                            (
                                                TextId::InputCategoryAction,
                                                InputAction::Button(Button::A),
                                            ),
                                            (
                                                TextId::InputCategoryAction,
                                                InputAction::Button(Button::B),
                                            ),
                                            (
                                                TextId::InputCategoryAction,
                                                InputAction::Turbo(Button::A),
                                            ),
                                            (
                                                TextId::InputCategoryAction,
                                                InputAction::Turbo(Button::B),
                                            ),
                                            (
                                                TextId::InputCategorySystem,
                                                InputAction::Button(Button::Select),
                                            ),
                                            (
                                                TextId::InputCategorySystem,
                                                InputAction::Button(Button::Start),
                                            ),
                                        ];

                                        for (category_id, action) in mapping_rows {
                                            let category_label = ui_state.i18n.text(*category_id);
                                            let name = match action {
                                                InputAction::Button(button) => {
                                                    controller::format_button_name(*button)
                                                        .to_string()
                                                }
                                                InputAction::Turbo(button) => match button {
                                                    Button::A => ui_state
                                                        .i18n
                                                        .text(TextId::InputButtonTurboA)
                                                        .to_string(),
                                                    Button::B => ui_state
                                                        .i18n
                                                        .text(TextId::InputButtonTurboB)
                                                        .to_string(),
                                                    _ => format!(
                                                        "Turbo {}",
                                                        controller::format_button_name(*button)
                                                    ),
                                                },
                                            };
                                            let ctrl = &mut ui_state.controllers[port];

                                            ui.label(category_label);
                                            ui.label(name);

                                            let capture_target = ctrl.capture_target();
                                            let is_capturing = capture_target == Some(*action);
                                            if is_capturing {
                                                ui.colored_label(Color32::LIGHT_BLUE, prompt_label);
                                            } else {
                                                let bound = match action {
                                                    InputAction::Button(button) => {
                                                        ctrl.binding_for(*button)
                                                    }
                                                    InputAction::Turbo(button) => {
                                                        ctrl.turbo_binding_for(*button)
                                                    }
                                                };
                                                if let Some(key) = bound {
                                                    ui.monospace(format!("{key:?}"));
                                                } else {
                                                    ui.colored_label(
                                                        Color32::DARK_GRAY,
                                                        not_bound_label,
                                                    );
                                                }
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
                                                    ctrl.begin_capture(*action);
                                                }
                                            }

                                            ui.end_row();
                                        }
                                    });

                                ui.horizontal(|ui| {
                                    ui.label(
                                        ui_state.i18n.text(TextId::InputCurrentlyPressedLabel),
                                    );
                                    ui.horizontal_wrapped(|ui| {
                                        let ctrl = &ui_state.controllers[port];
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
                                ui.collapsing(
                                    ui_state.i18n.text(TextId::InputGamepadMappingSection),
                                    |ui| {
                                        ui.label(
                                            ui_state.i18n.text(TextId::InputGamepadMappingTitle),
                                        );
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
                                                let header_category = ui_state
                                                    .i18n
                                                    .text(TextId::InputGamepadGridHeaderCategory);
                                                let header_button = ui_state
                                                    .i18n
                                                    .text(TextId::InputGamepadGridHeaderButton);
                                                let header_gamepad_button = ui_state.i18n.text(
                                                    TextId::InputGamepadGridHeaderGamepadButton,
                                                );
                                                let not_bound_label =
                                                    ui_state.i18n.text(TextId::InputNotBound);

                                                ui.strong(header_category);
                                                ui.strong(header_button);
                                                ui.strong(header_gamepad_button);
                                                ui.end_row();

                                                let mapping_rows: &[(TextId, InputAction)] = &[
                                                    (
                                                        TextId::InputCategoryDirection,
                                                        InputAction::Button(Button::Up),
                                                    ),
                                                    (
                                                        TextId::InputCategoryDirection,
                                                        InputAction::Button(Button::Down),
                                                    ),
                                                    (
                                                        TextId::InputCategoryDirection,
                                                        InputAction::Button(Button::Left),
                                                    ),
                                                    (
                                                        TextId::InputCategoryDirection,
                                                        InputAction::Button(Button::Right),
                                                    ),
                                                    (
                                                        TextId::InputCategoryAction,
                                                        InputAction::Button(Button::A),
                                                    ),
                                                    (
                                                        TextId::InputCategoryAction,
                                                        InputAction::Button(Button::B),
                                                    ),
                                                    (
                                                        TextId::InputCategoryAction,
                                                        InputAction::Turbo(Button::A),
                                                    ),
                                                    (
                                                        TextId::InputCategoryAction,
                                                        InputAction::Turbo(Button::B),
                                                    ),
                                                    (
                                                        TextId::InputCategorySystem,
                                                        InputAction::Button(Button::Select),
                                                    ),
                                                    (
                                                        TextId::InputCategorySystem,
                                                        InputAction::Button(Button::Start),
                                                    ),
                                                ];

                                                for (category_id, action) in mapping_rows {
                                                    let category_label =
                                                        ui_state.i18n.text(*category_id);
                                                    let name = match action {
                                                        InputAction::Button(button) => {
                                                            controller::format_button_name(*button)
                                                                .to_string()
                                                        }
                                                        InputAction::Turbo(button) => {
                                                            match button {
                                                                Button::A => ui_state
                                                                    .i18n
                                                                    .text(TextId::InputButtonTurboA)
                                                                    .to_string(),
                                                                Button::B => ui_state
                                                                    .i18n
                                                                    .text(TextId::InputButtonTurboB)
                                                                    .to_string(),
                                                                _ => format!(
                                                                    "Turbo {}",
                                                                    controller::format_button_name(
                                                                        *button
                                                                    )
                                                                ),
                                                            }
                                                        }
                                                    };
                                                    let ctrl = &mut ui_state.controllers[port];
                                                    let mut binding = match action {
                                                        InputAction::Button(button) => {
                                                            ctrl.gamepad_binding_for(*button)
                                                        }
                                                        InputAction::Turbo(button) => {
                                                            ctrl.turbo_gamepad_binding_for(*button)
                                                        }
                                                    };

                                                    ui.label(category_label);
                                                    ui.label(&name);

                                                    let current_label = binding
                                                        .map(|b| format!("{b:?}"))
                                                        .unwrap_or_else(|| {
                                                            not_bound_label.to_string()
                                                        });

                                                    egui::ComboBox::from_id_salt(format!(
                                                        "gp_map_{port}_{name}"
                                                    ))
                                                    .selected_text(current_label)
                                                    .show_ui(ui, |ui| {
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
                                                    });

                                                    match action {
                                                        InputAction::Button(button) => ctrl
                                                            .set_gamepad_binding(*button, binding),
                                                        InputAction::Turbo(button) => ctrl
                                                            .set_turbo_gamepad_binding(
                                                                *button, binding,
                                                            ),
                                                    }
                                                    ui.end_row();
                                                }
                                            });

                                        let restore_label =
                                            ui_state.i18n.text(TextId::InputRestoreDefaults);
                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui.button(restore_label).clicked() {
                                                    ui_state.controllers[port] =
                                                    controller::ControllerInput::new_with_defaults(
                                                    );
                                                }
                                            },
                                        );
                                    },
                                );
                            });
                    });
                    close_requested
                },
            );
        }

        if self.viewports.is_open(AppViewport::Audio) {
            let builder = ViewportBuilder::default()
                .with_title(self.t(TextId::MenuWindowAudio))
                .with_inner_size([320.0, 320.0]);
            let ui_state = Arc::clone(&self.ui_state);
            let runtime_handle = self.runtime_handle.clone();
            let close_flag = self.viewports.close_flag(AppViewport::Audio);
            show_viewport_with_close(
                ctx,
                AppViewport::Audio.id(),
                builder,
                close_flag,
                move |ctx, class| {
                    let (
                        title,
                        heading,
                        master_label,
                        bg_label,
                        mute_bg_label,
                        reduce_bg_label,
                        reduce_ff_label,
                        reduce_amount_label,
                        reverb_section_label,
                        reverb_enable_label,
                        reverb_strength_label,
                        reverb_delay_label,
                        crossfeed_section_label,
                        crossfeed_enable_label,
                        crossfeed_ratio_label,
                        eq_section_label,
                        eq_enable_label,
                        eq_gain_label,
                        mut cfg,
                    ) = {
                        let ui_state = ui_state.lock().unwrap();
                        (
                            ui_state.i18n.text(TextId::MenuWindowAudio),
                            ui_state.i18n.text(TextId::AudioHeading),
                            ui_state.i18n.text(TextId::AudioMasterVolumeLabel),
                            ui_state.i18n.text(TextId::AudioBgFastBehaviorLabel),
                            ui_state.i18n.text(TextId::AudioMuteInBackground),
                            ui_state.i18n.text(TextId::AudioReduceInBackground),
                            ui_state.i18n.text(TextId::AudioReduceInFastForward),
                            ui_state.i18n.text(TextId::AudioReduceAmount),
                            ui_state.i18n.text(TextId::AudioReverbSection),
                            ui_state.i18n.text(TextId::AudioEnableReverb),
                            ui_state.i18n.text(TextId::AudioReverbStrength),
                            ui_state.i18n.text(TextId::AudioReverbDelayMs),
                            ui_state.i18n.text(TextId::AudioCrossfeedSection),
                            ui_state.i18n.text(TextId::AudioEnableCrossfeed),
                            ui_state.i18n.text(TextId::AudioCrossfeedRatio),
                            ui_state.i18n.text(TextId::AudioEqSection),
                            ui_state.i18n.text(TextId::AudioEnableEq),
                            ui_state.i18n.text(TextId::AudioEqGlobalGain),
                            ui_state.audio_cfg,
                        )
                    };

                    let mut changed = false;
                    let close_requested = show_viewport_content(ctx, class, title, |ui| {
                        ui.heading(heading);
                        ui.separator();

                        // Master volume
                        let mut vol_percent = (cfg.master_volume * 100.0).clamp(0.0, 100.0);
                        ui.horizontal(|ui| {
                            ui.label(master_label);
                            if ui
                                .add(egui::Slider::new(&mut vol_percent, 0.0..=100.0).suffix("%"))
                                .changed()
                            {
                                cfg.master_volume = (vol_percent / 100.0).clamp(0.0, 1.0);
                                changed = true;
                            }
                        });

                        ui.separator();
                        ui.label(bg_label);
                        if ui
                            .checkbox(&mut cfg.mute_in_background, mute_bg_label)
                            .changed()
                        {
                            changed = true;
                        }
                        if ui
                            .checkbox(&mut cfg.reduce_in_background, reduce_bg_label)
                            .changed()
                        {
                            changed = true;
                        }
                        if ui
                            .checkbox(&mut cfg.reduce_in_fast_forward, reduce_ff_label)
                            .changed()
                        {
                            changed = true;
                        }
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
                            changed = true;
                        }

                        ui.separator();
                        ui.collapsing(reverb_section_label, |ui| {
                            if ui
                                .checkbox(&mut cfg.reverb_enabled, reverb_enable_label)
                                .changed()
                            {
                                changed = true;
                            }
                            if cfg.reverb_enabled {
                                if ui
                                    .add(
                                        egui::Slider::new(&mut cfg.reverb_strength, 0.0..=1.0)
                                            .text(reverb_strength_label),
                                    )
                                    .changed()
                                {
                                    changed = true;
                                }
                                if ui
                                    .add(
                                        egui::Slider::new(&mut cfg.reverb_delay_ms, 1.0..=250.0)
                                            .text(reverb_delay_label),
                                    )
                                    .changed()
                                {
                                    changed = true;
                                }
                            }
                        });

                        ui.collapsing(crossfeed_section_label, |ui| {
                            if ui
                                .checkbox(&mut cfg.crossfeed_enabled, crossfeed_enable_label)
                                .changed()
                            {
                                changed = true;
                            }
                            if cfg.crossfeed_enabled
                                && ui
                                    .add(
                                        egui::Slider::new(&mut cfg.crossfeed_ratio, 0.0..=1.0)
                                            .text(crossfeed_ratio_label),
                                    )
                                    .changed()
                            {
                                changed = true;
                            }
                        });

                        ui.collapsing(eq_section_label, |ui| {
                            if ui
                                .checkbox(&mut cfg.enable_equalizer, eq_enable_label)
                                .changed()
                            {
                                changed = true;
                            }
                            if cfg.enable_equalizer {
                                // Use a single gain slider for all bands.
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
                                    changed = true;
                                }
                            }
                        });

                        if changed {
                            let _ = runtime_handle.set_audio_config(cfg);
                        }
                    });

                    if changed {
                        if let Ok(mut ui_state) = ui_state.lock() {
                            ui_state.audio_cfg = cfg;
                        }
                    }
                    close_requested
                },
            );
        }
    }
}
