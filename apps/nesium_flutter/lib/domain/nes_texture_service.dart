import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../bridge/api/emulation.dart' as emulation;

final nesTextureServiceProvider = Provider((ref) => NesTextureService());

/// Handles platform channel for creating the external NES texture.

class NesTextureService {
  static const MethodChannel _channel = MethodChannel('nesium');
  static const MethodChannel _auxChannel = MethodChannel('nesium_aux');

  Future<void> _invokeSetAndroidSurfaceSize({
    required int width,
    required int height,
  }) => _channel.invokeMethod<void>('setAndroidSurfaceSize', {
    'width': width,
    'height': height,
  });

  Future<int?> createTexture() =>
      _channel.invokeMethod<int>('createNesTexture');

  Future<int?> disposeTexture() =>
      _channel.invokeMethod<int>('disposeNesTexture');

  /// Updates the platform presentation buffer size for the main NES output.
  ///
  /// This does **not** change the Rust runtime output size; that is controlled via FRB
  /// (video pipeline config). On some platforms (notably Android), setting the presentation
  /// buffer size is required to avoid system compositor scaling/blurring.
  Future<void> setPresentBufferSize({
    required int width,
    required int height,
  }) => _channel.invokeMethod<void>('setPresentBufferSize', {
    'width': width,
    'height': height,
  });

  /// Sets the Android native SurfaceView buffer size (PlatformView path).
  ///
  /// - If `width`/`height` are > 0: uses `SurfaceHolder.setFixedSize`.
  /// - Otherwise: resets to `SurfaceHolder.setSizeFromLayout`.
  /// Forces the Android native SurfaceView buffer size (PlatformView path).
  Future<void> setAndroidSurfaceFixedSize({
    required int width,
    required int height,
  }) {
    assert(width > 0 && height > 0);
    return _invokeSetAndroidSurfaceSize(width: width, height: height);
  }

  /// Resets the Android native SurfaceView buffer size to be driven by layout.
  Future<void> resetAndroidSurfaceSizeFromLayout() =>
      _invokeSetAndroidSurfaceSize(width: 0, height: 0);

  /// Switches the Android video backend.
  ///
  /// - `0`: Kotlin GL uploader (software upload)
  /// - `1`: AHardwareBuffer swapchain + Rust EGL/GL renderer
  ///
  /// Note: takes effect on next app restart.
  Future<void> setVideoBackend(int mode) =>
      _channel.invokeMethod<void>('setVideoBackend', {'mode': mode});

  Future<void> setAndroidHighPriority(bool enabled) =>
      emulation.setHighPriorityEnabled(enabled: enabled);

  /// Enables/disables the Rust-side librashader chain (Android hardware backend).
  ///
  /// This only affects the Android "Hardware" backend (AHardwareBuffer + Rust renderer).
  Future<void> setShaderEnabled(bool enabled) =>
      _channel.invokeMethod<void>('setShaderEnabled', {'enabled': enabled});

  /// Sets the shader preset path for librashader (Android hardware backend).
  ///
  /// Pass an empty string to clear.
  Future<void> setShaderPreset(String path) =>
      _channel.invokeMethod<void>('setShaderPreset', {'path': path});

  /// Switches the Windows video backend.
  ///
  /// - `true`: D3D11 GPU texture sharing (zero-copy)
  /// - `false`: CPU PixelBufferTexture fallback
  Future<int?> setWindowsVideoBackend(bool useGpu) =>
      _channel.invokeMethod<int>('setWindowsVideoBackend', {'useGpu': useGpu});

  Future<void> setWindowsHighPriority(bool enabled) async {
    await emulation.setHighPriorityEnabled(enabled: enabled);
  }

  Future<void> setAppleHighPriority(bool enabled) async {
    await emulation.setHighPriorityEnabled(enabled: enabled);
  }

  Future<void> setLinuxHighPriority(bool enabled) async {
    await emulation.setHighPriorityEnabled(enabled: enabled);
  }

  Future<void> setNativeOverlay({
    required bool enabled,
    double x = 0,
    double y = 0,
    double width = 0,
    double height = 0,
  }) async {
    await _channel.invokeMethod('setNativeOverlay', {
      'enabled': enabled,
      'x': x,
      'y': y,
      'width': width,
      'height': height,
    });
  }

  Future<void> setVideoFilter(int filter) async {
    await _channel.invokeMethod('setVideoFilter', {'filter': filter});
  }

  // ---------------------------------------------------------------------------
  // Auxiliary Textures (Tilemap, Pattern, etc.)
  // ---------------------------------------------------------------------------

  /// Creates an auxiliary texture with the given ID and dimensions.
  /// Returns the Flutter texture ID to use with a [Texture] widget.
  Future<int?> createAuxTexture({
    required int id,
    required int width,
    required int height,
  }) => _auxChannel.invokeMethod<int>('createAuxTexture', {
    'id': id,
    'width': width,
    'height': height,
  });

  /// Pauses updates for an auxiliary texture.
  /// Call this before dispose to prevent race conditions.
  Future<void> pauseAuxTexture(int id) =>
      _auxChannel.invokeMethod<void>('pauseAuxTexture', {'id': id});

  /// Disposes an auxiliary texture.
  Future<void> disposeAuxTexture(int id) =>
      _auxChannel.invokeMethod<void>('disposeAuxTexture', {'id': id});
}
