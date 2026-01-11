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
class GamepadMapping {
  final GamepadButton? a;
  final GamepadButton? b;
  final GamepadButton? select;
  final GamepadButton? start;
  final GamepadButton? up;
  final GamepadButton? down;
  final GamepadButton? left;
  final GamepadButton? right;
  final GamepadButton? turboA;
  final GamepadButton? turboB;

  // Extended actions
  final GamepadButton? rewind;
  final GamepadButton? fastForward;
  final GamepadButton? saveState;
  final GamepadButton? loadState;
  final GamepadButton? pause;

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
    this.rewind,
    this.fastForward,
    this.saveState,
    this.loadState,
    this.pause,
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
    rewind: GamepadButton.leftTrigger,
    fastForward: GamepadButton.rightTrigger,
    pause: null,
  );

  GamepadMapping copyWith({
    GamepadButton? a,
    bool clearA = false,
    GamepadButton? b,
    bool clearB = false,
    GamepadButton? select,
    bool clearSelect = false,
    GamepadButton? start,
    bool clearStart = false,
    GamepadButton? up,
    bool clearUp = false,
    GamepadButton? down,
    bool clearDown = false,
    GamepadButton? left,
    bool clearLeft = false,
    GamepadButton? right,
    bool clearRight = false,
    GamepadButton? turboA,
    bool clearTurboA = false,
    GamepadButton? turboB,
    bool clearTurboB = false,
    GamepadButton? rewind,
    bool clearRewind = false,
    GamepadButton? fastForward,
    bool clearFastForward = false,
    GamepadButton? saveState,
    bool clearSaveState = false,
    GamepadButton? loadState,
    bool clearLoadState = false,
    GamepadButton? pause,
    bool clearPause = false,
  }) {
    return GamepadMapping(
      a: clearA ? null : (a ?? this.a),
      b: clearB ? null : (b ?? this.b),
      select: clearSelect ? null : (select ?? this.select),
      start: clearStart ? null : (start ?? this.start),
      up: clearUp ? null : (up ?? this.up),
      down: clearDown ? null : (down ?? this.down),
      left: clearLeft ? null : (left ?? this.left),
      right: clearRight ? null : (right ?? this.right),
      turboA: clearTurboA ? null : (turboA ?? this.turboA),
      turboB: clearTurboB ? null : (turboB ?? this.turboB),
      rewind: clearRewind ? null : (rewind ?? this.rewind),
      fastForward: clearFastForward ? null : (fastForward ?? this.fastForward),
      saveState: clearSaveState ? null : (saveState ?? this.saveState),
      loadState: clearLoadState ? null : (loadState ?? this.loadState),
      pause: clearPause ? null : (pause ?? this.pause),
    );
  }

  Map<String, dynamic> toJson() => {
    'a': a?.toJson(),
    'b': b?.toJson(),
    'select': select?.toJson(),
    'start': start?.toJson(),
    'up': up?.toJson(),
    'down': down?.toJson(),
    'left': left?.toJson(),
    'right': right?.toJson(),
    'turboA': turboA?.toJson(),
    'turboB': turboB?.toJson(),
    'rewind': rewind?.toJson(),
    'fastForward': fastForward?.toJson(),
    'saveState': saveState?.toJson(),
    'loadState': loadState?.toJson(),
    'pause': pause?.toJson(),
  };

  factory GamepadMapping.fromJson(Map<String, dynamic> json) => GamepadMapping(
    a: json['a'] != null ? GamepadButton.fromJson(json['a'] as String) : null,
    b: json['b'] != null ? GamepadButton.fromJson(json['b'] as String) : null,
    select: json['select'] != null
        ? GamepadButton.fromJson(json['select'] as String)
        : null,
    start: json['start'] != null
        ? GamepadButton.fromJson(json['start'] as String)
        : null,
    up: json['up'] != null
        ? GamepadButton.fromJson(json['up'] as String)
        : null,
    down: json['down'] != null
        ? GamepadButton.fromJson(json['down'] as String)
        : null,
    left: json['left'] != null
        ? GamepadButton.fromJson(json['left'] as String)
        : null,
    right: json['right'] != null
        ? GamepadButton.fromJson(json['right'] as String)
        : null,
    turboA: json['turboA'] != null
        ? GamepadButton.fromJson(json['turboA'] as String)
        : null,
    turboB: json['turboB'] != null
        ? GamepadButton.fromJson(json['turboB'] as String)
        : null,
    rewind: json['rewind'] != null
        ? GamepadButton.fromJson(json['rewind'] as String)
        : null,
    fastForward: json['fastForward'] != null
        ? GamepadButton.fromJson(json['fastForward'] as String)
        : null,
    saveState: json['saveState'] != null
        ? GamepadButton.fromJson(json['saveState'] as String)
        : null,
    loadState: json['loadState'] != null
        ? GamepadButton.fromJson(json['loadState'] as String)
        : null,
    pause: json['pause'] != null
        ? GamepadButton.fromJson(json['pause'] as String)
        : null,
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
