import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path/path.dart' as p;

import '../shaders/shader_asset_service.dart';

import '../../logging/app_logger.dart';
import '../../persistence/app_storage.dart';
import '../../persistence/keys.dart';
import '../../platform/nes_video.dart' as nes_video;

@immutable
class AndroidShaderSettings {
  const AndroidShaderSettings({
    required this.enabled,
    required this.presetPath,
  });

  final bool enabled;
  final String? presetPath;
}

class AndroidShaderSettingsController extends Notifier<AndroidShaderSettings> {
  @override
  AndroidShaderSettings build() {
    final storage = ref.read(appStorageProvider);
    final storedEnabled = storage.get(StorageKeys.settingsAndroidShaderEnabled);
    final storedPath = storage.get(StorageKeys.settingsAndroidShaderPresetPath);

    final settings = AndroidShaderSettings(
      enabled: storedEnabled is bool ? storedEnabled : false,
      presetPath: storedPath is String && storedPath.trim().isNotEmpty
          ? storedPath.trim()
          : null,
    );

    scheduleMicrotask(() {
      unawaitedLogged(
        _applyToRuntime(settings),
        message: 'applyToRuntime (init)',
        logger: 'android_shader_settings',
      );
    });

    // Listen for storage changes
    final subscription = ref.read(appStorageProvider).onKeyChanged.listen((
      event,
    ) {
      if (event.key == StorageKeys.settingsAndroidShaderEnabled ||
          event.key == StorageKeys.settingsAndroidShaderPresetPath) {
        state = build();
      }
    });

    ref.onDispose(() => subscription.cancel());

    return settings;
  }

  bool get _isAndroid =>
      !kIsWeb && defaultTargetPlatform == TargetPlatform.android;

  Future<void> _applyToRuntime(AndroidShaderSettings settings) async {
    if (!_isAndroid) return;

    String? absolutePath;
    if (settings.presetPath != null) {
      // The settings store the relative path (e.g. 'crt/crt-geom.slangp')
      // We need to resolve it to an absolute path for the native backend.
      try {
        final assetService = ref.read(shaderAssetServiceProvider);
        final root = await assetService.getShadersRoot();
        if (root != null) {
          absolutePath = p.join(root, settings.presetPath!);
        }
      } catch (e) {
        logWarning(
          e,
          message: 'Failed to resolve shader root',
          logger: 'android_shader_settings',
        );
      }
    }

    logInfo(
      'Applying shader: enabled=${settings.enabled}, path=$absolutePath (rel=${settings.presetPath})',
      logger: 'android_shader_settings',
    );

    await nes_video.setShaderPresetPath(path: absolutePath);
    await nes_video.setShaderEnabled(enabled: settings.enabled);
  }

  Future<void> setEnabled(bool enabled) async {
    if (enabled == state.enabled) return;
    final next = AndroidShaderSettings(
      enabled: enabled,
      presetPath: state.presetPath,
    );
    state = next;

    try {
      await ref
          .read(appStorageProvider)
          .put(StorageKeys.settingsAndroidShaderEnabled, enabled);
    } catch (e, st) {
      logError(
        e,
        stackTrace: st,
        message: 'Failed to persist android shader enabled',
        logger: 'android_shader_settings',
      );
    }

    try {
      await _applyToRuntime(next);
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'setShaderEnabled failed',
        logger: 'android_shader_settings',
      );
    }
  }

  Future<void> setPresetPath(String? path) async {
    final normalized = (path == null || path.trim().isEmpty)
        ? null
        : path.trim();
    if (normalized == state.presetPath) return;

    final next = AndroidShaderSettings(
      enabled: state.enabled,
      presetPath: normalized,
    );
    state = next;

    try {
      if (normalized == null) {
        await ref
            .read(appStorageProvider)
            .delete(StorageKeys.settingsAndroidShaderPresetPath);
      } else {
        await ref
            .read(appStorageProvider)
            .put(StorageKeys.settingsAndroidShaderPresetPath, normalized);
      }
    } catch (e, st) {
      logError(
        e,
        stackTrace: st,
        message: 'Failed to persist android shader preset path',
        logger: 'android_shader_settings',
      );
    }

    try {
      await _applyToRuntime(next);
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'setShaderPresetPath failed',
        logger: 'android_shader_settings',
      );
    }
  }
}

final androidShaderSettingsProvider =
    NotifierProvider<AndroidShaderSettingsController, AndroidShaderSettings>(
      AndroidShaderSettingsController.new,
    );
