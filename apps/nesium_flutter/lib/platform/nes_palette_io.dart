export '../bridge/api/palette.dart' show PaletteKind;

import '../bridge/api/palette.dart' as frb_palette;

Future<void> setPalettePreset({required frb_palette.PaletteKind kind}) =>
    frb_palette.setPalettePreset(kind: kind);

Future<void> setPalettePalData({required List<int> data}) =>
    frb_palette.setPalettePalData(data: data);
