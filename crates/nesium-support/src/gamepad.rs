//! Gamepad input handling using gilrs.
//!
//! This module provides a `GamepadManager` that:
//! - Polls connected gamepads for button/axis state
//! - Auto-assigns gamepads to NES controller ports
//! - Converts gamepad input to NES button masks
//! - Supports vibration/force feedback

use std::collections::HashMap;
use std::time::Duration;

use gilrs::{
    Button, Event, EventType, GamepadId, Gilrs,
    ff::{BaseEffect, BaseEffectType, EffectBuilder, Replay, Ticks},
};

use crate::error::SupportError;

/// Maximum number of NES controller ports.
const MAX_PORTS: usize = 2;

/// Information about a connected gamepad.
#[derive(Debug, Clone)]
pub struct GamepadInfo {
    /// Unique identifier for this gamepad session.
    pub id: usize,
    /// Human-readable name of the gamepad.
    pub name: String,
    /// Whether this gamepad is currently connected.
    pub connected: bool,
    /// Which NES port this gamepad is assigned to (if any).
    pub port: Option<usize>,
}

/// NES button mask bits (matches nesium-core controller format).
pub mod button_mask {
    pub const A: u8 = 0x01;
    pub const B: u8 = 0x02;
    pub const SELECT: u8 = 0x04;
    pub const START: u8 = 0x08;
    pub const UP: u8 = 0x10;
    pub const DOWN: u8 = 0x20;
    pub const LEFT: u8 = 0x40;
    pub const RIGHT: u8 = 0x80;
}

/// Turbo button mask bits (separate from NES standard buttons).
pub mod turbo_mask {
    pub const TURBO_A: u8 = 0x01;
    pub const TURBO_B: u8 = 0x02;
}

/// Default button mapping from gilrs buttons to NES buttons.
#[derive(Debug, Clone)]
pub struct ButtonMapping {
    // Standard NES buttons
    pub a: Option<Button>,
    pub b: Option<Button>,
    pub select: Option<Button>,
    pub start: Option<Button>,
    pub up: Option<Button>,
    pub down: Option<Button>,
    pub left: Option<Button>,
    pub right: Option<Button>,
    // Turbo buttons
    pub turbo_a: Option<Button>,
    pub turbo_b: Option<Button>,
}

impl Default for ButtonMapping {
    fn default() -> Self {
        Self {
            a: Some(Button::South), // Xbox A / PS Cross
            b: Some(Button::East),  // Xbox B / PS Circle
            select: Some(Button::Select),
            start: Some(Button::Start),
            up: Some(Button::DPadUp),
            down: Some(Button::DPadDown),
            left: Some(Button::DPadLeft),
            right: Some(Button::DPadRight),
            turbo_a: Some(Button::North), // Y / △ (Xbox Y / PS Triangle)
            turbo_b: Some(Button::West),  // X / □ (Xbox X / PS Square)
        }
    }
}

/// Extended gamepad actions beyond NES controller buttons.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct GamepadActions {
    /// Rewind emulation state.
    pub rewind: bool,
    /// Fast forward emulation.
    pub fast_forward: bool,
    /// Save state to slot.
    pub save_state: bool,
    /// Load state from slot.
    pub load_state: bool,
    /// Toggle pause.
    pub pause: bool,
}

/// Mapping for extended actions.
#[derive(Debug, Clone)]
pub struct ActionMapping {
    pub rewind: Option<Button>,
    pub fast_forward: Option<Button>,
    pub save_state: Option<Button>,
    pub load_state: Option<Button>,
    pub pause: Option<Button>,
}

impl Default for ActionMapping {
    fn default() -> Self {
        Self {
            rewind: Some(Button::RightTrigger2),      // RT / R2
            fast_forward: Some(Button::LeftTrigger2), // LT / L2
            save_state: None,
            load_state: None,
            pause: None,
        }
    }
}

/// Result of polling gamepads.
#[derive(Debug, Clone, Default)]
pub struct GamepadPollResult {
    /// NES button masks for each port.
    pub pad_masks: [u8; MAX_PORTS],
    /// Turbo button masks for each port.
    pub turbo_masks: [u8; MAX_PORTS],
    /// Extended actions (combined from all gamepads).
    pub actions: GamepadActions,
}

