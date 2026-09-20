import 'package:flutter/material.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer_models.dart';

String formatHexValue(int value, {int width = 2}) =>
    '\$${value.toRadixString(16).toUpperCase().padLeft(width, '0')}';

int spriteActualY(bridge.SpriteInfo sprite) => (sprite.y + 1) & 0xFF;

String spriteSizeLabel(bridge.SpriteSnapshot snapshot) =>
    snapshot.largeSprites ? '8×16' : '8×8';

int spriteTileBaseAddr(
  bridge.SpriteSnapshot snapshot,
  bridge.SpriteInfo sprite,
) {
  if (!snapshot.largeSprites) {
    return snapshot.patternBase + sprite.tileIndex * 16;
  }

  final tableBase = (sprite.tileIndex & 0x01) != 0 ? 0x1000 : 0x0000;
  final baseTile = sprite.tileIndex & 0xFE;
  return tableBase + baseTile * 16;
}

int spritePaletteAddr(bridge.SpriteInfo sprite) => 0x3F10 + sprite.palette * 4;

String spriteFlipLabel(bridge.SpriteInfo sprite) =>
    '${sprite.flipH ? 'H' : '-'}${sprite.flipV ? 'V' : '-'}';

SpriteMetadata buildSpriteMetadata(
  bridge.SpriteSnapshot snapshot,
  bridge.SpriteInfo sprite,
) {
  final tileAddr = spriteTileBaseAddr(snapshot, sprite);
  return SpriteMetadata(
    actualY: spriteActualY(sprite),
    sizeLabel: spriteSizeLabel(snapshot),
    tileAddr: tileAddr,
    tileAddr2: snapshot.largeSprites ? tileAddr + 16 : null,
    paletteAddr: spritePaletteAddr(sprite),
    flipLabel: spriteFlipLabel(sprite),
  );
}

String spriteTileAddressLabel(SpriteMetadata metadata) =>
    metadata.tileAddr2 == null
    ? formatHexValue(metadata.tileAddr, width: 4)
    : '${formatHexValue(metadata.tileAddr, width: 4)} / ${formatHexValue(metadata.tileAddr2!, width: 4)}';

bool matrixNear(Matrix4 a, Matrix4 b, {double eps = 1e-3}) {
  final as = a.storage;
  final bs = b.storage;
  for (var i = 0; i < 16; i++) {
    if ((as[i] - bs[i]).abs() > eps) return false;
  }
  return true;
}

Offset transformToPreviewContent(Matrix4 transform, Offset screenPos) {
  final inverted = Matrix4.inverted(transform);
  return MatrixUtils.transformPoint(inverted, screenPos);
}

Matrix4 computePreviewDefaultTransform({
  required Size viewportSize,
  required double contentW,
  required double contentH,
  required double maxScale,
}) {
  final fitScale = viewportSize.width <= 0
      ? 1.0
      : viewportSize.width / contentW;
  final scale = fitScale > maxScale ? maxScale : fitScale;
  final dx = (viewportSize.width - contentW * scale) / 2.0;
  final dy = (viewportSize.height - contentH * scale) / 2.0;
  return Matrix4.identity()
    ..translateByDouble(dx, dy, 0, 1)
    ..scaleByDouble(scale, scale, 1, 1);
}

int? gridIndexAtPosition({
  required Offset localPosition,
  required Size viewportSize,
  required bridge.SpriteSnapshot snapshot,
}) {
  if (viewportSize.width <= 0 || viewportSize.height <= 0) return null;
  if (localPosition.dx < 0 ||
      localPosition.dy < 0 ||
      localPosition.dx > viewportSize.width ||
      localPosition.dy > viewportSize.height) {
    return null;
  }

  final totalW = SpriteViewerLayout.gridTextureWidth(snapshot).toDouble();
  final totalH = SpriteViewerLayout.gridTextureHeight(snapshot).toDouble();
  final x = (localPosition.dx / viewportSize.width) * totalW;
  final y = (localPosition.dy / viewportSize.height) * totalH;
  final col = (x / snapshot.thumbnailWidth).floor().clamp(
    0,
    SpriteViewerLayout.gridCols - 1,
  );
  final row = (y / snapshot.thumbnailHeight).floor().clamp(
    0,
    SpriteViewerLayout.gridRows - 1,
  );
  return row * SpriteViewerLayout.gridCols + col;
}

