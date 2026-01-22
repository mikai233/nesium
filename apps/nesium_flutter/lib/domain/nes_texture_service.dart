import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

final nesTextureServiceProvider = Provider((ref) => NesTextureService());

/// Handles platform channel for creating the external NES texture.

class NesTextureService {
  static const MethodChannel _channel = MethodChannel('nesium');
  static const MethodChannel _auxChannel = MethodChannel('nesium_aux');

  Future<int?> createTexture() =>
      _channel.invokeMethod<int>('createNesTexture');

  Future<int?> disposeTexture() =>
      _channel.invokeMethod<int>('disposeNesTexture');

  /// Switches the Android video backend.
  ///
  /// - `0`: Kotlin GL uploader (software upload)
  /// - `1`: AHardwareBuffer swapchain + Rust EGL/GL renderer
  ///
  /// Note: takes effect on next app restart.
  Future<void> setVideoBackend(int mode) =>
      _channel.invokeMethod<void>('setVideoBackend', {'mode': mode});

  Future<void> setAndroidHighPriority(bool enabled) => _channel
      .invokeMethod<void>('setAndroidHighPriority', {'enabled': enabled});

  /// Switches the Windows video backend.
  ///
  /// - `true`: D3D11 GPU texture sharing (zero-copy)
  /// - `false`: CPU PixelBufferTexture fallback
  Future<int?> setWindowsVideoBackend(bool useGpu) =>
      _channel.invokeMethod<int>('setWindowsVideoBackend', {'useGpu': useGpu});

  Future<void> setWindowsHighPriority(bool enabled) async {
    await _channel.invokeMethod('setWindowsHighPriority', {'enabled': enabled});
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