/// Manages gamepad input and converts it to NES controller state.
pub struct GamepadManager {
    gilrs: Gilrs,
    /// Maps gilrs GamepadId to assigned NES port.
    port_assignments: HashMap<GamepadId, usize>,
    /// Current button state for each NES port.
    pad_states: [u8; MAX_PORTS],
    /// Current turbo button state for each NES port.
    turbo_states: [u8; MAX_PORTS],
    /// Button mapping for each port.
    mappings: [ButtonMapping; MAX_PORTS],
    /// Action mapping for extended functions.
    action_mapping: ActionMapping,
    /// List of available port slots (for auto-assignment).
    available_ports: Vec<usize>,
}

impl GamepadManager {
    /// Creates a new GamepadManager.
    ///
    /// Initializes gilrs and scans for connected gamepads.
    pub fn new() -> Result<Self, SupportError> {
        let gilrs = Gilrs::new().map_err(|e| {
            tracing::error!("Failed to initialize gilrs: {}", e);
            SupportError::Gamepad(e.to_string())
        })?;

        let manager = Self {
            gilrs,
            port_assignments: HashMap::new(),
            pad_states: [0; MAX_PORTS],
            turbo_states: [0; MAX_PORTS],
            mappings: [ButtonMapping::default(), ButtonMapping::default()],
            action_mapping: ActionMapping::default(),
            available_ports: vec![1, 0], // Player 1 (0) will be pop()ed first
        };

        Ok(manager)
    }

    /// Polls gamepad events and updates button states.
    ///
    /// Call this once per frame. Returns NES button masks, turbo masks, and extended actions.
    pub fn poll(&mut self) -> GamepadPollResult {
        // Process all pending events
        while let Some(Event { id, event, .. }) = self.gilrs.next_event() {
            match event {
                EventType::Connected => {
                    let gp = self.gilrs.gamepad(id);
                    tracing::info!("Gamepad connected: {} ({:?})", gp.name(), id);
                }
                EventType::Disconnected => {
                    tracing::info!("Gamepad disconnected: {:?}", id);
                    self.unassign_gamepad(id);
                }
                _ => {}
            }
        }

        // Read current state from all assigned gamepads
        let mut actions = GamepadActions::default();
        for (gamepad_id, port) in &self.port_assignments {
            if let Some(gamepad) = self.gilrs.connected_gamepad(*gamepad_id) {
                let (pad, turbo) = self.read_gamepad_state(&gamepad, *port);
                self.pad_states[*port] = pad;
                self.turbo_states[*port] = turbo;

                // Read actions from any connected gamepad
                self.read_actions(&gamepad, &mut actions);
            }
        }

        GamepadPollResult {
            pad_masks: self.pad_states,
            turbo_masks: self.turbo_states,
            actions,
        }
    }

    /// Returns information about all connected gamepads.
    pub fn gamepads(&self) -> Vec<GamepadInfo> {
        self.gilrs
            .gamepads()
            .filter(|(_, gamepad)| gamepad.is_connected())
            .map(|(id, gamepad)| {
                let port = self.port_assignments.get(&id).copied();
                GamepadInfo {
                    id: id.into(),
                    name: gamepad.name().to_string(),
                    connected: gamepad.is_connected(),
                    port,
                }
            })
            .collect()
    }

    /// Returns a list of buttons that are currently pressed on the given gamepad.
    pub fn get_pressed_buttons(&self, id: usize) -> Vec<gilrs::Button> {
        if let Some((_, gamepad)) = self.gilrs.gamepads().find(|(gid, _)| {
            let gid_usize: usize = (*gid).into();
            gid_usize == id
        }) {
            let mut buttons: Vec<_> = [
                gilrs::Button::South,
                gilrs::Button::East,
                gilrs::Button::North,
                gilrs::Button::West,
                gilrs::Button::C,
                gilrs::Button::Z,
                gilrs::Button::LeftTrigger,
                gilrs::Button::LeftTrigger2,
                gilrs::Button::RightTrigger,
                gilrs::Button::RightTrigger2,
                gilrs::Button::Select,
                gilrs::Button::Start,
                gilrs::Button::Mode,
                gilrs::Button::LeftThumb,
                gilrs::Button::RightThumb,
                gilrs::Button::DPadUp,
                gilrs::Button::DPadDown,
                gilrs::Button::DPadLeft,
                gilrs::Button::DPadRight,
            ]
            .into_iter()
            .filter(|&b| gamepad.is_pressed(b))
            .collect();

            // Check axes for triggers manually if not detected as buttons
            let axis_threshold = 0.3;
            if let Some(axis) = gamepad.axis_data(gilrs::Axis::LeftZ) {
                if axis.value() > axis_threshold && !buttons.contains(&gilrs::Button::LeftTrigger2)
                {
                    buttons.push(gilrs::Button::LeftTrigger2);
                }
            }
            if let Some(axis) = gamepad.axis_data(gilrs::Axis::RightZ) {
                if axis.value() > axis_threshold && !buttons.contains(&gilrs::Button::RightTrigger2)
                {
                    buttons.push(gilrs::Button::RightTrigger2);
                }
            }

            buttons
        } else {
            Vec::new()
        }
    }

