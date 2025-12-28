import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../platform/nes_emulation.dart' as nes_emulation;
import '../../logging/app_logger.dart';
import '../../platform/platform_capabilities.dart';
import '../../persistence/app_storage.dart';
import '../../persistence/keys.dart';

@immutable
class EmulationSettings {
  const EmulationSettings({
    required this.integerFpsMode,
    required this.pauseInBackground,
  });

  final bool integerFpsMode;
  final bool pauseInBackground;

  EmulationSettings copyWith({bool? integerFpsMode, bool? pauseInBackground}) {
    return EmulationSettings(
      integerFpsMode: integerFpsMode ?? this.integerFpsMode,
      pauseInBackground: pauseInBackground ?? this.pauseInBackground,
    );
  }

  static EmulationSettings defaults() {
    return EmulationSettings(
      integerFpsMode: false,
      pauseInBackground: isNativeMobile,
    );
  }
}

class EmulationSettingsController extends Notifier<EmulationSettings> {
  @override
  EmulationSettings build() {
    final defaults = EmulationSettings.defaults();
    final loaded = _emulationSettingsFromStorage(
      ref.read(appStorageProvider).get(StorageKeys.settingsEmulation),
      defaults: defaults,
    );
    final settings = loaded ?? defaults;
    scheduleMicrotask(() => applyToRuntime());
    return settings;
  }

  void applyToRuntime() {
    unawaitedLogged(
      nes_emulation.setIntegerFpsMode(enabled: state.integerFpsMode),
      message: 'setIntegerFpsMode (apply)',
      logger: 'emulation_settings',
    );
  }

  void setIntegerFpsMode(bool enabled) {
    if (enabled == state.integerFpsMode) return;
    state = state.copyWith(integerFpsMode: enabled);
    unawaitedLogged(
      nes_emulation.setIntegerFpsMode(enabled: enabled),
      message: 'setIntegerFpsMode',
      logger: 'emulation_settings',
    );
    _persist(state);
  }

  void setPauseInBackground(bool enabled) {
    if (enabled == state.pauseInBackground) return;
    state = state.copyWith(pauseInBackground: enabled);
    _persist(state);
  }

  void _persist(EmulationSettings value) {
    unawaitedLogged(
      Future<void>.sync(
        () => ref
            .read(appStorageProvider)
            .put(
              StorageKeys.settingsEmulation,
              _emulationSettingsToStorage(value),
            ),
      ),
      message: 'Persist emulation settings',
      logger: 'emulation_settings',
    );
  }
}

final emulationSettingsProvider =
    NotifierProvider<EmulationSettingsController, EmulationSettings>(
      EmulationSettingsController.new,
    );

Map<String, Object?> _emulationSettingsToStorage(EmulationSettings value) =>
    <String, Object?>{
      'integerFpsMode': value.integerFpsMode,
      'pauseInBackground': value.pauseInBackground,
    };

EmulationSettings? _emulationSettingsFromStorage(
  Object? value, {
  required EmulationSettings defaults,
}) {
  if (value is! Map) return null;
  final map = value.cast<String, Object?>();
  final integerFpsMode = map['integerFpsMode'] is bool
      ? map['integerFpsMode'] as bool
      : null;
  final pauseInBackground = map['pauseInBackground'] is bool
      ? map['pauseInBackground'] as bool
      : null;
  return defaults.copyWith(
    integerFpsMode: integerFpsMode ?? defaults.integerFpsMode,
    pauseInBackground: pauseInBackground ?? defaults.pauseInBackground,
  );
}
