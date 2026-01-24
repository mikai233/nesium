import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

import '../../platform/nes_emulation.dart' as nes_emulation;
import '../../logging/app_logger.dart';
import '../../platform/platform_capabilities.dart';
import '../../persistence/app_storage.dart';
import '../../persistence/keys.dart';
import '../../windows/settings_sync.dart';

part 'emulation_settings.freezed.dart';
part 'emulation_settings.g.dart';

@freezed
sealed class EmulationSettings with _$EmulationSettings {
  const EmulationSettings._();

  const factory EmulationSettings({
    @Default(false) bool integerFpsMode,
    @Default(false) bool pauseInBackground,
    @Default(true) bool autoSaveEnabled,
    @Default(1) int autoSaveIntervalInMinutes,
    @Default(1) int quickSaveSlot,
    @Default(300) int fastForwardSpeedPercent,
    @Default(true) bool rewindEnabled,
    @Default(60) int rewindSeconds,
    @Default(100) int rewindSpeedPercent,
    @Default(true) bool showEmulationStatusOverlay,
  }) = _EmulationSettings;

  factory EmulationSettings.defaults() =>
      EmulationSettings(pauseInBackground: isNativeMobile);

  factory EmulationSettings.fromJson(Map<String, dynamic> json) =>
      _$EmulationSettingsFromJson(json);
}

class EmulationSettingsController extends Notifier<EmulationSettings> {
  @override
  EmulationSettings build() {
    final defaults = EmulationSettings.defaults();
    final settings = _loadSettingsFromStorage(defaults: defaults) ?? defaults;
    scheduleMicrotask(() {
      applyToRuntime();
    });
    return settings;
  }

  EmulationSettings? _loadSettingsFromStorage({
    required EmulationSettings defaults,
  }) {
    final stored = ref
        .read(appStorageProvider)
        .get(StorageKeys.settingsEmulation);
    if (stored is! Map) return null;
    try {
      final map = Map<String, dynamic>.from(stored);
      var settings = EmulationSettings.fromJson(map);
      if (!map.containsKey('pauseInBackground')) {
        settings = settings.copyWith(
          pauseInBackground: defaults.pauseInBackground,
        );
      }
      return settings;
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'Failed to load emulation settings',
        logger: 'emulation_settings',
      );
      return null;
    }
  }

  void applyToRuntime() {
    unawaitedLogged(
      nes_emulation.setIntegerFpsMode(enabled: state.integerFpsMode),
      message: 'setIntegerFpsMode (apply)',
      logger: 'emulation_settings',
    );
    _applyFastForwardSpeed();
    _applyRewindConfig();
    _applyRewindSpeed();
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
    final validated = seconds.clamp(60, 3600);
    if (validated == state.rewindSeconds) return;
    state = state.copyWith(rewindSeconds: validated);
    _applyRewindConfig();
    _persist(state);
  }

  void setRewindSpeedPercent(int percent) {
    final clamped = percent.clamp(100, 1000);
    if (clamped == state.rewindSpeedPercent) return;
    state = state.copyWith(rewindSpeedPercent: clamped);
    _applyRewindSpeed();
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

  void _applyRewindSpeed() {
    unawaitedLogged(
      nes_emulation.setRewindSpeed(speedPercent: state.rewindSpeedPercent),
      message: 'setRewindSpeed',
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
            .put(StorageKeys.settingsEmulation, value.toJson()),
      ),
      message: 'Persist emulation settings',
      logger: 'emulation_settings',
    );
    unawaited(
      SettingsSync.broadcast(group: 'emulation', payload: value.toJson()),
    );
  }

  void applySynced(EmulationSettings next) {
    if (next == state) return;
    state = next;
    applyToRuntime();
  }
}

final emulationSettingsProvider =
    NotifierProvider<EmulationSettingsController, EmulationSettings>(
      EmulationSettingsController.new,
    );
