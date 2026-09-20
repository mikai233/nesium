import 'package:flutter/material.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer_models.dart';

class SpriteGridPainter extends CustomPainter {
  SpriteGridPainter({
    required this.showGrid,
    required this.dimOffscreen,
    required this.visibleMask,
    required this.hoveredIndex,
    required this.selectedIndex,
  });

  final bool showGrid;
  final bool dimOffscreen;
  final List<bool> visibleMask;
  final int? hoveredIndex;
  final int? selectedIndex;

  @override
  void paint(Canvas canvas, Size size) {
    if (size.width <= 0 || size.height <= 0) return;
    final cellW = size.width / SpriteViewerLayout.gridCols;
    final cellH = size.height / SpriteViewerLayout.gridRows;

    if (dimOffscreen && visibleMask.isNotEmpty) {
      final paint = Paint()..color = Colors.black.withValues(alpha: 0.35);
      for (var i = 0; i < visibleMask.length; i++) {
        if (visibleMask[i]) continue;
        final col = i % SpriteViewerLayout.gridCols;
        final row = i ~/ SpriteViewerLayout.gridCols;
        final rect = Rect.fromLTWH(col * cellW, row * cellH, cellW, cellH);
        canvas.drawRect(rect, paint);
      }
    }

    if (showGrid) {
      final paint = Paint()
        ..color = Colors.white.withValues(alpha: 0.55)
        ..strokeWidth = 1.0
        ..style = PaintingStyle.stroke;

      for (var r = 0; r <= SpriteViewerLayout.gridRows; r++) {
        final y = r * cellH;
        canvas.drawLine(Offset(0, y), Offset(size.width, y), paint);
      }
      for (var c = 0; c <= SpriteViewerLayout.gridCols; c++) {
        final x = c * cellW;
        canvas.drawLine(Offset(x, 0), Offset(x, size.height), paint);
      }
    }

    if (hoveredIndex != null) {
      final paint = Paint()
        ..color = Colors.white.withValues(alpha: 0.9)
        ..strokeWidth = 2.0
        ..style = PaintingStyle.stroke;
      final col = hoveredIndex! % SpriteViewerLayout.gridCols;
      final row = hoveredIndex! ~/ SpriteViewerLayout.gridCols;
      canvas.drawRect(
        Rect.fromLTWH(col * cellW, row * cellH, cellW, cellH),
        paint,
      );
    }

    if (selectedIndex != null) {
      final paint = Paint()
        ..color = Colors.yellow.withValues(alpha: 0.95)
        ..strokeWidth = 2.5
        ..style = PaintingStyle.stroke;
      final col = selectedIndex! % SpriteViewerLayout.gridCols;
      final row = selectedIndex! ~/ SpriteViewerLayout.gridCols;
      canvas.drawRect(
        Rect.fromLTWH(col * cellW, row * cellH, cellW, cellH),
        paint,
      );
    }
  }

  @override
  bool shouldRepaint(SpriteGridPainter oldDelegate) {
    return showGrid != oldDelegate.showGrid ||
        dimOffscreen != oldDelegate.dimOffscreen ||
        hoveredIndex != oldDelegate.hoveredIndex ||
        selectedIndex != oldDelegate.selectedIndex ||
        visibleMask != oldDelegate.visibleMask;
  }
}
