import 'package:flutter/services.dart';

/// Handles platform channel for creating the external NES texture.
class NesTextureService {
  static const MethodChannel _channel = MethodChannel('nesium');

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

  /// Enables or disables the Android low-latency video mode.
  ///
  /// Note: takes effect on next app restart.
  Future<void> setLowLatencyVideo(bool enabled) =>
      _channel.invokeMethod<void>('setLowLatencyVideo', {'enabled': enabled});
}
