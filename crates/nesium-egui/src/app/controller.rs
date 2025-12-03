use eframe::egui::{Context as EguiContext, Key};
use nesium_core::{Nes, controller::Button};

#[derive(Default)]
pub struct ControllerInput {
    pressed: Vec<Button>,
}

impl ControllerInput {
    pub fn sync_from_input(&mut self, ctx: &EguiContext, nes: &mut Nes, keyboard_blocked: bool) {
        let keys = ctx.input(|i| i.keys_down.clone());
        let mut desired: Vec<Button> = Vec::new();

        if !keyboard_blocked {
            for key in keys {
                if let Some(button) = map_key(key) {
                    if !desired.contains(&button) {
                        desired.push(button);
                    }
                }
            }
        }

        // Release all, then re-apply desired. Simple and keeps in sync.
        for button in self.pressed.drain(..) {
            nes.set_button(0, button, false);
        }
        for &button in &desired {
            nes.set_button(0, button, true);
        }
        self.pressed = desired;
    }

    pub fn release_all(&mut self, nes: &mut Nes) {
        for button in self.pressed.drain(..) {
            nes.set_button(0, button, false);
        }
    }

    pub fn is_pressed(&self, button: Button) -> bool {
        self.pressed.iter().any(|b| *b == button)
    }
}

fn map_key(key: Key) -> Option<Button> {
    match key {
        Key::Z => Some(Button::A),
        Key::X => Some(Button::B),
        Key::Enter => Some(Button::Start),
        Key::Space | Key::C => Some(Button::Select),
        Key::ArrowUp => Some(Button::Up),
        Key::ArrowDown => Some(Button::Down),
        Key::ArrowLeft => Some(Button::Left),
        Key::ArrowRight => Some(Button::Right),
        _ => None,
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
