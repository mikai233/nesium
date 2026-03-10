mod debug;
mod gamepad;
mod input;
mod state;
mod windows;

use std::{
    ffi::c_void,
    path::{Path, PathBuf},
    ptr,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

use slint::winit_030::WinitWindowAccessor;

use anyhow::{Context, Result};
use slint::{CloseRequestResponse, ComponentHandle, Model, SharedString, Weak};

use crate::{MainWindow, runtime::RuntimeSession, video::GameRenderer};

use self::{
    debug::DebugPanelController,
    input::InputRouter,
    state::{AuxWindowKind, DisplayMode},
    windows::AuxWindows,
};

struct FrameReadyBridge {
    window: Weak<MainWindow>,
    redraw_pending: Arc<AtomicBool>,
}

struct FrameReadyRegistration {
    session: Arc<RuntimeSession>,
    user_data: *mut FrameReadyBridge,
}

impl Drop for FrameReadyRegistration {
    fn drop(&mut self) {
        let _ = self.session.set_frame_ready_callback(None, ptr::null_mut());
        if !self.user_data.is_null() {
            unsafe {
                drop(Box::from_raw(self.user_data));
            }
            self.user_data = ptr::null_mut();
        }
    }
}

pub fn run() -> Result<()> {
    slint::BackendSelector::new()
        .backend_name("winit".into())
        .require_wgpu_28(slint::wgpu_28::WGPUConfiguration::default())
        .select()
        .context("failed to initialize Slint backend")?;

    let window = MainWindow::new().context("failed to create main window")?;
    initialize_window(&window);

    let session = Arc::new(RuntimeSession::new()?);
    let renderer = Arc::new(Mutex::new(None::<GameRenderer>));
    let input_router = Arc::new(Mutex::new(InputRouter::new()));
    let debug_panel = Arc::new(Mutex::new(DebugPanelController::new()));
    let redraw_pending = Arc::new(AtomicBool::new(false));
    let aux_windows = Arc::new(AuxWindows::new(&window)?);

    debug_panel
        .lock()
        .expect("debug panel mutex poisoned")
        .apply_to_window(aux_windows.debugger());

    install_close_handler(&window);
    install_renderer(
        &window,
        Arc::clone(&session),
        Arc::clone(&renderer),
        Arc::clone(&debug_panel),
        Arc::clone(&aux_windows),
        Arc::clone(&redraw_pending),
    )?;
    install_callbacks(
        &window,
        Arc::clone(&session),
        Arc::clone(&input_router),
        Arc::clone(&debug_panel),
        Arc::clone(&aux_windows),
    );
    install_drop_handler(&window);
    let frame_ready_registration =
        install_frame_ready_callback(&window, Arc::clone(&session), redraw_pending)?;

    let mut gamepad_manager = crate::app::gamepad::GamepadManager::new();
    let gamepad_timer = slint::Timer::default();
    let gamepad_session = Arc::clone(&session);
    let gamepad_router = Arc::clone(&input_router);

    gamepad_timer.start(
        slint::TimerMode::Repeated,
        std::time::Duration::from_millis(16),
        move || {
            if let Some(gm) = gamepad_manager.as_mut() {
                gm.poll();
                let pads = gm.gamepads();

                let mut router = gamepad_router.lock().unwrap();
                let mut pad_masks = [0u8; 2];

                for (i, (id, _name)) in pads.iter().take(2).enumerate() {
                    let mut mask = 0u8;
                    if gm.is_pressed(*id, gilrs::Button::DPadUp) {
                        mask |= 1 << input::button_bit(nesium_core::controller::Button::Up);
                    }
                    if gm.is_pressed(*id, gilrs::Button::DPadDown) {
                        mask |= 1 << input::button_bit(nesium_core::controller::Button::Down);
                    }
                    if gm.is_pressed(*id, gilrs::Button::DPadLeft) {
                        mask |= 1 << input::button_bit(nesium_core::controller::Button::Left);
                    }
                    if gm.is_pressed(*id, gilrs::Button::DPadRight) {
                        mask |= 1 << input::button_bit(nesium_core::controller::Button::Right);
                    }
                    if gm.is_pressed(*id, gilrs::Button::East) {
                        mask |= 1 << input::button_bit(nesium_core::controller::Button::A);
                    }
                    if gm.is_pressed(*id, gilrs::Button::South) {
                        mask |= 1 << input::button_bit(nesium_core::controller::Button::B);
                    }
                    if gm.is_pressed(*id, gilrs::Button::Start) {
                        mask |= 1 << input::button_bit(nesium_core::controller::Button::Start);
                    }
                    if gm.is_pressed(*id, gilrs::Button::Select) {
                        mask |= 1 << input::button_bit(nesium_core::controller::Button::Select);
                    }

                    pad_masks[i] = mask;
                }

                router.update_gamepad_mask(&gamepad_session, 0, pad_masks[0]);
                router.update_gamepad_mask(&gamepad_session, 1, pad_masks[1]);
            }
        },
    );

    window.show().context("failed to show main window")?;
    let run_result = slint::run_event_loop().context("slint app exited with an error");

    drop(gamepad_timer);
    drop(frame_ready_registration);
    drop(aux_windows);
    drop(debug_panel);
    drop(input_router);
    drop(renderer);
    drop(session);
    drop(window);

    run_result
}

fn initialize_window(window: &MainWindow) {
    window.set_display_mode(DisplayMode::Square.as_index());
    window.set_integer_fps_mode(false);
    window.set_rom_name(SharedString::from("No ROM"));
    window.set_status_text(SharedString::from("Runtime ready"));
    window.set_fps_text(SharedString::from("FPS: --"));
    window.set_has_rom(false);
    window.set_paused(false);
}

fn install_close_handler(window: &MainWindow) {
    window.window().on_close_requested(|| {
        let _ = slint::quit_event_loop();
        CloseRequestResponse::HideWindow
    });
}

fn install_renderer(
    window: &MainWindow,
    session: Arc<RuntimeSession>,
    renderer: Arc<Mutex<Option<GameRenderer>>>,
    debug_panel: Arc<Mutex<DebugPanelController>>,
    aux_windows: Arc<AuxWindows>,
    redraw_pending: Arc<AtomicBool>,
) -> Result<()> {
    let window_weak = window.as_weak();
    let mut fps_last_update = Instant::now();
    let mut fps_frame_count = 0u32;

    window
        .window()
        .set_rendering_notifier(move |state, graphics_api| match (state, graphics_api) {
            (
                slint::RenderingState::RenderingSetup,
                slint::GraphicsAPI::WGPU28 { device, queue, .. },
            ) => {
                let Some(window) = window_weak.upgrade() else {
                    return;
                };

                let mut renderer_slot = renderer.lock().expect("renderer mutex poisoned");
                if renderer_slot.is_none() {
                    match GameRenderer::new(device, queue) {
                        Ok((new_renderer, image)) => {
                            window.set_game_frame(image);
                            *renderer_slot = Some(new_renderer);
                        }
                        Err(err) => {
                            window.set_status_text(SharedString::from(format!(
                                "renderer init failed: {err}"
                            )));
                        }
                    }
                }
            }
            (slint::RenderingState::BeforeRendering, _) => {
                redraw_pending.store(false, Ordering::Release);

                let Some(window) = window_weak.upgrade() else {
                    return;
                };

                {
                    let mut renderer_slot = renderer.lock().expect("renderer mutex poisoned");
                    if let Some(renderer) = renderer_slot.as_mut()
                        && session.upload_latest_frame(renderer)
                    {
                        fps_frame_count = fps_frame_count.saturating_add(1);
                    }
                }

                let elapsed = fps_last_update.elapsed();
                if elapsed.as_secs_f32() >= 1.0 {
                    if window.get_has_rom() && !window.get_paused() && fps_frame_count > 0 {
                        let fps = fps_frame_count as f32 / elapsed.as_secs_f32();
                        window.set_fps_text(SharedString::from(format!("FPS: {:.1}", fps)));
                    } else if !window.get_has_rom() || window.get_paused() {
                        window.set_fps_text(SharedString::from("FPS: --"));
                    }
                    fps_last_update = Instant::now();
                    fps_frame_count = 0;
                }

                let mut debug_panel = debug_panel.lock().expect("debug panel mutex poisoned");
                debug_panel.drain();
                debug_panel.apply_to_window(aux_windows.debugger());
            }
            (slint::RenderingState::RenderingTeardown, _) => {
                redraw_pending.store(false, Ordering::Release);
                let mut renderer_slot = renderer.lock().expect("renderer mutex poisoned");
                *renderer_slot = None;
            }
            _ => {}
        })
        .context("failed to install rendering notifier")
}

fn install_callbacks(
    window: &MainWindow,
    session: Arc<RuntimeSession>,
    input_router: Arc<Mutex<InputRouter>>,
    debug_panel: Arc<Mutex<DebugPanelController>>,
    aux_windows: Arc<AuxWindows>,
) {
    let window_weak = window.as_weak();
    let load_session = Arc::clone(&session);
    let load_input = Arc::clone(&input_router);
    let load_aux_windows = Arc::clone(&aux_windows);
    window.on_load_rom_requested(move || {
        let Some(path) = pick_rom_path() else {
            return;
        };

        let Some(window) = window_weak.upgrade() else {
            return;
        };

        match load_session.load_rom(&path) {
            Ok(()) => {
                load_input
                    .lock()
                    .expect("input router mutex poisoned")
                    .clear(&load_session);
                let rom_name = rom_name_for_path(&path);
                window.set_has_rom(true);
                window.set_paused(false);
                window.set_rom_name(rom_name.clone());
                window.set_fps_text(SharedString::from("FPS: --"));
                window.set_status_text(SharedString::from(format!("Loaded {}", path.display())));
                load_aux_windows.set_rom_state(rom_name, true);
            }
            Err(err) => {
                window.set_error_message(SharedString::from(format!("Failed to load ROM:\n{err}")));
            }
        }
    });

    let window_weak = window.as_weak();
    let pause_session = Arc::clone(&session);
    window.on_pause_toggled(move || {
        let Some(window) = window_weak.upgrade() else {
            return;
        };
        if !window.get_has_rom() {
            return;
        }

        let paused = pause_session.toggle_pause();
        window.set_paused(paused);
        window.set_status_text(SharedString::from(if paused {
            "Paused"
        } else {
            "Running"
        }));
        if paused {
            window.set_fps_text(SharedString::from("FPS: --"));
        }
    });

    let window_weak = window.as_weak();
    let reset_session = Arc::clone(&session);
    let reset_input = Arc::clone(&input_router);
    window.on_reset_requested(move || {
        let Some(window) = window_weak.upgrade() else {
            return;
        };
        if !window.get_has_rom() {
            return;
        }

        match reset_session.reset() {
            Ok(()) => {
                reset_input
                    .lock()
                    .expect("input router mutex poisoned")
                    .clear(&reset_session);
                window.set_paused(false);
                window.set_status_text(SharedString::from("Soft reset"));
            }
            Err(err) => {
                window.set_status_text(SharedString::from(format!("Reset failed: {err}")));
            }
        }
    });

    let window_weak = window.as_weak();
    let power_reset_session = Arc::clone(&session);
    let power_reset_input = Arc::clone(&input_router);
    window.on_power_reset_requested(move || {
        let Some(window) = window_weak.upgrade() else {
            return;
        };
        if !window.get_has_rom() {
            return;
        }

        match power_reset_session.power_reset() {
            Ok(()) => {
                power_reset_input
                    .lock()
                    .expect("input router mutex poisoned")
                    .clear(&power_reset_session);
                window.set_paused(false);
                window.set_status_text(SharedString::from("Power reset"));
            }
            Err(err) => {
                window.set_status_text(SharedString::from(format!("Power reset failed: {err}")));
            }
        }
    });

    let window_weak = window.as_weak();
    let power_off_session = Arc::clone(&session);
    let power_off_input = Arc::clone(&input_router);
    let power_off_debug = Arc::clone(&debug_panel);
    let power_off_aux_windows = Arc::clone(&aux_windows);
    window.on_power_off_requested(move || {
        let Some(window) = window_weak.upgrade() else {
            return;
        };
        if !window.get_has_rom() {
            return;
        }

        match power_off_session.power_off() {
            Ok(()) => {
                power_off_input
                    .lock()
                    .expect("input router mutex poisoned")
                    .clear(&power_off_session);
                let debug_panel = power_off_debug.lock().expect("debug panel mutex poisoned");
                power_off_aux_windows.set_rom_state(SharedString::from("No ROM"), false);
                debug_panel.apply_to_window(power_off_aux_windows.debugger());
                window.set_has_rom(false);
                window.set_paused(false);
                window.set_rom_name(SharedString::from("No ROM"));
                window.set_fps_text(SharedString::from("FPS: --"));
                window.set_status_text(SharedString::from("Cartridge ejected"));
                window.window().request_redraw();
            }
            Err(err) => {
                window.set_status_text(SharedString::from(format!("Power off failed: {err}")));
            }
        }
    });

    let window_weak = window.as_weak();
    window.on_display_mode_requested(move |mode| {
        let Some(window) = window_weak.upgrade() else {
            return;
        };

        let mode = DisplayMode::from_index(mode);
        window.set_display_mode(mode.as_index());
        window.set_status_text(SharedString::from(format!("View: {}", mode.label())));
        window.window().request_redraw();
    });

    let window_weak = window.as_weak();
    let fps_session = Arc::clone(&session);
    window.on_integer_fps_mode_requested(move |_| {
        let Some(window) = window_weak.upgrade() else {
            return;
        };
        if !window.get_has_rom() {
            return;
        }
        let enabled = !window.get_integer_fps_mode();

        match fps_session.set_integer_fps_mode(enabled) {
            Ok(()) => {
                window.set_integer_fps_mode(enabled);
                window.set_status_text(SharedString::from(if enabled {
                    "Enabled 60 FPS integer pacing"
                } else {
                    "Disabled integer FPS pacing"
                }));
            }
            Err(err) => {
                window.set_status_text(SharedString::from(format!("Pacing change failed: {err}")));
            }
        }
    });

    let window_weak = window.as_weak();
    let aux_session = Arc::clone(&session);
    let aux_debug_panel = Arc::clone(&debug_panel);
    let aux_windows_for_open = Arc::clone(&aux_windows);
    window.on_aux_window_open_requested(move |kind| {
        let Some(window) = window_weak.upgrade() else {
            return;
        };
        let Some(kind) = AuxWindowKind::from_index(kind) else {
            return;
        };

        if kind == AuxWindowKind::Debugger {
            let mut debug_panel = aux_debug_panel.lock().expect("debug panel mutex poisoned");
            match debug_panel.set_enabled(&aux_session, true) {
                Ok(()) => {
                    debug_panel.apply_to_window(aux_windows_for_open.debugger());
                }
                Err(err) => {
                    window.set_status_text(SharedString::from(format!(
                        "Debugger toggle failed: {err}"
                    )));
                    return;
                }
            }
        }

        match aux_windows_for_open.open_or_focus(kind) {
            Ok(()) => {}
            Err(err) => {
                window.set_status_text(SharedString::from(format!("Window update failed: {err}")));
            }
        }
    });

    let window_weak = window.as_weak();
    let closed_session = Arc::clone(&session);
    let closed_debug_panel = Arc::clone(&debug_panel);
    let closed_aux_windows = Arc::clone(&aux_windows);
    window.on_aux_window_closed(move |kind| {
        let Some(window) = window_weak.upgrade() else {
            return;
        };
        let Some(kind) = AuxWindowKind::from_index(kind) else {
            return;
        };

        if kind != AuxWindowKind::Debugger {
            return;
        }

        let mut debug_panel = closed_debug_panel
            .lock()
            .expect("debug panel mutex poisoned");
        if let Err(err) = debug_panel.set_enabled(&closed_session, false) {
            window.set_status_text(SharedString::from(format!("Debugger close failed: {err}")));
            return;
        }
        debug_panel.apply_to_window(closed_aux_windows.debugger());
    });

    let game_input_session = Arc::clone(&session);
    let game_input_router = Arc::clone(&input_router);
    window.on_game_key_changed(move |key, pressed| {
        let mut input_router = game_input_router
            .lock()
            .expect("input router mutex poisoned");
        input_router.handle_key(&game_input_session, key.as_str(), pressed)
    });

    let bind_req_window = aux_windows.input().clone_strong();
    aux_windows
        .input()
        .on_keyboard_bind_requested(move |port, btn_index| {
            bind_req_window.set_capturing_port_index(port);
            bind_req_window.set_capturing_button_index(btn_index);
            bind_req_window.invoke_focus_bind_catcher();
        });

    let cancel_bind_window = aux_windows.input().clone_strong();
    aux_windows.input().on_cancel_bind_requested(move || {
        cancel_bind_window.set_capturing_port_index(-1);
        cancel_bind_window.set_capturing_button_index(-1);
    });

    let bind_done_window = aux_windows.input().clone_strong();
    let bind_done_router = Arc::clone(&input_router);
    aux_windows
        .input()
        .on_keyboard_bind_completed(move |port, btn_index, key| {
            bind_done_window.set_capturing_port_index(-1);
            bind_done_window.set_capturing_button_index(-1);

            let mut router = bind_done_router.lock().unwrap();
            let button_names = bind_done_window.get_button_names();
            let Some(name_val) = button_names.row_data(btn_index as usize) else {
                return;
            };

            if let Some(button) = input::button_from_name(name_val.as_str()) {
                let is_turbo = name_val.as_str().contains("Turbo");
                router.bind_key(port as usize, key.as_str(), button, is_turbo);
                crate::app::update_input_ui_mappings(&bind_done_window, &router);
            }
        });

    let input_reset_session = Arc::clone(&session);
    let input_reset_router = Arc::clone(&input_router);
    let input_reset_window = aux_windows.input().clone_strong();
    window.on_game_input_reset_requested(move || {
        let mut router = input_reset_router
            .lock()
            .expect("input router mutex poisoned");
        router.clear(&input_reset_session);
        router.apply_preset();
        crate::app::update_input_ui_mappings(&input_reset_window, &router);
    });

    let reset_btn_router = Arc::clone(&input_router);
    let reset_btn_window = aux_windows.input().clone_strong();
    let reset_btn_session = Arc::clone(&session);
    aux_windows.input().on_input_reset_requested(move |_port| {
        let mut router = reset_btn_router.lock().unwrap();
        router.clear(&reset_btn_session);
        router.apply_preset();
        crate::app::update_input_ui_mappings(&reset_btn_window, &router);
    });

    // Palette callbacks
    let palette_session = Arc::clone(&session);
    let palette_window = aux_windows.palette().clone_strong();
    aux_windows.palette().on_palette_changed(move |index| {
        let kinds = nesium_core::ppu::palette::PaletteKind::all();
        if let Some(kind) = kinds.get(index as usize) {
            let error = match palette_session.set_palette_kind(*kind) {
                Ok(()) => "".into(),
                Err(err) => err.to_string().into(),
            };
            palette_window.set_palette_error(error);
        }
    });

    let load_ext_palette_session = Arc::clone(&session);
    let load_ext_palette_window = aux_windows.palette().clone_strong();
    aux_windows.palette().on_load_external_palette(move || {
        if let Some(path) = pick_palette_path() {
            let error = match load_ext_palette_session.set_palette_from_pal_file(&path) {
                Ok(()) => "".into(),
                Err(err) => err.to_string().into(),
            };
            load_ext_palette_window.set_palette_error(error);
        }
    });

    // Audio callbacks
    let audio_session = Arc::clone(&session);
    let audio_window = aux_windows.audio().clone_strong();
    aux_windows.audio().on_audio_config_changed(move || {
        let mut eq_gains = [0.0; 20];
        eq_gains.fill(audio_window.get_eq_gain_db() as f32);

        let config = nesium_core::audio::bus::AudioBusConfig {
            master_volume: audio_window.get_master_volume_percent() as f32 / 100.0,
            mute_in_background: audio_window.get_mute_in_background(),
            reduce_in_background: audio_window.get_reduce_in_background(),
            reduce_in_fast_forward: audio_window.get_reduce_in_fast_forward(),
            volume_reduction: audio_window.get_volume_reduction_percent() as f32 / 100.0,
            in_background: false,
            is_fast_forward: false,
            enable_equalizer: audio_window.get_eq_enabled(),
            eq_band_gains: eq_gains,
            reverb_enabled: audio_window.get_reverb_enabled(),
            reverb_strength: audio_window.get_reverb_strength_percent() as f32 / 100.0,
            reverb_delay_ms: audio_window.get_reverb_delay_ms() as f32,
            crossfeed_enabled: audio_window.get_crossfeed_enabled(),
            crossfeed_ratio: audio_window.get_crossfeed_ratio_percent() as f32 / 100.0,
        };
        let _ = audio_session.set_audio_config(config);
    });

    // Input Window Turbos callback
    let input_window_session = Arc::clone(&session);
    aux_windows
        .input()
        .on_turbo_timing_changed(move |on_frames, off_frames| {
            input_window_session.set_turbo_timing(on_frames as u8, off_frames as u8);
        });

    // ROM drag-and-drop handler
    let window_weak = window.as_weak();
    let drop_session = Arc::clone(&session);
    let drop_input = Arc::clone(&input_router);
    let drop_aux_windows = Arc::clone(&aux_windows);
    window.on_rom_dropped(move |path_str| {
        let Some(window) = window_weak.upgrade() else {
            return;
        };
        let path = PathBuf::from(path_str.as_str());

        match drop_session.load_rom(&path) {
            Ok(()) => {
                drop_input
                    .lock()
                    .expect("input router mutex poisoned")
                    .clear(&drop_session);
                let rom_name = rom_name_for_path(&path);
                window.set_has_rom(true);
                window.set_paused(false);
                window.set_rom_name(rom_name.clone());
                window.set_fps_text(SharedString::from("FPS: --"));
                window.set_status_text(SharedString::from(format!("Loaded {}", path.display())));
                drop_aux_windows.set_rom_state(rom_name, true);
            }
            Err(err) => {
                window.set_error_message(SharedString::from(format!(
                    "Failed to load ROM:\n{err}"
                )));
            }
        }
    });

    // Error dismissed — no extra action needed, the .slint side already clears error-message.
    window.on_error_dismissed(|| {});

    window.on_quit_requested(move || {
        let _ = slint::quit_event_loop();
    });
}

fn install_drop_handler(window: &MainWindow) {
    let window_weak = window.as_weak();
    window.window().on_winit_window_event(move |_winit_window, event| {
        use slint::winit_030::{self, winit};

        if let winit::event::WindowEvent::DroppedFile(path) = event {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            if ext == "nes" || ext == "fds" {
                let path_str = path.to_string_lossy().to_string();
                let weak = window_weak.clone();
                let _ = weak.upgrade_in_event_loop(move |window| {
                    window.invoke_rom_dropped(SharedString::from(path_str));
                });
                return winit_030::EventResult::PreventDefault;
            }
        }
        winit_030::EventResult::Propagate
    });
}

fn install_frame_ready_callback(
    window: &MainWindow,
    session: Arc<RuntimeSession>,
    redraw_pending: Arc<AtomicBool>,
) -> Result<FrameReadyRegistration> {
    let bridge = Box::new(FrameReadyBridge {
        window: window.as_weak(),
        redraw_pending,
    });
    let user_data = Box::into_raw(bridge);

    session
        .set_frame_ready_callback(Some(frame_ready_callback), user_data.cast::<c_void>())
        .context("failed to install frame-ready callback")?;

    Ok(FrameReadyRegistration { session, user_data })
}

fn pick_rom_path() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("NES ROM", &["nes", "fds"])
        .pick_file()
}

