import 'dart:async';

import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/domain/aux_texture_ids.dart';
import 'package:nesium_flutter/domain/nes_texture_service.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer_models.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer_state.dart';
import 'package:nesium_flutter/logging/app_logger.dart';

class SpriteViewerController {
  SpriteViewerController({NesTextureService? textureService})
    : _textureService = textureService ?? NesTextureService();

  final NesTextureService _textureService;

  StreamSubscription<bridge.SpriteSnapshot>? _subscription;
  int? _spriteTextureId;
  int? _spriteScreenTextureId;
  bool _disposed = false;
  int _streamGeneration = 0;
  final Set<Future<void>> _textureOperations = {};

  Future<void> applyCaptureMode(SpriteViewerCaptureState captureState) async {
    switch (captureState.mode) {
      case SpriteCaptureMode.frameStart:
        await bridge.setSpriteCaptureFrameStart();
      case SpriteCaptureMode.vblankStart:
        await bridge.setSpriteCaptureVblankStart();
      case SpriteCaptureMode.scanline:
        await bridge.setSpriteCaptureScanline(
          scanline: captureState.scanline,
          dot: captureState.dot,
        );
    }
  }

  Future<void> startStreaming({
    required SpriteViewerCaptureState captureState,
    required void Function(bridge.SpriteSnapshot snapshot) onSnapshot,
    void Function(Object error)? onError,
  }) async {
    if (_disposed) return;
    final generation = ++_streamGeneration;
    await applyCaptureMode(captureState);
    if (_disposed || generation != _streamGeneration) return;
    await _subscription?.cancel();
    if (_disposed || generation != _streamGeneration) return;
    _subscription = bridge.spriteStateStream().listen(
      onSnapshot,
      onError: (error) => onError?.call(error),
    );
  }

  Future<void> stopStreaming() async {
    _streamGeneration++;
    await _subscription?.cancel();
    _subscription = null;
    await unsubscribe();
  }

  Future<void> unsubscribe() async {
    try {
      await bridge.unsubscribeSpriteState();
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'Failed to unsubscribe sprite state',
        logger: 'sprite_viewer',
      );
    }
  }

  Future<SpriteTextureHandle> ensureThumbTexture({
    required bridge.SpriteSnapshot snapshot,
    SpriteTextureHandle? currentHandle,
  }) => _trackTextureOperation(() async {
    final ids = await AuxTextureIdsCache.get();
    if (_disposed) throw StateError('Sprite viewer is disposed');
    _spriteTextureId ??= ids.sprite;

    final width = SpriteViewerLayout.gridTextureWidth(snapshot);
    final height = SpriteViewerLayout.gridTextureHeight(snapshot);
    if (width <= 0 || height <= 0) {
      throw StateError('Invalid sprite thumbnail texture size');
    }

    if (currentHandle != null &&
        currentHandle.width == width &&
        currentHandle.height == height) {
      return currentHandle;
    }

    await _textureService.pauseAuxTexture(_spriteTextureId!);
    await _textureService.disposeAuxTexture(_spriteTextureId!);

    final textureId = await _textureService.createAuxTexture(
      id: _spriteTextureId!,
      width: width,
      height: height,
    );
    if (textureId == null) {
      throw StateError('createAuxTexture returned null');
    }

    return SpriteTextureHandle(
      textureId: textureId,
      width: width,
      height: height,
    );
  });

  Future<SpriteTextureHandle> ensureScreenTexture({
    SpriteTextureHandle? currentHandle,
  }) => _trackTextureOperation(() async {
    final ids = await AuxTextureIdsCache.get();
    if (_disposed) throw StateError('Sprite viewer is disposed');
    _spriteScreenTextureId ??= ids.spriteScreen;

    const width = SpriteViewerLayout.previewWidth;
    const height = SpriteViewerLayout.previewHeight;

    if (currentHandle != null &&
        currentHandle.width == width &&
        currentHandle.height == height) {
      return currentHandle;
    }

    await _textureService.pauseAuxTexture(_spriteScreenTextureId!);
    await _textureService.disposeAuxTexture(_spriteScreenTextureId!);

    final textureId = await _textureService.createAuxTexture(
      id: _spriteScreenTextureId!,
      width: width,
      height: height,
    );
    if (textureId == null) {
      throw StateError('createAuxTexture returned null');
    }

    return SpriteTextureHandle(
      textureId: textureId,
      width: width,
      height: height,
    );
  });

  Future<SpriteTextureHandle> _trackTextureOperation(
    Future<SpriteTextureHandle> Function() operation,
  ) {
    if (_disposed) return Future.error(StateError('Sprite viewer is disposed'));
    final result = operation();
    // The caller handles errors; disposal only needs to wait for completion.
    final completed = result.then<void>(
      (_) {},
      onError: (Object _, StackTrace _) {},
    );
    _textureOperations.add(completed);
    unawaited(
      completed.whenComplete(() => _textureOperations.remove(completed)),
    );
    return result;
  }

  Future<void> dispose() async {
    if (_disposed) return;
    _disposed = true;
    await stopStreaming();
    // A native create call may still be in flight when the window closes.
    await Future.wait(_textureOperations.toList());
    final spriteTextureId = _spriteTextureId;
    final spriteScreenTextureId = _spriteScreenTextureId;

    if (spriteTextureId != null) {
      await _textureService.pauseAuxTexture(spriteTextureId);
    }
    if (spriteScreenTextureId != null) {
      await _textureService.pauseAuxTexture(spriteScreenTextureId);
    }

    if (spriteTextureId != null) {
      await _textureService.disposeAuxTexture(spriteTextureId);
    }
    if (spriteScreenTextureId != null) {
      await _textureService.disposeAuxTexture(spriteScreenTextureId);
    }
  }
}
