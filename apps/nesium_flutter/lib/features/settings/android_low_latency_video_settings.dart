import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../domain/nes_texture_service.dart';
import '../../logging/app_logger.dart';
import '../../persistence/app_storage.dart';
import '../../persistence/keys.dart';

@immutable
class AndroidLowLatencyVideoSettings {
  const AndroidLowLatencyVideoSettings({required this.enabled});
  final bool enabled;
}

class AndroidLowLatencyVideoSettingsController
    extends Notifier<AndroidLowLatencyVideoSettings> {
  late final NesTextureService _textureService;

  @override
  AndroidLowLatencyVideoSettings build() {
    _textureService = NesTextureService();
    final stored = ref
        .read(appStorageProvider)
        .get(StorageKeys.settingsAndroidLowLatency);
    return AndroidLowLatencyVideoSettings(enabled: stored == true);
  }

  Future<void> setEnabled(bool enabled) async {
    if (enabled == state.enabled) return;
    state = AndroidLowLatencyVideoSettings(enabled: enabled);
    try {
      await ref
          .read(appStorageProvider)
          .put(StorageKeys.settingsAndroidLowLatency, enabled);
    } catch (_) {}

    final isAndroid =
        !kIsWeb && defaultTargetPlatform == TargetPlatform.android;
    if (!isAndroid) return;

    unawaitedLogged(
      _textureService.setLowLatencyVideo(enabled),
      message: 'setLowLatencyVideo($enabled)',
      logger: 'android_low_latency_video_settings',
    );
  }
}

final androidLowLatencyVideoSettingsProvider =
    NotifierProvider<
      AndroidLowLatencyVideoSettingsController,
      AndroidLowLatencyVideoSettings
    >(AndroidLowLatencyVideoSettingsController.new);
