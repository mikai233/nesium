part of '../tilemap_viewer.dart';

class _TilePreview extends StatelessWidget {
  const _TilePreview({required this.snapshot, required this.info});

  final bridge.TilemapSnapshot snapshot;
  final _TileInfo info;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SizedBox(
          width: 64,
          height: 64,
          child: CustomPaint(
            painter: _TilePreviewPainter(snapshot: snapshot, info: info),
          ),
        ),
        const SizedBox(height: 8),
        _PaletteStrip(snapshot: snapshot, info: info),
      ],
    );
  }
}

class _PaletteStrip extends StatelessWidget {
  const _PaletteStrip({required this.snapshot, required this.info});

  final bridge.TilemapSnapshot snapshot;
  final _TileInfo info;

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

class _TileInfoTable extends StatelessWidget {
  const _TileInfoTable({required this.info});

  final _TileInfo info;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    String hex(int v, {int width = 4}) =>
        '\$${v.toRadixString(16).toUpperCase().padLeft(width, '0')}';

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _row(l10n.tilemapLabelColumnRow, '${info.tileX}, ${info.tileY}'),
        _row(l10n.tilemapLabelXY, '${info.tileX * 8}, ${info.tileY * 8}'),
        _row(l10n.tilemapLabelSize, '8×8'),
        const Divider(height: 16),
        _row(l10n.tilemapLabelTilemapAddress, hex(info.tilemapAddress)),
        _row(l10n.tilemapLabelTileIndex, hex(info.tileIndex, width: 2)),
        _row(l10n.tilemapLabelTileAddressPpu, hex(info.tileAddressPpu)),
        const Divider(height: 16),
        _row(l10n.tilemapLabelPaletteIndex, '${info.paletteIndex}'),
        _row(l10n.tilemapLabelPaletteAddress, hex(info.paletteAddress)),
        const Divider(height: 16),
        _row(l10n.tilemapLabelAttributeAddress, hex(info.attrAddress)),
        _row(l10n.tilemapLabelAttributeData, hex(info.attrByte, width: 2)),
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

class _TileInfoCard extends StatelessWidget {
  const _TileInfoCard({required this.info, required this.snapshot});

  final _TileInfo info;
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

    String hex(int v, {int width = 4}) =>
        '\$${v.toRadixString(16).toUpperCase().padLeft(width, '0')}';

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
              child: _TilePreview(snapshot: snapshot, info: info),
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
        kv(l10n.tilemapSelectedTileTilemap, hex(info.tilemapAddress)),
        kv(l10n.tilemapSelectedTileTileIdx, hex(info.tileIndex, width: 2)),
        kv(l10n.tilemapSelectedTileTilePpu, hex(info.tileAddressPpu)),
        kv(
          l10n.tilemapSelectedTilePalette,
          '${info.paletteIndex}  ${hex(info.paletteAddress)}',
        ),
        kv(
          l10n.tilemapSelectedTileAttr,
          '${hex(info.attrAddress)}  ${hex(info.attrByte, width: 2)}',
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
