import 'package:flutter/material.dart';
import 'package:nesium_flutter/features/debugger/tile/tile_viewer_geometry.dart';
import 'package:nesium_flutter/features/debugger/tile/tile_viewer_models.dart';

class TileGridPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = Colors.white.withValues(alpha: 0.2)
      ..strokeWidth = 1.0
      ..style = PaintingStyle.stroke;

    final tileWidth = size.width / tileViewerColumns;
    final tileHeight = size.height / tileViewerRows;

    for (var i = 0; i <= tileViewerColumns; i++) {
      final x = i * tileWidth;
      canvas.drawLine(Offset(x, 0), Offset(x, size.height), paint);
    }

    for (var i = 0; i <= tileViewerRows; i++) {
      final y = i * tileHeight;
      canvas.drawLine(Offset(0, y), Offset(size.width, y), paint);
    }

    final separatorY = 16 * tileHeight;
    final separatorPaint = Paint()
      ..color = Colors.yellow.withValues(alpha: 0.5)
      ..strokeWidth = 2.0;
    canvas.drawLine(
      Offset(0, separatorY),
      Offset(size.width, separatorY),
      separatorPaint,
    );
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}

/// Paints highlight for hovered and selected tiles.
class TileHighlightPainter extends CustomPainter {
  TileHighlightPainter({
    required this.tileWidth,
    required this.tileHeight,
    this.hoveredTile,
    this.selectedTile,
  });

  final TileCoord? hoveredTile;
  final TileCoord? selectedTile;
  final double tileWidth;
  final double tileHeight;

  @override
  void paint(Canvas canvas, Size size) {
    if (hoveredTile != null) {
      final rect = Rect.fromLTWH(
        hoveredTile!.x * tileWidth,
        hoveredTile!.y * tileHeight,
        tileWidth,
        tileHeight,
      );
      final paint = Paint()
        ..color = Colors.cyan.withValues(alpha: 0.3)
        ..style = PaintingStyle.fill;
      canvas.drawRect(rect, paint);

      final borderPaint = Paint()
        ..color = Colors.cyan
        ..strokeWidth = 1.5
        ..style = PaintingStyle.stroke;
      canvas.drawRect(rect, borderPaint);
    }

    if (selectedTile != null && selectedTile != hoveredTile) {
      final rect = Rect.fromLTWH(
        selectedTile!.x * tileWidth,
        selectedTile!.y * tileHeight,
        tileWidth,
        tileHeight,
      );
      final paint = Paint()
        ..color = Colors.yellow.withValues(alpha: 0.4)
        ..style = PaintingStyle.fill;
      canvas.drawRect(rect, paint);

      final borderPaint = Paint()
        ..color = Colors.yellow
        ..strokeWidth = 2.0
        ..style = PaintingStyle.stroke;
      canvas.drawRect(rect, borderPaint);
    }
  }

  @override
  bool shouldRepaint(covariant TileHighlightPainter oldDelegate) =>
      hoveredTile != oldDelegate.hoveredTile ||
      selectedTile != oldDelegate.selectedTile;
}
