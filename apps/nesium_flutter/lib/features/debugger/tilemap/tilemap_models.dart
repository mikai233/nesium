import 'package:flutter/foundation.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/l10n/app_localizations.dart';

enum TilemapDisplayMode { defaultMode, grayscale, attributeView }

enum TilemapCaptureMode { frameStart, vblankStart, scanline }

@immutable
class TileCoord {
  const TileCoord(this.x, this.y);

  final int x;
  final int y;

  @override
  bool operator ==(Object other) =>
      other is TileCoord && other.x == x && other.y == y;

  @override
  int get hashCode => Object.hash(x, y);
}

@immutable
class TileInfo {
  const TileInfo({
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

  static TileInfo? compute(bridge.TilemapSnapshot snap, TileCoord tile) {
    final tileX = tile.x;
    final tileY = tile.y;
    final ntX = tileX >= 32 ? 1 : 0;
    final ntY = tileY >= 30 ? 1 : 0;
    final ntIndex = ntY * 2 + ntX;

    final tileXInNt = tileX % 32;
    final tileYInNt = tileY % 30;

    final ntLocalAddr = tileYInNt * 32 + tileXInNt;
    final tilemapAddress = 0x2000 + ntIndex * 0x400 + ntLocalAddr;

    final ciramBase = mirrorNametableToCiramOffset(ntIndex, snap.mirroring);
    final tileCiramAddr = ciramBase + ntLocalAddr;
    if (tileCiramAddr < 0 || tileCiramAddr >= snap.ciram.length) return null;

    final tileIndex = snap.ciram[tileCiramAddr];

    final attrLocalAddr = 0x3C0 + (tileYInNt ~/ 4) * 8 + (tileXInNt ~/ 4);
    final attrAddress = 0x2000 + ntIndex * 0x400 + attrLocalAddr;
    final attrCiramAddr = ciramBase + attrLocalAddr;
    final attrByte = attrCiramAddr >= 0 && attrCiramAddr < snap.ciram.length
        ? snap.ciram[attrCiramAddr]
        : 0;

    final shift = ((tileYInNt % 4) ~/ 2) * 4 + ((tileXInNt % 4) ~/ 2) * 2;
    final paletteIndex = (attrByte >> shift) & 0x03;
    final paletteAddress = 0x3F00 + paletteIndex * 4;

    final tileAddressPpu = snap.bgPatternBase + tileIndex * 16;

    return TileInfo(
      tileX: tileX,
      tileY: tileY,
      ntIndex: ntIndex,
      tileIndex: tileIndex,
      tilemapAddress: tilemapAddress,
      tileAddressPpu: tileAddressPpu,
      paletteIndex: paletteIndex,
      paletteAddress: paletteAddress,
      attrAddress: attrAddress,
      attrByte: attrByte,
    );
  }

  static int mirrorNametableToCiramOffset(
    int ntIndex,
    bridge.TilemapMirroring mirroring,
  ) {
    final physicalNt = switch (mirroring) {
      bridge.TilemapMirroring.horizontal =>
        (ntIndex == 0 || ntIndex == 1) ? 0 : 1,
      bridge.TilemapMirroring.vertical =>
        (ntIndex == 0 || ntIndex == 2) ? 0 : 1,
      bridge.TilemapMirroring.fourScreen => ntIndex.clamp(0, 1),
      bridge.TilemapMirroring.singleScreenLower => 0,
      bridge.TilemapMirroring.singleScreenUpper => 1,
      bridge.TilemapMirroring.mapperControlled => ntIndex.clamp(0, 1),
    };
    return physicalNt * 0x400;
  }
}

String formatHex(int value, {int width = 4}) {
  return '\$${value.toRadixString(16).toUpperCase().padLeft(width, '0')}';
}

String mirroringLabel(AppLocalizations l10n, bridge.TilemapMirroring m) {
  switch (m) {
    case bridge.TilemapMirroring.horizontal:
      return l10n.tilemapMirroringHorizontal;
    case bridge.TilemapMirroring.vertical:
      return l10n.tilemapMirroringVertical;
    case bridge.TilemapMirroring.fourScreen:
      return l10n.tilemapMirroringFourScreen;
    case bridge.TilemapMirroring.singleScreenLower:
      return l10n.tilemapMirroringSingleScreenLower;
    case bridge.TilemapMirroring.singleScreenUpper:
      return l10n.tilemapMirroringSingleScreenUpper;
    case bridge.TilemapMirroring.mapperControlled:
      return l10n.tilemapMirroringMapperControlled;
  }
}
