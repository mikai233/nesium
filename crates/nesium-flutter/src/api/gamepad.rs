//! Gamepad input handling via FFI for desktop platforms.
//!
//! This module exposes gilrs-based gamepad functionality to Flutter.

use flutter_rust_bridge::frb;
use std::sync::Mutex;
use std::time::Duration;

use nesium_support::gamepad::{GamepadActions, GamepadInfo, GamepadManager, GamepadPollResult};

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::{self, JoinHandle};

static GAMEPAD_MANAGER: Mutex<Option<GamepadManager>> = Mutex::new(None);
static GAMEPAD_LOOP_STOP: AtomicBool = AtomicBool::new(false);
static POLLING_THREAD: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);

/// Initializes the gamepad subsystem and starts the background polling thread.
///
/// Call this once at app startup. Returns an error if gilrs fails to initialize.
#[frb]
pub fn init_gamepad() -> Result<(), String> {
    let mut manager = GAMEPAD_MANAGER.lock().unwrap();
    if manager.is_some() {
        return Ok(()); // Already initialized
    }

    let gm = GamepadManager::new().map_err(|e| e.to_string())?;
    *manager = Some(gm);

    // Start background polling thread
    let mut thread_guard = POLLING_THREAD.lock().unwrap();
    if thread_guard.is_none() {
        GAMEPAD_LOOP_STOP.store(false, Ordering::Release);
        *thread_guard = Some(thread::spawn(move || {
            tracing::info!("Starting gamepad polling thread (Rust)");
            loop {
                if GAMEPAD_LOOP_STOP.load(Ordering::Acquire) {
                    tracing::info!("Gamepad polling thread stopping");
                    break;
                }

                // Lock and poll
                {
                    let mut gm_lock = GAMEPAD_MANAGER.lock().unwrap();
                    if let Some(gm) = gm_lock.as_mut() {
                        let result = gm.poll();

                        // Update masks in input API
                        for port in 0..2 {
                            crate::api::input::set_gamepad_masks(
                                port,
                                result.pad_masks[port],
                                result.turbo_masks[port],
                            );
                        }

                        // Handle actions
                        let handle = crate::runtime_handle();
                        if result.actions.save_state {
                            // TODO: This uses a hardcoded slot/path if we want to support it cleanly
                            // For now maybe we don't trigger save/load automatically without a path
                        }
                        if result.actions.pause {
                            handle.set_paused(!handle.paused());
                        }
                        // Rewind/FastForward would need more logic or just set state
                    } else {
                        // Manager gone? Stop loop.
                        break;
                    }
                }

                // Poll at ~250Hz
                thread::sleep(Duration::from_millis(4));
            }
        }));
    }

    Ok(())
}

/// Polls all connected gamepads and returns the current input state.
///
/// Returns NES button masks, turbo button masks, and extended actions.
/// Call this once per frame.
#[frb]
pub fn poll_gamepads() -> Result<GamepadPollResultFfi, String> {
    let mut manager = GAMEPAD_MANAGER.lock().unwrap();
    let gm = manager.as_mut().ok_or("Gamepad not initialized")?;

    let result = gm.poll();
    Ok(GamepadPollResultFfi::from(result))
}

/// Returns information about all connected gamepads.
#[frb]
pub fn list_gamepads() -> Result<Vec<GamepadInfoFfi>, String> {
    let mut manager = GAMEPAD_MANAGER.lock().unwrap();
    let gm = manager.as_mut().ok_or("Gamepad not initialized")?;

    // Pump events before listing to ensure new connections are picked up
    let _ = gm.poll();

    let list = gm.gamepads();
    Ok(list.into_iter().map(GamepadInfoFfi::from).collect())
}

/// Triggers vibration on the gamepad assigned to the given port.
///
/// - `port`: NES port (0 or 1)
/// - `strength`: Vibration strength (0.0 to 1.0)
/// - `duration_ms`: How long to vibrate in milliseconds
#[frb]
pub fn rumble_gamepad(port: u8, strength: f32, duration_ms: u32) -> Result<(), String> {
    let mut manager = GAMEPAD_MANAGER.lock().unwrap();
    let gm = manager.as_mut().ok_or("Gamepad not initialized")?;

    gm.rumble(
        port as usize,
        strength,
        Duration::from_millis(duration_ms as u64),
    )
    .map_err(|e| e.to_string())
}

