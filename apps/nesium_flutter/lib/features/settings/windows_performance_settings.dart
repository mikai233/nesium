import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/domain/nes_texture_service.dart';
import 'package:nesium_flutter/persistence/app_storage.dart';
import 'package:nesium_flutter/persistence/keys.dart';
import 'package:nesium_flutter/windows/current_window_kind.dart';
import 'package:nesium_flutter/windows/window_types.dart';

class WindowsPerformanceSettings {
  final bool highPerformance;

  WindowsPerformanceSettings({required this.highPerformance});

  WindowsPerformanceSettings copyWith({bool? highPerformance}) {
    return WindowsPerformanceSettings(
      highPerformance: highPerformance ?? this.highPerformance,
    );
  }
}

class WindowsPerformanceSettingsController
    extends Notifier<WindowsPerformanceSettings> {
  bool get _isMainWindow =>
      ref.read(currentWindowKindProvider) == WindowKind.main;

  @override
  WindowsPerformanceSettings build() {
    // Listen for storage changes
    final storage = ref.read(appStorageProvider);
    final subscription = storage.onKeyChanged.listen((event) {
      if (event.key == StorageKeys.settingsWindowsHighPerformance) {
        _reloadFromStorage();
      }
    });

    ref.onDispose(() => subscription.cancel());

    final highPerformance =
        storage.get(StorageKeys.settingsWindowsHighPerformance) as bool? ??
        true;

    // Initial apply
    if (_isWindows && highPerformance && _isMainWindow) {
      _applyHighPerformance(highPerformance);
    }

    return WindowsPerformanceSettings(highPerformance: highPerformance);
  }

  void _reloadFromStorage() {
    final storage = ref.read(appStorageProvider);
    final highPerformance =
        storage.get(StorageKeys.settingsWindowsHighPerformance) as bool? ??
        true;

    if (highPerformance != state.highPerformance) {
      state = state.copyWith(highPerformance: highPerformance);
      if (_isMainWindow) {
        _applyHighPerformance(highPerformance);
      }
    }
  }

  bool get _isWindows =>
      !kIsWeb && defaultTargetPlatform == TargetPlatform.windows;

  Future<void> setHighPerformance(bool enabled) async {
    if (state.highPerformance == enabled) return;

    final storage = ref.read(appStorageProvider);
    await storage.put(StorageKeys.settingsWindowsHighPerformance, enabled);
    state = state.copyWith(highPerformance: enabled);

    if (!_isMainWindow) return;
    await _applyHighPerformance(enabled);
  }

  Future<void> _applyHighPerformance(bool enabled) async {
    final isWindows =
        !kIsWeb && defaultTargetPlatform == TargetPlatform.windows;
    if (!isWindows) return;
    await ref.read(nesTextureServiceProvider).setWindowsHighPriority(enabled);
  }
}

final windowsPerformanceSettingsControllerProvider =
    NotifierProvider<
      WindowsPerformanceSettingsController,
      WindowsPerformanceSettings
    >(WindowsPerformanceSettingsController.new);
