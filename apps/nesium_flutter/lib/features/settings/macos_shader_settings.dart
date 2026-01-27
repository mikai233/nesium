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
class MacosShaderSettings {
  const MacosShaderSettings({required this.enabled, required this.presetPath});

  final bool enabled;
  final String? presetPath;
}

class MacosShaderSettingsController extends Notifier<MacosShaderSettings> {
  @override
  MacosShaderSettings build() {
    final storage = ref.read(appStorageProvider);
    final storedEnabled = storage.get(StorageKeys.settingsMacosShaderEnabled);
    final storedPath = storage.get(StorageKeys.settingsMacosShaderPresetPath);

    final settings = MacosShaderSettings(
      enabled: storedEnabled is bool ? storedEnabled : false,
      presetPath: storedPath is String && storedPath.trim().isNotEmpty
          ? storedPath.trim()
          : null,
    );

    scheduleMicrotask(() {
      unawaitedLogged(
        _applyToRuntime(settings),
        message: 'applyToRuntime (init)',
        logger: 'macos_shader_settings',
      );
    });

    return settings;
  }

  bool get _isMacos => !kIsWeb && defaultTargetPlatform == TargetPlatform.macOS;

  Future<void> _applyToRuntime(MacosShaderSettings settings) async {
    if (!_isMacos) return;

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
          logger: 'macos_shader_settings',
        );
      }
    }

    logInfo(
      'Applying shader: enabled=${settings.enabled}, path=$absolutePath (rel=${settings.presetPath})',
      logger: 'macos_shader_settings',
    );

    await nes_video.setShaderPresetPath(path: absolutePath);
    await nes_video.setShaderEnabled(enabled: settings.enabled);
  }

  Future<void> setEnabled(bool enabled) async {
    if (enabled == state.enabled) return;
    final next = MacosShaderSettings(
      enabled: enabled,
      presetPath: state.presetPath,
    );
    state = next;

    try {
      await ref
          .read(appStorageProvider)
          .put(StorageKeys.settingsMacosShaderEnabled, enabled);
    } catch (e, st) {
      logError(
        e,
        stackTrace: st,
        message: 'Failed to persist macos shader enabled',
        logger: 'macos_shader_settings',
      );
    }

    try {
      await _applyToRuntime(next);
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'setShaderEnabled failed',
        logger: 'macos_shader_settings',
      );
    }
  }

  Future<void> setPresetPath(String? path) async {
    final normalized = (path == null || path.trim().isEmpty)
        ? null
        : path.trim();
    if (normalized == state.presetPath) return;

    final next = MacosShaderSettings(
      enabled: state.enabled,
      presetPath: normalized,
    );
    state = next;

    try {
      if (normalized == null) {
        await ref
            .read(appStorageProvider)
            .delete(StorageKeys.settingsMacosShaderPresetPath);
      } else {
        await ref
            .read(appStorageProvider)
            .put(StorageKeys.settingsMacosShaderPresetPath, normalized);
      }
    } catch (e, st) {
      logError(
        e,
        stackTrace: st,
        message: 'Failed to persist macos shader preset path',
        logger: 'macos_shader_settings',
      );
    }

    try {
      await _applyToRuntime(next);
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'setShaderPresetPath failed',
        logger: 'macos_shader_settings',
      );
    }
  }
}

final macosShaderSettingsProvider =
    NotifierProvider<MacosShaderSettingsController, MacosShaderSettings>(
      MacosShaderSettingsController.new,
    );