fn pick_palette_path() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("NES Palette", &["pal"])
        .pick_file()
}

fn rom_name_for_path(path: &Path) -> SharedString {
    SharedString::from(
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Loaded ROM"),
    )
}

pub fn update_input_ui_mappings(window: &crate::InputWindow, router: &input::InputRouter) {
    let button_names = window.get_button_names();
    let count = button_names.row_count();

    let mut p0 = vec![slint::SharedString::default(); count];
    let mut p1 = vec![slint::SharedString::default(); count];

    for (slint_key, (port, mapping)) in &router.key_bindings {
        let name_to_find = match (mapping.button, mapping.turbo) {
            (nesium_core::controller::Button::A, false) => "A",
            (nesium_core::controller::Button::A, true) => "Turbo A",
            (nesium_core::controller::Button::B, false) => "B",
            (nesium_core::controller::Button::B, true) => "Turbo B",
            (nesium_core::controller::Button::Up, _) => "Up",
            (nesium_core::controller::Button::Down, _) => "Down",
            (nesium_core::controller::Button::Left, _) => "Left",
            (nesium_core::controller::Button::Right, _) => "Right",
            (nesium_core::controller::Button::Start, _) => "Start",
            (nesium_core::controller::Button::Select, _) => "Select",
        };

        for i in 0..count {
            if let Some(target) = button_names.row_data(i) {
                if target.as_str() == name_to_find {
                    if *port == 0 {
                        p0[i] = slint_key.clone();
                    } else if *port == 1 {
                        p1[i] = slint_key.clone();
                    }
                    break;
                }
            }
        }
    }

    window.set_p0_mappings(std::rc::Rc::new(slint::VecModel::from(p0)).into());
    window.set_p1_mappings(std::rc::Rc::new(slint::VecModel::from(p1)).into());
}

extern "C" fn frame_ready_callback(
    _buffer_index: u32,
    _width: u32,
    _height: u32,
    _pitch: u32,
    user_data: *mut c_void,
) {
    let Some(bridge) =
        (!user_data.is_null()).then(|| unsafe { &*(user_data.cast::<FrameReadyBridge>()) })
    else {
        return;
    };

    if bridge.redraw_pending.swap(true, Ordering::AcqRel) {
        return;
    }

    let window_weak = bridge.window.clone();
    let _ = window_weak.upgrade_in_event_loop(move |window| {
        window.window().request_redraw();
    });
}
