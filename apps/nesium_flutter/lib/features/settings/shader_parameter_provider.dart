import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../bridge/api/video.dart' as video;
import '../../domain/nes_texture_service.dart';
import '../../persistence/app_storage.dart';
import '../../logging/app_logger.dart';

/// Provides the list of parameters for the currently active shader, with persistence.
class ShaderParameterNotifier
    extends Notifier<AsyncValue<List<video.ShaderParameter>>> {
  String? _activePresetPath;

  @override
  AsyncValue<List<video.ShaderParameter>> build() {
    // State is managed by platform controllers calling [onShaderLoaded].
    return const AsyncValue.data([]);
  }

  /// Clears the current parameters and active path.
  void clear() {
    _activePresetPath = null;
    state = const AsyncValue.data([]);
  }

  Future<void> _handleUpdate(
    video.ShaderParameters parameters,
    String requestedPreset,
  ) async {
    try {
      // Map Rust-provided parameters to local state while applying saved overrides.

      final storage = ref.read(appStorageProvider);
      final service = ref.read(nesTextureServiceProvider);
      final List<video.ShaderParameter> updatedParams = [];
      final seenNames = <String>{};

      for (final meta in parameters.parameters) {
        final name = meta.name;

        if (seenNames.contains(name)) continue;
        seenNames.add(name);

        final storageKey = _getStorageKey(requestedPreset, name);
        final savedValue = storage.get(storageKey);

        if (savedValue is double &&
            (savedValue - meta.current).abs() > 0.0001) {
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
      appLog.severe('ShaderParamProvider: Error in _handleUpdate', e, st);
      state = AsyncValue.error(e, st);
    }
  }

  String _getStorageKey(String presetPath, String paramName) {
    return 'settings.shader.params.$presetPath.$paramName';
  }

  Future<void> updateParameter(String name, double value) async {
    final currentState = state.asData?.value;
    if (currentState == null) return;

    final presetPath = _activePresetPath;
    if (presetPath == null) return;

    final service = ref.read(nesTextureServiceProvider);
    final storage = ref.read(appStorageProvider);

    await service.setShaderParameter(name, value);
    await storage.put(_getStorageKey(presetPath, name), value);

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

    final presetPath = _activePresetPath;
    if (presetPath == null) return;

    final service = ref.read(nesTextureServiceProvider);
    final storage = ref.read(appStorageProvider);
    final List<video.ShaderParameter> nextState = [];

    for (final meta in currentState) {
      final name = meta.name;
      await service.setShaderParameter(name, meta.initial);
      await storage.delete(_getStorageKey(presetPath, name));

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

  /// Called when a shader is loaded and its parameters are received directly.
  Future<void> onShaderLoaded(
    video.ShaderParameters parameters,
    String requestedPreset,
  ) async {
    _activePresetPath = requestedPreset;
    await _handleUpdate(parameters, requestedPreset);
  }

  String? get activePresetPath => _activePresetPath;
}

final shaderParametersProvider =
    NotifierProvider<
      ShaderParameterNotifier,
      AsyncValue<List<video.ShaderParameter>>
    >(ShaderParameterNotifier.new);
