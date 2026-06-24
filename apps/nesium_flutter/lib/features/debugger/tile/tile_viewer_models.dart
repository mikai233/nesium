/// Memory source for tile data (matches Rust backend).
enum TileSource { ppu, chrRom, chrRam, prgRom }

/// Tile layout mode (matches Rust backend).
enum TileLayout { normal, singleLine8x16, singleLine16x16 }

/// Tile background color (matches Rust backend).
enum TileBackground {
  defaultBg,
  transparent,
  paletteColor,
  black,
  white,
  magenta,
}

/// All Mesen2-style presets (both source and palette presets).
enum TilePreset { ppu, chr, rom, bg, oam }

enum TileCaptureMode { frameStart, vblankStart, scanline }

/// Tile coordinate in the CHR grid (0-15 for x, 0-31 for y).
class TileCoord {
  const TileCoord(this.x, this.y);

  final int x;
  final int y;

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is TileCoord && x == other.x && y == other.y;

  @override
  int get hashCode => Object.hash(x, y);
}

/// Tile information for tooltip display.
class TileInfo {
  const TileInfo({
    required this.tileIndex,
    required this.patternTable,
    required this.tileIndexInTable,
    required this.chrAddress,
  });

  /// 0-511 global index across both pattern tables.
  final int tileIndex;

  /// 0 or 1.
  final int patternTable;

  /// 0-255 index within the pattern table.
  final int tileIndexInTable;

  /// CHR address ($0000-$1FFF).
  final int chrAddress;
}
