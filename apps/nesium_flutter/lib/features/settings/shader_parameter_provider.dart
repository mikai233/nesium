import 'dart:async';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:path/path.dart' as p;
import '../../bridge/api/video.dart' as video;
import '../../domain/nes_texture_service.dart';
import '../../persistence/app_storage.dart';
import '../shaders/shader_asset_service.dart';
import 'android_shader_settings.dart';
import 'apple_shader_settings.dart';
import 'windows_shader_settings.dart';
import '../../logging/app_logger.dart';

/// Provides the list of parameters for the currently active shader, with persistence.
class ShaderParameterNotifier
    extends Notifier<AsyncValue<List<video.ShaderParameter>>> {
  @override
  AsyncValue<List<video.ShaderParameter>> build() {
    // Only watch 'enabled' status. We do NOT want to rebuild when presetPath changes,
    // because that would tear down our stream listener right when we need it most
    // (during a shader switch).
    bool enabled = false;

    if (kIsWeb) {
      // Not supported
    } else if (defaultTargetPlatform == TargetPlatform.android) {
      enabled = ref.watch(
        androidShaderSettingsProvider.select((s) => s.enabled),
      );
    } else if (defaultTargetPlatform == TargetPlatform.macOS ||
        defaultTargetPlatform == TargetPlatform.iOS) {
      enabled = ref.watch(appleShaderSettingsProvider.select((s) => s.enabled));
    } else if (defaultTargetPlatform == TargetPlatform.windows) {
      enabled = ref.watch(
        windowsShaderSettingsProvider.select((s) => s.enabled),
      );
    }

    if (!enabled) {
      return const AsyncValue.data([]);
    }

    // Listen to the persistent stream for updates
    final service = ref.read(nesTextureServiceProvider);
    final subscription = service.shaderParametersStream().listen(
      (parameters) {
        // Dynamically get the current target path
        final currentPath = _getActivePresetPath();
        if (currentPath != null) {
          _handleUpdate(parameters, currentPath);
        }
      },
      onError: (err, st) {
        state = AsyncValue.error(err, st);
      },
    );
    ref.onDispose(subscription.cancel);

    return const AsyncValue.loading();
  }

  Future<void> _handleUpdate(
    video.ShaderParameters parameters,
    String requestedPreset,
  ) async {
    try {
      final assetService = ref.read(shaderAssetServiceProvider);
      final root = await assetService.getShadersRoot();
      final expectedPath = root != null ? p.join(root, requestedPreset) : null;

      // Normalization: Rust side might resolve symlinks or send absolute paths differently.
      // We should be lenient. If the filenames match, it's likely the right one.
      final bool pathMatches =
          (expectedPath != null &&
          parameters.path.isNotEmpty &&
          p.equals(parameters.path, expectedPath));

      // If mismatch, check if it's just a timing issue (receiving old shader data)
      if (!pathMatches && expectedPath != null) {
        // Fallback check: check basenames
        if (p.basename(parameters.path) == p.basename(expectedPath)) {
          // Accept it
        } else {
          appLog.info(
            'Shader Stream: Path mismatch. Received "${parameters.path}", Expected "$expectedPath". Ignoring.',
          );
          return;
        }
      }

      // Apply saved overrides from storage.
      final storage = ref.read(appStorageProvider);
      final service = ref.read(nesTextureServiceProvider);
      final List<video.ShaderParameter> updatedParams = [];

      for (final meta in parameters.parameters) {
        final name = meta.name;
        final storageKey = _getStorageKey(requestedPreset, name);
        final savedValue = storage.get(storageKey);

        if (savedValue is double &&
            (savedValue - meta.current).abs() > 0.0001) {
          // Apply to backend
          await service.setShaderParameter(name, savedValue);
          updatedParams.add(
            video.ShaderParameter(
              name: meta.name,
              description: meta.description,
              initial: meta.initial,
              current: savedValue,
              minimum: meta.minimum,
              maximum: meta.maximum,
              step: meta.step,
            ),
          );
        } else {
          updatedParams.add(meta);
        }
      }

      state = AsyncValue.data(updatedParams);
    } catch (e, st) {
      state = AsyncValue.error(e, st);
    }
  }

  String _getStorageKey(String presetPath, String paramName) {
    return 'settings.shader.params.$presetPath.$paramName';
  }

  Future<void> updateParameter(String name, double value) async {
    final currentState = state.asData?.value;
    if (currentState == null) return;

    final presetPath = _getActivePresetPath();
    if (presetPath == null) return;

    final service = ref.read(nesTextureServiceProvider);
    final storage = ref.read(appStorageProvider);

    // 1. Apply to backend
    await service.setShaderParameter(name, value);

    // 2. Save to storage
    await storage.put(_getStorageKey(presetPath, name), value);

    // 3. Update local state
    final nextState = currentState.map((p) {
      if (p.name == name) {
        return video.ShaderParameter(
          name: p.name,
          description: p.description,
          initial: p.initial,
          current: value,
          minimum: p.minimum,
          maximum: p.maximum,
          step: p.step,
        );
      }
      return p;
    }).toList();
    state = AsyncValue.data(nextState);
  }

  Future<void> resetParameters() async {
    final currentState = state.asData?.value;
    if (currentState == null) return;

    final presetPath = _getActivePresetPath();
    if (presetPath == null) return;

    final service = ref.read(nesTextureServiceProvider);
    final storage = ref.read(appStorageProvider);
    final List<video.ShaderParameter> nextState = [];

    for (final meta in currentState) {
      final name = meta.name;

      // 1. Reset backend to initial
      await service.setShaderParameter(name, meta.initial);

      // 2. Remove override from storage
      await storage.delete(_getStorageKey(presetPath, name));

      // 3. Update local state item
      nextState.add(
        video.ShaderParameter(
          name: meta.name,
          description: meta.description,
          initial: meta.initial,
          current: meta.initial,
          minimum: meta.minimum,
          maximum: meta.maximum,
          step: meta.step,
        ),
      );
    }
    state = AsyncValue.data(nextState);
  }

  String? _getActivePresetPath() {
    if (kIsWeb) return null;
    if (defaultTargetPlatform == TargetPlatform.android) {
      return ref.read(androidShaderSettingsProvider).presetPath;
    } else if (defaultTargetPlatform == TargetPlatform.macOS ||
        defaultTargetPlatform == TargetPlatform.iOS) {
      return ref.read(appleShaderSettingsProvider).presetPath;
    } else if (defaultTargetPlatform == TargetPlatform.windows) {
      return ref.read(windowsShaderSettingsProvider).presetPath;
    }
    return null;
  }
}

final shaderParametersProvider =
    NotifierProvider<
      ShaderParameterNotifier,
      AsyncValue<List<video.ShaderParameter>>
    >(ShaderParameterNotifier.new);
