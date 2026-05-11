import 'dart:ui';

import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/features/debugger/tilemap/tilemap_models.dart';

const int tilemapLogicalWidth = 512;
const int tilemapLogicalHeight = 480;

String tilemapHex(int value, {int width = 4}) =>
    '\$${value.toRadixString(16).toUpperCase().padLeft(width, '0')}';

TilemapCoord? tilemapTileAtPosition(Offset position, Size size) {
  if (size.width <= 0 || size.height <= 0) return null;
  if (position.dx < 0 ||
      position.dy < 0 ||
      position.dx > size.width ||
      position.dy > size.height) {
    return null;
  }

  final x = (position.dx / size.width) * tilemapLogicalWidth;
  final y = (position.dy / size.height) * tilemapLogicalHeight;
  final tileX = (x / 8).floor().clamp(0, 63);
  final tileY = (y / 8).floor().clamp(0, 59);
  return TilemapCoord(tileX, tileY);
}

TilemapTileInfo? computeTilemapTileInfo(
  bridge.TilemapSnapshot snap,
  TilemapCoord tile,
) {
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

  return TilemapTileInfo(
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

int mirrorNametableToCiramOffset(
  int ntIndex,
  bridge.TilemapMirroring mirroring,
) {
  final physicalNt = switch (mirroring) {
    bridge.TilemapMirroring.horizontal =>
      (ntIndex == 0 || ntIndex == 1) ? 0 : 1,
    bridge.TilemapMirroring.vertical => (ntIndex == 0 || ntIndex == 2) ? 0 : 1,
    bridge.TilemapMirroring.fourScreen => ntIndex.clamp(0, 1),
    bridge.TilemapMirroring.singleScreenLower => 0,
    bridge.TilemapMirroring.singleScreenUpper => 1,
    bridge.TilemapMirroring.mapperControlled => ntIndex.clamp(0, 1),
  };
  return physicalNt * 0x400;
}

List<Rect> scrollOverlayRectsFromTilemapSnapshot(bridge.TilemapSnapshot snap) {
  // Use the PPU `t` (temp) address for scroll origin. The `v` address is
  // advanced by background fetches/pipeline and is not stable for viewport math.
  final v = snap.tempAddr & 0x7FFF;
  final fineX = snap.fineX & 0x07;
  final coarseX = v & 0x1F;
  final coarseY = (v >> 5) & 0x1F;
  final ntX = (v >> 10) & 0x01;
  final ntY = (v >> 11) & 0x01;
  final fineY = (v >> 12) & 0x07;

  final scrollX = coarseX * 8 + fineX;
  final scrollY = coarseY * 8 + fineY;
  final baseX = ntX * 256;
  final baseY = ntY * 240;

  final x0 = (baseX + scrollX) % tilemapLogicalWidth;
  final y0 = (baseY + scrollY) % tilemapLogicalHeight;

  return splitWrappedRect(
    x: x0.toDouble(),
    y: y0.toDouble(),
    w: 256.0,
    h: 240.0,
    wrapW: tilemapLogicalWidth.toDouble(),
    wrapH: tilemapLogicalHeight.toDouble(),
  );
}

List<Rect> splitWrappedRect({
  required double x,
  required double y,
  required double w,
  required double h,
  required double wrapW,
  required double wrapH,
}) {
  final x0 = x % wrapW;
  final y0 = y % wrapH;
  final right = x0 + w;
  final bottom = y0 + h;

  final w1 = (right <= wrapW) ? w : (wrapW - x0);
  final h1 = (bottom <= wrapH) ? h : (wrapH - y0);
  final w2 = w - w1;
  final h2 = h - h1;

  final rects = <Rect>[Rect.fromLTWH(x0, y0, w1, h1)];
  if (w2 > 0) {
    rects.add(Rect.fromLTWH(0, y0, w2, h1));
  }
  if (h2 > 0) {
    rects.add(Rect.fromLTWH(x0, 0, w1, h2));
    if (w2 > 0) {
      rects.add(Rect.fromLTWH(0, 0, w2, h2));
    }
  }
  return rects;
}
