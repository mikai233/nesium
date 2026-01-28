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
    final storage = ref.read(appStorageProvider);
    final highPerformance =
        storage.get(StorageKeys.settingsWindowsHighPerformance) as bool? ??
        true;

    final isWindows =
        !kIsWeb && defaultTargetPlatform == TargetPlatform.windows;
    // Initial apply
    if (isWindows && highPerformance && _isMainWindow) {
      _applyHighPerformance(highPerformance);
    }

    // Listen for storage changes
    final subscription = ref.read(appStorageProvider).onKeyChanged.listen((
      event,
    ) {
      if (event.key == StorageKeys.settingsWindowsHighPerformance) {
        state = build();
      }
    });

    ref.onDispose(() => subscription.cancel());

    return WindowsPerformanceSettings(highPerformance: highPerformance);
  }

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
