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
    required this.autoSaveEnabled,
    required this.autoSaveIntervalInMinutes,
    required this.quickSaveSlot,
    required this.fastForwardSpeedPercent,
    required this.rewindEnabled,
    required this.rewindSeconds,
    required this.showEmulationStatusOverlay,
  });

  final bool integerFpsMode;
  final bool pauseInBackground;
  final bool autoSaveEnabled;
  final int autoSaveIntervalInMinutes;
  final int quickSaveSlot;
  final int fastForwardSpeedPercent;
  final bool rewindEnabled;
  final int rewindSeconds;
  final bool showEmulationStatusOverlay;

  EmulationSettings copyWith({
    bool? integerFpsMode,
    bool? pauseInBackground,
    bool? autoSaveEnabled,
    int? autoSaveIntervalInMinutes,
    int? quickSaveSlot,
    int? fastForwardSpeedPercent,
    bool? rewindEnabled,
    int? rewindSeconds,
    bool? showEmulationStatusOverlay,
  }) {
    return EmulationSettings(
      integerFpsMode: integerFpsMode ?? this.integerFpsMode,
      pauseInBackground: pauseInBackground ?? this.pauseInBackground,
      autoSaveEnabled: autoSaveEnabled ?? this.autoSaveEnabled,
      autoSaveIntervalInMinutes:
          autoSaveIntervalInMinutes ?? this.autoSaveIntervalInMinutes,
      quickSaveSlot: quickSaveSlot ?? this.quickSaveSlot,
      fastForwardSpeedPercent:
          fastForwardSpeedPercent ?? this.fastForwardSpeedPercent,
      rewindEnabled: rewindEnabled ?? this.rewindEnabled,
      rewindSeconds: rewindSeconds ?? this.rewindSeconds,
      showEmulationStatusOverlay:
          showEmulationStatusOverlay ?? this.showEmulationStatusOverlay,
    );
  }

  static EmulationSettings defaults() {
    return EmulationSettings(
      integerFpsMode: false,
      pauseInBackground: isNativeMobile,
      autoSaveEnabled: true,
      autoSaveIntervalInMinutes: 1,
      quickSaveSlot: 1,
      fastForwardSpeedPercent: 300,
      rewindEnabled: true,
      rewindSeconds: 10,
      showEmulationStatusOverlay: true,
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
    scheduleMicrotask(() {
      applyToRuntime();
    });
    return settings;
  }

  void applyToRuntime() {
    unawaitedLogged(
      nes_emulation.setIntegerFpsMode(enabled: state.integerFpsMode),
      message: 'setIntegerFpsMode (apply)',
      logger: 'emulation_settings',
    );
    _applyFastForwardSpeed();
    _applyRewindConfig();
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

  void setAutoSaveEnabled(bool enabled) {
    if (enabled == state.autoSaveEnabled) return;
    state = state.copyWith(autoSaveEnabled: enabled);
    _persist(state);
  }

  void setAutoSaveIntervalInMinutes(int minutes) {
    final clamped = minutes.clamp(1, 60);
    if (clamped == state.autoSaveIntervalInMinutes) return;
    state = state.copyWith(autoSaveIntervalInMinutes: clamped);
    _persist(state);
  }

  void setQuickSaveSlot(int slot) {
    final clamped = slot.clamp(1, 10);
    if (clamped == state.quickSaveSlot) return;
    state = state.copyWith(quickSaveSlot: clamped);
    _persist(state);
  }

  void setFastForwardSpeedPercent(int percent) {
    final clamped = percent.clamp(100, 1000);
    if (clamped == state.fastForwardSpeedPercent) return;
    state = state.copyWith(fastForwardSpeedPercent: clamped);
    _applyFastForwardSpeed();
    _persist(state);
  }

  void setRewindEnabled(bool enabled) {
    if (enabled == state.rewindEnabled) return;
    state = state.copyWith(rewindEnabled: enabled);
    _applyRewindConfig();
    _persist(state);
  }

  void setRewindSeconds(int seconds) {
    final clamped = seconds.clamp(10, 300);
    if (clamped == state.rewindSeconds) return;
    state = state.copyWith(rewindSeconds: clamped);
    _applyRewindConfig();
    _persist(state);
  }

  void setShowEmulationStatusOverlay(bool enabled) {
    if (enabled == state.showEmulationStatusOverlay) return;
    state = state.copyWith(showEmulationStatusOverlay: enabled);
    _persist(state);
  }

  void _applyRewindConfig() {
    unawaitedLogged(
      nes_emulation.setRewindConfig(
        enabled: state.rewindEnabled,
        capacity: BigInt.from(state.rewindSeconds * 60),
      ),
      message: 'setRewindConfig',
      logger: 'emulation_settings',
    );
  }

  void _applyFastForwardSpeed() {
    unawaitedLogged(
      nes_emulation.setFastForwardSpeed(
        speedPercent: state.fastForwardSpeedPercent,
      ),
      message: 'setFastForwardSpeed',
      logger: 'emulation_settings',
    );
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
      'autoSaveEnabled': value.autoSaveEnabled,
      'autoSaveIntervalInMinutes': value.autoSaveIntervalInMinutes,
      'quickSaveSlot': value.quickSaveSlot,
      'fastForwardSpeedPercent': value.fastForwardSpeedPercent,
      'rewindEnabled': value.rewindEnabled,
      'rewindSeconds': value.rewindSeconds,
      'showEmulationStatusOverlay': value.showEmulationStatusOverlay,
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
  final autoSaveEnabled = map['autoSaveEnabled'] is bool
      ? map['autoSaveEnabled'] as bool
      : null;
  final autoSaveIntervalInMinutes = map['autoSaveIntervalInMinutes'] is int
      ? map['autoSaveIntervalInMinutes'] as int
      : null;
  final quickSaveSlot = map['quickSaveSlot'] is int
      ? map['quickSaveSlot'] as int
      : null;
  final fastForwardSpeedPercent = map['fastForwardSpeedPercent'] is int
      ? map['fastForwardSpeedPercent'] as int
      : null;
  final rewindEnabled = map['rewindEnabled'] is bool
      ? map['rewindEnabled'] as bool
      : null;
  final rewindSeconds = map['rewindSeconds'] is int
      ? map['rewindSeconds'] as int
      : null;
  final showEmulationStatusOverlay = map['showEmulationStatusOverlay'] is bool
      ? map['showEmulationStatusOverlay'] as bool
      : null;
  return defaults.copyWith(
    integerFpsMode: integerFpsMode ?? defaults.integerFpsMode,
    pauseInBackground: pauseInBackground ?? defaults.pauseInBackground,
    autoSaveEnabled: autoSaveEnabled ?? defaults.autoSaveEnabled,
    autoSaveIntervalInMinutes:
        autoSaveIntervalInMinutes ?? defaults.autoSaveIntervalInMinutes,
    quickSaveSlot: quickSaveSlot ?? defaults.quickSaveSlot,
    fastForwardSpeedPercent:
        fastForwardSpeedPercent ?? defaults.fastForwardSpeedPercent,
    rewindEnabled: rewindEnabled ?? defaults.rewindEnabled,
    rewindSeconds: rewindSeconds ?? defaults.rewindSeconds,
    showEmulationStatusOverlay:
        showEmulationStatusOverlay ?? defaults.showEmulationStatusOverlay,
  );
}
