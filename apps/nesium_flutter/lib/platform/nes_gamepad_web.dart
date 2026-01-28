// Gamepad support for Web platform.
//
// On Web, gamepad support can be implemented using the browser's Gamepad API.

import 'dart:js_interop';
import 'package:web/web.dart' as web;

import 'nes_gamepad_types.dart';

export 'nes_gamepad_types.dart';

/// Whether gamepad support is available on this platform.
bool get isGamepadSupported => true;

/// Initializes the gamepad subsystem.
Future<void> initGamepad() async {
  // Web: No-op, browser handles gamepad natively
}

/// Shuts down the gamepad subsystem.
Future<void> shutdownGamepad() async {
  // Web: No-op
}

// Gamepad mappings for each port (0-3).
final Map<int, GamepadMapping> _portMappings = {};

// Bindings from NES Port (0-3) to Gamepad Index.
// On Web, we start with no bindings and let the user assign gamepads in settings.
final Map<int, int> _bindings = {};
// Remember the gamepad's `id` string for each bound port so we can re-resolve
// the correct index after disconnect/reconnect (the index is not stable).
final Map<int, String> _bindingNames = {};
// Best-effort stable fingerprint from the `id` string (e.g. Vendor/Product) to
// survive browsers changing the prefix of the `id` across reconnects.
final Map<int, String> _bindingVidPid = {};

String? _extractVidPid(String id) {
  final match = RegExp(
    r'Vendor:\s*([0-9a-fA-F]{4})\s*Product:\s*([0-9a-fA-F]{4})',
  ).firstMatch(id);
  if (match == null) return null;
  final vid = match.group(1)!.toLowerCase();
  final pid = match.group(2)!.toLowerCase();
  return '$vid:$pid';
}

int? _resolveBoundGamepadIndex({
  required int port,
  required JSArray<web.Gamepad?> gamepads,
}) {
  final boundIndex = _bindings[port];
  if (boundIndex == null) return null;

  final boundName = _bindingNames[port];
  final hasBoundName = boundName != null && boundName.isNotEmpty;

  if (boundIndex >= 0 && boundIndex < gamepads.length) {
    final gp = gamepads.toDart[boundIndex];
    if (gp != null && gp.connected && (!hasBoundName || gp.id == boundName)) {
      return boundIndex;
    }
  }

  if (hasBoundName) {
    for (var i = 0; i < gamepads.length; i++) {
      final gp = gamepads.toDart[i];
      if (gp != null && gp.connected && gp.id == boundName) {
        _bindings[port] = i;
        return i;
      }
    }
  }

  final expectedVidPid = _bindingVidPid[port];
  if (expectedVidPid != null && expectedVidPid.isNotEmpty) {
    for (var i = 0; i < gamepads.length; i++) {
      final gp = gamepads.toDart[i];
      if (gp == null || !gp.connected) continue;
      final vidPid = _extractVidPid(gp.id);
      if (vidPid != null && vidPid == expectedVidPid) {
        _bindings[port] = i;
        _bindingNames[port] = gp.id;
        return i;
      }
    }
  }

  return null;
}

