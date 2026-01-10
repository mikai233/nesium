import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../domain/nes_input_masks.dart';
import '../../logging/app_logger.dart';
import '../../platform/platform_capabilities.dart';
import '../../persistence/app_storage.dart';
import '../../persistence/keys.dart';

enum InputDevice { keyboard, virtualController }

enum KeyboardPreset { nesStandard, fightStick, arcadeLayout, custom }

enum KeyboardBindingAction {
  up,
  down,
  left,
  right,
  a,
  b,
  select,
  start,
  turboA,
  turboB,
}

extension KeyboardPresetLabel on KeyboardPreset {
  String get label => switch (this) {
    KeyboardPreset.nesStandard => 'NES standard',
    KeyboardPreset.fightStick => 'Fight stick',
    KeyboardPreset.arcadeLayout => 'Arcade layout',
    KeyboardPreset.custom => 'Custom',
  };
}

extension KeyboardBindingActionLabel on KeyboardBindingAction {
  String get label => switch (this) {
    KeyboardBindingAction.up => 'Up',
    KeyboardBindingAction.down => 'Down',
    KeyboardBindingAction.left => 'Left',
    KeyboardBindingAction.right => 'Right',
    KeyboardBindingAction.a => 'A',
    KeyboardBindingAction.b => 'B',
    KeyboardBindingAction.select => 'Select',
    KeyboardBindingAction.start => 'Start',
    KeyboardBindingAction.turboA => 'Turbo A',
    KeyboardBindingAction.turboB => 'Turbo B',
  };
}

@immutable
class InputSettings {
  static const Object _unset = Object();

  const InputSettings({
    required this.device,
    required this.keyboardPreset,
    required this.customUp,
    required this.customDown,
    required this.customLeft,
    required this.customRight,
    required this.customA,
    required this.customB,
    required this.customSelect,
    required this.customStart,
    required this.customTurboA,
    required this.customTurboB,
  });

  final InputDevice device;
  final KeyboardPreset keyboardPreset;

  final LogicalKeyboardKey? customUp;
  final LogicalKeyboardKey? customDown;
  final LogicalKeyboardKey? customLeft;
  final LogicalKeyboardKey? customRight;
  final LogicalKeyboardKey? customA;
  final LogicalKeyboardKey? customB;
  final LogicalKeyboardKey? customSelect;
  final LogicalKeyboardKey? customStart;
  final LogicalKeyboardKey? customTurboA;
  final LogicalKeyboardKey? customTurboB;

  InputSettings copyWith({
    InputDevice? device,
    KeyboardPreset? keyboardPreset,
    Object? customUp = _unset,
    Object? customDown = _unset,
    Object? customLeft = _unset,
    Object? customRight = _unset,
    Object? customA = _unset,
    Object? customB = _unset,
    Object? customSelect = _unset,
    Object? customStart = _unset,
    Object? customTurboA = _unset,
    Object? customTurboB = _unset,
  }) => InputSettings(
    device: device ?? this.device,
    keyboardPreset: keyboardPreset ?? this.keyboardPreset,
    customUp: identical(customUp, _unset)
        ? this.customUp
        : customUp as LogicalKeyboardKey?,
    customDown: identical(customDown, _unset)
        ? this.customDown
        : customDown as LogicalKeyboardKey?,
    customLeft: identical(customLeft, _unset)
        ? this.customLeft
        : customLeft as LogicalKeyboardKey?,
    customRight: identical(customRight, _unset)
        ? this.customRight
        : customRight as LogicalKeyboardKey?,
    customA: identical(customA, _unset)
        ? this.customA
        : customA as LogicalKeyboardKey?,
    customB: identical(customB, _unset)
        ? this.customB
        : customB as LogicalKeyboardKey?,
    customSelect: identical(customSelect, _unset)
        ? this.customSelect
        : customSelect as LogicalKeyboardKey?,
    customStart: identical(customStart, _unset)
        ? this.customStart
        : customStart as LogicalKeyboardKey?,
    customTurboA: identical(customTurboA, _unset)
        ? this.customTurboA
        : customTurboA as LogicalKeyboardKey?,
    customTurboB: identical(customTurboB, _unset)
        ? this.customTurboB
        : customTurboB as LogicalKeyboardKey?,
  );

