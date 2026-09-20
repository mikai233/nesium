import 'package:flutter/material.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer_utils.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/widgets/sprite_thumbnail_preview.dart';
import 'package:nesium_flutter/l10n/app_localizations.dart';

class SpriteInfoCard extends StatelessWidget {
  const SpriteInfoCard({
    super.key,
    required this.sprite,
    required this.snapshot,
    required this.textureId,
  });

  final bridge.SpriteInfo sprite;
  final bridge.SpriteSnapshot snapshot;
  final int textureId;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final metadata = buildSpriteMetadata(snapshot, sprite);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            SpriteThumbnailPreview(
              textureId: textureId,
              index: sprite.index,
              thumbWidth: snapshot.thumbnailWidth,
              thumbHeight: snapshot.thumbnailHeight,
              scale: 7,
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Column(
                children: [
                  _SpriteInfoMetadataRow(
                    label: l10n.spriteViewerLabelIndex,
                    value: '#${sprite.index}',
                    dense: false,
                  ),
                  _SpriteInfoMetadataRow(
                    label: l10n.spriteViewerLabelPos,
                    value: '${sprite.x}, ${metadata.actualY}',
                    dense: false,
                  ),
                  _SpriteInfoMetadataRow(
                    label: l10n.spriteViewerLabelSize,
                    value: metadata.sizeLabel,
                    dense: false,
                  ),
                  _SpriteInfoMetadataRow(
                    label: l10n.spriteViewerLabelTile,
                    value: formatHexValue(sprite.tileIndex),
                    dense: false,
                  ),
                  _SpriteInfoMetadataRow(
                    label: l10n.spriteViewerLabelTileAddr,
                    value: spriteTileAddressLabel(metadata),
                    dense: false,
                  ),
                ],
              ),
            ),
          ],
        ),
        const SizedBox(height: 10),
        const Divider(height: 1),
        const SizedBox(height: 10),
        _SpriteInfoMetadataRow(
          label: l10n.spriteViewerLabelPalette,
          value: '${sprite.palette}',
          dense: false,
        ),
        _SpriteInfoMetadataRow(
          label: l10n.spriteViewerLabelPaletteAddr,
          value: formatHexValue(metadata.paletteAddr, width: 4),
          dense: false,
        ),
        _SpriteInfoMetadataRow(
          label: l10n.spriteViewerLabelFlip,
          value: metadata.flipLabel,
          dense: false,
        ),
        _SpriteInfoMetadataRow(
          label: l10n.spriteViewerLabelPriority,
          value: sprite.behindBg
              ? l10n.spriteViewerPriorityBehindBg
              : l10n.spriteViewerPriorityInFront,
          dense: false,
        ),
        _SpriteInfoMetadataRow(
          label: l10n.spriteViewerLabelVisible,
          value: sprite.visible
              ? l10n.spriteViewerValueYes
              : l10n.spriteViewerValueNoOffscreen,
          dense: false,
        ),
      ],
    );
  }
}

class _SpriteInfoMetadataRow extends StatelessWidget {
  const _SpriteInfoMetadataRow({
    required this.label,
    required this.value,
    this.dense = true,
  });

  final String label;
  final String value;
  final bool dense;

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
      padding: EdgeInsets.symmetric(vertical: dense ? 2 : 4),
      child: Row(
        children: [
          Expanded(child: Text(label, style: labelStyle)),
          Text(value, style: valueStyle),
        ],
      ),
    );
  }
}
