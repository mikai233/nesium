part of '../tile_viewer.dart';

/// Memory source for tile data (matches Rust backend)
enum _TileSource { ppu, chrRom, chrRam, prgRom }

/// Tile layout mode (matches Rust backend)
enum _TileLayout { normal, singleLine8x16, singleLine16x16 }

/// Tile background color (matches Rust backend)
enum _TileBackground {
  defaultBg,
  transparent,
  paletteColor,
  black,
  white,
  magenta,
}

/// All Mesen2-style presets (both source and palette presets)
enum _Preset { ppu, chr, rom, bg, oam }

enum _CaptureMode { frameStart, vblankStart, scanline }
