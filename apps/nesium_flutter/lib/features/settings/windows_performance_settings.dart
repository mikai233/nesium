import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/domain/nes_texture_service.dart';
import 'package:nesium_flutter/persistence/app_storage.dart';
import 'package:nesium_flutter/persistence/keys.dart';

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
  @override
  WindowsPerformanceSettings build() {
    final storage = ref.read(appStorageProvider);
    final highPerformance =
        storage.get(StorageKeys.settingsWindowsHighPerformance) as bool? ??
        true;

    final isWindows =
        !kIsWeb && defaultTargetPlatform == TargetPlatform.windows;
    // Initial apply
    if (isWindows && highPerformance) {
      _applyHighPerformance(highPerformance);
    }

    return WindowsPerformanceSettings(highPerformance: highPerformance);
  }

  Future<void> setHighPerformance(bool enabled) async {
    if (state.highPerformance == enabled) return;

    final storage = ref.read(appStorageProvider);
    await storage.put(StorageKeys.settingsWindowsHighPerformance, enabled);
    state = state.copyWith(highPerformance: enabled);

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
