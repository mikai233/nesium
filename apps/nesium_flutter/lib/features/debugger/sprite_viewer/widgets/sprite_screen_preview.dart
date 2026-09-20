import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/features/debugger/sprite_viewer/painters/sprite_preview_overlay_painter.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer_models.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer_utils.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/widgets/sprite_hover_card.dart';
import 'package:nesium_flutter/features/debugger/viewer_skeletonizer.dart';
import 'package:nesium_flutter/platform/platform_capabilities.dart';

class SpriteScreenPreview extends StatelessWidget {
  const SpriteScreenPreview({
    super.key,
    required this.snapshot,
    required this.screenTextureId,
    required this.thumbTextureId,
    required this.overlayDataListenable,
    required this.backgroundBuilder,
    required this.showOutline,
    required this.showOffscreenRegions,
    required this.hoveredIndex,
    required this.selectedIndex,
    required this.selectedPosition,
    required this.transformationController,
    required this.maxScale,
    required this.onViewportChanged,
    required this.onHover,
    required this.onHoverExit,
    required this.onTap,
  });

  final bridge.SpriteSnapshot snapshot;
  final int? screenTextureId;
  final int? thumbTextureId;
  final ValueListenable<SpriteOverlayData> overlayDataListenable;
  final Widget backgroundBuilder;
  final bool showOutline;
  final bool showOffscreenRegions;
  final int? hoveredIndex;
  final int? selectedIndex;
  final Offset? selectedPosition;
  final TransformationController transformationController;
  final double maxScale;
  final void Function(Size viewportSize, int displayW, int displayH)
  onViewportChanged;
  final void Function(int? index, Offset localPosition, Offset? globalPosition)
  onHover;
  final VoidCallback onHoverExit;
  final void Function(int? index, Offset localPosition) onTap;

