part of '../tile_viewer.dart';

class _TileGridPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = Colors.white.withValues(alpha: 0.2)
      ..strokeWidth = 1.0
      ..style = PaintingStyle.stroke;

    // 16 tiles wide × 32 tiles tall
    const tilesX = 16;
    const tilesY = 32;

    final tileWidth = size.width / tilesX;
    final tileHeight = size.height / tilesY;

    // Draw vertical lines
    for (int i = 0; i <= tilesX; i++) {
      final x = i * tileWidth;
      canvas.drawLine(Offset(x, 0), Offset(x, size.height), paint);
    }

    // Draw horizontal lines
    for (int i = 0; i <= tilesY; i++) {
      final y = i * tileHeight;
      canvas.drawLine(Offset(0, y), Offset(size.width, y), paint);
    }

    // Draw separator between pattern tables (after row 16)
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

/// Paints highlight for hovered and selected tiles
class _TileHighlightPainter extends CustomPainter {
  final _TileCoord? hoveredTile;
  final _TileCoord? selectedTile;
  final double tileWidth;
  final double tileHeight;

  _TileHighlightPainter({
    this.hoveredTile,
    this.selectedTile,
    required this.tileWidth,
    required this.tileHeight,
  });

  @override
  void paint(Canvas canvas, Size size) {
    // Draw hover highlight
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

    // Draw selection highlight (stronger)
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
  bool shouldRepaint(covariant _TileHighlightPainter oldDelegate) =>
      hoveredTile != oldDelegate.hoveredTile ||
      selectedTile != oldDelegate.selectedTile;
}

class _ChrTilePreviewPainter extends CustomPainter {
  _ChrTilePreviewPainter({required this.snapshot, required this.info});

  final bridge.TileSnapshot snapshot;
  final _TileInfo info;

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint();
    final paletteIndex = snapshot.selectedPalette.clamp(0, 7);
    final palBase = paletteIndex < 4
        ? paletteIndex * 4
        : 0x10 + (paletteIndex - 4) * 4;

    // Draw 2x2 grid showing the 4 palette colors as a placeholder
    final cellW = size.width / 2;
    final cellH = size.height / 2;

    for (var i = 0; i < 4; i++) {
      final pal = snapshot.palette;
      final idx = i == 0 ? 0 : palBase + i;
      final nesColor = (idx < pal.length ? pal[idx] : 0) & 0x3F;
      paint.color = _colorFromNes(nesColor);

      final x = (i % 2) * cellW;
      final y = (i ~/ 2) * cellH;
      canvas.drawRect(Rect.fromLTWH(x, y, cellW, cellH), paint);
    }

    // Draw tile index in center
    final textPainter = TextPainter(
      text: TextSpan(
        text:
            '\$${info.tileIndexInTable.toRadixString(16).toUpperCase().padLeft(2, '0')}',
        style: TextStyle(
          color: Colors.white,
          fontSize: size.width / 4,
          fontFamily: 'monospace',
          fontWeight: FontWeight.bold,
          shadows: const [Shadow(blurRadius: 2, color: Colors.black)],
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    textPainter.paint(
      canvas,
      Offset(
        (size.width - textPainter.width) / 2,
        (size.height - textPainter.height) / 2,
      ),
    );
  }

  Color _colorFromNes(int nesColor) {
    final rgba = snapshot.rgbaPalette;
    final base = (nesColor & 0x3F) * 4;
    if (base + 3 >= rgba.length) return Colors.black;
    return Color.fromARGB(
      rgba[base + 3],
      rgba[base],
      rgba[base + 1],
      rgba[base + 2],
    );
  }

  @override
  bool shouldRepaint(covariant _ChrTilePreviewPainter old) =>
      info.tileIndex != old.info.tileIndex ||
      snapshot.selectedPalette != old.snapshot.selectedPalette;
}
