import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/domain/nes_texture_service.dart';
import 'package:nesium_flutter/persistence/app_storage.dart';
import 'package:nesium_flutter/persistence/keys.dart';

class AndroidPerformanceSettings {
  final bool highPerformance;

  AndroidPerformanceSettings({required this.highPerformance});

  AndroidPerformanceSettings copyWith({bool? highPerformance}) {
    return AndroidPerformanceSettings(
      highPerformance: highPerformance ?? this.highPerformance,
    );
  }
}

class AndroidPerformanceSettingsController
    extends Notifier<AndroidPerformanceSettings> {
  @override
  AndroidPerformanceSettings build() {
    final storage = ref.read(appStorageProvider);
    final highPerformance =
        storage.get(StorageKeys.settingsAndroidHighPerformance) as bool? ??
        false;

    // Initial apply (best-effort).
    if (highPerformance) {
      _applyHighPerformance(highPerformance);
    }

    // Listen for storage changes
    final subscription = ref.read(appStorageProvider).onKeyChanged.listen((
      event,
    ) {
      if (event.key == StorageKeys.settingsAndroidHighPerformance) {
        state = build();
      }
    });

    ref.onDispose(() => subscription.cancel());

    return AndroidPerformanceSettings(highPerformance: highPerformance);
  }

  Future<void> setHighPerformance(bool enabled) async {
    if (state.highPerformance == enabled) return;

    final storage = ref.read(appStorageProvider);
    await storage.put(StorageKeys.settingsAndroidHighPerformance, enabled);
    state = state.copyWith(highPerformance: enabled);

    await _applyHighPerformance(enabled);
  }

  Future<void> _applyHighPerformance(bool enabled) async {
    final isAndroid =
        !kIsWeb && defaultTargetPlatform == TargetPlatform.android;
    if (!isAndroid) return;
    await ref.read(nesTextureServiceProvider).setAndroidHighPriority(enabled);
  }
}

final androidPerformanceSettingsControllerProvider =
    NotifierProvider<
      AndroidPerformanceSettingsController,
      AndroidPerformanceSettings
    >(AndroidPerformanceSettingsController.new);