/// Polls all connected gamepads and returns the current input state.
Future<GamepadPollResult?> pollGamepads() async {
  final gamepads = web.window.navigator.getGamepads();

  final padMasks = List<int>.filled(4, 0);
  final turboMasks = List<int>.filled(4, 0);
  bool rewind = false;
  bool fastForward = false;
  bool saveState = false;
  bool loadState = false;
  bool pause = false;
  bool fullScreen = false;

  // Iterate over NES ports (0-3) to populate masks based on bindings
  for (var port = 0; port < 4; port++) {
    final gamepadIndex = _resolveBoundGamepadIndex(
      port: port,
      gamepads: gamepads,
    );
    if (gamepadIndex == null) continue;

    final gamepad = gamepads.toDart[gamepadIndex];
    if (gamepad == null || !gamepad.connected) continue;

    int mask = 0;
    int turboMask = 0;
    final buttons = gamepad.buttons;

    // Safety check for button count
    bool isPressed(int index) =>
        index >= 0 && index < buttons.length && buttons.toDart[index].pressed;

    // Get mapping for this port, or default to standard
    final mapping = _portMappings[port] ?? GamepadMapping.standard();

    // Helper to check if a specific mapped button is pressed on this specific gamepad
    bool isMappedPressed(GamepadButton? button) {
      if (button == null) return false;
      // We need to map the enum `GamepadButton` back to the Standard Gamepad Layout index.
      // This is the reverse of what `nes_gamepad_io` does (which maps index to enum).
      // Standard Layout: https://w3c.github.io/gamepad/#remapping
      final index = switch (button) {
        GamepadButton.south => 0, // A
        GamepadButton.east => 1, // B
        GamepadButton.west => 2, // X
        GamepadButton.north => 3, // Y
        GamepadButton.leftTrigger => 4, // LB
        GamepadButton.rightTrigger => 5, // RB
        GamepadButton.leftTrigger2 => 6, // LT
        GamepadButton.rightTrigger2 => 7, // RT
        GamepadButton.select => 8, // Back / Select
        GamepadButton.start => 9, // Start
        GamepadButton.leftThumb => 10, // LS
        GamepadButton.rightThumb => 11, // RS
        GamepadButton.dpadUp => 12, // D-Pad Up
        GamepadButton.dpadDown => 13, // D-Pad Down
        GamepadButton.dpadLeft => 14, // D-Pad Left
        GamepadButton.dpadRight => 15, // D-Pad Right
        GamepadButton.mode => 16, // Mode / Guide
        _ => -1,
      };
      return isPressed(index);
    }

    if (isMappedPressed(mapping.a)) mask |= 1 << 0;
    if (isMappedPressed(mapping.b)) mask |= 1 << 1;
    if (isMappedPressed(mapping.select)) mask |= 1 << 2;
    if (isMappedPressed(mapping.start)) mask |= 1 << 3;
    if (isMappedPressed(mapping.up)) mask |= 1 << 4;
    if (isMappedPressed(mapping.down)) mask |= 1 << 5;
    if (isMappedPressed(mapping.left)) mask |= 1 << 6;
    if (isMappedPressed(mapping.right)) mask |= 1 << 7;

    if (isMappedPressed(mapping.turboA)) turboMask |= 1 << 0;
    if (isMappedPressed(mapping.turboB)) turboMask |= 1 << 1;

    padMasks[port] = mask;
    turboMasks[port] = turboMask;

    rewind = rewind || isMappedPressed(mapping.rewind);
    fastForward = fastForward || isMappedPressed(mapping.fastForward);
    saveState = saveState || isMappedPressed(mapping.saveState);
    loadState = loadState || isMappedPressed(mapping.loadState);
    pause = pause || isMappedPressed(mapping.pause);
    fullScreen = fullScreen || isMappedPressed(mapping.fullScreen);
  }

  return GamepadPollResult(
    padMasks: padMasks,
    turboMasks: turboMasks,
    actions: GamepadActions(
      rewind: rewind,
      fastForward: fastForward,
      saveState: saveState,
      loadState: loadState,
      pause: pause,
      fullScreen: fullScreen,
    ),
  );
}

/// Returns information about all connected gamepads.
Future<List<GamepadInfo>> listGamepads() async {
  final gamepads = web.window.navigator.getGamepads();
  final infoList = <GamepadInfo>[];

  // Repair stale bindings first (indices can change after reconnect).
  final boundPorts = List<int>.from(_bindings.keys);
  for (final port in boundPorts) {
    _resolveBoundGamepadIndex(port: port, gamepads: gamepads);
  }

  for (var i = 0; i < gamepads.length; i++) {
    final gamepad = gamepads.toDart[i];
    if (gamepad != null && gamepad.connected) {
      // Find which port this gamepad is assigned to
      int? assignedPort;
      for (final entry in _bindings.entries) {
        if (entry.value == i) {
          assignedPort = entry.key;
          break;
        }
      }

      infoList.add(
        GamepadInfo(
          id: i,
          name: gamepad.id,
          connected: gamepad.connected,
          port: assignedPort,
        ),
      );
    }
  }
  return infoList;
}