  Map<LogicalKeyboardKey, KeyboardBindingAction> resolveKeyboardBindings() {
    final bindings = <LogicalKeyboardKey, KeyboardBindingAction>{};
    void bind(KeyboardBindingAction action, LogicalKeyboardKey? key) {
      if (key == null) return;
      bindings[key] = action;
    }

    for (final action in KeyboardBindingAction.values) {
      bind(action, bindingForAction(action));
    }
    return bindings;
  }

  LogicalKeyboardKey? bindingForAction(KeyboardBindingAction action) {
    final preset = keyboardPreset;
    if (preset == KeyboardPreset.nesStandard) {
      return switch (action) {
        KeyboardBindingAction.up => LogicalKeyboardKey.arrowUp,
        KeyboardBindingAction.down => LogicalKeyboardKey.arrowDown,
        KeyboardBindingAction.left => LogicalKeyboardKey.arrowLeft,
        KeyboardBindingAction.right => LogicalKeyboardKey.arrowRight,
        KeyboardBindingAction.a => LogicalKeyboardKey.keyZ,
        KeyboardBindingAction.b => LogicalKeyboardKey.keyX,
        KeyboardBindingAction.select => LogicalKeyboardKey.space,
        KeyboardBindingAction.start => LogicalKeyboardKey.enter,
        KeyboardBindingAction.turboA => LogicalKeyboardKey.keyC,
        KeyboardBindingAction.turboB => LogicalKeyboardKey.keyV,
      };
    }

    if (preset == KeyboardPreset.fightStick) {
      return switch (action) {
        KeyboardBindingAction.up => LogicalKeyboardKey.keyW,
        KeyboardBindingAction.down => LogicalKeyboardKey.keyS,
        KeyboardBindingAction.left => LogicalKeyboardKey.keyA,
        KeyboardBindingAction.right => LogicalKeyboardKey.keyD,
        KeyboardBindingAction.a => LogicalKeyboardKey.keyJ,
        KeyboardBindingAction.b => LogicalKeyboardKey.keyK,
        KeyboardBindingAction.select => LogicalKeyboardKey.space,
        KeyboardBindingAction.start => LogicalKeyboardKey.enter,
        KeyboardBindingAction.turboA => LogicalKeyboardKey.keyU,
        KeyboardBindingAction.turboB => LogicalKeyboardKey.keyI,
      };
    }

    if (preset == KeyboardPreset.arcadeLayout) {
      return switch (action) {
        KeyboardBindingAction.up => LogicalKeyboardKey.arrowUp,
        KeyboardBindingAction.down => LogicalKeyboardKey.arrowDown,
        KeyboardBindingAction.left => LogicalKeyboardKey.arrowLeft,
        KeyboardBindingAction.right => LogicalKeyboardKey.arrowRight,
        KeyboardBindingAction.a => LogicalKeyboardKey.keyJ,
        KeyboardBindingAction.b => LogicalKeyboardKey.keyH,
        KeyboardBindingAction.select => LogicalKeyboardKey.space,
        KeyboardBindingAction.start => LogicalKeyboardKey.enter,
        KeyboardBindingAction.turboA => LogicalKeyboardKey.keyK,
        KeyboardBindingAction.turboB => LogicalKeyboardKey.keyL,
      };
    }

    return customBindingFor(action);
  }

  LogicalKeyboardKey? customBindingFor(KeyboardBindingAction action) =>
      switch (action) {
        KeyboardBindingAction.up => customUp,
        KeyboardBindingAction.down => customDown,
        KeyboardBindingAction.left => customLeft,
        KeyboardBindingAction.right => customRight,
        KeyboardBindingAction.a => customA,
        KeyboardBindingAction.b => customB,
        KeyboardBindingAction.select => customSelect,
        KeyboardBindingAction.start => customStart,
        KeyboardBindingAction.turboA => customTurboA,
        KeyboardBindingAction.turboB => customTurboB,
      };
}

