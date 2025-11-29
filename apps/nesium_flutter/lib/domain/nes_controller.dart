import 'package:flutter_riverpod/flutter_riverpod.dart';

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
    state = state.copyWith(loading: true, clearError: true);
    try {
      final textureId = await _textureService.createTexture();
      state = state.copyWith(loading: false, textureId: textureId);
    } catch (e) {
      state = state.copyWith(loading: false, error: e.toString());
    }
  }
}

final nesControllerProvider = NotifierProvider<NesController, NesState>(
  NesController.new,
);
