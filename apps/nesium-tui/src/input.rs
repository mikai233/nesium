use std::collections::HashMap;
use std::time::{Duration, Instant};

use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use nesium_core::{controller::Button, reset_kind::ResetKind};
use nesium_runtime::Runtime;

pub enum AppAction {
    Quit,
    Reset,
    None,
}

pub struct InputManager {
    /// Tracks the last time a key was seen pressed.
    pressed_keys: HashMap<KeyCode, Instant>,
    /// Time without a repeat event after which a key is considered released.
    release_timeout: Duration,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            pressed_keys: HashMap::new(),
            // 75ms is a reasonable balance. 
            // Too short -> keys flicker off between repeats.
            // Too long -> input lag on release.
            // Typical repeat rates are 30-50ms.
            // Increased to 200ms to handle slower terminals/SSH.
            release_timeout: Duration::from_millis(200),
        }
    }

    pub fn handle_event(&mut self, key: KeyEvent, runtime: &Option<Runtime>) -> AppAction {
        // We accept Press and Repeat.
        let is_active = key.kind == KeyEventKind::Press || key.kind == KeyEventKind::Repeat;
        
        // Explicit release event (if supported by terminal)
        if key.kind == KeyEventKind::Release {
            if let Some(rt) = runtime {
                self.set_game_key(rt, key.code, false);
            }
            self.pressed_keys.remove(&key.code);
            return AppAction::None;
        }

        if !is_active {
            return AppAction::None;
        }

        // Handle System Keys (Quit, Reset) immediately
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return AppAction::Quit,
            KeyCode::Char('r') => {
                if let Some(rt) = runtime {
                    let _ = rt.handle().reset(ResetKind::Soft);
                }
                return AppAction::Reset;
            }
            _ => {}
        }

        // Handle Game Keys
        // Mark as pressed and update timestamp
        if let Some(rt) = runtime {
            if self.map_button(key.code).is_some() {
                self.pressed_keys.insert(key.code, Instant::now());
                self.set_game_key(rt, key.code, true);
            }
        }

        AppAction::None
    }

    /// Called every frame to check for stale keys
    pub fn update(&mut self, runtime: &Option<Runtime>) {
        let Some(rt) = runtime else { return };
        let now = Instant::now();

        // Identify keys that have timed out
        let mut to_remove = Vec::new();
        for (code, last_seen) in &self.pressed_keys {
            if now.duration_since(*last_seen) > self.release_timeout {
                to_remove.push(*code);
            }
        }

        // Release them
        for code in to_remove {
            self.pressed_keys.remove(&code);
            self.set_game_key(rt, code, false);
        }
    }

    fn set_game_key(&self, runtime: &Runtime, code: KeyCode, pressed: bool) {
        if let Some(btn) = self.map_button(code) {
             runtime.handle().set_button(0, btn, pressed);
        }
    }

    fn map_button(&self, code: KeyCode) -> Option<Button> {
        match code {
            KeyCode::Up => Some(Button::Up),
            KeyCode::Down => Some(Button::Down),
            KeyCode::Left => Some(Button::Left),
            KeyCode::Right => Some(Button::Right),
            KeyCode::Char('z') => Some(Button::A),
            KeyCode::Char('x') => Some(Button::B),
            KeyCode::Enter => Some(Button::Start),
            KeyCode::Char(' ') => Some(Button::Select),
            _ => None,
        }
    }
}