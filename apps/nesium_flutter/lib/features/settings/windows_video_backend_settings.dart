import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../domain/nes_texture_service.dart';
import '../../persistence/keys.dart';
import '../../persistence/app_storage.dart';
import '../../domain/nes_controller.dart';
import '../../windows/current_window_kind.dart';
import '../../windows/window_types.dart';

enum WindowsVideoBackend { d3d11Gpu, softwareCpu }

class WindowsVideoBackendSettings {
  final WindowsVideoBackend backend;
  final bool useNativeOverlay;

  const WindowsVideoBackendSettings({
    required this.backend,
    this.useNativeOverlay = false,
  });

  bool get useGpu => backend == WindowsVideoBackend.d3d11Gpu;

  WindowsVideoBackendSettings copyWith({
    WindowsVideoBackend? backend,
    bool? useNativeOverlay,
  }) {
    return WindowsVideoBackendSettings(
      backend: backend ?? this.backend,
      useNativeOverlay: useNativeOverlay ?? this.useNativeOverlay,
    );
  }
}

class WindowsVideoBackendSettingsController
    extends Notifier<WindowsVideoBackendSettings> {
  bool get _isMainWindow =>
      ref.read(currentWindowKindProvider) == WindowKind.main;

  @override
  WindowsVideoBackendSettings build() {
    final storage = ref.read(appStorageProvider);
    final useGpuValue =
        storage.get(StorageKeys.settingsWindowsVideoBackend) as bool?;
    final useNativeOverlayValue =
        storage.get(StorageKeys.settingsWindowsNativeOverlay) as bool?;

    // Default to GPU (D3D11) if not set.
    final backend = (useGpuValue ?? true)
        ? WindowsVideoBackend.d3d11Gpu
        : WindowsVideoBackend.softwareCpu;

    final isWindows =
        !kIsWeb && defaultTargetPlatform == TargetPlatform.windows;
    if (isWindows) {
      // Apply the preference to the native plugin on startup.
      if (_isMainWindow) {
        Future.microtask(
          () => _applyBackend(backend == WindowsVideoBackend.d3d11Gpu),
        );
      }
    }

    // Listen for storage changes
    final subscription = ref.read(appStorageProvider).onKeyChanged.listen((
      event,
    ) {
      if (event.key == StorageKeys.settingsWindowsVideoBackend ||
          event.key == StorageKeys.settingsWindowsNativeOverlay) {
        state = build();
      }
    });

    ref.onDispose(() => subscription.cancel());

    return WindowsVideoBackendSettings(
      backend: backend,
      useNativeOverlay: useNativeOverlayValue ?? false,
    );
  }

  Future<void> setBackend(WindowsVideoBackend backend) async {
    if (state.backend == backend) return;

    final storage = ref.read(appStorageProvider);
    await storage.put(
      StorageKeys.settingsWindowsVideoBackend,
      backend == WindowsVideoBackend.d3d11Gpu,
    );

    state = state.copyWith(backend: backend);

    final isWindows =
        !kIsWeb && defaultTargetPlatform == TargetPlatform.windows;
    if (!isWindows) return;
    if (!_isMainWindow) return;
    await _applyBackend(backend == WindowsVideoBackend.d3d11Gpu);
  }

  Future<void> setNativeOverlay(bool value) async {
    if (state.useNativeOverlay == value) return;

    final storage = ref.read(appStorageProvider);
    await storage.put(StorageKeys.settingsWindowsNativeOverlay, value);

    state = state.copyWith(useNativeOverlay: value);
  }

  Future<void> _applyBackend(bool useGpu) async {
    final textureService = ref.read(nesTextureServiceProvider);
    final newId = await textureService.setWindowsVideoBackend(useGpu);
    if (newId != null) {
      ref.read(nesControllerProvider.notifier).updateTextureId(newId);
    }
  }
}

final windowsVideoBackendSettingsProvider =
    NotifierProvider<
      WindowsVideoBackendSettingsController,
      WindowsVideoBackendSettings
    >(WindowsVideoBackendSettingsController.new);
