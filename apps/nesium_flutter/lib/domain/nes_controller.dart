import 'package:flutter_riverpod/flutter_riverpod.dart';

import 'nes_state.dart';
import 'nes_texture_service.dart';

class NesController extends StateNotifier<NesState> {
  NesController(this._textureService) : super(NesState.initial());

  final NesTextureService _textureService;

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

final nesTextureServiceProvider = Provider<NesTextureService>((ref) {
  return NesTextureService();
});

final nesControllerProvider = StateNotifierProvider<NesController, NesState>((
  ref,
) {
  final service = ref.watch(nesTextureServiceProvider);
  return NesController(service);
});
