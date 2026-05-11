import 'package:flutter/foundation.dart';

enum TilemapDisplayMode { defaultMode, grayscale, attributeView }

enum TilemapCaptureMode { frameStart, vblankStart, scanline }

@immutable
class TilemapCoord {
  const TilemapCoord(this.x, this.y);

  final int x;
  final int y;

  @override
  bool operator ==(Object other) =>
      other is TilemapCoord && other.x == x && other.y == y;

  @override
  int get hashCode => Object.hash(x, y);
}

@immutable
class TilemapTileInfo {
  const TilemapTileInfo({
    required this.tileX,
    required this.tileY,
    required this.ntIndex,
    required this.tileIndex,
    required this.tilemapAddress,
    required this.tileAddressPpu,
    required this.paletteIndex,
    required this.paletteAddress,
    required this.attrAddress,
    required this.attrByte,
  });

  final int tileX;
  final int tileY;
  final int ntIndex;
  final int tileIndex;
  final int tilemapAddress;
  final int tileAddressPpu;
  final int paletteIndex;
  final int paletteAddress;
  final int attrAddress;
  final int attrByte;
}