    /// Triggers vibration/rumble on the gamepad assigned to the given port.
    ///
    /// - `port`: NES port (0 or 1)
    /// - `strength`: Vibration strength (0.0 to 1.0)
    /// - `duration`: How long to vibrate
    pub fn rumble(
        &mut self,
        port: usize,
        strength: f32,
        duration: Duration,
    ) -> Result<(), SupportError> {
        if port >= MAX_PORTS {
            return Err(SupportError::Gamepad(format!("Invalid port: {}", port)));
        }

        // Find the gamepad assigned to this port
        let gamepad_id = self
            .port_assignments
            .iter()
            .find(|(_, p)| **p == port)
            .map(|(id, _)| *id);

        let Some(id) = gamepad_id else {
            return Ok(()); // No gamepad on this port, silently succeed
        };

        let strength = strength.clamp(0.0, 1.0);
        let duration_ms = duration.as_millis() as u32;

        // Build and play the effect
        let effect = EffectBuilder::new()
            .add_effect(BaseEffect {
                kind: BaseEffectType::Strong {
                    magnitude: (strength * 65535.0) as u16,
                },
                scheduling: Replay {
                    play_for: Ticks::from_ms(duration_ms),
                    ..Default::default()
                },
                ..Default::default()
            })
            .gamepads(&[id])
            .finish(&mut self.gilrs)
            .map_err(|e| {
                SupportError::Gamepad(format!("Failed to create rumble effect: {:?}", e))
            })?;

        effect
            .play()
            .map_err(|e| SupportError::Gamepad(format!("Failed to play rumble: {:?}", e)))?;

        Ok(())
    }

    /// Sets a custom button mapping for the given port.
    pub fn set_mapping(&mut self, port: usize, mapping: ButtonMapping) {
        if port < MAX_PORTS {
            self.mappings[port] = mapping;
        }
    }

    /// Gets the current button mapping for the given port.
    pub fn mapping(&self, port: usize) -> Option<&ButtonMapping> {
        self.mappings.get(port)
    }

    /// Manually assigns a gamepad to a port, or unassigns it.
    ///
    /// - `id_raw`: The raw ID from GamepadInfo.
    /// - `port`: Some(0) for Player 1, Some(1) for Player 2, or None to unassign.
    pub fn bind_gamepad(&mut self, id_raw: usize, port: Option<usize>) {
        let gamepad_id = self
            .gilrs
            .gamepads()
            .find(|(id, gamepad)| usize::from(*id) == id_raw && gamepad.is_connected())
            .map(|(id, _)| id);

        let Some(id) = gamepad_id else {
            tracing::warn!("Failed to find connected gamepad with raw ID {}", id_raw);
            return;
        };

        // 1. Unassign from current port if it has one
        self.unassign_gamepad(id);

        // 2. If assigning to a new port
        if let Some(target_port) = port {
            if target_port >= MAX_PORTS {
                tracing::error!("Invalid port index: {}", target_port);
                return;
            }

            // Kick out any other gamepad currently assigned to target_port
            let occupant = self
                .port_assignments
                .iter()
                .find(|(_, p)| **p == target_port)
                .map(|(id, _)| *id);

            if let Some(other_id) = occupant {
                tracing::info!(
                    "Unassigning previous occupant of port {} to make room",
                    target_port
                );
                self.unassign_gamepad(other_id);
            }

            // Assign new
            tracing::info!(
                "Manually binding gamepad '{}' ({:?}) to port {}",
                self.gilrs.gamepad(id).name(),
                id,
                target_port
            );
            self.port_assignments.insert(id, target_port);
            self.available_ports.retain(|&p| p != target_port);
        }
    }

    fn unassign_gamepad(&mut self, id: GamepadId) {
        if let Some(port) = self.port_assignments.remove(&id) {
            self.available_ports.push(port);
            self.available_ports.sort_by(|a, b| b.cmp(a)); // Keep lower ports available first
            self.pad_states[port] = 0; // Clear the port state
        }
    }

