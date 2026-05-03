part of '../tilemap_viewer.dart';

class _TilePreviewPainter extends CustomPainter {
  _TilePreviewPainter({required this.snapshot, required this.info});

  final bridge.TilemapSnapshot snapshot;
  final _TileInfo info;

  @override
  void paint(Canvas canvas, Size size) {
    final sx = size.width / 8;
    final sy = size.height / 8;
    final paint = Paint();

    final base = snapshot.bgPatternBase + info.tileIndex * 16;
    for (var py = 0; py < 8; py++) {
      final o = base + py;
      if (o + 8 >= snapshot.chr.length) break;
      final plane0 = snapshot.chr[o];
      final plane1 = snapshot.chr[o + 8];
      for (var px = 0; px < 8; px++) {
        final bit = 7 - px;
        final lo = (plane0 >> bit) & 1;
        final hi = (plane1 >> bit) & 1;
        final colorIndex = (hi << 1) | lo;
        final nesColor = _nesColorIndex(
          snapshot,
          paletteIndex: info.paletteIndex,
          colorIndex: colorIndex,
        );
        paint.color = _colorFromNes(snapshot.rgbaPalette, nesColor);
        canvas.drawRect(Rect.fromLTWH(px * sx, py * sy, sx, sy), paint);
      }
    }
  }

  int _nesColorIndex(
    bridge.TilemapSnapshot snap, {
    required int paletteIndex,
    required int colorIndex,
  }) {
    final pal = snap.palette;
    if (pal.isEmpty) return 0;
    if (colorIndex == 0) return pal[0] & 0x3F;
    final idx = paletteIndex * 4 + colorIndex;
    if (idx < 0 || idx >= pal.length) return 0;
    return pal[idx] & 0x3F;
  }

  Color _colorFromNes(Uint8List rgbaPalette, int nesColor) {
    final base = (nesColor & 0x3F) * 4;
    if (base + 3 >= rgbaPalette.length) return const Color(0xFF000000);
    final r = rgbaPalette[base];
    final g = rgbaPalette[base + 1];
    final b = rgbaPalette[base + 2];
    final a = rgbaPalette[base + 3];
    return Color.fromARGB(a, r, g, b);
  }

  @override
  bool shouldRepaint(_TilePreviewPainter oldDelegate) {
    return snapshot != oldDelegate.snapshot || info != oldDelegate.info;
  }
}

/// CustomPainter for drawing grid overlays on tilemap texture.
/// Uses vector graphics for resolution-independent sharp rendering.
class _TilemapGridPainter extends CustomPainter {
  final bool showTileGrid;
  final bool showAttributeGrid;
  final bool showAttributeGrid32;
  final bool showNametableDelimiters;
  final bool showScrollOverlay;
  final List<Rect> scrollOverlayRects;
  final _TileCoord? hoveredTile;
  final _TileCoord? selectedTile;

  _TilemapGridPainter({
    required this.showTileGrid,
    required this.showAttributeGrid,
    required this.showAttributeGrid32,
    required this.showNametableDelimiters,
    required this.showScrollOverlay,
    required this.scrollOverlayRects,
    required this.hoveredTile,
    required this.selectedTile,
  });

  @override
  void paint(Canvas canvas, Size size) {
    // Tilemap is 512x480 (2x2 nametables of 256x240 each)
    const logicalWidth = 512.0;
    const logicalHeight = 480.0;

    // Calculate scaling factor
    final scaleX = size.width / logicalWidth;
    final scaleY = size.height / logicalHeight;

    // Draw 8×8 tile grid
    if (showTileGrid) {
      final paint = Paint()
        ..color = Colors.white.withValues(alpha: 0.5)
        ..strokeWidth =
            1.0 // Clear 1px lines
        ..style = PaintingStyle.stroke;

      for (var y = 0; y <= logicalHeight; y += 8) {
        final scaledY = y * scaleY;
        canvas.drawLine(Offset(0, scaledY), Offset(size.width, scaledY), paint);
      }
      for (var x = 0; x <= logicalWidth; x += 8) {
        final scaledX = x * scaleX;
        canvas.drawLine(
          Offset(scaledX, 0),
          Offset(scaledX, size.height),
          paint,
        );
      }
    }

    // Draw 16×16 attribute grid
    if (showAttributeGrid) {
      final paint = Paint()
        ..color = Colors.cyan
            .withValues(alpha: 0.7) // Increased visibility
        ..strokeWidth =
            0.8 // Increased from 0.6
        ..style = PaintingStyle.stroke;

      for (var y = 0; y <= logicalHeight; y += 16) {
        final scaledY = y * scaleY;
        canvas.drawLine(Offset(0, scaledY), Offset(size.width, scaledY), paint);
      }
      for (var x = 0; x <= logicalWidth; x += 16) {
        final scaledX = x * scaleX;
        canvas.drawLine(
          Offset(scaledX, 0),
          Offset(scaledX, size.height),
          paint,
        );
      }
    }

    // Draw 32×32 attribute grid (attribute bytes boundaries)
    if (showAttributeGrid32) {
      final paint = Paint()
        ..color = Colors.orange.withValues(alpha: 0.55)
        ..strokeWidth = 1.0
        ..style = PaintingStyle.stroke;

      for (var y = 0; y <= logicalHeight; y += 32) {
        final scaledY = y * scaleY;
        canvas.drawLine(Offset(0, scaledY), Offset(size.width, scaledY), paint);
      }
      for (var x = 0; x <= logicalWidth; x += 32) {
        final scaledX = x * scaleX;
        canvas.drawLine(
          Offset(scaledX, 0),
          Offset(scaledX, size.height),
          paint,
        );
      }
    }

    // Draw nametable delimiters (256×240 boundaries)
    if (showNametableDelimiters) {
      final paint = Paint()
        ..color = Colors.white.withValues(alpha: 0.9)
        ..strokeWidth =
            1.0 // Crisp single-pixel line
        ..style = PaintingStyle.stroke;

      // Vertical delimiter at x=256
      final verticalX = 256.0 * scaleX;
      canvas.drawLine(
        Offset(verticalX, 0),
        Offset(verticalX, size.height),
        paint,
      );

      // Horizontal delimiter at y=240
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

    // Hovered tile outline
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

    // Selected tile outline
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
  bool shouldRepaint(_TilemapGridPainter oldDelegate) {
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
