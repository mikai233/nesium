// Gamepad support for native desktop platforms (Windows, macOS, Linux).
//
// Uses gilrs via flutter_rust_bridge for gamepad input and vibration.

import 'dart:async';
import 'dart:io';

import '../bridge/api/gamepad.dart' as frb_gamepad;

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
    return GamepadMapping.fromFfi(result);
  } catch (_) {
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
    return result.map((b) => GamepadButton.values[b.index]).toList();
  } catch (_) {
    return [];
  }
}

/// Sets a custom button mapping for a NES port.
Future<void> setGamepadMapping(int port, GamepadMapping mapping) async {
  if (!isGamepadSupported) return;
  await frb_gamepad.setGamepadMapping(port: port, mapping: mapping.toFfi());
}

// === Dart-friendly types ===

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

  static GamepadActions fromFfi(frb_gamepad.GamepadActionsFfi actions) =>
      GamepadActions(
        rewind: actions.rewind,
        fastForward: actions.fastForward,
        saveState: actions.saveState,
        loadState: actions.loadState,
        pause: actions.pause,
      );
}

enum GamepadButton {
  south,
  east,
  north,
  west,
  c,
  z,
  leftTrigger,
  leftTrigger2,
  rightTrigger,
  rightTrigger2,
  select,
  start,
  mode,
  leftThumb,
  rightThumb,
  dpadUp,
  dpadDown,
  dpadLeft,
  dpadRight,
  unknown;

  static GamepadButton fromFfi(frb_gamepad.GamepadButtonFfi b) => switch (b) {
    frb_gamepad.GamepadButtonFfi.south => south,
    frb_gamepad.GamepadButtonFfi.east => east,
    frb_gamepad.GamepadButtonFfi.north => north,
    frb_gamepad.GamepadButtonFfi.west => west,
    frb_gamepad.GamepadButtonFfi.c => c,
    frb_gamepad.GamepadButtonFfi.z => z,
    frb_gamepad.GamepadButtonFfi.leftTrigger => leftTrigger,
    frb_gamepad.GamepadButtonFfi.leftTrigger2 => leftTrigger2,
    frb_gamepad.GamepadButtonFfi.rightTrigger => rightTrigger,
    frb_gamepad.GamepadButtonFfi.rightTrigger2 => rightTrigger2,
    frb_gamepad.GamepadButtonFfi.select => select,
    frb_gamepad.GamepadButtonFfi.start => start,
    frb_gamepad.GamepadButtonFfi.mode => mode,
    frb_gamepad.GamepadButtonFfi.leftThumb => leftThumb,
    frb_gamepad.GamepadButtonFfi.rightThumb => rightThumb,
    frb_gamepad.GamepadButtonFfi.dPadUp => dpadUp,
    frb_gamepad.GamepadButtonFfi.dPadDown => dpadDown,
    frb_gamepad.GamepadButtonFfi.dPadLeft => dpadLeft,
    frb_gamepad.GamepadButtonFfi.dPadRight => dpadRight,
    frb_gamepad.GamepadButtonFfi.unknown => unknown,
  };

  frb_gamepad.GamepadButtonFfi toFfi() => switch (this) {
    south => frb_gamepad.GamepadButtonFfi.south,
    east => frb_gamepad.GamepadButtonFfi.east,
    north => frb_gamepad.GamepadButtonFfi.north,
    west => frb_gamepad.GamepadButtonFfi.west,
    c => frb_gamepad.GamepadButtonFfi.c,
    z => frb_gamepad.GamepadButtonFfi.z,
    leftTrigger => frb_gamepad.GamepadButtonFfi.leftTrigger,
    leftTrigger2 => frb_gamepad.GamepadButtonFfi.leftTrigger2,
    rightTrigger => frb_gamepad.GamepadButtonFfi.rightTrigger,
    rightTrigger2 => frb_gamepad.GamepadButtonFfi.rightTrigger2,
    select => frb_gamepad.GamepadButtonFfi.select,
    start => frb_gamepad.GamepadButtonFfi.start,
    mode => frb_gamepad.GamepadButtonFfi.mode,
    leftThumb => frb_gamepad.GamepadButtonFfi.leftThumb,
    rightThumb => frb_gamepad.GamepadButtonFfi.rightThumb,
    dpadUp => frb_gamepad.GamepadButtonFfi.dPadUp,
    dpadDown => frb_gamepad.GamepadButtonFfi.dPadDown,
    dpadLeft => frb_gamepad.GamepadButtonFfi.dPadLeft,
    dpadRight => frb_gamepad.GamepadButtonFfi.dPadRight,
    unknown => frb_gamepad.GamepadButtonFfi.unknown,
  };