/// Manually binds a gamepad to a NES port.
///
/// - `id`: The gamepad ID (from GamepadInfoFfi).
/// - `port`: 0 for Player 1, 1 for Player 2, or null to unbind.
#[frb]
pub fn bind_gamepad(id: u64, port: Option<u8>) -> Result<(), String> {
    let mut manager = GAMEPAD_MANAGER.lock().unwrap();
    let gm = manager.as_mut().ok_or("Gamepad not initialized")?;

    gm.bind_gamepad(id as usize, port.map(|p| p as usize));
    Ok(())
}

/// Shuts down the gamepad subsystem.
#[frb]
pub fn shutdown_gamepad() {
    GAMEPAD_LOOP_STOP.store(true, Ordering::Release);

    let thread = POLLING_THREAD.lock().unwrap().take();
    if let Some(handle) = thread {
        let _ = handle.join();
    }

    let mut manager = GAMEPAD_MANAGER.lock().unwrap();
    *manager = None;
}

/// Returns a list of buttons currently pressed on the given gamepad.
#[frb]
pub fn get_gamepad_pressed_buttons(id: u64) -> Result<Vec<GamepadButtonFfi>, String> {
    let manager = GAMEPAD_MANAGER.lock().unwrap();
    let gm = manager.as_ref().ok_or("Gamepad not initialized")?;

    let buttons = gm.get_pressed_buttons(id as usize);
    Ok(buttons.into_iter().map(GamepadButtonFfi::from).collect())
}

/// Sets a custom button mapping for the given port.
#[frb]
pub fn set_gamepad_mapping(port: u8, mapping: GamepadMappingFfi) -> Result<(), String> {
    let mut manager = GAMEPAD_MANAGER.lock().unwrap();
    let gm = manager.as_mut().ok_or("Gamepad not initialized")?;

    gm.set_mapping(port as usize, mapping.into());
    Ok(())
}

// === FFI-friendly types ===

/// FFI-safe version of GamepadPollResult.
#[frb]
#[derive(Debug, Clone, Default)]
pub struct GamepadPollResultFfi {
    /// NES button masks for ports 0 and 1.
    pub pad_masks: Vec<u8>,
    /// Turbo button masks for ports 0 and 1.
    pub turbo_masks: Vec<u8>,
    /// Extended actions.
    pub actions: GamepadActionsFfi,
}

impl From<GamepadPollResult> for GamepadPollResultFfi {
    fn from(r: GamepadPollResult) -> Self {
        Self {
            pad_masks: r.pad_masks.to_vec(),
            turbo_masks: r.turbo_masks.to_vec(),
            actions: GamepadActionsFfi::from(r.actions),
        }
    }
}

/// FFI-safe version of GamepadActions.
#[frb]
#[derive(Debug, Clone, Copy, Default)]
pub struct GamepadActionsFfi {
    pub rewind: bool,
    pub fast_forward: bool,
    pub save_state: bool,
    pub load_state: bool,
    pub pause: bool,
}

impl From<GamepadActions> for GamepadActionsFfi {
    fn from(a: GamepadActions) -> Self {
        Self {
            rewind: a.rewind,
            fast_forward: a.fast_forward,
            save_state: a.save_state,
            load_state: a.load_state,
            pause: a.pause,
        }
    }
}

/// FFI-safe version of GamepadInfo.
#[frb]
#[derive(Debug, Clone)]
pub struct GamepadInfoFfi {
    pub id: u64,
    pub name: String,
    pub connected: bool,
    pub port: Option<u8>,
}

impl From<GamepadInfo> for GamepadInfoFfi {
    fn from(i: GamepadInfo) -> Self {
        Self {
            id: i.id as u64,
            name: i.name,
            connected: i.connected,
            port: i.port.map(|p| p as u8),
        }
    }
}

/// FFI-safe version of gilrs::Button.
#[frb]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamepadButtonFfi {
    South,
    East,
    North,
    West,
    C,
    Z,
    LeftTrigger,
    LeftTrigger2,
    RightTrigger,
    RightTrigger2,
    Select,
    Start,
    Mode,
    LeftThumb,
    RightThumb,
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    Unknown,
}

