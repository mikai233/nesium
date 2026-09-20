import 'package:flutter/material.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/features/debugger/sprite_viewer/painters/sprite_grid_painter.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer_models.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer_utils.dart';
import 'package:nesium_flutter/features/debugger/viewer_skeletonizer.dart';

class SpriteThumbnailGrid extends StatelessWidget {
  const SpriteThumbnailGrid({
    super.key,
    required this.snapshot,
    required this.background,
    required this.backgroundBuilder,
    required this.textureId,
    required this.showGrid,
    required this.dimOffscreen,
    required this.hoveredIndex,
    required this.selectedIndex,
    required this.onHover,
    required this.onHoverExit,
    required this.onTap,
  });

  final bridge.SpriteSnapshot snapshot;
  final SpriteViewerBackground background;
  final Widget backgroundBuilder;
  final int? textureId;
  final bool showGrid;
  final bool dimOffscreen;
  final int? hoveredIndex;
  final int? selectedIndex;
  final void Function(int? index, Offset localPosition, Offset? globalPosition)
  onHover;
  final VoidCallback onHoverExit;
  final ValueChanged<int?> onTap;

  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;
    final gridW = SpriteViewerLayout.gridTextureWidth(snapshot);
    final gridH = SpriteViewerLayout.gridTextureHeight(snapshot);
    final aspectRatio = gridH == 0 ? 1.0 : gridW / gridH;

    final visibleMask = List<bool>.generate(
      SpriteViewerLayout.gridCols * SpriteViewerLayout.gridRows,
      (i) => i < snapshot.sprites.length ? snapshot.sprites[i].visible : false,
    );

    return Center(
      child: AspectRatio(
        aspectRatio: aspectRatio,
        child: LayoutBuilder(
          builder: (context, constraints) {
            final viewportSize = constraints.biggest;
            return Container(
              decoration: BoxDecoration(
                border: Border.all(color: colorScheme.outlineVariant, width: 1),
                borderRadius: BorderRadius.circular(4),
              ),
              clipBehavior: Clip.antiAlias,
              child: MouseRegion(
                onHover: (event) {
                  final index = gridIndexAtPosition(
                    localPosition: event.localPosition,
                    viewportSize: viewportSize,
                    snapshot: snapshot,
                  );
                  final box = context.findRenderObject() as RenderBox?;
                  onHover(
                    index,
                    event.localPosition,
                    box?.localToGlobal(event.localPosition),
                  );
                },
                onExit: (_) => onHoverExit(),
                child: GestureDetector(
                  behavior: HitTestBehavior.opaque,
                  onTapDown: (details) {
                    final index = gridIndexAtPosition(
                      localPosition: details.localPosition,
                      viewportSize: viewportSize,
                      snapshot: snapshot,
                    );
                    onTap(index);
                  },
                  child: Stack(
                    children: [
                      backgroundBuilder,
                      if (textureId != null &&
                          !ViewerSkeletonScope.enabledOf(context))
                        Texture(
                          textureId: textureId!,
                          filterQuality: FilterQuality.none,
                        )
                      else
                        Positioned.fill(
                          child: DecoratedBox(
                            decoration: BoxDecoration(
                              color: colorScheme.surfaceContainerHighest,
                            ),
                          ),
                        ),
                      CustomPaint(
                        painter: SpriteGridPainter(
                          showGrid: showGrid,
                          dimOffscreen: dimOffscreen,
                          visibleMask: visibleMask,
                          hoveredIndex: hoveredIndex,
                          selectedIndex: selectedIndex,
                        ),
                        size: Size.infinite,
                      ),
                    ],
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
