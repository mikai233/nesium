part of '../tile_viewer.dart';

/// Tile coordinate in the CHR grid (0-15 for x, 0-31 for y)
class _TileCoord {
  final int x;
  final int y;

  const _TileCoord(this.x, this.y);

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is _TileCoord && x == other.x && y == other.y;

  @override
  int get hashCode => x.hashCode ^ y.hashCode;
}

/// Tile information for tooltip display
class _TileInfo {
  final int tileIndex; // 0-511 (global index across both pattern tables)
  final int patternTable; // 0 or 1
  final int tileIndexInTable; // 0-255 (index within the pattern table)
  final int chrAddress; // CHR address ($0000-$1FFF)

  const _TileInfo({
    required this.tileIndex,
    required this.patternTable,
    required this.tileIndexInTable,
    required this.chrAddress,
  });
}
