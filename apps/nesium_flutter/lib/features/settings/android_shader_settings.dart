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
    // Listen for storage changes
    final storage = ref.read(appStorageProvider);
    final subscription = storage.onKeyChanged.listen((event) {
      if (event.key == StorageKeys.settingsAndroidShaderEnabled ||
          event.key == StorageKeys.settingsAndroidShaderPresetPath) {
        _reloadFromStorage();
      }
    });
    ref.onDispose(() => subscription.cancel());

    final settings = _loadSettings();

    scheduleMicrotask(() {
      unawaitedLogged(
        _applyToRuntime(settings),
        message: 'applyToRuntime (init)',
        logger: 'android_shader_settings',
      );
    });

    return settings;
  }

  AndroidShaderSettings _loadSettings() {
    final storage = ref.read(appStorageProvider);
    final storedEnabled = storage.get(StorageKeys.settingsAndroidShaderEnabled);
    final storedPath = storage.get(StorageKeys.settingsAndroidShaderPresetPath);

    return AndroidShaderSettings(
      enabled: storedEnabled is bool ? storedEnabled : false,
      presetPath: storedPath is String && storedPath.trim().isNotEmpty
          ? storedPath.trim()
          : null,
    );
  }

  void _reloadFromStorage() {
    final newState = _loadSettings();
    if (newState.enabled != state.enabled ||
        newState.presetPath != state.presetPath) {
      state = newState;
      _debounceApply(newState);
    }
  }

  Timer? _debounceTimer;

  bool get _isAndroid =>
      !kIsWeb && defaultTargetPlatform == TargetPlatform.android;

  @override
  bool updateShouldNotify(
    AndroidShaderSettings previous,
    AndroidShaderSettings next,
  ) {
    return previous.enabled != next.enabled ||
        previous.presetPath != next.presetPath;
  }

  void _debounceApply(AndroidShaderSettings settings) {
    _debounceTimer?.cancel();
    _debounceTimer = Timer(const Duration(milliseconds: 300), () {
      unawaitedLogged(
        _applyToRuntime(settings),
        message: 'applyToRuntime (debounced)',
        logger: 'android_shader_settings',
      );
    });
  }

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

    _debounceApply(next);
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

    _debounceApply(next);
  }
}

final androidShaderSettingsProvider =
    NotifierProvider<AndroidShaderSettingsController, AndroidShaderSettings>(
      AndroidShaderSettingsController.new,
    );
