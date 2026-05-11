import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/features/debugger/tilemap/tilemap_geometry.dart';
import 'package:nesium_flutter/features/debugger/tilemap/tilemap_grid_painter.dart';
import 'package:nesium_flutter/features/debugger/tilemap/tilemap_models.dart';
import 'package:nesium_flutter/features/debugger/tilemap/tilemap_tile_info_widgets.dart';
import 'package:nesium_flutter/features/debugger/viewer_skeletonizer.dart';
import 'package:nesium_flutter/platform/platform_capabilities.dart';

class TilemapCanvas extends StatelessWidget {
  const TilemapCanvas({
    required this.textureId,
    required this.snapshot,
    required this.transformationController,
    required this.minScale,
    required this.maxScale,
    required this.showTileGrid,
    required this.showAttributeGrid,
    required this.showAttributeGrid32,
    required this.showNametableDelimiters,
    required this.showScrollOverlay,
    required this.scrollOverlayRects,
    required this.hoveredTile,
    required this.selectedTile,
    required this.hoverPosition,
    required this.selectedPosition,
    required this.lastHoverTooltipSize,
    required this.hoverTooltipKey,
    required this.onHover,
    required this.onTap,
    required this.onHoverExit,
    required this.onScheduleHoverTooltipMeasure,
    super.key,
  });

  final int? textureId;
  final bridge.TilemapSnapshot? snapshot;
  final TransformationController transformationController;
  final double minScale;
  final double maxScale;
  final bool showTileGrid;
  final bool showAttributeGrid;
  final bool showAttributeGrid32;
  final bool showNametableDelimiters;
  final bool showScrollOverlay;
  final ValueListenable<List<Rect>> scrollOverlayRects;
  final TilemapCoord? hoveredTile;
  final TilemapCoord? selectedTile;
  final Offset? hoverPosition;
  final Offset? selectedPosition;
  final Size lastHoverTooltipSize;
  final GlobalKey hoverTooltipKey;
  final void Function(Offset localPosition, Size size) onHover;
  final void Function(Offset localPosition, Size size) onTap;
  final VoidCallback onHoverExit;
  final VoidCallback onScheduleHoverTooltipMeasure;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final showHoverTooltip =
        isNativeDesktop && hoveredTile != null && snapshot != null;
    final showSelectedTooltip =
        !isNativeDesktop && selectedTile != null && snapshot != null;

    return Container(
      color: colorScheme.surfaceContainerLowest,
      child: Center(
        child: AspectRatio(
          aspectRatio: tilemapLogicalWidth / tilemapLogicalHeight,
          child: Container(
            decoration: BoxDecoration(
              border: Border.all(color: colorScheme.outlineVariant),
              borderRadius: BorderRadius.circular(4),
            ),
            clipBehavior: Clip.antiAlias,
            child: LayoutBuilder(
              builder: (context, constraints) {
                final size = constraints.biggest;
                return InteractiveViewer(
                  transformationController: transformationController,
                  minScale: minScale,
                  maxScale: maxScale,
                  panEnabled: true,
                  scaleEnabled: true,
                  boundaryMargin: const EdgeInsets.all(double.infinity),
                  constrained: false,
                  child: SizedBox(
                    width: size.width,
                    height: size.height,
                    child: MouseRegion(
                      onHover: (event) => onHover(event.localPosition, size),
                      onExit: (_) => onHoverExit(),
                      child: GestureDetector(
                        behavior: HitTestBehavior.opaque,
                        onTapDown: (details) =>
                            onTap(details.localPosition, size),
                        child: Stack(
                          children: [
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
                                    borderRadius: BorderRadius.circular(12),
                                  ),
                                ),
                              ),
                            ValueListenableBuilder<List<Rect>>(
                              valueListenable: scrollOverlayRects,
                              builder: (context, scrollRects, _) {
                                return CustomPaint(
                                  painter: TilemapGridPainter(
                                    showTileGrid: showTileGrid,
                                    showAttributeGrid: showAttributeGrid,
                                    showAttributeGrid32: showAttributeGrid32,
                                    showNametableDelimiters:
                                        showNametableDelimiters,
                                    showScrollOverlay: showScrollOverlay,
                                    scrollOverlayRects: scrollRects,
                                    hoveredTile: hoveredTile,
                                    selectedTile: selectedTile,
                                  ),
                                  size: Size.infinite,
                                );
                              },
                            ),
                            if (showHoverTooltip)
                              _TilemapHoverTooltip(
                                tile: hoveredTile!,
                                snapshot: snapshot!,
                                position: hoverPosition,
                                canvasSize: size,
                                width: 320,
                                maxHeight: 420,
                                lastTooltipSize: lastHoverTooltipSize,
                                tooltipKey: hoverTooltipKey,
                                onScheduleMeasure:
                                    onScheduleHoverTooltipMeasure,
                              ),
                            if (showSelectedTooltip)
                              _TilemapHoverTooltip(
                                tile: selectedTile!,
                                snapshot: snapshot!,
                                position: selectedPosition,
                                canvasSize: size,
                                width: 280,
                                maxHeight: 380,
                                lastTooltipSize: lastHoverTooltipSize,
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
        ),
      ),
    );
  }
}

class _TilemapHoverTooltip extends StatelessWidget {
  const _TilemapHoverTooltip({
    required this.tile,
    required this.snapshot,
    required this.position,
    required this.canvasSize,
    required this.width,
    required this.maxHeight,
    required this.lastTooltipSize,
    this.tooltipKey,
    this.onScheduleMeasure,
  });

  final TilemapCoord tile;
  final bridge.TilemapSnapshot snapshot;
  final Offset? position;
  final Size canvasSize;
  final double width;
  final double maxHeight;
  final Size lastTooltipSize;
  final GlobalKey? tooltipKey;
  final VoidCallback? onScheduleMeasure;

  @override
  Widget build(BuildContext context) {
    final pos = position;
    if (pos == null) return const SizedBox();

    final info = computeTilemapTileInfo(snapshot, tile);
    if (info == null) return const SizedBox();

    final maxAllowedHeight = (canvasSize.height - 16).clamp(140.0, maxHeight);
    final tooltipHeight = lastTooltipSize.height.clamp(120.0, maxAllowedHeight);

    final preferRight = pos.dx < canvasSize.width * 0.55;
    final preferDown = pos.dy < canvasSize.height * 0.55;

    final dxCandidate = preferRight ? pos.dx + 16 : pos.dx - width - 16;
    final dyCandidate = preferDown ? pos.dy + 16 : pos.dy - tooltipHeight - 16;

    final dx = dxCandidate.clamp(8.0, canvasSize.width - width - 8.0);
    final dy = dyCandidate.clamp(8.0, canvasSize.height - tooltipHeight - 8.0);

    onScheduleMeasure?.call();

    return Positioned(
      left: dx,
      top: dy,
      child: ConstrainedBox(
        constraints: BoxConstraints(maxHeight: maxAllowedHeight),
        child: SizedBox(
          width: width,
          child: KeyedSubtree(
            key: tooltipKey,
            child: Card(
              clipBehavior: Clip.antiAlias,
              elevation: 8,
              child: SingleChildScrollView(
                physics: const ClampingScrollPhysics(),
                child: Padding(
                  padding: const EdgeInsets.all(12),
                  child: Row(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      TilemapTilePreview(snapshot: snapshot, info: info),
                      const SizedBox(width: 12),
                      Expanded(
                        child: DefaultTextStyle(
                          style: Theme.of(context).textTheme.bodySmall!,
                          child: TilemapTileInfoTable(info: info),
                        ),
                      ),
                    ],
                  ),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}
