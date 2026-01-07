import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/bridge/api/load_rom.dart' as nes_api;
import 'package:nesium_flutter/logging/app_logger.dart';

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
    try {
      final textureId = await _textureService.createTexture();
      state = state.copyWith(textureId: textureId);
    } catch (e) {
      state = state.copyWith(error: e.toString());
    }
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
