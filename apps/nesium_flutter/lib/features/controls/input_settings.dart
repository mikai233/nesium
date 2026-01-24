import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

import '../../domain/nes_input_masks.dart';
import '../../logging/app_logger.dart';
import '../../platform/platform_capabilities.dart';
import '../../persistence/app_storage.dart';
import '../../persistence/keys.dart';
import '../../platform/nes_gamepad.dart';
import '../../persistence/json_converters.dart';
import '../../windows/settings_sync.dart';

part 'input_settings.freezed.dart';
part 'input_settings.g.dart';

enum InputDevice { keyboard, gamepad, virtualController }

enum InputMethod { keyboard, gamepad }

class LastInputMethodNotifier extends Notifier<InputMethod> {
  @override
  InputMethod build() => InputMethod.keyboard;

  void set(InputMethod method) => state = method;
}

final lastInputMethodProvider =
    NotifierProvider<LastInputMethodNotifier, InputMethod>(
      LastInputMethodNotifier.new,
    );

final keyboardPressedKeysProvider = StreamProvider<Set<LogicalKeyboardKey>>((
  ref,
) async* {
  while (true) {
    yield HardwareKeyboard.instance.logicalKeysPressed;
    await Future.delayed(const Duration(milliseconds: 50));
  }
});

enum KeyboardPreset { none, nesStandard, fightStick, arcadeLayout, custom }

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
  rewind,
  fastForward,
  saveState,
  loadState,
  pause,
}

extension KeyboardBindingActionExt on KeyboardBindingAction {
  bool get isCore => index <= KeyboardBindingAction.turboB.index;
  bool get isExtended => index > KeyboardBindingAction.turboB.index;
}

extension KeyboardPresetLabel on KeyboardPreset {
  String get label => switch (this) {
    KeyboardPreset.none => 'None',
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
    KeyboardBindingAction.rewind => 'Rewind',
    KeyboardBindingAction.fastForward => 'Fast Forward',
    KeyboardBindingAction.saveState => 'Save State',
    KeyboardBindingAction.loadState => 'Load State',
    KeyboardBindingAction.pause => 'Pause',
  };
}

final actionHintProvider = Provider.family<String, KeyboardBindingAction>((
  ref,
  action,
) {
  final method = ref.watch(lastInputMethodProvider);
  final settings = ref.watch(inputSettingsProvider).selectedSettings;

  if (method == InputMethod.keyboard) {
    final key = settings.bindingForAction(action);
    return key?.keyLabel ?? 'Unassigned';
  } else {
    // For simplicity, we map KeyboardBindingAction to NesButtonAction where they overlap
    // and then look up the gamepad binding.
    final gamepadMapping = ref.watch(gamepadMappingProvider(0)).asData?.value;
    if (gamepadMapping == null) return 'Unassigned';

    final button = switch (action) {
      KeyboardBindingAction.up => gamepadMapping.up,
      KeyboardBindingAction.down => gamepadMapping.down,
      KeyboardBindingAction.left => gamepadMapping.left,
      KeyboardBindingAction.right => gamepadMapping.right,
      KeyboardBindingAction.a => gamepadMapping.a,
      KeyboardBindingAction.b => gamepadMapping.b,
      KeyboardBindingAction.select => gamepadMapping.select,
      KeyboardBindingAction.start => gamepadMapping.start,
      KeyboardBindingAction.turboA => gamepadMapping.turboA,
      KeyboardBindingAction.turboB => gamepadMapping.turboB,
      KeyboardBindingAction.rewind => gamepadMapping.rewind,
      KeyboardBindingAction.fastForward => gamepadMapping.fastForward,
      KeyboardBindingAction.saveState => gamepadMapping.saveState,
      KeyboardBindingAction.loadState => gamepadMapping.loadState,
      KeyboardBindingAction.pause => gamepadMapping.pause,
    };

    return button?.toFriendlyName() ?? 'Unassigned';
  }
});

@freezed
sealed class InputSettings with _$InputSettings {
  const InputSettings._();

  @LogicalKeyboardKeyNullableConverter()
  const factory InputSettings({
    required InputDevice device,
    required KeyboardPreset keyboardPreset,
    LogicalKeyboardKey? customUp,
    LogicalKeyboardKey? customDown,
    LogicalKeyboardKey? customLeft,
    LogicalKeyboardKey? customRight,
    LogicalKeyboardKey? customA,
    LogicalKeyboardKey? customB,
    LogicalKeyboardKey? customSelect,
    LogicalKeyboardKey? customStart,
    LogicalKeyboardKey? customTurboA,
    LogicalKeyboardKey? customTurboB,
    LogicalKeyboardKey? customRewind,
    LogicalKeyboardKey? customFastForward,
    LogicalKeyboardKey? customSaveState,
    LogicalKeyboardKey? customLoadState,
    LogicalKeyboardKey? customPause,
  }) = _InputSettings;

  factory InputSettings.fromJson(Map<String, dynamic> json) =>
      _$InputSettingsFromJson(json);

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