int? hitTestSprite(bridge.SpriteSnapshot snapshot, Offset screenCoord) {
  final x = screenCoord.dx;
  final y = screenCoord.dy;
  final spriteH = snapshot.largeSprites ? 16.0 : 8.0;

  for (var i = 0; i < snapshot.sprites.length; i++) {
    final sprite = snapshot.sprites[i];
    final sx = sprite.x.toDouble();
    final sy = spriteActualY(sprite).toDouble();
    if (x >= sx && x < sx + 8 && y >= sy && y < sy + spriteH) {
      return i;
    }
  }
  return null;
}

Offset previewScreenCoordAtContentPosition(Offset contentPos) => contentPos;

Offset computeTooltipOffset({
  required Offset position,
  required Size viewportSize,
  required double tooltipWidth,
  required double tooltipHeight,
  double cursorOffset = 20.0,
}) {
  final spaceRight = viewportSize.width - position.dx - cursorOffset;
  final spaceLeft = position.dx - cursorOffset;
  final spaceBottom = viewportSize.height - position.dy - cursorOffset;
  final spaceTop = position.dy - cursorOffset;

  double dx;
  double dy;

  if (spaceRight >= tooltipWidth) {
    dx = position.dx + cursorOffset;
  } else if (spaceLeft >= tooltipWidth) {
    dx = position.dx - cursorOffset - tooltipWidth;
  } else {
    dx = (viewportSize.width - tooltipWidth).clamp(0.0, double.infinity);
  }

  if (spaceBottom >= tooltipHeight) {
    dy = position.dy + cursorOffset;
  } else if (spaceTop >= tooltipHeight) {
    dy = position.dy - cursorOffset - tooltipHeight;
  } else {
    dy = 0.0;
  }

  return Offset(
    dx.clamp(
      0.0,
      (viewportSize.width - tooltipWidth).clamp(0.0, double.infinity),
    ),
    dy.clamp(
      0.0,
      (viewportSize.height - tooltipHeight).clamp(0.0, double.infinity),
    ),
  );
}

Rect computeOverlayTooltipRect({
  required Offset globalPosition,
  required Size screenSize,
  required double tooltipWidth,
  required double tooltipHeight,
  double cursorGap = 16.0,
  double screenPadding = 8.0,
}) {
  final bounds = Rect.fromLTWH(
    screenPadding,
    screenPadding,
    (screenSize.width - screenPadding * 2).clamp(0.0, double.infinity),
    (screenSize.height - screenPadding * 2).clamp(0.0, double.infinity),
  );

  Rect rectFor(double left, double top) =>
      Rect.fromLTWH(left, top, tooltipWidth, tooltipHeight);

  bool fits(Rect rect) =>
      bounds.contains(rect.topLeft) && bounds.contains(rect.bottomRight);

  final cx = globalPosition.dx;
  final cy = globalPosition.dy;
  final candidates = <Rect>[
    rectFor(cx + cursorGap, cy + cursorGap),
    rectFor(cx - cursorGap - tooltipWidth, cy + cursorGap),
    rectFor(cx + cursorGap, cy - cursorGap - tooltipHeight),
    rectFor(cx - cursorGap - tooltipWidth, cy - cursorGap - tooltipHeight),
  ];

  var rect = candidates.firstWhere(
    fits,
    orElse: () {
      final left = (cx + cursorGap).clamp(
        bounds.left,
        (bounds.right - tooltipWidth).clamp(bounds.left, bounds.right),
      );
      final top = (cy + cursorGap).clamp(
        bounds.top,
        (bounds.bottom - tooltipHeight).clamp(bounds.top, bounds.bottom),
      );
      return rectFor(left, top);
    },
  );

  if (!rect.contains(Offset(cx, cy))) return rect;

  final tryRects =
      <Rect>[
            rectFor(cx + cursorGap, rect.top),
            rectFor(cx - cursorGap - tooltipWidth, rect.top),
            rectFor(rect.left, cy + cursorGap),
            rectFor(rect.left, cy - cursorGap - tooltipHeight),
          ]
          .map(
            (candidate) => rectFor(
              candidate.left.clamp(
                bounds.left,
                (bounds.right - tooltipWidth).clamp(bounds.left, bounds.right),
              ),
              candidate.top.clamp(
                bounds.top,
                (bounds.bottom - tooltipHeight).clamp(
                  bounds.top,
                  bounds.bottom,
                ),
              ),
            ),
          )
          .toList();

  return tryRects.firstWhere(
    (candidate) => !candidate.contains(Offset(cx, cy)) && fits(candidate),
    orElse: () => rect,
  );
}
