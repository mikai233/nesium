import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/features/debugger/tilemap/tilemap_geometry.dart';
import 'package:nesium_flutter/features/debugger/tilemap/tilemap_models.dart';
import 'package:nesium_flutter/l10n/app_localizations.dart';

class TilemapTilePreview extends StatelessWidget {
  const TilemapTilePreview({
    required this.snapshot,
    required this.info,
    super.key,
  });

  final bridge.TilemapSnapshot snapshot;
  final TilemapTileInfo info;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SizedBox(
          width: 64,
          height: 64,
          child: CustomPaint(
            painter: _TilemapTilePreviewPainter(snapshot: snapshot, info: info),
          ),
        ),
        const SizedBox(height: 8),
        _TilemapPaletteStrip(snapshot: snapshot, info: info),
      ],
    );
  }
}

class _TilemapTilePreviewPainter extends CustomPainter {
  _TilemapTilePreviewPainter({required this.snapshot, required this.info});

  final bridge.TilemapSnapshot snapshot;
  final TilemapTileInfo info;

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
  bool shouldRepaint(_TilemapTilePreviewPainter oldDelegate) {
    return snapshot != oldDelegate.snapshot || info != oldDelegate.info;
  }
}

class _TilemapPaletteStrip extends StatelessWidget {
  const _TilemapPaletteStrip({required this.snapshot, required this.info});

  final bridge.TilemapSnapshot snapshot;
  final TilemapTileInfo info;

  @override
  Widget build(BuildContext context) {
    final colors = List<Color>.generate(4, (i) {
      int nes;
      if (snapshot.palette.isEmpty) {
        nes = 0;
      } else if (i == 0) {
        nes = snapshot.palette[0];
      } else {
        final idx = info.paletteIndex * 4 + i;
        nes = snapshot.palette[idx.clamp(0, snapshot.palette.length - 1)];
      }

      final base = (nes & 0x3F) * 4;
      if (base + 3 >= snapshot.rgbaPalette.length) {
        return const Color(0xFF000000);
      }
      return Color.fromARGB(
        snapshot.rgbaPalette[base + 3],
        snapshot.rgbaPalette[base],
        snapshot.rgbaPalette[base + 1],
        snapshot.rgbaPalette[base + 2],
      );
    });

    return Row(
      children: [
        for (final c in colors)
          Container(
            width: 16,
            height: 16,
            margin: const EdgeInsets.only(right: 4),
            decoration: BoxDecoration(
              color: c,
              border: Border.all(color: Colors.black26),
            ),
          ),
      ],
    );
  }
}

class TilemapTileInfoTable extends StatelessWidget {
  const TilemapTileInfoTable({required this.info, super.key});

  final TilemapTileInfo info;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _row(l10n.tilemapLabelColumnRow, '${info.tileX}, ${info.tileY}'),
        _row(l10n.tilemapLabelXY, '${info.tileX * 8}, ${info.tileY * 8}'),
        _row(l10n.tilemapLabelSize, '8×8'),
        const Divider(height: 16),
        _row(l10n.tilemapLabelTilemapAddress, tilemapHex(info.tilemapAddress)),
        _row(l10n.tilemapLabelTileIndex, tilemapHex(info.tileIndex, width: 2)),
        _row(l10n.tilemapLabelTileAddressPpu, tilemapHex(info.tileAddressPpu)),
        const Divider(height: 16),
        _row(l10n.tilemapLabelPaletteIndex, '${info.paletteIndex}'),
        _row(l10n.tilemapLabelPaletteAddress, tilemapHex(info.paletteAddress)),
        const Divider(height: 16),
        _row(l10n.tilemapLabelAttributeAddress, tilemapHex(info.attrAddress)),
        _row(
          l10n.tilemapLabelAttributeData,
          tilemapHex(info.attrByte, width: 2),
        ),
      ],
    );
  }

  Widget _row(String k, String v) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 1),
      child: Row(
        children: [
          Expanded(
            child: Text(k, style: const TextStyle(color: Colors.black54)),
          ),
          Text(v),
        ],
      ),
    );
  }
}

class TilemapTileInfoCard extends StatelessWidget {
  const TilemapTileInfoCard({
    required this.info,
    required this.snapshot,
    super.key,
  });

  final TilemapTileInfo info;
  final bridge.TilemapSnapshot snapshot;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final labelStyle = theme.textTheme.bodySmall?.copyWith(
      color: colorScheme.onSurfaceVariant,
    );
    final valueStyle = theme.textTheme.bodySmall?.copyWith(
      fontWeight: FontWeight.w600,
    );

    Widget kv(String label, String value) {
      return Padding(
        padding: const EdgeInsets.symmetric(vertical: 4),
        child: Row(
          children: [
            Expanded(child: Text(label, style: labelStyle)),
            Text(value, style: valueStyle),
          ],
        ),
      );
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            SizedBox(
              width: 84,
              child: TilemapTilePreview(snapshot: snapshot, info: info),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Column(
                children: [
                  _metaRow(
                    label: l10n.tilemapLabelColumnRow,
                    value: '${info.tileX}, ${info.tileY}',
                    labelStyle: labelStyle,
                    valueStyle: valueStyle,
                  ),
                  _metaRow(
                    label: l10n.tilemapLabelXY,
                    value: '${info.tileX * 8}, ${info.tileY * 8}',
                    labelStyle: labelStyle,
                    valueStyle: valueStyle,
                  ),
                  _metaRow(
                    label: l10n.tilemapLabelSize,
                    value: '8×8',
                    labelStyle: labelStyle,
                    valueStyle: valueStyle,
                  ),
                ],
              ),
            ),
          ],
        ),
        const SizedBox(height: 10),
        const Divider(height: 1),
        const SizedBox(height: 10),
        kv(l10n.tilemapSelectedTileTilemap, tilemapHex(info.tilemapAddress)),
        kv(
          l10n.tilemapSelectedTileTileIdx,
          tilemapHex(info.tileIndex, width: 2),
        ),
        kv(l10n.tilemapSelectedTileTilePpu, tilemapHex(info.tileAddressPpu)),
        kv(
          l10n.tilemapSelectedTilePalette,
          '${info.paletteIndex}  ${tilemapHex(info.paletteAddress)}',
        ),
        kv(
          l10n.tilemapSelectedTileAttr,
          '${tilemapHex(info.attrAddress)}  '
          '${tilemapHex(info.attrByte, width: 2)}',
        ),
      ],
    );
  }

  Widget _metaRow({
    required String label,
    required String value,
    required TextStyle? labelStyle,
    required TextStyle? valueStyle,
  }) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 3),
      child: Row(
        children: [
          Expanded(child: Text(label, style: labelStyle)),
          Text(value, style: valueStyle),
        ],
      ),
    );
  }
}
