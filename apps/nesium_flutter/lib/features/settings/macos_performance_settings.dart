import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/domain/nes_texture_service.dart';
import 'package:nesium_flutter/persistence/app_storage.dart';
import 'package:nesium_flutter/persistence/keys.dart';

class MacosPerformanceSettings {
  final bool highPerformance;

  MacosPerformanceSettings({required this.highPerformance});

  MacosPerformanceSettings copyWith({bool? highPerformance}) {
    return MacosPerformanceSettings(
      highPerformance: highPerformance ?? this.highPerformance,
    );
  }
}

class MacosPerformanceSettingsController
    extends Notifier<MacosPerformanceSettings> {
  @override
  MacosPerformanceSettings build() {
    final storage = ref.read(appStorageProvider);
    final highPerformance =
        storage.get(StorageKeys.settingsMacosHighPerformance) as bool? ?? true;

    final isMacos = !kIsWeb && defaultTargetPlatform == TargetPlatform.macOS;
    if (isMacos && highPerformance) {
      _applyHighPerformance(highPerformance);
    }

    return MacosPerformanceSettings(highPerformance: highPerformance);
  }

  Future<void> setHighPerformance(bool enabled) async {
    if (state.highPerformance == enabled) return;

    final storage = ref.read(appStorageProvider);
    await storage.put(StorageKeys.settingsMacosHighPerformance, enabled);
    state = state.copyWith(highPerformance: enabled);

    await _applyHighPerformance(enabled);
  }

  Future<void> _applyHighPerformance(bool enabled) async {
    final isMacos = !kIsWeb && defaultTargetPlatform == TargetPlatform.macOS;
    if (!isMacos) return;
    await ref.read(nesTextureServiceProvider).setMacosHighPriority(enabled);
  }
}

final macosPerformanceSettingsControllerProvider =
    NotifierProvider<
      MacosPerformanceSettingsController,
      MacosPerformanceSettings
    >(MacosPerformanceSettingsController.new);
