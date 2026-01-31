import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/features/settings/android_video_backend_settings.dart';
import 'package:nesium_flutter/bridge/api/load_rom.dart' as nes_api;
import 'package:nesium_flutter/logging/app_logger.dart';
import 'package:nesium_flutter/platform/nes_video.dart' as nes_video;

import 'nes_state.dart';
import 'nes_texture_service.dart';

class NesController extends Notifier<NesState> {
  late final NesTextureService _textureService;

  bool get _isAndroid =>
      !kIsWeb && defaultTargetPlatform == TargetPlatform.android;

  bool get _useAndroidNativeGameView {
    if (!_isAndroid) return false;
    final backend = ref.read(androidVideoBackendSettingsProvider).backend;
    return backend == AndroidVideoBackend.hardware;
  }

  @override
  NesState build() {
    _textureService = NesTextureService();
    return NesState.initial();
  }

  Future<void> initTexture() async {
    state = state.copyWith(clearError: true);
    if (_useAndroidNativeGameView) {
      // Native SurfaceView path (Android hardware backend): no Flutter external
      // texture is created.
      return;
    }
    try {
      final textureId = await _textureService.createTexture();
      state = state.copyWith(textureId: textureId);
    } catch (e) {
      state = state.copyWith(error: e.toString());
    }
  }

  /// Applies the selected single video filter.
  ///
  /// Some filters are scaling filters and will resize the runtime's output framebuffer.
  /// In that case, we also best-effort resize the platform presentation buffer to match.
  Future<void> setVideoFilter(nes_video.VideoFilter filter) async {
    state = state.copyWith(clearError: true);
    try {
      final prevW = state.videoOutputWidth;
      final prevH = state.videoOutputHeight;
      final info = await nes_video.setVideoFilter(filter: filter);
      final w = info.outputWidth;
      final h = info.outputHeight;

      final needsResize = w != prevW || h != prevH;
      if (needsResize) {
        if (kIsWeb) {
          // Web renders via OffscreenCanvas in a Worker; there is no platform presentation buffer.
        } else if (_useAndroidNativeGameView) {
          // Keep the SurfaceView buffer size driven by layout so scaling happens in our
          // renderer with nearest-neighbor sampling (avoids system compositor bilinear scaling).
          await _textureService.resetAndroidSurfaceSizeFromLayout();
        } else {
          await _textureService.setPresentBufferSize(width: w, height: h);
        }
      }

      state = state.copyWith(videoOutputWidth: w, videoOutputHeight: h);
    } catch (e) {
      state = state.copyWith(error: e.toString());
    }
  }

  void updateTextureId(int? id) {
    state = state.copyWith(textureId: id);
  }

  Future<void> refreshRomHash() async {
    try {
      final hashBytes = await nes_api.getRomHash();
      if (hashBytes != null) {
        final hashStr = hashBytes
            .map((b) => b.toRadixString(16).padLeft(2, '0'))
            .join();
        updateRomInfo(hash: hashStr);
      } else {
        updateRomInfo(hash: null, name: null);
      }
    } catch (e, st) {
      logError(
        e,
        stackTrace: st,
        message: 'refreshRomHash failed',
        logger: 'nes_controller',
      );
    }
  }

  void updateRomInfo({String? hash, String? name}) {
    state = state.copyWith(romHash: hash, romName: name);
  }

  void updateRomHash(String? hash) {
    state = state.copyWith(romHash: hash);
  }

  void updateRomBytes(Uint8List? bytes) {
    if (bytes != null) {
      state = state.copyWith(romBytes: bytes);
    } else {
      state = state.copyWith(clearRomBytes: true);
    }
  }

  /// Updates the presentation buffer size to match the physical window size.
  ///
  /// This is critical for shaders on Windows to render at native resolution (HiDPI).
  Future<void> updateWindowOutputSize(int width, int height) async {
    if (_useAndroidNativeGameView) return;
    try {
      await _textureService.setPresentBufferSize(width: width, height: height);
    } catch (e) {
      logError(e, message: 'updateWindowOutputSize failed');
    }
  }
}

final nesControllerProvider = NotifierProvider<NesController, NesState>(
  NesController.new,
);
