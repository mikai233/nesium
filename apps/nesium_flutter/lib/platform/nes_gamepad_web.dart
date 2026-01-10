// Gamepad support for Web platform.
//
// On Web, gamepad support can be implemented using the browser's Gamepad API.
// For now, this is a stub that returns empty values.
// TODO: Implement browser Gamepad API integration in nes_worker.js.

import 'nes_gamepad_types.dart';

export 'nes_gamepad_types.dart';

/// Whether gamepad support is available on this platform.
bool get isGamepadSupported =>
    false; // TODO: Enable after implementing browser API

/// Initializes the gamepad subsystem.
Future<void> initGamepad() async {
  // Web: No-op, browser handles gamepad natively
}

/// Shuts down the gamepad subsystem.
Future<void> shutdownGamepad() async {
  // Web: No-op
}

/// Polls all connected gamepads and returns the current input state.
Future<GamepadPollResult?> pollGamepads() async {
  // TODO: Implement via JS interop with navigator.getGamepads()
  return null;
}

/// Returns information about all connected gamepads.
Future<List<GamepadInfo>> listGamepads() async {
  // TODO: Implement via JS interop
  return [];
}

/// Triggers vibration on the gamepad assigned to the given port.
Future<void> rumbleGamepad({
  required int port,
  required double strength,
  required int durationMs,
}) async {
  // TODO: Implement via Gamepad.vibrationActuator
}

/// Manually binds a gamepad to a NES port.
Future<void> bindGamepad({required int id, int? port}) async {
  // Web: No-op
}

/// Returns the current button mapping for a NES port.
Future<GamepadMapping?> getGamepadMapping(int port) async {
  // Web: Stub
  return null;
}

/// Returns a list of currently pressed buttons on a gamepad.
Future<List<GamepadButton>> getGamepadPressedButtons(int id) async {
  // Web: Stub
  return [];
}

/// Sets a custom button mapping for a NES port.
Future<void> setGamepadMapping(int port, GamepadMapping mapping) async {
  // Web: Stub
}
