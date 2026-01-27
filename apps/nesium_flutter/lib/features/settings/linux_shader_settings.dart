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
class LinuxShaderSettings {
  const LinuxShaderSettings({required this.enabled, required this.presetPath});

  final bool enabled;
  final String? presetPath;
}

class LinuxShaderSettingsController extends Notifier<LinuxShaderSettings> {
  @override
  LinuxShaderSettings build() {
    final storage = ref.read(appStorageProvider);
    final storedEnabled = storage.get(StorageKeys.settingsLinuxShaderEnabled);
    final storedPath = storage.get(StorageKeys.settingsLinuxShaderPresetPath);

    final settings = LinuxShaderSettings(
      enabled: storedEnabled is bool ? storedEnabled : false,
      presetPath: storedPath is String && storedPath.trim().isNotEmpty
          ? storedPath.trim()
          : null,
    );

    scheduleMicrotask(() {
      unawaitedLogged(
        _applyToRuntime(settings),
        message: 'applyToRuntime (init)',
        logger: 'linux_shader_settings',
      );
    });

    return settings;
  }

  bool get _isLinux => !kIsWeb && defaultTargetPlatform == TargetPlatform.linux;

  Future<void> _applyToRuntime(LinuxShaderSettings settings) async {
    if (!_isLinux) return;

    String? absolutePath;
    if (settings.presetPath != null) {
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
          logger: 'linux_shader_settings',
        );
      }
    }

    logInfo(
      'Applying shader (Linux): enabled=${settings.enabled}, path=$absolutePath',
      logger: 'linux_shader_settings',
    );

    await nes_video.setShaderPresetPath(path: absolutePath);
    await nes_video.setShaderEnabled(enabled: settings.enabled);
  }

  Future<void> setEnabled(bool enabled) async {
    if (enabled == state.enabled) return;
    final next = LinuxShaderSettings(
      enabled: enabled,
      presetPath: state.presetPath,
    );
    state = next;

    try {
      await ref
          .read(appStorageProvider)
          .put(StorageKeys.settingsLinuxShaderEnabled, enabled);
    } catch (e, st) {
      logError(
        e,
        stackTrace: st,
        message: 'Failed to persist linux shader enabled',
        logger: 'linux_shader_settings',
      );
    }

    try {
      await _applyToRuntime(next);
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'setShaderEnabled failed',
        logger: 'linux_shader_settings',
      );
    }
  }

  Future<void> setPresetPath(String? path) async {
    final normalized = (path == null || path.trim().isEmpty)
        ? null
        : path.trim();
    if (normalized == state.presetPath) return;

    final next = LinuxShaderSettings(
      enabled: state.enabled,
      presetPath: normalized,
    );
    state = next;

    try {
      if (normalized == null) {
        await ref
            .read(appStorageProvider)
            .delete(StorageKeys.settingsLinuxShaderPresetPath);
      } else {
        await ref
            .read(appStorageProvider)
            .put(StorageKeys.settingsLinuxShaderPresetPath, normalized);
      }
    } catch (e, st) {
      logError(
        e,
        stackTrace: st,
        message: 'Failed to persist linux shader preset path',
        logger: 'linux_shader_settings',
      );
    }

    try {
      await _applyToRuntime(next);
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'setShaderPresetPath failed',
        logger: 'linux_shader_settings',
      );
    }
  }
}

final linuxShaderSettingsProvider =
    NotifierProvider<LinuxShaderSettingsController, LinuxShaderSettings>(
      LinuxShaderSettingsController.new,
    );
