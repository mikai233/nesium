import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../domain/nes_texture_service.dart';
import '../../logging/app_logger.dart';
import '../../persistence/app_storage.dart';
import '../../persistence/keys.dart';

enum AndroidVideoBackend { upload, hardware }

extension on AndroidVideoBackend {
  int get mode => switch (this) {
    AndroidVideoBackend.upload => 0,
    AndroidVideoBackend.hardware => 1,
  };
}

AndroidVideoBackend androidVideoBackendFromMode(Object? value) {
  if (value is int) {
    return value == 0
        ? AndroidVideoBackend.upload
        : AndroidVideoBackend.hardware;
  }
  return AndroidVideoBackend.hardware;
}

@immutable
class AndroidVideoBackendSettings {
  const AndroidVideoBackendSettings({required this.backend});
  final AndroidVideoBackend backend;
}

class AndroidVideoBackendSettingsController
    extends Notifier<AndroidVideoBackendSettings> {
  late final NesTextureService _textureService;

  @override
  AndroidVideoBackendSettings build() {
    _textureService = NesTextureService();
    final stored = ref
        .read(appStorageProvider)
        .get(StorageKeys.settingsAndroidVideoBackend);
    return AndroidVideoBackendSettings(
      backend: androidVideoBackendFromMode(stored),
    );
  }

  Future<void> setBackend(AndroidVideoBackend backend) async {
    if (backend == state.backend) return;
    state = AndroidVideoBackendSettings(backend: backend);
    try {
      await ref
          .read(appStorageProvider)
          .put(StorageKeys.settingsAndroidVideoBackend, backend.mode);
    } catch (e, st) {
      logError(
        e,
        stackTrace: st,
        message: 'Failed to persist android video backend settings',
        logger: 'android_video_backend_settings',
      );
    }

    final isAndroid =
        !kIsWeb && defaultTargetPlatform == TargetPlatform.android;
    if (!isAndroid) return;

    unawaitedLogged(
      _textureService.setVideoBackend(backend.mode),
      message: 'setVideoBackend(${backend.mode})',
      logger: 'android_video_backend_settings',
    );
  }
}

final androidVideoBackendSettingsProvider =
    NotifierProvider<
      AndroidVideoBackendSettingsController,
      AndroidVideoBackendSettings
    >(AndroidVideoBackendSettingsController.new);
