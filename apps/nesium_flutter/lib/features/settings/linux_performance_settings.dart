import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/domain/nes_texture_service.dart';
import 'package:nesium_flutter/persistence/app_storage.dart';
import 'package:nesium_flutter/persistence/keys.dart';

class LinuxPerformanceSettings {
  final bool highPerformance;

  LinuxPerformanceSettings({required this.highPerformance});

  LinuxPerformanceSettings copyWith({bool? highPerformance}) {
    return LinuxPerformanceSettings(
      highPerformance: highPerformance ?? this.highPerformance,
    );
  }
}

class LinuxPerformanceSettingsController
    extends Notifier<LinuxPerformanceSettings> {
  @override
  LinuxPerformanceSettings build() {
    // Listen for storage changes
    final storage = ref.read(appStorageProvider);
    final subscription = storage.onKeyChanged.listen((event) {
      if (event.key == StorageKeys.settingsLinuxHighPerformance) {
        _reloadFromStorage();
      }
    });

    ref.onDispose(() => subscription.cancel());

    final highPerformance =
        storage.get(StorageKeys.settingsLinuxHighPerformance) as bool? ?? true;

    if (_isLinux && highPerformance) {
      _applyHighPerformance(highPerformance);
    }

    return LinuxPerformanceSettings(highPerformance: highPerformance);
  }

  void _reloadFromStorage() {
    final storage = ref.read(appStorageProvider);
    final highPerformance =
        storage.get(StorageKeys.settingsLinuxHighPerformance) as bool? ?? true;

    if (highPerformance != state.highPerformance) {
      state = state.copyWith(highPerformance: highPerformance);
      _applyHighPerformance(highPerformance);
    }
  }

  bool get _isLinux => !kIsWeb && defaultTargetPlatform == TargetPlatform.linux;

  Future<void> setHighPerformance(bool enabled) async {
    if (state.highPerformance == enabled) return;

    final storage = ref.read(appStorageProvider);
    await storage.put(StorageKeys.settingsLinuxHighPerformance, enabled);
    state = state.copyWith(highPerformance: enabled);

    await _applyHighPerformance(enabled);
  }

  Future<void> _applyHighPerformance(bool enabled) async {
    final isLinux = !kIsWeb && defaultTargetPlatform == TargetPlatform.linux;
    if (!isLinux) return;
    await ref.read(nesTextureServiceProvider).setLinuxHighPriority(enabled);
  }
}

final linuxPerformanceSettingsControllerProvider =
    NotifierProvider<
      LinuxPerformanceSettingsController,
      LinuxPerformanceSettings
    >(LinuxPerformanceSettingsController.new);
