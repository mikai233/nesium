import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../bridge/api/palette.dart' as nes_palette;

enum PaletteMode { builtin, custom }

@immutable
class VideoSettings {
  static const Object _unset = Object();

  const VideoSettings({
    required this.paletteMode,
    required this.builtinPreset,
    this.customPaletteName,
  });

  final PaletteMode paletteMode;
  final nes_palette.PaletteKind builtinPreset;
  final String? customPaletteName;

  VideoSettings copyWith({
    PaletteMode? paletteMode,
    nes_palette.PaletteKind? builtinPreset,
    Object? customPaletteName = _unset,
  }) {
    return VideoSettings(
      paletteMode: paletteMode ?? this.paletteMode,
      builtinPreset: builtinPreset ?? this.builtinPreset,
      customPaletteName: identical(customPaletteName, _unset)
          ? this.customPaletteName
          : customPaletteName as String?,
    );
  }

  static VideoSettings defaults() {
    return const VideoSettings(
      paletteMode: PaletteMode.builtin,
      builtinPreset: nes_palette.PaletteKind.nesdevNtsc,
      customPaletteName: null,
    );
  }
}

class VideoSettingsController extends Notifier<VideoSettings> {
  @override
  VideoSettings build() => VideoSettings.defaults();

  Future<void> setBuiltinPreset(nes_palette.PaletteKind preset) async {
    if (preset == state.builtinPreset &&
        state.paletteMode == PaletteMode.builtin) {
      return;
    }
    state = state.copyWith(
      paletteMode: PaletteMode.builtin,
      builtinPreset: preset,
      customPaletteName: null,
    );
    await nes_palette.setPalettePreset(kind: preset);
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
