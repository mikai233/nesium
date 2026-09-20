import 'package:flutter/material.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer_utils.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/widgets/sprite_thumbnail_preview.dart';
import 'package:nesium_flutter/l10n/app_localizations.dart';

class SpriteHoverCard extends StatelessWidget {
  const SpriteHoverCard({
    super.key,
    required this.snapshot,
    required this.textureId,
    required this.index,
  });

  final bridge.SpriteSnapshot snapshot;
  final int textureId;
  final int index;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final l10n = AppLocalizations.of(context)!;
    final sprite = snapshot.sprites[index];
    final metadata = buildSpriteMetadata(snapshot, sprite);

    return Card(
      elevation: 8,
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
            SpriteThumbnailPreview(
              textureId: textureId,
              index: index,
              thumbWidth: snapshot.thumbnailWidth,
              thumbHeight: snapshot.thumbnailHeight,
              scale: 6,
            ),
            const SizedBox(width: 12),
            Expanded(
              child: DefaultTextStyle(
                style: theme.textTheme.bodySmall!,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      l10n.spriteViewerTooltipTitle(sprite.index),
                      style: theme.textTheme.titleSmall?.copyWith(
                        fontWeight: FontWeight.w700,
                      ),
                    ),
                    const SizedBox(height: 8),
                    _SpriteMetadataRow(
                      label: l10n.spriteViewerLabelPos,
                      value: '${sprite.x}, ${metadata.actualY}',
                    ),
                    _SpriteMetadataRow(
                      label: l10n.spriteViewerLabelSize,
                      value: metadata.sizeLabel,
                    ),
                    _SpriteMetadataRow(
                      label: l10n.spriteViewerLabelTile,
                      value: formatHexValue(sprite.tileIndex),
                    ),
                    _SpriteMetadataRow(
                      label: l10n.spriteViewerLabelTileAddr,
                      value: spriteTileAddressLabel(metadata),
                    ),
                    _SpriteMetadataRow(
                      label: l10n.spriteViewerLabelPalette,
                      value: '${sprite.palette}',
                    ),
                    _SpriteMetadataRow(
                      label: l10n.spriteViewerLabelPaletteAddr,
                      value: formatHexValue(metadata.paletteAddr, width: 4),
                    ),
                    _SpriteMetadataRow(
                      label: l10n.spriteViewerLabelFlip,
                      value: metadata.flipLabel,
                    ),
                    _SpriteMetadataRow(
                      label: l10n.spriteViewerLabelPriority,
                      value: sprite.behindBg
                          ? l10n.spriteViewerPriorityBehindBg
                          : l10n.spriteViewerPriorityInFront,
                    ),
                    _SpriteMetadataRow(
                      label: l10n.spriteViewerLabelVisible,
                      value: sprite.visible
                          ? l10n.spriteViewerValueYes
                          : l10n.spriteViewerValueNoOffscreen,
                    ),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _SpriteMetadataRow extends StatelessWidget {
  const _SpriteMetadataRow({required this.label, required this.value});

  final String label;
  final String value;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final labelStyle = theme.textTheme.bodySmall?.copyWith(
      color: theme.colorScheme.onSurfaceVariant,
    );
    final valueStyle = theme.textTheme.bodySmall?.copyWith(
      fontWeight: FontWeight.w600,
      fontFamily: 'monospace',
    );

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        children: [
          Expanded(child: Text(label, style: labelStyle)),
          Text(value, style: valueStyle),
        ],
      ),
    );
  }
}
