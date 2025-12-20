use super::gamepad::GamepadManager;
use eframe::egui::{Context as EguiContext, Event, Key};
use gilrs::{Button as GilrsButton, GamepadId};
use nesium_core::{Nes, controller::Button};

/// Logical input device for a single NES controller port.
///
/// Currently only keyboard input is implemented. Gamepad support can be
/// added later without changing the public NES core API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ControllerDevice {
    Disabled,
    #[default]
    Keyboard,
    Gamepad(GamepadId),
}

#[derive(Default, Clone)]
pub struct ControllerInput {
    pressed: Vec<Button>,
    /// Per-NES-button key bindings.
    bindings: Vec<(Button, Option<Key>)>,
    /// Per-NES-button gamepad bindings.
    gamepad_bindings: Vec<(Button, Option<GilrsButton>)>,
    /// If set, the next key press will be bound to this NES button.
    capture_target: Option<Button>,
}

impl ControllerInput {
    pub fn new_with_defaults() -> Self {
        let mut input = Self {
            pressed: Vec::new(),
            bindings: Vec::new(),
            gamepad_bindings: Vec::new(),
            capture_target: None,
        };
        input.apply_preset(InputPreset::NesStandard);
        input
    }

    /// Applies the current internal pressed state to the NES core.
    ///
    /// This is used by the emulator thread to sync inputs that were resolved
    /// on the UI thread.
    pub fn apply_to_nes(&self, nes: &mut Nes, pad_index: usize) {
        // We can't efficiently "diff" against the NES state here without reading it back,
        // so we just clear all likely buttons and set the pressed ones.
        // Or simpler: The NES core `set_button` is cheap.
        // Ideally we would iterate all standard buttons and set them.
        // But `pressed` only contains the pressed ones.
        // We need to ensure *released* buttons are also updated.
        // The previous `sync_from_input` logic was:
        // "Release all previous buttons for this pad (from `self.pressed`), then apply desired."
        //
        // But here `self` IS the desired state (sent from UI).
        // The issue is `nes` might have old state.
        //
        // Approach: Reset ALL buttons for this port, then apply pressed.
        // `nes.set_button` usually just updates a bitmask.
        //
        // Standard NES buttons: A, B, Select, Start, Up, Down, Left, Right.
        use nesium_core::controller::Button::*;
        let all_buttons = [A, B, Select, Start, Up, Down, Left, Right];

        for btn in all_buttons {
            let is_pressed = self.pressed.contains(&btn);
            nes.set_button(pad_index, btn, is_pressed);
        }
    }

    pub fn sync_from_input(
        &mut self,
        ctx: &EguiContext,
        _pad_index: usize, // Kept for API compatibility/logging if needed, but unused for now
        keyboard_blocked: bool,
    ) {
        // When in capture mode, listen for the next key press and update the
        // corresponding binding. Pressing Escape clears the binding.
        if let Some(target) = self.capture_target {
            let mut captured: Option<Key> = None;
            ctx.input(|i| {
                for ev in &i.events {
                    if let Event::Key { key, pressed, .. } = ev
                        && *pressed
                    {
                        captured = Some(*key);
                    }
                }
            });

            if let Some(key) = captured {
                let new_binding = if key == Key::Escape { None } else { Some(key) };
                if let Some((_, slot)) = self.bindings.iter_mut().find(|(btn, _)| *btn == target) {
                    *slot = new_binding;
                }
                self.capture_target = None;
            }
        }

        let keys = ctx.input(|i| i.keys_down.clone());
        let mut desired: Vec<Button> = Vec::new();

        if !keyboard_blocked {
            for (button, key_opt) in &self.bindings {
                if let Some(key) = key_opt
                    && keys.contains(key)
                    && !desired.contains(button)
                {
                    desired.push(*button);
                }
            }
        }

        self.pressed = desired;
    }

    pub fn sync_from_gamepad(
        &mut self,
        _pad_index: usize,
        gamepads: &GamepadManager,
        gamepad_id: GamepadId,
    ) {
        let mut desired: Vec<Button> = Vec::new();

        for (button, binding) in &self.gamepad_bindings {
            if let Some(gb) = binding
                && gamepads.is_pressed(gamepad_id, *gb)
            {
                desired.push(*button);
            }
        }

        self.pressed = desired;
    }

    pub fn release_all(&mut self) {
        self.pressed.clear();
    }

    pub fn is_pressed(&self, button: Button) -> bool {
        self.pressed.contains(&button)
    }

