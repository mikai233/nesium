import 'package:freezed_annotation/freezed_annotation.dart';

part 'nes_gamepad_types.freezed.dart';
part 'nes_gamepad_types.g.dart';

// Shared types for NES gamepad support.

/// Gamepad buttons supported by the mapping system.
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

  String toJson() => name;
  static GamepadButton fromJson(String json) =>
      values.firstWhere((e) => e.name == json, orElse: () => unknown);

  String toFriendlyName() {
    switch (this) {
      case GamepadButton.south:
        return "A / Cross";
      case GamepadButton.east:
        return "B / Circle";
      case GamepadButton.north:
        return "X / Triangle";
      case GamepadButton.west:
        return "Y / Square";
      case GamepadButton.leftTrigger:
        return "L1 / LB";
      case GamepadButton.rightTrigger:
        return "R1 / RB";
      case GamepadButton.leftTrigger2:
        return "L2 / LT";
      case GamepadButton.rightTrigger2:
        return "R2 / RT";
      case GamepadButton.leftThumb:
        return "L3 / LS";
      case GamepadButton.rightThumb:
        return "R3 / RS";
      case GamepadButton.select:
        return "Select / Back";
      case GamepadButton.start:
        return "Start";
      case GamepadButton.dpadUp:
        return "D-Pad Up";
      case GamepadButton.dpadDown:
        return "D-Pad Down";
      case GamepadButton.dpadLeft:
        return "D-Pad Left";
      case GamepadButton.dpadRight:
        return "D-Pad Right";
      case GamepadButton.mode:
        return "Mode / Home";
      case GamepadButton.c:
        return "C";
      case GamepadButton.z:
        return "Z";
      case GamepadButton.unknown:
        return "Unknown";
    }
  }
}

/// Mapping of physical gamepad buttons to NES controller inputs and system actions.
@freezed
sealed class GamepadMapping with _$GamepadMapping {
  const factory GamepadMapping({
    required GamepadButton? a,
    required GamepadButton? b,
    required GamepadButton? select,
    required GamepadButton? start,
    required GamepadButton? up,
    required GamepadButton? down,
    required GamepadButton? left,
    required GamepadButton? right,
    required GamepadButton? turboA,
    required GamepadButton? turboB,

    // Extended actions
    GamepadButton? rewind,
    GamepadButton? fastForward,
    GamepadButton? saveState,
    GamepadButton? loadState,
    GamepadButton? pause,
  }) = _GamepadMapping;

  factory GamepadMapping.fromJson(Map<String, dynamic> json) =>
      _$GamepadMappingFromJson(json);

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
    rewind: GamepadButton.leftTrigger,
    fastForward: GamepadButton.rightTrigger,
    pause: null,
  );
}

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
