import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path/path.dart' as p;

import '../shaders/shader_asset_service.dart';

import '../../logging/app_logger.dart';
import '../../persistence/app_storage.dart';
import '../../persistence/keys.dart';
import '../../platform/nes_video.dart' as nes_video;
import '../../domain/nes_texture_service.dart';
import 'video_settings.dart';
import '../../windows/current_window_kind.dart';
import '../../windows/window_types.dart';
import 'shader_parameter_provider.dart';

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
    // Listen for storage changes
    final storage = ref.read(appStorageProvider);
    final subscription = storage.onKeyChanged.listen((event) {
      if (event.key == StorageKeys.settingsWindowsShaderEnabled ||
          event.key == StorageKeys.settingsWindowsShaderPresetPath) {
        _reloadFromStorage();
      }
    });
    ref.onDispose(() => subscription.cancel());

    final settings = _loadSettings();

    scheduleMicrotask(() {
      unawaitedLogged(
        _applyToRuntime(settings),
        message: 'applyToRuntime (init)',
        logger: 'windows_shader_settings',
      );
    });

    return settings;
  }

  WindowsShaderSettings _loadSettings() {
    final storage = ref.read(appStorageProvider);
    final storedEnabled = storage.get(StorageKeys.settingsWindowsShaderEnabled);
    final storedPath = storage.get(StorageKeys.settingsWindowsShaderPresetPath);

    return WindowsShaderSettings(
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

  bool get _isWindows =>
      !kIsWeb && defaultTargetPlatform == TargetPlatform.windows;

  @override
  bool updateShouldNotify(
    WindowsShaderSettings previous,
    WindowsShaderSettings next,
  ) {
    return previous.enabled != next.enabled ||
        previous.presetPath != next.presetPath;
  }

  void _debounceApply(WindowsShaderSettings settings, {required bool persist}) {
    _debounceTimer?.cancel();
    _debounceTimer = Timer(const Duration(milliseconds: 200), () {
      if (persist) {
        _persist(settings);
      }
      unawaitedLogged(
        _applyToRuntime(settings),
        message: 'applyToRuntime (debounced)',
        logger: 'windows_shader_settings',
      );
    });
  }

  void _persist(WindowsShaderSettings settings) {
    final storage = ref.read(appStorageProvider);
    try {
      storage.put(StorageKeys.settingsWindowsShaderEnabled, settings.enabled);
      if (settings.presetPath == null) {
        storage.delete(StorageKeys.settingsWindowsShaderPresetPath);
      } else {
        storage.put(
          StorageKeys.settingsWindowsShaderPresetPath,
          settings.presetPath,
        );
      }
    } catch (e, st) {
      logError(
        e,
        stackTrace: st,
        message: 'Failed to persist windows shader settings',
        logger: 'windows_shader_settings',
      );
    }
  }

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
        logger: 'windows_shader_settings',
      );
    }

    final videoSettings = ref.read(videoSettingsProvider);
    final useLinear =
        videoSettings.videoFilter != nes_video.VideoFilter.none ||
        settings.enabled;

    // Only the main window has the native texture plugin registered.
    if (ref.read(currentWindowKindProvider) == WindowKind.main) {
      await ref
          .read(nesTextureServiceProvider)
          .setVideoFilter(useLinear ? 0 : 1);
    }
  }

  Future<void> setEnabled(bool enabled) async {
    if (enabled == state.enabled) return;
    final next = WindowsShaderSettings(
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

    final next = WindowsShaderSettings(
      enabled: state.enabled,
      presetPath: normalized,
    );
    state = next;
    _debounceApply(next, persist: true);
  }
}

final windowsShaderSettingsProvider =
    NotifierProvider<WindowsShaderSettingsController, WindowsShaderSettings>(
      WindowsShaderSettingsController.new,
    );
