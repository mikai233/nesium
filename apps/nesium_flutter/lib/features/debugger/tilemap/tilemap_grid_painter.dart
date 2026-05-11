import 'package:flutter/material.dart';
import 'package:nesium_flutter/features/debugger/tilemap/tilemap_geometry.dart';
import 'package:nesium_flutter/features/debugger/tilemap/tilemap_models.dart';

/// CustomPainter for drawing grid overlays on tilemap texture.
/// Uses vector graphics for resolution-independent sharp rendering.
class TilemapGridPainter extends CustomPainter {
  TilemapGridPainter({
    required this.showTileGrid,
    required this.showAttributeGrid,
    required this.showAttributeGrid32,
    required this.showNametableDelimiters,
    required this.showScrollOverlay,
    required this.scrollOverlayRects,
    required this.hoveredTile,
    required this.selectedTile,
  });

  final bool showTileGrid;
  final bool showAttributeGrid;
  final bool showAttributeGrid32;
  final bool showNametableDelimiters;
  final bool showScrollOverlay;
  final List<Rect> scrollOverlayRects;
  final TilemapCoord? hoveredTile;
  final TilemapCoord? selectedTile;

  @override
  void paint(Canvas canvas, Size size) {
    final scaleX = size.width / tilemapLogicalWidth;
    final scaleY = size.height / tilemapLogicalHeight;

    if (showTileGrid) {
      final paint = Paint()
        ..color = Colors.white.withValues(alpha: 0.5)
        ..strokeWidth = 1.0
        ..style = PaintingStyle.stroke;

      for (var y = 0; y <= tilemapLogicalHeight; y += 8) {
        final scaledY = y * scaleY;
        canvas.drawLine(Offset(0, scaledY), Offset(size.width, scaledY), paint);
      }
      for (var x = 0; x <= tilemapLogicalWidth; x += 8) {
        final scaledX = x * scaleX;
        canvas.drawLine(
          Offset(scaledX, 0),
          Offset(scaledX, size.height),
          paint,
        );
      }
    }

    if (showAttributeGrid) {
      final paint = Paint()
        ..color = Colors.cyan.withValues(alpha: 0.7)
        ..strokeWidth = 0.8
        ..style = PaintingStyle.stroke;

      for (var y = 0; y <= tilemapLogicalHeight; y += 16) {
        final scaledY = y * scaleY;
        canvas.drawLine(Offset(0, scaledY), Offset(size.width, scaledY), paint);
      }
      for (var x = 0; x <= tilemapLogicalWidth; x += 16) {
        final scaledX = x * scaleX;
        canvas.drawLine(
          Offset(scaledX, 0),
          Offset(scaledX, size.height),
          paint,
        );
      }
    }

    if (showAttributeGrid32) {
      final paint = Paint()
        ..color = Colors.orange.withValues(alpha: 0.55)
        ..strokeWidth = 1.0
        ..style = PaintingStyle.stroke;

      for (var y = 0; y <= tilemapLogicalHeight; y += 32) {
        final scaledY = y * scaleY;
        canvas.drawLine(Offset(0, scaledY), Offset(size.width, scaledY), paint);
      }
      for (var x = 0; x <= tilemapLogicalWidth; x += 32) {
        final scaledX = x * scaleX;
        canvas.drawLine(
          Offset(scaledX, 0),
          Offset(scaledX, size.height),
          paint,
        );
      }
    }

    if (showNametableDelimiters) {
      final paint = Paint()
        ..color = Colors.white.withValues(alpha: 0.9)
        ..strokeWidth = 1.0
        ..style = PaintingStyle.stroke;

      final verticalX = 256.0 * scaleX;
      canvas.drawLine(
        Offset(verticalX, 0),
        Offset(verticalX, size.height),
        paint,
      );

      final horizontalY = 240.0 * scaleY;
      canvas.drawLine(
        Offset(0, horizontalY),
        Offset(size.width, horizontalY),
        paint,
      );
    }

    if (showScrollOverlay && scrollOverlayRects.isNotEmpty) {
      final outline = Paint()
        ..color = Colors.pinkAccent.withValues(alpha: 0.9)
        ..strokeWidth = 2.0
        ..style = PaintingStyle.stroke;
      final fill = Paint()
        ..color = Colors.pinkAccent.withValues(alpha: 0.12)
        ..style = PaintingStyle.fill;

      for (final r in scrollOverlayRects) {
        final scaled = Rect.fromLTWH(
          r.left * scaleX,
          r.top * scaleY,
          r.width * scaleX,
          r.height * scaleY,
        );
        canvas.drawRect(scaled, fill);
        canvas.drawRect(scaled, outline);
      }
    }

    if (hoveredTile != null) {
      final paint = Paint()
        ..color = Colors.white.withValues(alpha: 0.9)
        ..strokeWidth = 2.0
        ..style = PaintingStyle.stroke;

      final rect = Rect.fromLTWH(
        hoveredTile!.x * 8.0 * scaleX,
        hoveredTile!.y * 8.0 * scaleY,
        8.0 * scaleX,
        8.0 * scaleY,
      );
      canvas.drawRect(rect, paint);
    }

    if (selectedTile != null) {
      final paint = Paint()
        ..color = Colors.yellow.withValues(alpha: 0.95)
        ..strokeWidth = 2.5
        ..style = PaintingStyle.stroke;

      final rect = Rect.fromLTWH(
        selectedTile!.x * 8.0 * scaleX,
        selectedTile!.y * 8.0 * scaleY,
        8.0 * scaleX,
        8.0 * scaleY,
      );
      canvas.drawRect(rect, paint);
    }
  }

  @override
  bool shouldRepaint(TilemapGridPainter oldDelegate) {
    return showTileGrid != oldDelegate.showTileGrid ||
        showAttributeGrid != oldDelegate.showAttributeGrid ||
        showAttributeGrid32 != oldDelegate.showAttributeGrid32 ||
        hoveredTile != oldDelegate.hoveredTile ||
        selectedTile != oldDelegate.selectedTile ||
        showNametableDelimiters != oldDelegate.showNametableDelimiters ||
        showScrollOverlay != oldDelegate.showScrollOverlay ||
        !_rectListEquals(scrollOverlayRects, oldDelegate.scrollOverlayRects);
  }

  bool _rectListEquals(List<Rect> a, List<Rect> b) {
    if (identical(a, b)) return true;
    if (a.length != b.length) return false;
    for (var i = 0; i < a.length; i++) {
      if (a[i] != b[i]) return false;
    }
    return true;
  }
}
