import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/api/palette.dart' as nes_palette;

enum PaletteMode { builtin, custom }

@immutable
class VideoSettings {
  static const Object _unset = Object();

  const VideoSettings({
    required this.paletteMode,
    required this.builtinPaletteId,
    this.customPaletteName,
  });

  final PaletteMode paletteMode;
  final String builtinPaletteId;
  final String? customPaletteName;

  VideoSettings copyWith({
    PaletteMode? paletteMode,
    String? builtinPaletteId,
    Object? customPaletteName = _unset,
  }) {
    return VideoSettings(
      paletteMode: paletteMode ?? this.paletteMode,
      builtinPaletteId: builtinPaletteId ?? this.builtinPaletteId,
      customPaletteName: identical(customPaletteName, _unset)
          ? this.customPaletteName
          : customPaletteName as String?,
    );
  }

  static VideoSettings defaults() {
    return const VideoSettings(
      paletteMode: PaletteMode.builtin,
      builtinPaletteId: 'nesdev-ntsc',
      customPaletteName: null,
    );
  }
}

class VideoSettingsController extends Notifier<VideoSettings> {
  @override
  VideoSettings build() => VideoSettings.defaults();

  Future<void> setBuiltinPalette(String id) async {
    if (id == state.builtinPaletteId &&
        state.paletteMode == PaletteMode.builtin) {
      return;
    }
    state = state.copyWith(
      paletteMode: PaletteMode.builtin,
      builtinPaletteId: id,
      customPaletteName: null,
    );
    await nes_palette.setPalettePreset(id: id);
  }

  Future<void> setCustomPalette(Uint8List data, {String? name}) async {
    state = state.copyWith(
      paletteMode: PaletteMode.custom,
      customPaletteName: name ?? state.customPaletteName ?? 'custom',
    );
    await nes_palette.setPalettePalData(data: data);
  }

  void useCustomIfAvailable() {
    if (state.customPaletteName == null) return;
    state = state.copyWith(paletteMode: PaletteMode.custom);
  }
}

final videoSettingsProvider =
    NotifierProvider<VideoSettingsController, VideoSettings>(
      VideoSettingsController.new,
    );