@immutable
class InputSettingsState {
  const InputSettingsState({required this.ports, required this.selectedPort});

  final Map<int, InputSettings> ports;
  final int selectedPort;

  InputSettings get selectedSettings => ports[selectedPort]!;

  InputSettingsState copyWith({
    Map<int, InputSettings>? ports,
    int? selectedPort,
  }) => InputSettingsState(
    ports: ports ?? this.ports,
    selectedPort: selectedPort ?? this.selectedPort,
  );

  InputBindingLocation? findConflict(
    LogicalKeyboardKey key, {
    int? excludePort,
    KeyboardBindingAction? excludeAction,
  }) {
    for (final entry in ports.entries) {
      final portIndex = entry.key;
      final settings = entry.value;
      for (final action in KeyboardBindingAction.values) {
        if (excludePort != null &&
            portIndex == excludePort &&
            excludeAction != null &&
            action == excludeAction) {
          continue;
        }
        if (settings.bindingForAction(action) == key) {
          return InputBindingLocation(port: portIndex, action: action);
        }
      }
    }
    return null;
  }
}

class InputBindingLocation {
  final int port;
  final KeyboardBindingAction action;

  InputBindingLocation({required this.port, required this.action});
}

class InputCollision {
  final int port;
  final KeyboardBindingAction action;
  final LogicalKeyboardKey key;

  InputCollision({required this.port, required this.action, required this.key});
}

bool _supportsVirtualController() {
  return supportsVirtualControls;
}

class InputSettingsController extends Notifier<InputSettingsState> {
  @override
  InputSettingsState build() {
    final loaded = _inputSettingsStateFromStorage(
      ref.read(appStorageProvider).get(StorageKeys.settingsInput),
      defaults: _allDefaults(),
    );
    return loaded ?? _allDefaults();
  }

  InputSettingsState _allDefaults() {
    return InputSettingsState(
      ports: {
        0: _defaults(port: 0),
        1: _defaults(port: 1),
        2: _defaults(port: 2),
        3: _defaults(port: 3),
      },
      selectedPort: 0,
    );
  }

  void setSelectedPort(int port) {
    state = state.copyWith(selectedPort: port);
  }

  void setDevice(InputDevice device) {
    if (device == InputDevice.virtualController &&
        !_supportsVirtualController()) {
      return;
    }
    ref.read(nesInputMasksProvider.notifier).clearAll();
    final nextPorts = Map<int, InputSettings>.from(state.ports);
    nextPorts[state.selectedPort] = state.selectedSettings.copyWith(
      device: device,
    );
    state = state.copyWith(ports: nextPorts);
    _persist(state);
  }

  void setKeyboardPreset(KeyboardPreset preset) {
    ref.read(nesInputMasksProvider.notifier).clearAll();
    final nextPorts = Map<int, InputSettings>.from(state.ports);
    nextPorts[state.selectedPort] = state.selectedSettings.copyWith(
      keyboardPreset: preset,
    );
    state = state.copyWith(ports: nextPorts);
    _persist(state);
  }

  final _collisionController = StreamController<InputCollision>.broadcast();
  Stream<InputCollision> get collisionStream => _collisionController.stream;

  void setCustomBinding(KeyboardBindingAction action, LogicalKeyboardKey? key) {
    final nextPorts = Map<int, InputSettings>.from(state.ports);

    if (key != null) {
      // Cross-port collision detection
      for (final entry in state.ports.entries) {
        final portIndex = entry.key;
        final settings = entry.value;

        // We check all actions in all ports
        for (final candidateAction in KeyboardBindingAction.values) {
          // Skip the one we are currently setting
          if (portIndex == state.selectedPort && candidateAction == action) {
            continue;
          }

          if (settings.customBindingFor(candidateAction) == key) {
            // Collision found! Clear it.
            var updatedSettings = nextPorts[portIndex]!;
            if (updatedSettings.keyboardPreset != KeyboardPreset.custom) {
              updatedSettings = updatedSettings.copyWith(
                keyboardPreset: KeyboardPreset.custom,
              );
            }

            nextPorts[portIndex] = _clearBinding(
              updatedSettings,
              candidateAction,
            );
            _collisionController.add(
              InputCollision(
                port: portIndex,
                action: candidateAction,
                key: key,
              ),
            );
          }
        }
      }
    }

    // Now set the new binding for the selected port
    var selected = nextPorts[state.selectedPort]!;
    if (selected.keyboardPreset != KeyboardPreset.custom) {
      selected = selected.copyWith(keyboardPreset: KeyboardPreset.custom);
    }

    nextPorts[state.selectedPort] = _setBinding(selected, action, key);

    ref.read(nesInputMasksProvider.notifier).clearAll();
    state = state.copyWith(ports: nextPorts);
    _persist(state);
  }

