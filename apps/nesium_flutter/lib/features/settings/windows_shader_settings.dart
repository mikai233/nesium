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
class WindowsShaderSettings {
  const WindowsShaderSettings({
    required this.enabled,
    required this.presetPath,
  });

  final bool enabled;
  final String? presetPath;
}

class WindowsShaderSettingsController extends Notifier<WindowsShaderSettings> {
  @override
  WindowsShaderSettings build() {
    final storage = ref.read(appStorageProvider);
    final storedEnabled = storage.get(StorageKeys.settingsWindowsShaderEnabled);
    final storedPath = storage.get(StorageKeys.settingsWindowsShaderPresetPath);

    final settings = WindowsShaderSettings(
      enabled: storedEnabled is bool ? storedEnabled : false,
      presetPath: storedPath is String && storedPath.trim().isNotEmpty
          ? storedPath.trim()
          : null,
    );

    scheduleMicrotask(() {
      unawaitedLogged(
        _applyToRuntime(settings),
        message: 'applyToRuntime (init)',
        logger: 'windows_shader_settings',
      );
    });

    return settings;
  }

  bool get _isWindows =>
      !kIsWeb && defaultTargetPlatform == TargetPlatform.windows;

  Future<void> _applyToRuntime(WindowsShaderSettings settings) async {
    if (!_isWindows) return;

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
          logger: 'windows_shader_settings',
        );
      }
    }

    logInfo(
      'Applying shader: enabled=${settings.enabled}, path=$absolutePath (rel=${settings.presetPath})',
      logger: 'windows_shader_settings',
    );

    await nes_video.setShaderPresetPath(path: absolutePath);
    await nes_video.setShaderEnabled(enabled: settings.enabled);
  }

  Future<void> setEnabled(bool enabled) async {
    if (enabled == state.enabled) return;
    final next = WindowsShaderSettings(
      enabled: enabled,
      presetPath: state.presetPath,
    );
    state = next;

    try {
      await ref
          .read(appStorageProvider)
          .put(StorageKeys.settingsWindowsShaderEnabled, enabled);
    } catch (e, st) {
      logError(
        e,
        stackTrace: st,
        message: 'Failed to persist windows shader enabled',
        logger: 'windows_shader_settings',
      );
    }

    try {
      await _applyToRuntime(next);
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'setShaderEnabled failed',
        logger: 'windows_shader_settings',
      );
    }
  }

  Future<void> setPresetPath(String? path) async {
    final normalized = (path == null || path.trim().isEmpty)
        ? null
        : path.trim();
    if (normalized == state.presetPath) return;

    final next = WindowsShaderSettings(
      enabled: state.enabled,
      presetPath: normalized,
    );
    state = next;

    try {
      if (normalized == null) {
        await ref
            .read(appStorageProvider)
            .delete(StorageKeys.settingsWindowsShaderPresetPath);
      } else {
        await ref
            .read(appStorageProvider)
            .put(StorageKeys.settingsWindowsShaderPresetPath, normalized);
      }
    } catch (e, st) {
      logError(
        e,
        stackTrace: st,
        message: 'Failed to persist windows shader preset path',
        logger: 'windows_shader_settings',
      );
    }

    try {
      await _applyToRuntime(next);
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'setShaderPresetPath failed',
        logger: 'windows_shader_settings',
      );
    }
  }
}

final windowsShaderSettingsProvider =
    NotifierProvider<WindowsShaderSettingsController, WindowsShaderSettings>(
      WindowsShaderSettingsController.new,
    );
