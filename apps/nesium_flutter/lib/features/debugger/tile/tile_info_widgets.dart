import 'package:flutter/material.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/features/debugger/tile/tile_viewer_geometry.dart';
import 'package:nesium_flutter/features/debugger/tile/tile_viewer_models.dart';
import 'package:nesium_flutter/l10n/app_localizations.dart';

/// CHR tile preview showing zoomed 8x8 tile with palette colors.
class ChrTilePreview extends StatelessWidget {
  const ChrTilePreview({required this.snapshot, required this.info, super.key});

  final bridge.TileSnapshot snapshot;
  final TileInfo info;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Container(
          width: 64,
          height: 64,
          decoration: BoxDecoration(
            border: Border.all(
              color: Theme.of(context).colorScheme.outlineVariant,
            ),
            borderRadius: BorderRadius.circular(4),
          ),
          child: CustomPaint(
            painter: _ChrTilePreviewPainter(snapshot: snapshot, info: info),
          ),
        ),
        const SizedBox(height: 8),
        _PaletteStrip(snapshot: snapshot),
      ],
    );
  }
}

class _ChrTilePreviewPainter extends CustomPainter {
  _ChrTilePreviewPainter({required this.snapshot, required this.info});

  final bridge.TileSnapshot snapshot;
  final TileInfo info;

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint();
    final paletteIndex = snapshot.selectedPalette.clamp(0, 7);
    final palBase = paletteIndex < 4
        ? paletteIndex * 4
        : 0x10 + (paletteIndex - 4) * 4;

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

    final textPainter = TextPainter(
      text: TextSpan(
        text: tileViewerHex(info.tileIndexInTable, width: 2),
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

class _PaletteStrip extends StatelessWidget {
  const _PaletteStrip({required this.snapshot});

  final bridge.TileSnapshot snapshot;

  @override
  Widget build(BuildContext context) {
    final paletteIndex = snapshot.selectedPalette.clamp(0, 7);
    final palBase = paletteIndex < 4
        ? paletteIndex * 4
        : 0x10 + (paletteIndex - 4) * 4;

    return Row(
      children: List.generate(4, (i) {
        final pal = snapshot.palette;
        final idx = palBase + i;
        final nesColor =
            (i == 0
                ? (pal.isNotEmpty ? pal[0] : 0)
                : (idx < pal.length ? pal[idx] : 0)) &
            0x3F;
        final rgba = snapshot.rgbaPalette;
        final base = nesColor * 4;
        final color = base + 3 < rgba.length
            ? Color.fromARGB(
                rgba[base + 3],
                rgba[base],
                rgba[base + 1],
                rgba[base + 2],
              )
            : Colors.black;

        return Container(width: 16, height: 8, color: color);
      }),
    );
  }
}

class TileInfoTable extends StatelessWidget {
  const TileInfoTable({required this.info, super.key});

  final TileInfo info;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final theme = Theme.of(context);
    final labelStyle = theme.textTheme.bodySmall?.copyWith(
      color: theme.colorScheme.onSurfaceVariant,
    );
    final valueStyle = theme.textTheme.bodySmall?.copyWith(
      fontWeight: FontWeight.w600,
      fontFamily: 'monospace',
    );

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _kv(
          l10n.tileViewerPatternTable,
          '${info.patternTable}',
          labelStyle,
          valueStyle,
        ),
        _kv(
          l10n.tileViewerTileIndex,
          tileViewerHex(info.tileIndexInTable, width: 2),
          labelStyle,
          valueStyle,
        ),
        _kv(
          l10n.tileViewerChrAddress,
          tileViewerHex(info.chrAddress),
          labelStyle,
          valueStyle,
        ),
      ],
    );
  }

  Widget _kv(
    String label,
    String value,
    TextStyle? labelStyle,
    TextStyle? valueStyle,
  ) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 3),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: labelStyle),
          Text(value, style: valueStyle),
        ],
      ),
    );
  }
}

class TileInfoCard extends StatelessWidget {
  const TileInfoCard({required this.tile, required this.snapshot, super.key});

  final TileCoord tile;
  final bridge.TileSnapshot snapshot;

  @override
  Widget build(BuildContext context) {
    final info = computeTileInfo(tile);
    final theme = Theme.of(context);
    final l10n = AppLocalizations.of(context)!;
    final labelStyle = theme.textTheme.bodySmall?.copyWith(
      color: theme.colorScheme.onSurfaceVariant,
    );
    final valueStyle = theme.textTheme.bodySmall?.copyWith(
      fontWeight: FontWeight.w600,
      fontFamily: 'monospace',
    );

    return Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SizedBox(
          width: 76,
          child: ChrTilePreview(snapshot: snapshot, info: info),
        ),
        const SizedBox(width: 12),
        Expanded(
          child: Column(
            children: [
              _infoRow(
                l10n.tileViewerPatternTable,
                '${info.patternTable}',
                labelStyle,
                valueStyle,
              ),
              _infoRow(
                l10n.tileViewerTileIndex,
                tileViewerHex(info.tileIndexInTable, width: 2),
                labelStyle,
                valueStyle,
              ),
              _infoRow(
                l10n.tileViewerChrAddress,
                tileViewerHex(info.chrAddress),
                labelStyle,
                valueStyle,
              ),
            ],
          ),
        ),
      ],
    );
  }

  Widget _infoRow(
    String label,
    String value,
    TextStyle? labelStyle,
    TextStyle? valueStyle,
  ) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        children: [
          Expanded(
            child: Text(
              label,
              style: labelStyle,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
            ),
          ),
          const SizedBox(width: 8),
          Text(value, style: valueStyle),
        ],
      ),
    );
  }
}

class TileHoverTooltip extends StatelessWidget {
  const TileHoverTooltip({
    required this.tile,
    required this.snapshot,
    required this.position,
    required this.viewportSize,
    super.key,
  });

  final TileCoord tile;
  final bridge.TileSnapshot snapshot;
  final Offset position;
  final Size viewportSize;

  @override
  Widget build(BuildContext context) {
    final info = computeTileInfo(tile);
    const tooltipWidth = 220.0;
    const tooltipHeight = 130.0;
    const cursorOffset = 16.0;

    final preferRight = position.dx < viewportSize.width * 0.55;
    final preferDown = position.dy < viewportSize.height * 0.5;
    final dxCandidate = preferRight
        ? position.dx + cursorOffset
        : position.dx - tooltipWidth - cursorOffset;
    final dyCandidate = preferDown
        ? position.dy + cursorOffset
        : position.dy - tooltipHeight - cursorOffset;

    final dx = dxCandidate.clamp(8.0, viewportSize.width - tooltipWidth - 8.0);
    final dy = dyCandidate.clamp(
      8.0,
      viewportSize.height - tooltipHeight - 8.0,
    );

    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

    return Positioned(
      left: dx,
      top: dy,
      child: SizedBox(
        width: tooltipWidth,
        child: Card(
          elevation: 4,
          shadowColor: colorScheme.shadow.withValues(alpha: 0.3),
          color: colorScheme.surfaceContainerHigh,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(12),
            side: BorderSide(color: colorScheme.outlineVariant),
          ),
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: Row(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                ChrTilePreview(snapshot: snapshot, info: info),
                const SizedBox(width: 12),
                Expanded(
                  child: DefaultTextStyle(
                    style: theme.textTheme.bodySmall!,
                    child: TileInfoTable(info: info),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