  InputSettings _clearBinding(InputSettings s, KeyboardBindingAction a) {
    return _setBinding(s, a, null);
  }

  InputSettings _setBinding(
    InputSettings s,
    KeyboardBindingAction a,
    LogicalKeyboardKey? k,
  ) {
    return switch (a) {
      KeyboardBindingAction.up => s.copyWith(customUp: k),
      KeyboardBindingAction.down => s.copyWith(customDown: k),
      KeyboardBindingAction.left => s.copyWith(customLeft: k),
      KeyboardBindingAction.right => s.copyWith(customRight: k),
      KeyboardBindingAction.a => s.copyWith(customA: k),
      KeyboardBindingAction.b => s.copyWith(customB: k),
      KeyboardBindingAction.select => s.copyWith(customSelect: k),
      KeyboardBindingAction.start => s.copyWith(customStart: k),
      KeyboardBindingAction.turboA => s.copyWith(customTurboA: k),
      KeyboardBindingAction.turboB => s.copyWith(customTurboB: k),
    };
  }

  InputSettings _defaults({int port = 0}) {
    final device = (port == 0 && preferVirtualControlsByDefault)
        ? InputDevice.virtualController
        : InputDevice.keyboard;

    // Default bindings for P1 and P2 (P3/P4 are unassigned by default)
    // Optimized split layout for local co-op:
    // P1 on the left (WASD + F/G), P2 on the right (Arrows + K/L)
    if (port == 0) {
      return InputSettings(
        device: device,
        keyboardPreset: KeyboardPreset.nesStandard,
        customUp: LogicalKeyboardKey.keyW,
        customDown: LogicalKeyboardKey.keyS,
        customLeft: LogicalKeyboardKey.keyA,
        customRight: LogicalKeyboardKey.keyD,
        customA: LogicalKeyboardKey.keyG,
        customB: LogicalKeyboardKey.keyF,
        customSelect: LogicalKeyboardKey.keyQ,
        customStart: LogicalKeyboardKey.keyE,
        customTurboA: LogicalKeyboardKey.keyT,
        customTurboB: LogicalKeyboardKey.keyR,
      );
    } else if (port == 1) {
      return InputSettings(
        device: InputDevice.keyboard,
        keyboardPreset: KeyboardPreset.nesStandard,
        customUp: LogicalKeyboardKey.arrowUp,
        customDown: LogicalKeyboardKey.arrowDown,
        customLeft: LogicalKeyboardKey.arrowLeft,
        customRight: LogicalKeyboardKey.arrowRight,
        customA: LogicalKeyboardKey.keyL,
        customB: LogicalKeyboardKey.keyK,
        customSelect: LogicalKeyboardKey.keyU,
        customStart: LogicalKeyboardKey.keyI,
        customTurboA: LogicalKeyboardKey.keyP,
        customTurboB: LogicalKeyboardKey.keyO,
      );
    }

    return InputSettings(
      device: InputDevice.keyboard,
      keyboardPreset: KeyboardPreset.nesStandard,
      customUp: null,
      customDown: null,
      customLeft: null,
      customRight: null,
      customA: null,
      customB: null,
      customSelect: null,
      customStart: null,
      customTurboA: null,
      customTurboB: null,
    );
  }

  void _persist(InputSettingsState value) {
    unawaitedLogged(
      Future<void>.sync(
        () => ref
            .read(appStorageProvider)
            .put(
              StorageKeys.settingsInput,
              _inputSettingsStateToStorage(value),
            ),
      ),
      message: 'Persist input settings',
      logger: 'input_settings',
    );
  }
}

