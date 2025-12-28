export '../bridge/api/palette.dart' show PaletteKind;

import 'dart:typed_data';

import '../bridge/api/palette.dart' as frb_palette;
import 'web_cmd_sender.dart';

Future<void> setPalettePreset({required frb_palette.PaletteKind kind}) async {
  if (!isWebNesReady) return;
  webPostCmd('setPalettePreset', {'kind': kind.name});
}

Future<void> setPalettePalData({required List<int> data}) async {
  if (!isWebNesReady) return;
  webPostCmd('setPalettePalData', {'data': Uint8List.fromList(data)});
}