    pub fn pressed_mask(&self) -> u8 {
        let mut mask: u8 = 0;
        for &button in &self.pressed {
            mask |= 1u8 << button_bit(button);
        }
        mask
    }

    /// Current key binding for a given NES button.
    pub fn binding_for(&self, button: Button) -> Option<Key> {
        self.bindings
            .iter()
            .find_map(|(b, key)| if *b == button { *key } else { None })
    }

    /// Start capturing the next key press to bind to `button`.
    pub fn begin_capture(&mut self, button: Button) {
        self.capture_target = Some(button);
    }

    /// NES button currently waiting for a new binding, if any.
    pub fn capture_target(&self) -> Option<Button> {
        self.capture_target
    }

    /// Cancel any pending key-capture operation.
    pub fn clear_capture(&mut self) {
        self.capture_target = None;
    }

    /// Current gamepad binding for a given NES button.
    pub fn gamepad_binding_for(&self, button: Button) -> Option<GilrsButton> {
        self.gamepad_bindings
            .iter()
            .find_map(|(b, btn)| if *b == button { *btn } else { None })
    }

    /// Set the gamepad binding for a given NES button.
    pub fn set_gamepad_binding(&mut self, button: Button, binding: Option<GilrsButton>) {
        if let Some((_, b)) = self
            .gamepad_bindings
            .iter_mut()
            .find(|(btn, _)| *btn == button)
        {
            *b = binding;
        } else {
            self.gamepad_bindings.push((button, binding));
        }
    }
}

fn button_bit(button: Button) -> u8 {
    match button {
        Button::A => 0,
        Button::B => 1,
        Button::Select => 2,
        Button::Start => 3,
        Button::Up => 4,
        Button::Down => 5,
        Button::Left => 6,
        Button::Right => 7,
    }
}

pub fn format_button_name(button: Button) -> &'static str {
    match button {
        Button::A => "A",
        Button::B => "B",
        Button::Select => "Select",
        Button::Start => "Start",
        Button::Up => "Up",
        Button::Down => "Down",
        Button::Left => "Left",
        Button::Right => "Right",
    }
}

/// High level presets for keyboard + gamepad layouts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputPreset {
    NesStandard,
    FightStick,
    ArcadeLayout,
}

impl ControllerInput {
    pub fn apply_preset(&mut self, preset: InputPreset) {
        use Button::*;

        // Keyboard defaults.
        self.bindings.clear();
        match preset {
            InputPreset::NesStandard => {
                self.bindings.extend_from_slice(&[
                    (Up, Some(Key::ArrowUp)),
                    (Down, Some(Key::ArrowDown)),
                    (Left, Some(Key::ArrowLeft)),
                    (Right, Some(Key::ArrowRight)),
                    (A, Some(Key::Z)),
                    (B, Some(Key::X)),
                    (Start, Some(Key::Enter)),
                    (Select, Some(Key::Space)),
                ]);
            }
            InputPreset::FightStick => {
                // WASD for directions, J/K for punches (A/B), Enter/RightShift for Start/Select.
                self.bindings.extend_from_slice(&[
                    (Up, Some(Key::W)),
                    (Down, Some(Key::S)),
                    (Left, Some(Key::A)),
                    (Right, Some(Key::D)),
                    (A, Some(Key::J)),
                    (B, Some(Key::K)),
                    (Start, Some(Key::Enter)),
                    (Select, Some(Key::Space)),
                ]);
            }
            InputPreset::ArcadeLayout => {
                // Arrow keys + H/J for A/B, Enter/Space for Start/Select.
                self.bindings.extend_from_slice(&[
                    (Up, Some(Key::ArrowUp)),
                    (Down, Some(Key::ArrowDown)),
                    (Left, Some(Key::ArrowLeft)),
                    (Right, Some(Key::ArrowRight)),
                    (A, Some(Key::J)),
                    (B, Some(Key::H)),
                    (Start, Some(Key::Enter)),
                    (Select, Some(Key::Space)),
                ]);
            }
        }

        // Gamepad defaults.
        self.gamepad_bindings.clear();
        self.gamepad_bindings.extend_from_slice(&[
            (Up, Some(GilrsButton::DPadUp)),
            (Down, Some(GilrsButton::DPadDown)),
            (Left, Some(GilrsButton::DPadLeft)),
            (Right, Some(GilrsButton::DPadRight)),
            // Face buttons: South = A, East = B (Nintendo-style).
            (A, Some(GilrsButton::South)),
            (B, Some(GilrsButton::East)),
            (Start, Some(GilrsButton::Start)),
            (Select, Some(GilrsButton::Select)),
        ]);
    }
}
