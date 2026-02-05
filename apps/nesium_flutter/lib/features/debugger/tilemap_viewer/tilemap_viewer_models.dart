part of '../tilemap_viewer.dart';

@immutable
class _TileCoord {
  const _TileCoord(this.x, this.y);

  final int x;
  final int y;

  @override
  bool operator ==(Object other) =>
      other is _TileCoord && other.x == x && other.y == y;

  @override
  int get hashCode => Object.hash(x, y);
}

@immutable
class _TileInfo {
  const _TileInfo({
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
