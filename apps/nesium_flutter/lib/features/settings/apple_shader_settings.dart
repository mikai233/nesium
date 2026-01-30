import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path/path.dart' as p;

import '../shaders/shader_asset_service.dart';

import '../../logging/app_logger.dart';
import '../../persistence/app_storage.dart';
import '../../persistence/keys.dart';
import '../../platform/nes_video.dart' as nes_video;
import 'shader_parameter_provider.dart';

@immutable
class AppleShaderSettings {
  const AppleShaderSettings({required this.enabled, required this.presetPath});

  final bool enabled;
  final String? presetPath;
}

class AppleShaderSettingsController extends Notifier<AppleShaderSettings> {
  @override
  AppleShaderSettings build() {
    // Listen for storage changes
    final storage = ref.read(appStorageProvider);
    final subscription = storage.onKeyChanged.listen((event) {
      if (event.key == StorageKeys.settingsAppleShaderEnabled ||
          event.key == StorageKeys.settingsAppleShaderPresetPath) {
        _reloadFromStorage();
      }
    });
    ref.onDispose(() => subscription.cancel());

    final settings = _loadSettings();

    // Initial apply
    scheduleMicrotask(() {
      unawaitedLogged(
        _applyToRuntime(settings),
        message: 'applyToRuntime (init)',
        logger: 'apple_shader_settings',
      );
    });

    return settings;
  }

  AppleShaderSettings _loadSettings() {
    final storage = ref.read(appStorageProvider);
    final storedEnabled = storage.get(StorageKeys.settingsAppleShaderEnabled);
    final storedPath = storage.get(StorageKeys.settingsAppleShaderPresetPath);

    return AppleShaderSettings(
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
      _debounceApply(newState, persist: false);
    }
  }

  Timer? _debounceTimer;

  bool get _isApple =>
      !kIsWeb &&
      (defaultTargetPlatform == TargetPlatform.macOS ||
          defaultTargetPlatform == TargetPlatform.iOS);

  @override
  bool updateShouldNotify(
    AppleShaderSettings previous,
    AppleShaderSettings next,
  ) {
    return previous.enabled != next.enabled ||
        previous.presetPath != next.presetPath;
  }

  void _debounceApply(AppleShaderSettings settings, {required bool persist}) {
    _debounceTimer?.cancel();
    _debounceTimer = Timer(const Duration(milliseconds: 200), () {
      if (persist) {
        _persist(settings);
      }
      unawaitedLogged(
        _applyToRuntime(settings),
        message: 'applyToRuntime (debounced)',
        logger: 'apple_shader_settings',
      );
    });
  }

  void _persist(AppleShaderSettings settings) {
    final storage = ref.read(appStorageProvider);
    try {
      storage.put(StorageKeys.settingsAppleShaderEnabled, settings.enabled);
      if (settings.presetPath == null) {
        storage.delete(StorageKeys.settingsAppleShaderPresetPath);
      } else {
        storage.put(
          StorageKeys.settingsAppleShaderPresetPath,
          settings.presetPath,
        );
      }
    } catch (e, st) {
      logError(
        e,
        stackTrace: st,
        message: 'Failed to persist apple shader settings',
        logger: 'apple_shader_settings',
      );
    }
  }

  Future<void> _applyToRuntime(AppleShaderSettings settings) async {
    if (!_isApple) return;

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
          logger: 'apple_shader_settings',
        );
      }
    }

    logInfo(
      'Applying shader: enabled=${settings.enabled}, path=$absolutePath (rel=${settings.presetPath})',
      logger: 'apple_shader_settings',
    );

    try {
      final parameters = await nes_video.setShaderConfig(
        enabled: settings.enabled,
        path: absolutePath,
      );

      if (!settings.enabled || absolutePath == null) {
        ref.read(shaderParametersProvider.notifier).clear();
      } else {
        if (settings.presetPath != null) {
          await ref
              .read(shaderParametersProvider.notifier)
              .onShaderLoaded(parameters, settings.presetPath!);
        }
      }
    } catch (e, st) {
      logError(
        e,
        stackTrace: st,
        message: 'Failed to set shader options',
        logger: 'apple_shader_settings',
      );
    }
  }

  Future<void> setEnabled(bool enabled) async {
    if (enabled == state.enabled) return;
    final next = AppleShaderSettings(
      enabled: enabled,
      presetPath: state.presetPath,
    );
    state = next;
    _debounceApply(next, persist: true);
  }

  Future<void> setPresetPath(String? path) async {
    final normalized = (path == null || path.trim().isEmpty)
        ? null
        : path.trim();
    if (normalized == state.presetPath) return;

    final next = AppleShaderSettings(
      enabled: state.enabled,
      presetPath: normalized,
    );
    state = next;
    _debounceApply(next, persist: true);
  }
}

final appleShaderSettingsProvider =
    NotifierProvider<AppleShaderSettingsController, AppleShaderSettings>(
      AppleShaderSettingsController.new,
    );
