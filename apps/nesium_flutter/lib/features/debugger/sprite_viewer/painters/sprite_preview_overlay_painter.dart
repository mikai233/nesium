import 'package:flutter/material.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer_models.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer_utils.dart';

class SpritePreviewOverlayPainter extends CustomPainter {
  const SpritePreviewOverlayPainter({
    required this.sprites,
    required this.largeSprites,
    required this.showOutline,
    required this.showOffscreenRegions,
    required this.hoveredIndex,
    required this.selectedIndex,
  });

  final List<bridge.SpriteInfo> sprites;
  final bool largeSprites;
  final bool showOutline;
  final bool showOffscreenRegions;
  final int? hoveredIndex;
  final int? selectedIndex;

  @override
  void paint(Canvas canvas, Size size) {
    if (size.width <= 0 || size.height <= 0) return;

    final baseW = SpriteViewerLayout.screenWidth.toDouble();
    final baseH =
        (showOffscreenRegions
                ? SpriteViewerLayout.previewHeight
                : SpriteViewerLayout.screenHeight)
            .toDouble();

    final sx = size.width / baseW;
    final sy = size.height / baseH;
    final spriteH = (largeSprites ? 16 : 8).toDouble();

    if (showOffscreenRegions) {
      final paint = Paint()
        ..color = Colors.white.withValues(alpha: 0.5)
        ..strokeWidth = 1.0
        ..style = PaintingStyle.stroke;

      final rect = Rect.fromLTWH(
        0,
        0,
        baseW * sx,
        SpriteViewerLayout.screenHeight.toDouble() * sy,
      );
      canvas.drawRect(rect, paint);
    }

    if (showOutline) {
      final paint = Paint()
        ..color = Colors.white.withValues(alpha: 0.35)
        ..strokeWidth = 1.0
        ..style = PaintingStyle.stroke;

      for (final sprite in sprites) {
        final x = sprite.x.toDouble() * sx;
        final y = spriteActualY(sprite).toDouble() * sy;
        final rect = Rect.fromLTWH(x, y, 8 * sx, spriteH * sy);
        canvas.drawRect(rect, paint);
      }
    }

    if (hoveredIndex != null &&
        hoveredIndex! >= 0 &&
        hoveredIndex! < sprites.length) {
      final sprite = sprites[hoveredIndex!];
      final paint = Paint()
        ..color = Colors.white.withValues(alpha: 0.9)
        ..strokeWidth = 1.0
        ..style = PaintingStyle.stroke;
      final x = sprite.x.toDouble() * sx;
      final y = spriteActualY(sprite).toDouble() * sy;
      canvas.drawRect(Rect.fromLTWH(x, y, 8 * sx, spriteH * sy), paint);
    }

    if (selectedIndex != null &&
        selectedIndex! >= 0 &&
        selectedIndex! < sprites.length) {
      final sprite = sprites[selectedIndex!];
      final paint = Paint()
        ..color = Colors.yellow.withValues(alpha: 0.95)
        ..strokeWidth = 1.0
        ..style = PaintingStyle.stroke;
      final x = sprite.x.toDouble() * sx;
      final y = spriteActualY(sprite).toDouble() * sy;
      canvas.drawRect(Rect.fromLTWH(x, y, 8 * sx, spriteH * sy), paint);
    }
  }

  @override
  bool shouldRepaint(covariant SpritePreviewOverlayPainter oldDelegate) {
    return sprites != oldDelegate.sprites ||
        largeSprites != oldDelegate.largeSprites ||
        showOutline != oldDelegate.showOutline ||
        showOffscreenRegions != oldDelegate.showOffscreenRegions ||
        hoveredIndex != oldDelegate.hoveredIndex ||
        selectedIndex != oldDelegate.selectedIndex;
  }
}
