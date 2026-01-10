/// Shared types for NES gamepad support.

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
}

/// Mapping of physical gamepad buttons to NES controller inputs.
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

  factory GamepadMapping.fromJson(Map<String, dynamic> json) => GamepadMapping(
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
