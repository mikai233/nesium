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
    final storage = ref.read(appStorageProvider);
    final highPerformance =
        storage.get(StorageKeys.settingsLinuxHighPerformance) as bool? ?? true;

    final isLinux = !kIsWeb && defaultTargetPlatform == TargetPlatform.linux;
    if (isLinux && highPerformance) {
      _applyHighPerformance(highPerformance);
    }

    return LinuxPerformanceSettings(highPerformance: highPerformance);
  }

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