  LogicalKeyboardKey? bindingForAction(KeyboardBindingAction action) =>
      switch (keyboardPreset) {
        KeyboardPreset.none => null,
        KeyboardPreset.nesStandard => switch (action) {
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
          KeyboardBindingAction.rewind => LogicalKeyboardKey.backspace,
          KeyboardBindingAction.fastForward => LogicalKeyboardKey.backslash,
          KeyboardBindingAction.saveState => LogicalKeyboardKey.f5,
          KeyboardBindingAction.loadState => LogicalKeyboardKey.f7,
          KeyboardBindingAction.pause => LogicalKeyboardKey.escape,
        },
        KeyboardPreset.fightStick => switch (action) {
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
          KeyboardBindingAction.rewind => LogicalKeyboardKey.backspace,
          KeyboardBindingAction.fastForward => LogicalKeyboardKey.backslash,
          KeyboardBindingAction.saveState => LogicalKeyboardKey.f5,
          KeyboardBindingAction.loadState => LogicalKeyboardKey.f7,
          KeyboardBindingAction.pause => LogicalKeyboardKey.escape,
        },
        KeyboardPreset.arcadeLayout => switch (action) {
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
          KeyboardBindingAction.rewind => LogicalKeyboardKey.backspace,
          KeyboardBindingAction.fastForward => LogicalKeyboardKey.backslash,
          KeyboardBindingAction.saveState => LogicalKeyboardKey.f5,
          KeyboardBindingAction.loadState => LogicalKeyboardKey.f7,
          KeyboardBindingAction.pause => LogicalKeyboardKey.escape,
        },
        KeyboardPreset.custom => customBindingFor(action),
      };

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
        KeyboardBindingAction.rewind => customRewind,
        KeyboardBindingAction.fastForward => customFastForward,
        KeyboardBindingAction.saveState => customSaveState,
        KeyboardBindingAction.loadState => customLoadState,
        KeyboardBindingAction.pause => customPause,
      };

  InputSettings solidify() {
    if (keyboardPreset == KeyboardPreset.custom) return this;
    return copyWith(
      keyboardPreset: KeyboardPreset.custom,
      customUp: bindingForAction(KeyboardBindingAction.up),
      customDown: bindingForAction(KeyboardBindingAction.down),
      customLeft: bindingForAction(KeyboardBindingAction.left),
      customRight: bindingForAction(KeyboardBindingAction.right),
      customA: bindingForAction(KeyboardBindingAction.a),
      customB: bindingForAction(KeyboardBindingAction.b),
      customSelect: bindingForAction(KeyboardBindingAction.select),
      customStart: bindingForAction(KeyboardBindingAction.start),
      customTurboA: bindingForAction(KeyboardBindingAction.turboA),
      customTurboB: bindingForAction(KeyboardBindingAction.turboB),
      customRewind: bindingForAction(KeyboardBindingAction.rewind),
      customFastForward: bindingForAction(KeyboardBindingAction.fastForward),
      customSaveState: bindingForAction(KeyboardBindingAction.saveState),
      customLoadState: bindingForAction(KeyboardBindingAction.loadState),
      customPause: bindingForAction(KeyboardBindingAction.pause),
    );
  }
}

@freezed
sealed class InputSettingsState with _$InputSettingsState {
  const InputSettingsState._();

  const factory InputSettingsState({
    required Map<int, InputSettings> ports,
    required int selectedPort,
  }) = _InputSettingsState;

  factory InputSettingsState.fromJson(Map<String, dynamic> json) =>
      _$InputSettingsStateFromJson(json);

  InputSettings get selectedSettings => ports[selectedPort]!;

  InputBindingLocation? findConflict(
    LogicalKeyboardKey key, {
    int? excludePort,
    KeyboardBindingAction? excludeAction,
  }) {
    for (final entry in ports.entries) {
      final portIndex = entry.key;
      final settings = entry.value;

      // Only keyboard-active ports can conflict with other keyboard/hotkey inputs
      if (settings.device != InputDevice.keyboard) {
        continue;
      }

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
    final stored = ref.read(appStorageProvider).get(StorageKeys.settingsInput);
    if (stored is Map) {
      try {
        return InputSettingsState.fromJson(Map<String, dynamic>.from(stored));
      } catch (e, st) {
        logWarning(
          e,
          stackTrace: st,
          message: 'Failed to load input settings',
          logger: 'input_settings',
        );
      }
    }
    return _allDefaults();
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

  void resetToDefault(int port) {
    ref.read(nesInputMasksProvider.notifier).clearAll();
    final nextPorts = Map<int, InputSettings>.from(state.ports);
    nextPorts[port] = _defaults(port: port);
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
      selected = selected.solidify();
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
      KeyboardBindingAction.rewind => s.copyWith(customRewind: k),
      KeyboardBindingAction.fastForward => s.copyWith(customFastForward: k),
      KeyboardBindingAction.saveState => s.copyWith(customSaveState: k),
      KeyboardBindingAction.loadState => s.copyWith(customLoadState: k),
      KeyboardBindingAction.pause => s.copyWith(customPause: k),
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
        customRewind: LogicalKeyboardKey.backspace,
        customFastForward: LogicalKeyboardKey.backslash,
        customSaveState: LogicalKeyboardKey.f5,
        customLoadState: LogicalKeyboardKey.f7,
        customPause: LogicalKeyboardKey.escape,
      );
    } else {
      return InputSettings(
        device: InputDevice.keyboard,
        keyboardPreset: KeyboardPreset.none,
      );
    }
  }

  void _persist(InputSettingsState value) {
    unawaitedLogged(
      Future<void>.sync(
        () => ref
            .read(appStorageProvider)
            .put(StorageKeys.settingsInput, value.toJson()),
      ),
      message: 'Persist input settings',
      logger: 'input_settings',
    );
    unawaited(SettingsSync.broadcast(group: 'input', payload: value.toJson()));
  }

  void applySynced(Object? payload) {
    if (payload is! Map) return;
    try {
      final next = InputSettingsState.fromJson(
        Map<String, dynamic>.from(payload),
      );
      if (next == state) return;
      state = next;
    } catch (e, st) {
      logWarning(e, stackTrace: st, message: 'Failed to apply synced input');
    }
  }
}

final inputSettingsProvider =
    NotifierProvider<InputSettingsController, InputSettingsState>(
      InputSettingsController.new,
    );
