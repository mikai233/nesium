import 'dart:math' as math;

import 'package:flutter/material.dart';
import 'package:nesium_flutter/features/debugger/tile/tile_viewer_models.dart';
import 'package:nesium_flutter/features/debugger/tile/tile_viewer_painters.dart';
import 'package:nesium_flutter/features/debugger/viewer_skeletonizer.dart';

class TileViewerCanvas extends StatelessWidget {
  const TileViewerCanvas({
    required this.textureId,
    required this.textureWidth,
    required this.textureHeight,
    required this.transformationController,
    required this.minScale,
    required this.maxScale,
    required this.showTileGrid,
    required this.hoveredTile,
    required this.selectedTile,
    required this.onHover,
    required this.onTap,
    required this.onHoverExit,
    super.key,
  });

  final int? textureId;
  final int textureWidth;
  final int textureHeight;
  final TransformationController transformationController;
  final double minScale;
  final double maxScale;
  final bool showTileGrid;
  final TileCoord? hoveredTile;
  final TileCoord? selectedTile;
  final void Function({
    required Offset position,
    required Offset contentOffset,
    required Size contentSize,
  })
  onHover;
  final void Function({
    required Offset position,
    required Offset contentOffset,
    required Size contentSize,
  })
  onTap;
  final VoidCallback onHoverExit;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

    return Container(
      color: colorScheme.surfaceContainerLowest,
      child: Container(
        decoration: BoxDecoration(
          border: Border.all(color: colorScheme.outlineVariant),
          borderRadius: BorderRadius.circular(4),
        ),
        clipBehavior: Clip.antiAlias,
        child: LayoutBuilder(
          builder: (context, constraints) {
            final viewportSize = constraints.biggest;
            if (viewportSize.width <= 0 || viewportSize.height <= 0) {
              return const SizedBox();
            }

            final scale = math.min(
              viewportSize.width / textureWidth,
              viewportSize.height / textureHeight,
            );
            final contentSize = Size(
              textureWidth * scale,
              textureHeight * scale,
            );
            final contentOffset = Offset(
              (viewportSize.width - contentSize.width) / 2,
              (viewportSize.height - contentSize.height) / 2,
            );

            return MouseRegion(
              onHover: (event) => onHover(
                position: event.localPosition,
                contentOffset: contentOffset,
                contentSize: contentSize,
              ),
              onExit: (_) => onHoverExit(),
              child: GestureDetector(
                behavior: HitTestBehavior.opaque,
                onTapDown: (details) => onTap(
                  position: details.localPosition,
                  contentOffset: contentOffset,
                  contentSize: contentSize,
                ),
                child: InteractiveViewer(
                  transformationController: transformationController,
                  minScale: minScale,
                  maxScale: maxScale,
                  panEnabled: true,
                  scaleEnabled: true,
                  boundaryMargin: const EdgeInsets.all(double.infinity),
                  constrained: false,
                  child: SizedBox(
                    width: viewportSize.width,
                    height: viewportSize.height,
                    child: Stack(
                      children: [
                        Positioned(
                          left: contentOffset.dx,
                          top: contentOffset.dy,
                          width: contentSize.width,
                          height: contentSize.height,
                          child: Stack(
                            fit: StackFit.expand,
                            children: [
                              if (textureId != null &&
                                  !ViewerSkeletonScope.enabledOf(context))
                                Texture(
                                  textureId: textureId!,
                                  filterQuality: FilterQuality.none,
                                )
                              else
                                DecoratedBox(
                                  decoration: BoxDecoration(
                                    color: colorScheme.surfaceContainerHighest,
                                    borderRadius: BorderRadius.circular(12),
                                  ),
                                ),
                              if (showTileGrid)
                                CustomPaint(painter: TileGridPainter()),
                              if (hoveredTile != null || selectedTile != null)
                                CustomPaint(
                                  painter: TileHighlightPainter(
                                    hoveredTile: hoveredTile,
                                    selectedTile: selectedTile,
                                    tileWidth: contentSize.width / 16,
                                    tileHeight: contentSize.height / 32,
                                  ),
                                ),
                            ],
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              ),
            );
          },
        ),
      ),
    );
  }
}