/// Triggers vibration on the gamepad assigned to the given port.
Future<void> rumbleGamepad({
  required int port,
  required double strength,
  required int durationMs,
}) async {
  final gamepads = web.window.navigator.getGamepads();

  // Use binding to find gamepad
  final gamepadIndex = _resolveBoundGamepadIndex(
    port: port,
    gamepads: gamepads,
  );
  if (gamepadIndex == null) return;

  final gamepad = gamepads.toDart[gamepadIndex];
  if (gamepad == null || !gamepad.connected) return;

  // Haptic Actuator API
  // Note: support varies by browser.
  final actuators = (gamepad as GamepadWithRumble).vibrationActuator;

  if (actuators != null) {
    // type: "dual-rumble" is standard
    actuators.playEffect(
      "dual-rumble",
      web.GamepadEffectParameters(
        duration: durationMs.toInt(), // package:web likely expects int
        startDelay: 0,
        strongMagnitude: strength,
        weakMagnitude: strength,
      ),
    );
  }
}

/// Manually binds a gamepad to a NES port.
Future<void> bindGamepad({required int id, int? port}) async {
  if (port != null && port >= 0 && port < 4) {
    // Assign gamepad `id` to `port`
    _bindings[port] = id;
    final gamepads = web.window.navigator.getGamepads();
    if (id >= 0 && id < gamepads.length) {
      final gp = gamepads.toDart[id];
      if (gp != null && gp.connected) {
        _bindingNames[port] = gp.id;
        final vidPid = _extractVidPid(gp.id);
        if (vidPid != null) {
          _bindingVidPid[port] = vidPid;
        } else {
          _bindingVidPid.remove(port);
        }
      } else {
        _bindingNames.remove(port);
        _bindingVidPid.remove(port);
      }
    } else {
      _bindingNames.remove(port);
      _bindingVidPid.remove(port);
    }

    // Clear other ports if they were assigned to this gamepad (optional, but good for exclusive binding)
    // Actually standard behavior: one gamepad can't be two players.
    _bindings.removeWhere((key, value) => key != port && value == id);
    _bindingNames.removeWhere((key, _) => !_bindings.containsKey(key));
    _bindingVidPid.removeWhere((key, _) => !_bindings.containsKey(key));
  } else {
    // Unbind gamepad `id` from any port
    _bindings.removeWhere((key, value) => value == id);
    _bindingNames.removeWhere((key, _) => !_bindings.containsKey(key));
    _bindingVidPid.removeWhere((key, _) => !_bindings.containsKey(key));
  }
}

/// Returns the current button mapping for a NES port.
Future<GamepadMapping?> getGamepadMapping(int port) async {
  if (_portMappings.containsKey(port)) {
    return _portMappings[port];
  }
  return GamepadMapping.standard();
}

/// Returns a list of currently pressed buttons on a gamepad.
Future<List<GamepadButton>> getGamepadPressedButtons(int id) async {
  // Web: Stub for UI indication
  // We can reuse the same polling logic or just read raw.
  // For UI "press any button to bind", we need to scan all buttons.
  final gamepads = web.window.navigator.getGamepads();
  // Finding by ID (index)
  if (id >= gamepads.length) return [];
  final gamepad = gamepads.toDart[id];
  if (gamepad == null || !gamepad.connected) return [];

  final pressed = <GamepadButton>[];
  final buttons = gamepad.buttons;

  void check(int index, GamepadButton btn) {
    if (index < buttons.length && buttons.toDart[index].pressed) {
      pressed.add(btn);
    }
  }

  check(0, GamepadButton.south);
  check(1, GamepadButton.east);
  check(2, GamepadButton.west);
  check(3, GamepadButton.north);
  check(4, GamepadButton.leftTrigger);
  check(5, GamepadButton.rightTrigger);
  check(6, GamepadButton.leftTrigger2);
  check(7, GamepadButton.rightTrigger2);
  check(8, GamepadButton.select);
  check(9, GamepadButton.start);
  check(10, GamepadButton.leftThumb);
  check(11, GamepadButton.rightThumb);
  check(12, GamepadButton.dpadUp);
  check(13, GamepadButton.dpadDown);
  check(14, GamepadButton.dpadLeft);
  check(15, GamepadButton.dpadRight);
  check(16, GamepadButton.mode);

  return pressed;
}

/// Sets a custom button mapping for a NES port.
Future<void> setGamepadMapping(int port, GamepadMapping mapping) async {
  _portMappings[port] = mapping;
}

// Extension to access vibrationActuator which might be missing in package:web definition
extension type GamepadWithRumble(web.Gamepad _) implements web.Gamepad {
  external web.GamepadHapticActuator? get vibrationActuator;
}