  @override
  Widget build(BuildContext context) {
    final colorScheme = Theme.of(context).colorScheme;
    final displayW = SpriteViewerLayout.previewWidth;
    final displayH = showOffscreenRegions
        ? SpriteViewerLayout.previewHeight
        : SpriteViewerLayout.screenHeight;
    final aspectRatio = displayH == 0 ? 1.0 : displayW / displayH;
    final showSelectedTooltip =
        !isNativeDesktop &&
        selectedIndex != null &&
        selectedPosition != null &&
        thumbTextureId != null &&
        selectedIndex! < snapshot.sprites.length;

    return Container(
      color: colorScheme.surfaceContainerLowest,
      child: Center(
        child: AspectRatio(
          aspectRatio: aspectRatio,
          child: Container(
            decoration: BoxDecoration(
              border: Border.all(color: colorScheme.outlineVariant, width: 1),
              borderRadius: BorderRadius.circular(4),
            ),
            clipBehavior: Clip.antiAlias,
            child: LayoutBuilder(
              builder: (context, constraints) {
                final viewportSize = constraints.biggest;
                onViewportChanged(viewportSize, displayW, displayH);

                final defaultScale = computePreviewDefaultTransform(
                  viewportSize: viewportSize,
                  contentW: displayW.toDouble(),
                  contentH: displayH.toDouble(),
                  maxScale: maxScale,
                ).entry(0, 0);
                final minScale = defaultScale < 1.0 ? defaultScale : 1.0;
                final showScreenTexture =
                    screenTextureId != null &&
                    !ViewerSkeletonScope.enabledOf(context);

                final content = SizedBox(
                  width: displayW.toDouble(),
                  height: displayH.toDouble(),
                  child: Stack(
                    children: [
                      backgroundBuilder,
                      Positioned.fill(
                        child: !showScreenTexture
                            ? DecoratedBox(
                                decoration: BoxDecoration(
                                  color: colorScheme.surfaceContainerHighest,
                                ),
                              )
                            : showOffscreenRegions
                            ? Texture(
                                textureId: screenTextureId!,
                                filterQuality: FilterQuality.none,
                              )
                            : ClipRect(
                                child: OverflowBox(
                                  alignment: Alignment.topLeft,
                                  minWidth: SpriteViewerLayout.previewWidth
                                      .toDouble(),
                                  maxWidth: SpriteViewerLayout.previewWidth
                                      .toDouble(),
                                  minHeight: SpriteViewerLayout.previewHeight
                                      .toDouble(),
                                  maxHeight: SpriteViewerLayout.previewHeight
                                      .toDouble(),
                                  child: SizedBox(
                                    width: SpriteViewerLayout.previewWidth
                                        .toDouble(),
                                    height: SpriteViewerLayout.previewHeight
                                        .toDouble(),
                                    child: Texture(
                                      textureId: screenTextureId!,
                                      filterQuality: FilterQuality.none,
                                    ),
                                  ),
                                ),
                              ),
                      ),
                      ValueListenableBuilder<SpriteOverlayData>(
                        valueListenable: overlayDataListenable,
                        builder: (context, overlayData, _) {
                          return CustomPaint(
                            painter: SpritePreviewOverlayPainter(
                              sprites: showOutline
                                  ? overlayData.sprites
                                  : snapshot.sprites,
                              largeSprites: showOutline
                                  ? overlayData.largeSprites
                                  : snapshot.largeSprites,
                              showOutline: showOutline,
                              showOffscreenRegions: showOffscreenRegions,
                              hoveredIndex: hoveredIndex,
                              selectedIndex: selectedIndex,
                            ),
                            size: Size.infinite,
                          );
                        },
                      ),
                    ],
                  ),
                );

                return Builder(
                  builder: (previewContext) {
                    return MouseRegion(
                      onHover: (event) {
                        final contentPos = transformToPreviewContent(
                          transformationController.value,
                          event.localPosition,
                        );
                        final screenCoord = previewScreenCoordAtContentPosition(
                          contentPos,
                        );
                        final index = hitTestSprite(snapshot, screenCoord);
                        final box =
                            previewContext.findRenderObject() as RenderBox?;
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
                          final contentPos = transformToPreviewContent(
                            transformationController.value,
                            details.localPosition,
                          );
                          final screenCoord =
                              previewScreenCoordAtContentPosition(contentPos);
                          onTap(
                            hitTestSprite(snapshot, screenCoord),
                            details.localPosition,
                          );
                        },
                        child: Stack(
                          children: [
                            Positioned.fill(
                              child: InteractiveViewer(
                                transformationController:
                                    transformationController,
                                minScale: minScale,
                                maxScale: maxScale,
                                panEnabled: true,
                                scaleEnabled: true,
                                boundaryMargin: const EdgeInsets.all(
                                  double.infinity,
                                ),
                                constrained: false,
                                child: content,
                              ),
                            ),
                            if (showSelectedTooltip)
                              IgnorePointer(
                                child: _SelectedTooltip(
                                  snapshot: snapshot,
                                  textureId: thumbTextureId!,
                                  index: selectedIndex!,
                                  position: selectedPosition!,
                                  viewportSize: viewportSize,
                                ),
                              ),
                          ],
                        ),
                      ),
                    );
                  },
                );
              },
            ),
          ),
        ),
      ),
    );
  }
}

class _SelectedTooltip extends StatelessWidget {
  const _SelectedTooltip({
    required this.snapshot,
    required this.textureId,
    required this.index,
    required this.position,
    required this.viewportSize,
  });

  final bridge.SpriteSnapshot snapshot;
  final int textureId;
  final int index;
  final Offset position;
  final Size viewportSize;

  @override
  Widget build(BuildContext context) {
    const tooltipWidth = 300.0;
    const tooltipHeight = 170.0;
    final offset = computeTooltipOffset(
      position: position,
      viewportSize: viewportSize,
      tooltipWidth: tooltipWidth,
      tooltipHeight: tooltipHeight,
    );

    return Positioned(
      left: offset.dx,
      top: offset.dy,
      child: SizedBox(
        width: tooltipWidth,
        child: SpriteHoverCard(
          snapshot: snapshot,
          textureId: textureId,
          index: index,
        ),
      ),
    );
  }
}