  String toJson() => name;
  static GamepadButton fromJson(String json) =>
      values.firstWhere((e) => e.name == json, orElse: () => unknown);
}

class GamepadMapping {
  final GamepadButton a;
  final GamepadButton b;
  final GamepadButton select;
  final GamepadButton start;
  final GamepadButton up;
  final GamepadButton down;
  final GamepadButton left;
  final GamepadButton right;
  final GamepadButton turboA;
  final GamepadButton turboB;

  const GamepadMapping({
    required this.a,
    required this.b,
    required this.select,
    required this.start,
    required this.up,
    required this.down,
    required this.left,
    required this.right,
    required this.turboA,
    required this.turboB,
  });

  static GamepadMapping fromFfi(frb_gamepad.GamepadMappingFfi m) =>
      GamepadMapping(
        a: GamepadButton.fromFfi(m.a),
        b: GamepadButton.fromFfi(m.b),
        select: GamepadButton.fromFfi(m.select),
        start: GamepadButton.fromFfi(m.start),
        up: GamepadButton.fromFfi(m.up),
        down: GamepadButton.fromFfi(m.down),
        left: GamepadButton.fromFfi(m.left),
        right: GamepadButton.fromFfi(m.right),
        turboA: GamepadButton.fromFfi(m.turboA),
        turboB: GamepadButton.fromFfi(m.turboB),
      );

  factory GamepadMapping.standard() => const GamepadMapping(
    a: GamepadButton.south,
    b: GamepadButton.west,
    select: GamepadButton.select,
    start: GamepadButton.start,
    up: GamepadButton.dpadUp,
    down: GamepadButton.dpadDown,
    left: GamepadButton.dpadLeft,
    right: GamepadButton.dpadRight,
    turboA: GamepadButton.east,
    turboB: GamepadButton.north,
  );

  frb_gamepad.GamepadMappingFfi toFfi() => frb_gamepad.GamepadMappingFfi(
    a: a.toFfi(),
    b: b.toFfi(),
    select: select.toFfi(),
    start: start.toFfi(),
    up: up.toFfi(),
    down: down.toFfi(),
    left: left.toFfi(),
    right: right.toFfi(),
    turboA: turboA.toFfi(),
    turboB: turboB.toFfi(),
  );

  Map<String, dynamic> toJson() => {
    'a': a.toJson(),
    'b': b.toJson(),
    'select': select.toJson(),
    'start': start.toJson(),
    'up': up.toJson(),
    'down': down.toJson(),
    'left': left.toJson(),
    'right': right.toJson(),
    'turboA': turboA.toJson(),
    'turboB': turboB.toJson(),
  };

  static GamepadMapping fromJson(Map<String, dynamic> json) => GamepadMapping(
    a: GamepadButton.fromJson(json['a'] as String),
    b: GamepadButton.fromJson(json['b'] as String),
    select: GamepadButton.fromJson(json['select'] as String),
    start: GamepadButton.fromJson(json['start'] as String),
    up: GamepadButton.fromJson(json['up'] as String),
    down: GamepadButton.fromJson(json['down'] as String),
    left: GamepadButton.fromJson(json['left'] as String),
    right: GamepadButton.fromJson(json['right'] as String),
    turboA: GamepadButton.fromJson(json['turboA'] as String),
    turboB: GamepadButton.fromJson(json['turboB'] as String),
  );
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
