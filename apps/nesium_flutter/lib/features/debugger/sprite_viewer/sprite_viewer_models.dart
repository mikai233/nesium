import 'package:flutter/material.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/l10n/app_localizations.dart';

class SpriteViewerLayout {
  const SpriteViewerLayout._();

  static const int gridCols = 8;
  static const int gridRows = 8;
  static const int screenWidth = 256;
  static const int screenHeight = 240;
  static const int offscreenBottomHeight = 16;
  static const int previewWidth = screenWidth;
  static const int previewHeight = screenHeight + offscreenBottomHeight;
  static const int minScanline = -1;
  static const int maxScanline = 260;
  static const int minDot = 0;
  static const int maxDot = 340;

  static int gridTextureWidth(bridge.SpriteSnapshot snapshot) =>
      gridCols * snapshot.thumbnailWidth;

  static int gridTextureHeight(bridge.SpriteSnapshot snapshot) =>
      gridRows * snapshot.thumbnailHeight;
}

enum SpriteViewerBackground { gray, black, white, magenta, transparent }

enum SpriteCaptureMode { frameStart, vblankStart, scanline }

enum SpriteDataSource { spriteRam, cpuMemory }

extension SpriteViewerBackgroundX on SpriteViewerBackground {
  String label(AppLocalizations l10n) => switch (this) {
    SpriteViewerBackground.gray => l10n.spriteViewerBgGray,
    SpriteViewerBackground.black => l10n.tileViewerBgBlack,
    SpriteViewerBackground.white => l10n.tileViewerBgWhite,
    SpriteViewerBackground.magenta => l10n.tileViewerBgMagenta,
    SpriteViewerBackground.transparent => l10n.tileViewerBgTransparent,
  };

  Color color(ThemeData theme) => switch (this) {
    SpriteViewerBackground.gray => const Color(0xFF808080),
    SpriteViewerBackground.black => Colors.black,
    SpriteViewerBackground.white => Colors.white,
    SpriteViewerBackground.magenta => const Color(0xFFFF00FF),
    SpriteViewerBackground.transparent => const Color(0x00000000),
  };
}

extension SpriteDataSourceX on SpriteDataSource {
  String label(AppLocalizations l10n) => switch (this) {
    SpriteDataSource.spriteRam => l10n.spriteViewerDataSourceSpriteRam,
    SpriteDataSource.cpuMemory => l10n.spriteViewerDataSourceCpuMemory,
  };
}

class SpriteOverlayData {
  const SpriteOverlayData({required this.sprites, required this.largeSprites});

  final List<bridge.SpriteInfo> sprites;
  final bool largeSprites;
}

class SpriteTextureHandle {
  const SpriteTextureHandle({
    required this.textureId,
    required this.width,
    required this.height,
  });

  final int textureId;
  final int width;
  final int height;
}

class SpriteMetadata {
  const SpriteMetadata({
    required this.actualY,
    required this.sizeLabel,
    required this.tileAddr,
    required this.tileAddr2,
    required this.paletteAddr,
    required this.flipLabel,
  });

  final int actualY;
  final String sizeLabel;
  final int tileAddr;
  final int? tileAddr2;
  final int paletteAddr;
  final String flipLabel;
}
