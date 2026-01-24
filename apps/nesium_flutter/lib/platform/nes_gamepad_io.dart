// Gamepad support for native desktop platforms (Windows, macOS, Linux).
//
// Uses gilrs via flutter_rust_bridge for gamepad input and vibration.

import 'dart:async';
import 'dart:io';

import '../bridge/api/gamepad.dart' as frb_gamepad;
import '../logging/app_logger.dart';

export 'nes_gamepad_types.dart';
import 'nes_gamepad_types.dart';

bool _loggedGetGamepadMappingError = false;
bool _loggedGetGamepadPressedButtonsError = false;

/// Whether gamepad support is available on this platform.
bool get isGamepadSupported =>
    Platform.isWindows || Platform.isMacOS || Platform.isLinux;

/// Initializes the gamepad subsystem.
///
/// Call this once at app startup after `initRustRuntime()`.
Future<void> initGamepad() async {
  if (!isGamepadSupported) return;
  await frb_gamepad.initGamepad();
}

/// Shuts down the gamepad subsystem.
Future<void> shutdownGamepad() async {
  if (!isGamepadSupported) return;
  await frb_gamepad.shutdownGamepad();
}

/// Polls all connected gamepads and returns the current input state.
///
/// Returns null if gamepad support is not available.
Future<GamepadPollResult?> pollGamepads() async {
  if (!isGamepadSupported) return null;
  final result = await frb_gamepad.pollGamepads();
  return GamepadPollResult(
    padMasks: result.padMasks,
    turboMasks: result.turboMasks,
    actions: GamepadActions(
      rewind: result.actions.rewind,
      fastForward: result.actions.fastForward,
      saveState: result.actions.saveState,
      loadState: result.actions.loadState,
      pause: result.actions.pause,
    ),
  );
}

/// Returns information about all connected gamepads.
Future<List<GamepadInfo>> listGamepads() async {
  if (!isGamepadSupported) return [];
  final result = await frb_gamepad.listGamepads();
  return result
      .map(
        (g) => GamepadInfo(
          id: g.id.toInt(),
          name: g.name,
          connected: g.connected,
          port: g.port,
        ),
      )
      .toList();
}

/// Triggers vibration on the gamepad assigned to the given port.
Future<void> rumbleGamepad({
  required int port,
  required double strength,
  required int durationMs,
}) async {
  if (!isGamepadSupported) return;
  await frb_gamepad.rumbleGamepad(
    port: port,
    strength: strength,
    durationMs: durationMs,
  );
}

/// Manually binds a gamepad to a NES port.
Future<void> bindGamepad({required int id, int? port}) async {
  if (!isGamepadSupported) return;
  await frb_gamepad.bindGamepad(id: BigInt.from(id), port: port);
}

/// Returns the current button mapping for a NES port.
Future<GamepadMapping?> getGamepadMapping(int port) async {
  if (!isGamepadSupported) return null;
  try {
    final result = await frb_gamepad.getGamepadMapping(port: port);
    return _gamepadMappingFromFfi(result);
  } catch (e, st) {
    if (!_loggedGetGamepadMappingError) {
      _loggedGetGamepadMappingError = true;
      logWarning(
        e,
        stackTrace: st,
        message: 'getGamepadMapping failed; returning null',
        logger: 'nes_gamepad_io',
      );
    }
    return null;
  }
}

/// Returns a list of currently pressed buttons on a gamepad.
Future<List<GamepadButton>> getGamepadPressedButtons(int id) async {
  if (!isGamepadSupported) return [];
  try {
    final result = await frb_gamepad.getGamepadPressedButtons(
      id: BigInt.from(id),
    );
    return result.map((b) => _gamepadButtonFromFfi(b)).toList();
  } catch (e, st) {
    if (!_loggedGetGamepadPressedButtonsError) {
      _loggedGetGamepadPressedButtonsError = true;
      logWarning(
        e,
        stackTrace: st,
        message: 'getGamepadPressedButtons failed; returning empty list',
        logger: 'nes_gamepad_io',
      );
    }
    return [];
  }
}

/// Sets a custom button mapping for a NES port.
Future<void> setGamepadMapping(int port, GamepadMapping mapping) async {
  if (!isGamepadSupported) return;
  await frb_gamepad.setGamepadMapping(port: port, mapping: mapping.toFfi());
}

// === FFI Conversion Extensions & Helpers ===

