import 'package:flutter/widgets.dart';
import 'package:nesium_flutter/features/debugger/tile/tile_viewer_models.dart';

const int tileViewerColumns = 16;
const int tileViewerRows = 32;

String tileViewerHex(int value, {int width = 4}) =>
    '\$${value.toRadixString(16).toUpperCase().padLeft(width, '0')}';

TileCoord? tileAtPosition(Offset position, Size contentSize) {
  if (contentSize.width <= 0 || contentSize.height <= 0) return null;
  if (position.dx < 0 ||
      position.dy < 0 ||
      position.dx > contentSize.width ||
      position.dy > contentSize.height) {
    return null;
  }

  final tileWidth = contentSize.width / tileViewerColumns;
  final tileHeight = contentSize.height / tileViewerRows;
  final x = (position.dx / tileWidth).floor().clamp(0, tileViewerColumns - 1);
  final y = (position.dy / tileHeight).floor().clamp(0, tileViewerRows - 1);
  return TileCoord(x, y);
}

TileInfo computeTileInfo(TileCoord tile) {
  final tileIndex = tile.y * tileViewerColumns + tile.x;
  final patternTable = tileIndex >= 256 ? 1 : 0;
  final tileIndexInTable = tileIndex % 256;
  final chrAddress = patternTable * 0x1000 + tileIndexInTable * 16;

  return TileInfo(
    tileIndex: tileIndex,
    patternTable: patternTable,
    tileIndexInTable: tileIndexInTable,
    chrAddress: chrAddress,
  );
}
