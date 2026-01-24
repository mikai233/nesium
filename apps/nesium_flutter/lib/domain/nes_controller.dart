import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/bridge/api/load_rom.dart' as nes_api;
import 'package:nesium_flutter/logging/app_logger.dart';
import 'package:nesium_flutter/platform/platform_capabilities.dart';
import 'package:nesium_flutter/platform/nes_video.dart' as nes_video;

import 'nes_state.dart';
import 'nes_texture_service.dart';

class NesController extends Notifier<NesState> {
  late final NesTextureService _textureService;

  @override
  NesState build() {
    _textureService = NesTextureService();
    return NesState.initial();
  }

  Future<void> initTexture() async {
    state = state.copyWith(clearError: true);
    if (useAndroidNativeGameView) {
      // Native SurfaceView path (Android): no Flutter external texture is created.
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
        if (useAndroidNativeGameView) {
          await _textureService.setAndroidSurfaceSize(width: w, height: h);
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
}

final nesControllerProvider = NotifierProvider<NesController, NesState>(
  NesController.new,
);
