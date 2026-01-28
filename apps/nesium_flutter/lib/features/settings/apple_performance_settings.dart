import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/domain/nes_texture_service.dart';
import 'package:nesium_flutter/persistence/app_storage.dart';
import 'package:nesium_flutter/persistence/keys.dart';

class ApplePerformanceSettings {
  final bool highPerformance;

  ApplePerformanceSettings({required this.highPerformance});

  ApplePerformanceSettings copyWith({bool? highPerformance}) {
    return ApplePerformanceSettings(
      highPerformance: highPerformance ?? this.highPerformance,
    );
  }
}

class ApplePerformanceSettingsController
    extends Notifier<ApplePerformanceSettings> {
  @override
  ApplePerformanceSettings build() {
    final storage = ref.read(appStorageProvider);
    final highPerformance =
        storage.get(StorageKeys.settingsAppleHighPerformance) as bool? ?? true;

    final isApple =
        !kIsWeb &&
        (defaultTargetPlatform == TargetPlatform.macOS ||
            defaultTargetPlatform == TargetPlatform.iOS);
    if (isApple && highPerformance) {
      _applyHighPerformance(highPerformance);
    }

    // Listen for storage changes
    final subscription = ref.read(appStorageProvider).onKeyChanged.listen((
      event,
    ) {
      if (event.key == StorageKeys.settingsAppleHighPerformance) {
        state = build();
      }
    });

    ref.onDispose(() => subscription.cancel());

    return ApplePerformanceSettings(highPerformance: highPerformance);
  }

  Future<void> setHighPerformance(bool enabled) async {
    if (state.highPerformance == enabled) return;

    final storage = ref.read(appStorageProvider);
    await storage.put(StorageKeys.settingsAppleHighPerformance, enabled);
    state = state.copyWith(highPerformance: enabled);

    await _applyHighPerformance(enabled);
  }

  Future<void> _applyHighPerformance(bool enabled) async {
    final isApple =
        !kIsWeb &&
        (defaultTargetPlatform == TargetPlatform.macOS ||
            defaultTargetPlatform == TargetPlatform.iOS);
    if (!isApple) return;
    await ref.read(nesTextureServiceProvider).setAppleHighPriority(enabled);
  }
}

final applePerformanceSettingsControllerProvider =
    NotifierProvider<
      ApplePerformanceSettingsController,
      ApplePerformanceSettings
    >(ApplePerformanceSettingsController.new);
