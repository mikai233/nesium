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

bool _supportsVirtualController() {
  return supportsVirtualControls;
}

class InputSettingsController extends Notifier<InputSettings> {
  @override
  InputSettings build() {
    final defaults = _defaults();
    final loaded = _inputSettingsFromStorage(
      ref.read(appStorageProvider).get(StorageKeys.settingsInput),
      defaults: defaults,
    );
    var settings = loaded ?? defaults;
    if (settings.device == InputDevice.virtualController &&
        !_supportsVirtualController()) {
      settings = settings.copyWith(device: InputDevice.keyboard);
    }
    return settings;
  }

  void setDevice(InputDevice device) {
    if (device == InputDevice.virtualController &&
        !_supportsVirtualController()) {
      return;
    }
    ref.read(nesInputMasksProvider.notifier).clearAll();
    state = state.copyWith(device: device);
    _persist(state);
  }

  void setKeyboardPreset(KeyboardPreset preset) {
    ref.read(nesInputMasksProvider.notifier).clearAll();
    state = state.copyWith(keyboardPreset: preset);
    _persist(state);
  }

  void setCustomBinding(KeyboardBindingAction action, LogicalKeyboardKey? key) {
    if (state.keyboardPreset != KeyboardPreset.custom) {
      state = state.copyWith(keyboardPreset: KeyboardPreset.custom);
    }

    LogicalKeyboardKey? nextUp = state.customUp;
    LogicalKeyboardKey? nextDown = state.customDown;
    LogicalKeyboardKey? nextLeft = state.customLeft;
    LogicalKeyboardKey? nextRight = state.customRight;
    LogicalKeyboardKey? nextA = state.customA;
    LogicalKeyboardKey? nextB = state.customB;
    LogicalKeyboardKey? nextSelect = state.customSelect;
    LogicalKeyboardKey? nextStart = state.customStart;
    LogicalKeyboardKey? nextTurboA = state.customTurboA;
    LogicalKeyboardKey? nextTurboB = state.customTurboB;

    void clearIfDup(KeyboardBindingAction candidateAction) {
      if (key == null) return;
      final current = state.customBindingFor(candidateAction);
      if (current == null || current != key) return;
      switch (candidateAction) {
        case KeyboardBindingAction.up:
          nextUp = null;
          break;
        case KeyboardBindingAction.down:
          nextDown = null;
          break;
        case KeyboardBindingAction.left:
          nextLeft = null;
          break;
        case KeyboardBindingAction.right:
          nextRight = null;
          break;
        case KeyboardBindingAction.a:
          nextA = null;
          break;
        case KeyboardBindingAction.b:
          nextB = null;
          break;
        case KeyboardBindingAction.select:
          nextSelect = null;
          break;
        case KeyboardBindingAction.start:
          nextStart = null;
          break;
        case KeyboardBindingAction.turboA:
          nextTurboA = null;
          break;
        case KeyboardBindingAction.turboB:
          nextTurboB = null;
          break;
      }
    }

    for (final other in KeyboardBindingAction.values) {
      if (other == action) continue;
      clearIfDup(other);
    }

    switch (action) {
      case KeyboardBindingAction.up:
        nextUp = key;
        break;
      case KeyboardBindingAction.down:
        nextDown = key;
        break;
      case KeyboardBindingAction.left:
        nextLeft = key;
        break;
      case KeyboardBindingAction.right:
        nextRight = key;
        break;
      case KeyboardBindingAction.a:
        nextA = key;
        break;
      case KeyboardBindingAction.b:
        nextB = key;
        break;
      case KeyboardBindingAction.select:
        nextSelect = key;
        break;
      case KeyboardBindingAction.start:
        nextStart = key;
        break;
      case KeyboardBindingAction.turboA:
        nextTurboA = key;
        break;
      case KeyboardBindingAction.turboB:
        nextTurboB = key;
        break;
    }

    ref.read(nesInputMasksProvider.notifier).clearAll();
    state = state.copyWith(
      customUp: nextUp,
      customDown: nextDown,
      customLeft: nextLeft,
      customRight: nextRight,
      customA: nextA,
      customB: nextB,
      customSelect: nextSelect,
      customStart: nextStart,
      customTurboA: nextTurboA,
      customTurboB: nextTurboB,
    );
    _persist(state);
  }

  InputSettings _defaults() {
    final device = preferVirtualControlsByDefault
        ? InputDevice.virtualController
        : InputDevice.keyboard;
    return InputSettings(
      device: device,
      keyboardPreset: KeyboardPreset.nesStandard,
      customUp: LogicalKeyboardKey.arrowUp,
      customDown: LogicalKeyboardKey.arrowDown,
      customLeft: LogicalKeyboardKey.arrowLeft,
      customRight: LogicalKeyboardKey.arrowRight,
      customA: LogicalKeyboardKey.keyZ,
      customB: LogicalKeyboardKey.keyX,
      customSelect: LogicalKeyboardKey.space,
      customStart: LogicalKeyboardKey.enter,
      customTurboA: LogicalKeyboardKey.keyC,
      customTurboB: LogicalKeyboardKey.keyV,
    );
  }

  void _persist(InputSettings value) {
    unawaitedLogged(
      Future<void>.sync(
        () => ref
            .read(appStorageProvider)
            .put(StorageKeys.settingsInput, _inputSettingsToStorage(value)),
      ),
      message: 'Persist input settings',
      logger: 'input_settings',
    );
  }
}

final inputSettingsProvider =
    NotifierProvider<InputSettingsController, InputSettings>(
      InputSettingsController.new,
    );

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
      } catch (_) {}
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
