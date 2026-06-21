part of '../tile_viewer.dart';

/// CHR tile preview showing zoomed 8×8 tile with palette colors
class _ChrTilePreview extends StatelessWidget {
  const _ChrTilePreview({required this.snapshot, required this.info});

  final bridge.TileSnapshot snapshot;
  final _TileInfo info;

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

/// Shows the 4 colors of the selected palette
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

/// Table showing tile metadata
class _TileInfoTable extends StatelessWidget {
  const _TileInfoTable({required this.context, required this.info});

  final BuildContext context;
  final _TileInfo info;

  @override
  Widget build(BuildContext ctx) {
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
          '\$${info.tileIndexInTable.toRadixString(16).toUpperCase().padLeft(2, '0')}',
          labelStyle,
          valueStyle,
        ),
        _kv(
          l10n.tileViewerChrAddress,
          '\$${info.chrAddress.toRadixString(16).toUpperCase().padLeft(4, '0')}',
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

/// Hex address input with 4-button navigation (Mesen2 style)
/// Buttons: << (prev page), < (prev byte), [value], > (next byte), >> (next page)
class _AddressInput extends StatelessWidget {
  const _AddressInput({
    required this.value,
    required this.maxValue,
    required this.pageIncrement,
    required this.onChanged,
    this.byteIncrement = 1,
  });

  final int value;
  final int maxValue;
  final int pageIncrement; // Large step (page)
  final int byteIncrement; // Small step (byte/tile)
  final ValueChanged<int> onChanged;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final hexValue =
        '\$${value.toRadixString(16).toUpperCase().padLeft(4, '0')}';

    Widget navButton(String label, int delta, {bool enabled = true}) {
      return InkWell(
        onTap: enabled
            ? () => onChanged((value + delta).clamp(0, maxValue))
            : null,
        borderRadius: BorderRadius.circular(4),
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 4),
          child: Text(
            label,
            style: theme.textTheme.bodyMedium?.copyWith(
              fontFamily: 'monospace',
              fontWeight: FontWeight.bold,
              color: enabled
                  ? theme.colorScheme.primary
                  : theme.colorScheme.onSurface.withValues(alpha: 0.3),
            ),
          ),
        ),
      );
    }

    return Row(
      children: [
        navButton('«', -pageIncrement, enabled: value > 0),
        navButton('<', -byteIncrement, enabled: value > 0),
        Expanded(
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            decoration: BoxDecoration(
              border: Border.all(color: theme.colorScheme.outlineVariant),
              borderRadius: BorderRadius.circular(4),
            ),
            child: Text(
              hexValue,
              textAlign: TextAlign.center,
              style: theme.textTheme.bodyMedium?.copyWith(
                fontFamily: 'monospace',
                fontWeight: FontWeight.w500,
              ),
            ),
          ),
        ),
        // Next byte > (Mesen2: CanIncrementSmall = Value < Maximum)
        navButton('>', byteIncrement, enabled: value < maxValue),
        // Next page >> (Mesen2: CanIncrementLarge = Value < Maximum - LargeIncrement + 1)
        navButton(
          '»',
          pageIncrement,
          enabled: value < maxValue - pageIncrement + 1,
        ),
      ],
    );
  }
}

/// Compact numeric input with label for size controls
class _SizeInput extends StatelessWidget {
  const _SizeInput({
    required this.label,
    required this.value,
    required this.min,
    required this.max,
    required this.step,
    required this.onChanged,
  });

  final String label;
  final int value;
  final int min;
  final int max;
  final int step;
  final ValueChanged<int> onChanged;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        Text(
          label,
          style: theme.textTheme.labelSmall?.copyWith(
            color: theme.colorScheme.onSurfaceVariant,
          ),
        ),
        const SizedBox(height: 4),
        Row(
          children: [
            InkWell(
              onTap: value > min
                  ? () => onChanged((value - step).clamp(min, max))
                  : null,
              borderRadius: BorderRadius.circular(4),
              child: Container(
                padding: const EdgeInsets.all(4),
                child: Icon(
                  Icons.remove,
                  size: 16,
                  color: value > min
                      ? theme.colorScheme.onSurface
                      : theme.colorScheme.onSurface.withValues(alpha: 0.3),
                ),
              ),
            ),
            Expanded(
              child: Text(
                '$value',
                textAlign: TextAlign.center,
                style: theme.textTheme.bodyMedium?.copyWith(
                  fontWeight: FontWeight.w600,
                ),
              ),
            ),
            InkWell(
              onTap: value < max
                  ? () => onChanged((value + step).clamp(min, max))
                  : null,
              borderRadius: BorderRadius.circular(4),
              child: Container(
                padding: const EdgeInsets.all(4),
                child: Icon(
                  Icons.add,
                  size: 16,
                  color: value < max
                      ? theme.colorScheme.onSurface
                      : theme.colorScheme.onSurface.withValues(alpha: 0.3),
                ),
              ),
            ),
          ],
        ),
      ],
    );
  }
}