final inputSettingsProvider =
    NotifierProvider<InputSettingsController, InputSettingsState>(
      InputSettingsController.new,
    );

Map<String, Object?> _inputSettingsStateToStorage(InputSettingsState state) {
  final ports = <String, Map<String, Object?>>{};
  state.ports.forEach((index, settings) {
    ports[index.toString()] = _inputSettingsToStorage(settings);
  });

  return <String, Object?>{'ports': ports, 'selectedPort': state.selectedPort};
}

Map<String, Object?> _inputSettingsToStorage(InputSettings value) {
  int? keyId(LogicalKeyboardKey? key) => key?.keyId;

  return <String, Object?>{
    'device': value.device.name,
    'keyboardPreset': value.keyboardPreset.name,
    'customUp': keyId(value.customUp),
    'customDown': keyId(value.customDown),
    'customLeft': keyId(value.customLeft),
    'customRight': keyId(value.customRight),
    'customA': keyId(value.customA),
    'customB': keyId(value.customB),
    'customSelect': keyId(value.customSelect),
    'customStart': keyId(value.customStart),
    'customTurboA': keyId(value.customTurboA),
    'customTurboB': keyId(value.customTurboB),
  };
}

InputSettingsState? _inputSettingsStateFromStorage(
  Object? value, {
  required InputSettingsState defaults,
}) {
  if (value is! Map) return null;
  final map = value.cast<String, Object?>();

  final rawPorts = map['ports'];
  final ports = Map<int, InputSettings>.from(defaults.ports);

  if (rawPorts is Map) {
    rawPorts.forEach((key, val) {
      final index = int.tryParse(key.toString());
      if (index != null && index >= 0 && index < 4) {
        final settings = _inputSettingsFromStorage(
          val,
          defaults: defaults.ports[index]!,
        );
        if (settings != null) {
          ports[index] = settings;
        }
      }
    });
  } else {
    // Migration: if 'ports' doesn't exist, maybe it's the old format
    final legacySettings = _inputSettingsFromStorage(
      value,
      defaults: defaults.ports[0]!,
    );
    if (legacySettings != null) {
      ports[0] = legacySettings;
    }
  }

  return defaults.copyWith(
    ports: ports,
    selectedPort: map['selectedPort'] as int? ?? defaults.selectedPort,
  );
}

InputSettings? _inputSettingsFromStorage(
  Object? value, {
  required InputSettings defaults,
}) {
  if (value is! Map) return null;
  final map = value.cast<String, Object?>();

  T byNameOr<T extends Enum>(List<T> values, Object? raw, T fallback) {
    if (raw is String) {
      try {
        return values.byName(raw);
      } catch (e, st) {
        logWarning(
          e,
          stackTrace: st,
          message: 'Failed to lookup enum $T by name: $raw',
          logger: 'input_settings',
        );
      }
    }
    return fallback;
  }

  LogicalKeyboardKey? key(Object? raw, LogicalKeyboardKey? fallback) {
    if (raw == null) return null;
    if (raw is num) return LogicalKeyboardKey(raw.toInt());
    return fallback;
  }

  return defaults.copyWith(
    device: byNameOr(InputDevice.values, map['device'], defaults.device),
    keyboardPreset: byNameOr(
      KeyboardPreset.values,
      map['keyboardPreset'],
      defaults.keyboardPreset,
    ),
    customUp: key(map['customUp'], defaults.customUp),
    customDown: key(map['customDown'], defaults.customDown),
    customLeft: key(map['customLeft'], defaults.customLeft),
    customRight: key(map['customRight'], defaults.customRight),
    customA: key(map['customA'], defaults.customA),
    customB: key(map['customB'], defaults.customB),
    customSelect: key(map['customSelect'], defaults.customSelect),
    customStart: key(map['customStart'], defaults.customStart),
    customTurboA: key(map['customTurboA'], defaults.customTurboA),
    customTurboB: key(map['customTurboB'], defaults.customTurboB),
  );
}
