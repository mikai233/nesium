use slint::SharedString;
use std::collections::HashMap;

use nesium_core::controller::Button;

use crate::runtime::RuntimeSession;

#[derive(Clone, Debug)]
pub struct KeyMapping {
    pub button: Button,
    pub turbo: bool,
}

pub struct InputRouter {
    kb_mask: [u8; 2],
    gp_mask: [u8; 2],
    turbo_mask: [u8; 2],
    pub key_bindings: HashMap<SharedString, (usize, KeyMapping)>,
}

impl InputRouter {
    pub fn new() -> Self {
        let mut router = Self {
            kb_mask: [0; 2],
            gp_mask: [0; 2],
            turbo_mask: [0; 2],
            key_bindings: HashMap::new(),
        };
        router.apply_preset();
        router
    }

    pub fn apply_preset(&mut self) {
        self.key_bindings.clear();
        // Default player 1 mapping
        self.bind_key(0, "\u{0011}", Button::Up, false); // UpArrow
        self.bind_key(0, "\u{0013}", Button::Down, false); // DownArrow
        self.bind_key(0, "\u{0012}", Button::Left, false); // LeftArrow
        self.bind_key(0, "\u{0014}", Button::Right, false); // RightArrow
        self.bind_key(0, "Z", Button::A, false);
        self.bind_key(0, "z", Button::A, false);
        self.bind_key(0, "X", Button::B, false);
        self.bind_key(0, "x", Button::B, false);
        self.bind_key(0, "C", Button::A, true);
        self.bind_key(0, "c", Button::A, true);
        self.bind_key(0, "V", Button::B, true);
        self.bind_key(0, "v", Button::B, true);
        self.bind_key(0, "\n", Button::Start, false); // Return
        self.bind_key(0, " ", Button::Select, false); // Space
    }

    pub fn bind_key(&mut self, port: usize, key: &str, button: Button, turbo: bool) {
        // Automatically unbind previous binding for this key if it exists
        self.key_bindings
            .insert(key.into(), (port, KeyMapping { button, turbo }));
    }

    pub fn clear(&mut self, session: &RuntimeSession) {
        self.kb_mask = [0; 2];
        self.turbo_mask = [0; 2];
        session.set_pad_mask(0, self.kb_mask[0] | self.gp_mask[0]);
        session.set_turbo_mask(0, 0);
        session.set_pad_mask(1, self.kb_mask[1] | self.gp_mask[1]);
        session.set_turbo_mask(1, 0);
    }

    pub fn handle_key(&mut self, session: &RuntimeSession, key: &str, pressed: bool) -> bool {
        let mapping = self.key_bindings.get(key).cloned();

        let Some((port, mapped)) = mapping else {
            return false;
        };

        if port > 1 {
            return false;
        }

        let bit = 1u8 << button_bit(mapped.button);
        let target = if mapped.turbo {
            &mut self.turbo_mask[port]
        } else {
            &mut self.kb_mask[port]
        };
        let previous = *target;

        if pressed {
            *target |= bit;
        } else {
            *target &= !bit;
        }

        if *target == previous {
            return true;
        }

        if mapped.turbo {
            session.set_turbo_mask(port, self.turbo_mask[port]);
        } else {
            session.set_pad_mask(port, self.kb_mask[port] | self.gp_mask[port]);
        }

        true
    }

    pub fn update_gamepad_mask(&mut self, session: &RuntimeSession, port: usize, mask: u8) {
        if port > 1 {
            return;
        }
        if self.gp_mask[port] == mask {
            return;
        }
        self.gp_mask[port] = mask;
        session.set_pad_mask(port, self.kb_mask[port] | self.gp_mask[port]);
    }
}

pub fn button_from_name(name: &str) -> Option<Button> {
    match name {
        "Up" => Some(Button::Up),
        "Down" => Some(Button::Down),
        "Left" => Some(Button::Left),
        "Right" => Some(Button::Right),
        "A" | "Turbo A" => Some(Button::A),
        "B" | "Turbo B" => Some(Button::B),
        "Start" => Some(Button::Start),
        "Select" => Some(Button::Select),
        _ => None,
    }
}

pub fn button_bit(button: Button) -> u8 {
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
