import 'package:flutter/material.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer_models.dart';
import 'package:nesium_flutter/features/debugger/viewer_skeletonizer.dart';

class SpriteThumbnailPreview extends StatelessWidget {
  const SpriteThumbnailPreview({
    super.key,
    required this.textureId,
    required this.index,
    required this.thumbWidth,
    required this.thumbHeight,
    required this.scale,
  });

  final int textureId;
  final int index;
  final int thumbWidth;
  final int thumbHeight;
  final double scale;

  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;
    final gridX = index % SpriteViewerLayout.gridCols;
    final gridY = index ~/ SpriteViewerLayout.gridCols;

    final srcX = (gridX * thumbWidth).toDouble();
    final srcY = (gridY * thumbHeight).toDouble();

    final dstW = thumbWidth * scale;
    final dstH = thumbHeight * scale;

    final totalW = SpriteViewerLayout.gridCols * thumbWidth * scale;
    final totalH = SpriteViewerLayout.gridRows * thumbHeight * scale;

    return ClipRRect(
      borderRadius: BorderRadius.circular(8),
      child: SizedBox(
        width: dstW,
        height: dstH,
        child: ViewerSkeletonScope.enabledOf(context)
            ? DecoratedBox(
                decoration: BoxDecoration(
                  color: colorScheme.surfaceContainerHighest,
                ),
              )
            : OverflowBox(
                alignment: Alignment.topLeft,
                minWidth: totalW,
                maxWidth: totalW,
                minHeight: totalH,
                maxHeight: totalH,
                child: Transform.translate(
                  offset: Offset(-srcX * scale, -srcY * scale),
                  child: SizedBox(
                    width: totalW,
                    height: totalH,
                    child: Texture(
                      textureId: textureId,
                      filterQuality: FilterQuality.none,
                    ),
                  ),
                ),
              ),
      ),
    );
  }
}
