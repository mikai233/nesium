// Gamepad support for Web platform.
//
// On Web, gamepad support can be implemented using the browser's Gamepad API.
// For now, this is a stub that returns empty values.
// TODO: Implement browser Gamepad API integration in nes_worker.js.

import 'dart:async';

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

// === Dart-friendly types (same as IO version) ===

/// Result of polling gamepads.
class GamepadPollResult {
  final List<int> padMasks;
  final List<int> turboMasks;
  final GamepadActions actions;

  const GamepadPollResult({
    required this.padMasks,
    required this.turboMasks,
    required this.actions,
  });
}

/// Extended gamepad actions.
class GamepadActions {
  final bool rewind;
  final bool fastForward;
  final bool saveState;
  final bool loadState;
  final bool pause;

  const GamepadActions({
    required this.rewind,
    required this.fastForward,
    required this.saveState,
    required this.loadState,
    required this.pause,
  });
}

/// Information about a connected gamepad.
class GamepadInfo {
  final int id;
  final String name;
  final bool connected;
  final int? port;

  const GamepadInfo({
    required this.id,
    required this.name,
    required this.connected,
    this.port,
  });
}
