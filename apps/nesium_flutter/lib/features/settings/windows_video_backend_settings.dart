import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../domain/nes_texture_service.dart';
import '../../persistence/keys.dart';
import '../../persistence/app_storage.dart';
import '../../domain/nes_controller.dart';
import '../../windows/current_window_kind.dart';
import '../../windows/settings_sync.dart';
import '../../windows/window_types.dart';

enum WindowsVideoBackend { d3d11Gpu, softwareCpu }

class WindowsVideoBackendSettings {
  final WindowsVideoBackend backend;

  const WindowsVideoBackendSettings({required this.backend});

  bool get useGpu => backend == WindowsVideoBackend.d3d11Gpu;

  WindowsVideoBackendSettings copyWith({WindowsVideoBackend? backend}) {
    return WindowsVideoBackendSettings(backend: backend ?? this.backend);
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

    return WindowsVideoBackendSettings(backend: backend);
  }

  Future<void> setBackend(WindowsVideoBackend backend) async {
    if (state.backend == backend) return;

    final storage = ref.read(appStorageProvider);
    await storage.put(
      StorageKeys.settingsWindowsVideoBackend,
      backend == WindowsVideoBackend.d3d11Gpu,
    );
    unawaited(
      SettingsSync.broadcast(
        group: 'windowsVideoBackend',
        fields: const ['backend'],
      ),
    );

    state = state.copyWith(backend: backend);

    final isWindows =
        !kIsWeb && defaultTargetPlatform == TargetPlatform.windows;
    if (!isWindows) return;
    if (!_isMainWindow) return;
    await _applyBackend(backend == WindowsVideoBackend.d3d11Gpu);
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
