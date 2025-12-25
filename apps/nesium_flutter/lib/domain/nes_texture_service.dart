import 'package:flutter/services.dart';

/// Handles platform channel for creating the external NES texture.
class NesTextureService {
  static const MethodChannel _channel = MethodChannel('nesium');

  Future<int?> createTexture() =>
      _channel.invokeMethod<int>('createNesTexture');

  Future<int?> disposeTexture() =>
      _channel.invokeMethod<int>('disposeNesTexture');
}
