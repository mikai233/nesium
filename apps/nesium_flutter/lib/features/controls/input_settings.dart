import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../domain/nes_input_masks.dart';

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

    final preset = keyboardPreset;
    if (preset == KeyboardPreset.nesStandard) {
      bind(KeyboardBindingAction.up, LogicalKeyboardKey.arrowUp);
      bind(KeyboardBindingAction.down, LogicalKeyboardKey.arrowDown);
      bind(KeyboardBindingAction.left, LogicalKeyboardKey.arrowLeft);
      bind(KeyboardBindingAction.right, LogicalKeyboardKey.arrowRight);
      bind(KeyboardBindingAction.a, LogicalKeyboardKey.keyZ);
      bind(KeyboardBindingAction.b, LogicalKeyboardKey.keyX);
      bind(KeyboardBindingAction.start, LogicalKeyboardKey.enter);
      bind(KeyboardBindingAction.select, LogicalKeyboardKey.space);
      bind(KeyboardBindingAction.turboA, LogicalKeyboardKey.keyC);
      bind(KeyboardBindingAction.turboB, LogicalKeyboardKey.keyV);
      return bindings;
    }

    if (preset == KeyboardPreset.fightStick) {
      bind(KeyboardBindingAction.up, LogicalKeyboardKey.keyW);
      bind(KeyboardBindingAction.down, LogicalKeyboardKey.keyS);
      bind(KeyboardBindingAction.left, LogicalKeyboardKey.keyA);
      bind(KeyboardBindingAction.right, LogicalKeyboardKey.keyD);
      bind(KeyboardBindingAction.a, LogicalKeyboardKey.keyJ);
      bind(KeyboardBindingAction.b, LogicalKeyboardKey.keyK);
      bind(KeyboardBindingAction.start, LogicalKeyboardKey.enter);
      bind(KeyboardBindingAction.select, LogicalKeyboardKey.space);
      bind(KeyboardBindingAction.turboA, LogicalKeyboardKey.keyU);
      bind(KeyboardBindingAction.turboB, LogicalKeyboardKey.keyI);
      return bindings;
    }

    if (preset == KeyboardPreset.arcadeLayout) {
      bind(KeyboardBindingAction.up, LogicalKeyboardKey.arrowUp);
      bind(KeyboardBindingAction.down, LogicalKeyboardKey.arrowDown);
      bind(KeyboardBindingAction.left, LogicalKeyboardKey.arrowLeft);
      bind(KeyboardBindingAction.right, LogicalKeyboardKey.arrowRight);
      bind(KeyboardBindingAction.a, LogicalKeyboardKey.keyJ);
      bind(KeyboardBindingAction.b, LogicalKeyboardKey.keyH);
      bind(KeyboardBindingAction.start, LogicalKeyboardKey.enter);
      bind(KeyboardBindingAction.select, LogicalKeyboardKey.space);
      bind(KeyboardBindingAction.turboA, LogicalKeyboardKey.keyK);
      bind(KeyboardBindingAction.turboB, LogicalKeyboardKey.keyL);
      return bindings;
    }

    bind(KeyboardBindingAction.up, customUp);
    bind(KeyboardBindingAction.down, customDown);
    bind(KeyboardBindingAction.left, customLeft);
    bind(KeyboardBindingAction.right, customRight);
    bind(KeyboardBindingAction.a, customA);
    bind(KeyboardBindingAction.b, customB);
    bind(KeyboardBindingAction.start, customStart);
    bind(KeyboardBindingAction.select, customSelect);
    bind(KeyboardBindingAction.turboA, customTurboA);
    bind(KeyboardBindingAction.turboB, customTurboB);
    return bindings;
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
  if (kIsWeb) return false;
  return defaultTargetPlatform == TargetPlatform.android ||
      defaultTargetPlatform == TargetPlatform.iOS;
}

class InputSettingsController extends Notifier<InputSettings> {
  @override
  InputSettings build() {
    final device = _supportsVirtualController()
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

  void setDevice(InputDevice device) {
    if (device == InputDevice.virtualController &&
        !_supportsVirtualController()) {
      return;
    }
    ref.read(nesInputMasksProvider.notifier).clearAll();
    state = state.copyWith(device: device);
  }

  void setKeyboardPreset(KeyboardPreset preset) {
    ref.read(nesInputMasksProvider.notifier).clearAll();
    state = state.copyWith(keyboardPreset: preset);
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
  }
}

final inputSettingsProvider =
    NotifierProvider<InputSettingsController, InputSettings>(
      InputSettingsController.new,
    );