impl From<gilrs::Button> for GamepadButtonFfi {
    fn from(b: gilrs::Button) -> Self {
        match b {
            gilrs::Button::South => Self::South,
            gilrs::Button::East => Self::East,
            gilrs::Button::North => Self::North,
            gilrs::Button::West => Self::West,
            gilrs::Button::C => Self::C,
            gilrs::Button::Z => Self::Z,
            gilrs::Button::LeftTrigger => Self::LeftTrigger,
            gilrs::Button::LeftTrigger2 => Self::LeftTrigger2,
            gilrs::Button::RightTrigger => Self::RightTrigger,
            gilrs::Button::RightTrigger2 => Self::RightTrigger2,
            gilrs::Button::Select => Self::Select,
            gilrs::Button::Start => Self::Start,
            gilrs::Button::Mode => Self::Mode,
            gilrs::Button::LeftThumb => Self::LeftThumb,
            gilrs::Button::RightThumb => Self::RightThumb,
            gilrs::Button::DPadUp => Self::DPadUp,
            gilrs::Button::DPadDown => Self::DPadDown,
            gilrs::Button::DPadLeft => Self::DPadLeft,
            gilrs::Button::DPadRight => Self::DPadRight,
            gilrs::Button::Unknown => Self::Unknown,
        }
    }
}

impl From<GamepadButtonFfi> for gilrs::Button {
    fn from(b: GamepadButtonFfi) -> Self {
        match b {
            GamepadButtonFfi::South => Self::South,
            GamepadButtonFfi::East => Self::East,
            GamepadButtonFfi::North => Self::North,
            GamepadButtonFfi::West => Self::West,
            GamepadButtonFfi::C => Self::C,
            GamepadButtonFfi::Z => Self::Z,
            GamepadButtonFfi::LeftTrigger => Self::LeftTrigger,
            GamepadButtonFfi::LeftTrigger2 => Self::LeftTrigger2,
            GamepadButtonFfi::RightTrigger => Self::RightTrigger,
            GamepadButtonFfi::RightTrigger2 => Self::RightTrigger2,
            GamepadButtonFfi::Select => Self::Select,
            GamepadButtonFfi::Start => Self::Start,
            GamepadButtonFfi::Mode => Self::Mode,
            GamepadButtonFfi::LeftThumb => Self::LeftThumb,
            GamepadButtonFfi::RightThumb => Self::RightThumb,
            GamepadButtonFfi::DPadUp => Self::DPadUp,
            GamepadButtonFfi::DPadDown => Self::DPadDown,
            GamepadButtonFfi::DPadLeft => Self::DPadLeft,
            GamepadButtonFfi::DPadRight => Self::DPadRight,
            GamepadButtonFfi::Unknown => Self::Unknown,
        }
    }
}

/// FFI-safe version of ButtonMapping.
#[frb]
#[derive(Debug, Clone, Copy)]
pub struct GamepadMappingFfi {
    pub a: GamepadButtonFfi,
    pub b: GamepadButtonFfi,
    pub select: GamepadButtonFfi,
    pub start: GamepadButtonFfi,
    pub up: GamepadButtonFfi,
    pub down: GamepadButtonFfi,
    pub left: GamepadButtonFfi,
    pub right: GamepadButtonFfi,
    pub turbo_a: GamepadButtonFfi,
    pub turbo_b: GamepadButtonFfi,
}

impl From<nesium_support::gamepad::ButtonMapping> for GamepadMappingFfi {
    fn from(m: nesium_support::gamepad::ButtonMapping) -> Self {
        Self {
            a: GamepadButtonFfi::from(m.a),
            b: GamepadButtonFfi::from(m.b),
            select: GamepadButtonFfi::from(m.select),
            start: GamepadButtonFfi::from(m.start),
            up: GamepadButtonFfi::from(m.up),
            down: GamepadButtonFfi::from(m.down),
            left: GamepadButtonFfi::from(m.left),
            right: GamepadButtonFfi::from(m.right),
            turbo_a: GamepadButtonFfi::from(m.turbo_a),
            turbo_b: GamepadButtonFfi::from(m.turbo_b),
        }
    }
}

/// Returns the current button mapping for a NES port.
#[frb]
pub fn get_gamepad_mapping(port: u8) -> Result<GamepadMappingFfi, String> {
    let mut manager = GAMEPAD_MANAGER.lock().unwrap();
    let gm = manager.as_mut().ok_or("Gamepad not initialized")?;

    let mapping = gm
        .mapping(port as usize)
        .cloned()
        .ok_or_else(|| format!("No mapping for port {}", port))?;
    Ok(GamepadMappingFfi::from(mapping))
}

impl From<GamepadMappingFfi> for nesium_support::gamepad::ButtonMapping {
    fn from(m: GamepadMappingFfi) -> Self {
        Self {
            a: m.a.into(),
            b: m.b.into(),
            select: m.select.into(),
            start: m.start.into(),
            up: m.up.into(),
            down: m.down.into(),
            left: m.left.into(),
            right: m.right.into(),
            turbo_a: m.turbo_a.into(),
            turbo_b: m.turbo_b.into(),
        }
    }
}