    fn read_gamepad_state(&self, gamepad: &gilrs::Gamepad, port: usize) -> (u8, u8) {
        let mapping = &self.mappings[port];
        let mut pad_mask = 0u8;
        let mut turbo_mask = 0u8;

        let check_btn = |btn_opt: Option<gilrs::Button>| -> bool {
            if let Some(btn) = btn_opt {
                if btn == gilrs::Button::Unknown {
                    return false;
                }
                gamepad.is_pressed(btn)
            } else {
                false
            }
        };

        // Standard NES buttons
        if check_btn(mapping.a) {
            pad_mask |= button_mask::A;
        }
        if check_btn(mapping.b) {
            pad_mask |= button_mask::B;
        }
        if check_btn(mapping.select) {
            pad_mask |= button_mask::SELECT;
        }
        if check_btn(mapping.start) {
            pad_mask |= button_mask::START;
        }
        if check_btn(mapping.up) {
            pad_mask |= button_mask::UP;
        }
        if check_btn(mapping.down) {
            pad_mask |= button_mask::DOWN;
        }
        if check_btn(mapping.left) {
            pad_mask |= button_mask::LEFT;
        }
        if check_btn(mapping.right) {
            pad_mask |= button_mask::RIGHT;
        }

        // Turbo buttons
        if check_btn(mapping.turbo_a) {
            turbo_mask |= turbo_mask::TURBO_A;
        }
        if check_btn(mapping.turbo_b) {
            turbo_mask |= turbo_mask::TURBO_B;
        }

        // Also check analog stick for D-Pad emulation
        let axis_threshold = 0.5;
        if let Some(axis) = gamepad.axis_data(gilrs::Axis::LeftStickY) {
            if axis.value() > axis_threshold {
                pad_mask |= button_mask::UP;
            } else if axis.value() < -axis_threshold {
                pad_mask |= button_mask::DOWN;
            }
        }
        if let Some(axis) = gamepad.axis_data(gilrs::Axis::LeftStickX) {
            if axis.value() > axis_threshold {
                pad_mask |= button_mask::RIGHT;
            } else if axis.value() < -axis_threshold {
                pad_mask |= button_mask::LEFT;
            }
        }

        (pad_mask, turbo_mask)
    }

    fn read_actions(&self, gamepad: &gilrs::Gamepad, actions: &mut GamepadActions) {
        let is_pressed_safe = |btn: gilrs::Button| -> bool {
            btn != gilrs::Button::Unknown && gamepad.is_pressed(btn)
        };

        if let Some(btn) = self.action_mapping.rewind {
            if is_pressed_safe(btn) {
                actions.rewind = true;
            }
        }
        if let Some(btn) = self.action_mapping.fast_forward {
            if is_pressed_safe(btn) {
                actions.fast_forward = true;
            }
        }
        if let Some(btn) = self.action_mapping.save_state {
            if is_pressed_safe(btn) {
                actions.save_state = true;
            }
        }
        if let Some(btn) = self.action_mapping.load_state {
            if is_pressed_safe(btn) {
                actions.load_state = true;
            }
        }
        if let Some(btn) = self.action_mapping.pause {
            if is_pressed_safe(btn) {
                actions.pause = true;
            }
        }
    }

    /// Sets the action mapping for extended functions.
    pub fn set_action_mapping(&mut self, mapping: ActionMapping) {
        self.action_mapping = mapping;
    }

    /// Gets the current action mapping.
    pub fn action_mapping(&self) -> &ActionMapping {
        &self.action_mapping
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_mask_values() {
        assert_eq!(button_mask::A, 0x01);
        assert_eq!(button_mask::B, 0x02);
        assert_eq!(button_mask::SELECT, 0x04);
        assert_eq!(button_mask::START, 0x08);
        assert_eq!(button_mask::UP, 0x10);
        assert_eq!(button_mask::DOWN, 0x20);
        assert_eq!(button_mask::LEFT, 0x40);
        assert_eq!(button_mask::RIGHT, 0x80);
    }

    #[test]
    fn test_default_button_mapping() {
        let mapping = ButtonMapping::default();
        assert_eq!(mapping.a, Some(Button::South));
        assert_eq!(mapping.b, Some(Button::East));
        assert_eq!(mapping.start, Some(Button::Start));
        assert_eq!(mapping.select, Some(Button::Select));
    }
}