extension _GamepadButtonFfiExt on GamepadButton {
  frb_gamepad.GamepadButtonFfi toFfi() => switch (this) {
    GamepadButton.south => frb_gamepad.GamepadButtonFfi.south,
    GamepadButton.east => frb_gamepad.GamepadButtonFfi.east,
    GamepadButton.north => frb_gamepad.GamepadButtonFfi.north,
    GamepadButton.west => frb_gamepad.GamepadButtonFfi.west,
    GamepadButton.c => frb_gamepad.GamepadButtonFfi.c,
    GamepadButton.z => frb_gamepad.GamepadButtonFfi.z,
    GamepadButton.leftTrigger => frb_gamepad.GamepadButtonFfi.leftTrigger,
    GamepadButton.leftTrigger2 => frb_gamepad.GamepadButtonFfi.leftTrigger2,
    GamepadButton.rightTrigger => frb_gamepad.GamepadButtonFfi.rightTrigger,
    GamepadButton.rightTrigger2 => frb_gamepad.GamepadButtonFfi.rightTrigger2,
    GamepadButton.select => frb_gamepad.GamepadButtonFfi.select,
    GamepadButton.start => frb_gamepad.GamepadButtonFfi.start,
    GamepadButton.mode => frb_gamepad.GamepadButtonFfi.mode,
    GamepadButton.leftThumb => frb_gamepad.GamepadButtonFfi.leftThumb,
    GamepadButton.rightThumb => frb_gamepad.GamepadButtonFfi.rightThumb,
    GamepadButton.dpadUp => frb_gamepad.GamepadButtonFfi.dPadUp,
    GamepadButton.dpadDown => frb_gamepad.GamepadButtonFfi.dPadDown,
    GamepadButton.dpadLeft => frb_gamepad.GamepadButtonFfi.dPadLeft,
    GamepadButton.dpadRight => frb_gamepad.GamepadButtonFfi.dPadRight,
    GamepadButton.unknown => frb_gamepad.GamepadButtonFfi.unknown,
  };
}

GamepadButton _gamepadButtonFromFfi(frb_gamepad.GamepadButtonFfi b) =>
    switch (b) {
      frb_gamepad.GamepadButtonFfi.south => GamepadButton.south,
      frb_gamepad.GamepadButtonFfi.east => GamepadButton.east,
      frb_gamepad.GamepadButtonFfi.north => GamepadButton.north,
      frb_gamepad.GamepadButtonFfi.west => GamepadButton.west,
      frb_gamepad.GamepadButtonFfi.c => GamepadButton.c,
      frb_gamepad.GamepadButtonFfi.z => GamepadButton.z,
      frb_gamepad.GamepadButtonFfi.leftTrigger => GamepadButton.leftTrigger,
      frb_gamepad.GamepadButtonFfi.leftTrigger2 => GamepadButton.leftTrigger2,
      frb_gamepad.GamepadButtonFfi.rightTrigger => GamepadButton.rightTrigger,
      frb_gamepad.GamepadButtonFfi.rightTrigger2 => GamepadButton.rightTrigger2,
      frb_gamepad.GamepadButtonFfi.select => GamepadButton.select,
      frb_gamepad.GamepadButtonFfi.start => GamepadButton.start,
      frb_gamepad.GamepadButtonFfi.mode => GamepadButton.mode,
      frb_gamepad.GamepadButtonFfi.leftThumb => GamepadButton.leftThumb,
      frb_gamepad.GamepadButtonFfi.rightThumb => GamepadButton.rightThumb,
      frb_gamepad.GamepadButtonFfi.dPadUp => GamepadButton.dpadUp,
      frb_gamepad.GamepadButtonFfi.dPadDown => GamepadButton.dpadDown,
      frb_gamepad.GamepadButtonFfi.dPadLeft => GamepadButton.dpadLeft,
      frb_gamepad.GamepadButtonFfi.dPadRight => GamepadButton.dpadRight,
      frb_gamepad.GamepadButtonFfi.unknown => GamepadButton.unknown,
    };

extension _GamepadMappingFfiExt on GamepadMapping {
  frb_gamepad.GamepadMappingFfi toFfi() => frb_gamepad.GamepadMappingFfi(
    a: a?.toFfi(),
    b: b?.toFfi(),
    select: select?.toFfi(),
    start: start?.toFfi(),
    up: up?.toFfi(),
    down: down?.toFfi(),
    left: left?.toFfi(),
    right: right?.toFfi(),
    turboA: turboA?.toFfi(),
    turboB: turboB?.toFfi(),
    rewind: rewind?.toFfi(),
    fastForward: fastForward?.toFfi(),
    saveState: saveState?.toFfi(),
    loadState: loadState?.toFfi(),
    pause: pause?.toFfi(),
  );
}

GamepadMapping _gamepadMappingFromFfi(
  frb_gamepad.GamepadMappingFfi m,
) => GamepadMapping(
  a: m.a != null ? _gamepadButtonFromFfi(m.a!) : null,
  b: m.b != null ? _gamepadButtonFromFfi(m.b!) : null,
  select: m.select != null ? _gamepadButtonFromFfi(m.select!) : null,
  start: m.start != null ? _gamepadButtonFromFfi(m.start!) : null,
  up: m.up != null ? _gamepadButtonFromFfi(m.up!) : null,
  down: m.down != null ? _gamepadButtonFromFfi(m.down!) : null,
  left: m.left != null ? _gamepadButtonFromFfi(m.left!) : null,
  right: m.right != null ? _gamepadButtonFromFfi(m.right!) : null,
  turboA: m.turboA != null ? _gamepadButtonFromFfi(m.turboA!) : null,
  turboB: m.turboB != null ? _gamepadButtonFromFfi(m.turboB!) : null,
  rewind: m.rewind != null ? _gamepadButtonFromFfi(m.rewind!) : null,
  fastForward: m.fastForward != null
      ? _gamepadButtonFromFfi(m.fastForward!)
      : null,
  saveState: m.saveState != null ? _gamepadButtonFromFfi(m.saveState!) : null,
  loadState: m.loadState != null ? _gamepadButtonFromFfi(m.loadState!) : null,
  pause: m.pause != null ? _gamepadButtonFromFfi(m.pause!) : null,
);
