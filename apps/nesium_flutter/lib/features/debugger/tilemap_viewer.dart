import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/domain/aux_texture_ids.dart';
import 'package:nesium_flutter/domain/nes_controller.dart';
import 'package:nesium_flutter/domain/nes_texture_service.dart';
import 'package:nesium_flutter/features/debugger/viewer_skeletonizer.dart';
import 'package:nesium_flutter/l10n/app_localizations.dart';
import 'package:nesium_flutter/logging/app_logger.dart';
import 'package:nesium_flutter/platform/platform_capabilities.dart';

import 'tilemap/tilemap_models.dart';
import 'tilemap/tilemap_painters.dart';
import 'tilemap/tilemap_settings_dialog.dart';
import 'tilemap/tilemap_side_panel.dart';
import 'tilemap/tilemap_widgets.dart';

/// Tilemap Viewer that displays NES nametables via a Flutter Texture.
class TilemapViewer extends ConsumerStatefulWidget {
  const TilemapViewer({super.key});

  @override
  ConsumerState<TilemapViewer> createState() => _TilemapViewerState();
}

class _TilemapViewerState extends ConsumerState<TilemapViewer> {
  static const int _width = 512;
  static const int _height = 480;

  final NesTextureService _textureService = NesTextureService();
  int? _tilemapTextureId;
  int? _flutterTextureId;
  bool _isCreating = false;
  String? _error;
  StreamSubscription<bridge.TilemapSnapshot>? _tilemapSnapshotSub;
  bridge.TilemapSnapshot? _tilemapSnapshot;

  // Capture mode state
  TilemapCaptureMode _captureMode = TilemapCaptureMode.vblankStart;
  int _scanline = 0;
  int _dot = 0;
  late final TextEditingController _scanlineController = TextEditingController(
    text: _scanline.toString(),
  );
  late final TextEditingController _dotController = TextEditingController(
    text: _dot.toString(),
  );

  // Overlay options
  bool _showTileGrid = false;
  bool _showAttributeGrid = false;
  bool _showAttributeGrid32 = false;
  bool _showNametableDelimiters = true;
  bool _showScrollOverlay = false;
  TilemapDisplayMode _displayMode = TilemapDisplayMode.defaultMode;

  // Hover/selection
  TileCoord? _hoveredTile;
  TileCoord? _selectedTile;
  Offset? _hoverPosition;
  Offset? _selectedPosition; // For mobile tooltip positioning
  Size _lastHoverTooltipSize = const Size(320, 240);
  final GlobalKey _hoverTooltipKey = GlobalKey();
  bool _hoverTooltipMeasurePending = false;
  bool _showSidePanel = true;

  // ValueNotifier for scroll overlay rects - allows isolated repaint
  final ValueNotifier<List<Rect>> _scrollOverlayRects = ValueNotifier(const []);

  // Zoom and pan state
  final TransformationController _transformationController =
      TransformationController();
  static const double _minScale = 1.0;
  static const double _maxScale = 5.0;
  bool _isCanvasTransformed = false;

  @override
  void initState() {
    super.initState();
    _transformationController.addListener(_onTransformChanged);
    _createTexture();
  }

  void _onTransformChanged() {
    final matrix = _transformationController.value;
    final isTransformed = matrix != Matrix4.identity();
    if (_isCanvasTransformed != isTransformed) {
      setState(() => _isCanvasTransformed = isTransformed);
    }
  }

  void _resetCanvasTransform() {
    _transformationController.value = Matrix4.identity();
  }

  Future<void> _createTexture() async {
    if (_isCreating) return;
    setState(() {
      _isCreating = true;
      _error = null;
    });

    try {
      final ids = await AuxTextureIdsCache.get();
      _tilemapTextureId ??= ids.tilemap;

      final textureId = await _textureService.createAuxTexture(
        id: _tilemapTextureId!,
        width: _width,
        height: _height,
      );

      await _tilemapSnapshotSub?.cancel();
      _tilemapSnapshotSub = bridge.tilemapStateStream().listen(
        (snap) {
          if (!mounted) return;
          _tilemapSnapshot = snap;
          if (_showScrollOverlay) {
            _scrollOverlayRects.value = _scrollOverlayRectsFromSnapshot(snap);
          }
        },
        onError: (e, st) {
          logError(
            e,
            stackTrace: st,
            message: 'Tilemap state stream error',
            logger: 'tilemap_viewer',
          );
        },
      );
      unawaitedLogged(
        _applyCaptureMode(),
        message: 'Failed to set tilemap capture point',
      );
      unawaitedLogged(
        _applyTextureRenderMode(),
        message: 'Failed to set tilemap display mode',
      );

      if (mounted) {
        setState(() {
          _flutterTextureId = textureId;
          _isCreating = false;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() {
          _error = e.toString();
          _isCreating = false;
        });
      }
    }
  }

  @override
  void dispose() {
    final textureId = _tilemapTextureId;
    if (textureId != null) {
      _textureService.pauseAuxTexture(textureId);
    }
    unawaited(_tilemapSnapshotSub?.cancel());
    bridge.unsubscribeTilemapTexture();
    if (textureId != null) {
      _textureService.disposeAuxTexture(textureId);
    }
    _scanlineController.dispose();
    _dotController.dispose();
    _transformationController.removeListener(_onTransformChanged);
    _transformationController.dispose();
    _scrollOverlayRects.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (_error != null) {
      return _buildErrorState();
    }
    final hasRom = ref.watch(nesControllerProvider).romHash != null;
    final loading = !hasRom || _isCreating || _flutterTextureId == null;
    return ViewerSkeletonizer(
      enabled: loading,
      child: _buildMainLayout(context),
    );
  }

  Widget _buildErrorState() {
    final l10n = AppLocalizations.of(context)!;
    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Icon(Icons.error_outline, size: 48, color: Colors.red),
          const SizedBox(height: 16),
          Text(l10n.tilemapError(_error ?? '')),
          const SizedBox(height: 16),
          FilledButton.tonal(
            onPressed: _createTexture,
            child: Text(l10n.tilemapRetry),
          ),
        ],
      ),
    );
  }

  Widget _buildMainLayout(BuildContext context) {
    final tilemap = _buildTilemapView(context);

    if (isNativeDesktop) {
      return Stack(
        children: [
          Row(
            children: [
              Expanded(child: tilemap),
              _buildDesktopSidePanelWrapper(context),
            ],
          ),
          Positioned(
            top: 12,
            right: 12,
            child: _buildPanelToggleButton(context),
          ),
          if (_isCanvasTransformed)
            Positioned(
              bottom: 12,
              left: 12,
              child: _buildResetZoomButton(context),
            ),
        ],
      );
    }

    return GestureDetector(
      onTap: _clearSelection,
      behavior: HitTestBehavior.translucent,
      child: Stack(
        children: [
          tilemap,
          Positioned(top: 12, right: 12, child: _buildSettingsButton(context)),
          if (_isCanvasTransformed)
            Positioned(
              bottom: 12,
              left: 12,
              child: _buildResetZoomButton(context),
            ),
        ],
      ),
    );
  }

  void _clearSelection() {
    if (_selectedTile != null) {
      setState(() {
        _selectedTile = null;
        _selectedPosition = null;
      });
    }
  }

  Widget _buildResetZoomButton(BuildContext context) {
    final theme = Theme.of(context);
    final l10n = AppLocalizations.of(context)!;

    return AnimatedOpacity(
      opacity: _isCanvasTransformed ? 1.0 : 0.0,
      duration: const Duration(milliseconds: 200),
      child: Material(
        color: theme.colorScheme.surfaceContainerHighest.withValues(alpha: 0.9),
        borderRadius: BorderRadius.circular(8),
        elevation: 4,
        child: InkWell(
          borderRadius: BorderRadius.circular(8),
          onTap: _resetCanvasTransform,
          child: Padding(
            padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            child: Row(
              mainAxisSize: MainAxisSize.min,
              children: [
                Icon(
                  Icons.zoom_out_map,
                  size: 18,
                  color: theme.colorScheme.onSurface,
                ),
                const SizedBox(width: 6),
                Text(
                  l10n.tilemapResetZoom,
                  style: theme.textTheme.labelMedium?.copyWith(
                    color: theme.colorScheme.onSurface,
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }

  Widget _buildTilemapView(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final showHoverTooltip =
        isNativeDesktop && _hoveredTile != null && _tilemapSnapshot != null;
    final showSelectedTooltip =
        !isNativeDesktop && _selectedTile != null && _tilemapSnapshot != null;

    return Container(
      color: colorScheme.surfaceContainerLowest,
      child: Center(
        child: AspectRatio(
          aspectRatio: _width / _height,
          child: Container(
            decoration: BoxDecoration(
              border: Border.all(color: colorScheme.outlineVariant, width: 1),
              borderRadius: BorderRadius.circular(4),
            ),
            clipBehavior: Clip.antiAlias,
            child: LayoutBuilder(
              builder: (context, constraints) {
                final size = constraints.biggest;
                return InteractiveViewer(
                  transformationController: _transformationController,
                  minScale: _minScale,
                  maxScale: _maxScale,
                  panEnabled: true,
                  scaleEnabled: true,
                  boundaryMargin: const EdgeInsets.all(double.infinity),
                  constrained: false,
                  child: SizedBox(
                    width: size.width,
                    height: size.height,
                    child: MouseRegion(
                      onHover: (event) =>
                          _handleHover(event.localPosition, size),
                      onExit: (_) => setState(() {
                        _hoveredTile = null;
                        _hoverPosition = null;
                      }),
                      child: GestureDetector(
                        behavior: HitTestBehavior.opaque,
                        onTapDown: (details) =>
                            _handleTap(details.localPosition, size),
                        child: Stack(
                          children: [
                            if (_flutterTextureId != null &&
                                !ViewerSkeletonScope.enabledOf(context))
                              Texture(
                                textureId: _flutterTextureId!,
                                filterQuality: FilterQuality.none,
                              )
                            else
                              Positioned.fill(
                                child: DecoratedBox(
                                  decoration: BoxDecoration(
                                    color: Theme.of(
                                      context,
                                    ).colorScheme.surfaceContainerHighest,
                                    borderRadius: BorderRadius.circular(12),
                                  ),
                                ),
                              ),
                            ValueListenableBuilder<List<Rect>>(
                              valueListenable: _scrollOverlayRects,
                              builder: (context, scrollRects, _) {
                                return CustomPaint(
                                  painter: TilemapGridPainter(
                                    showTileGrid: _showTileGrid,
                                    showAttributeGrid: _showAttributeGrid,
                                    showAttributeGrid32: _showAttributeGrid32,
                                    showNametableDelimiters:
                                        _showNametableDelimiters,
                                    showScrollOverlay: _showScrollOverlay,
                                    scrollOverlayRects: scrollRects,
                                    hoveredTile: _hoveredTile,
                                    selectedTile: _selectedTile,
                                  ),
                                  size: Size.infinite,
                                );
                              },
                            ),
                            if (showHoverTooltip)
                              _buildHoverTooltip(context, size),
                            if (showSelectedTooltip)
                              _buildSelectedTooltip(context, size),
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

  void _handleHover(Offset position, Size size) {
    final tile = _tileAtPosition(position, size);
    if (tile == _hoveredTile && position == _hoverPosition) return;
    setState(() {
      _hoveredTile = tile;
      _hoverPosition = position;
    });
  }

  void _handleTap(Offset position, Size size) {
    final tile = _tileAtPosition(position, size);
    setState(() {
      _selectedTile = tile;
      _selectedPosition = position;
    });
  }

  TileCoord? _tileAtPosition(Offset position, Size size) {
    if (size.width <= 0 || size.height <= 0) return null;
    if (position.dx < 0 ||
        position.dy < 0 ||
        position.dx > size.width ||
        position.dy > size.height) {
      return null;
    }

    final x = (position.dx / size.width) * _width;
    final y = (position.dy / size.height) * _height;
    final tileX = (x / 8).floor().clamp(0, 63);
    final tileY = (y / 8).floor().clamp(0, 59);
    return TileCoord(tileX, tileY);
  }

  Widget _buildHoverTooltip(BuildContext context, Size size) {
    final tile = _hoveredTile;
    final snap = _tilemapSnapshot;
    final pos = _hoverPosition;
    if (tile == null || snap == null || pos == null) return const SizedBox();

    final info = TileInfo.compute(snap, tile);
    if (info == null) return const SizedBox();

    const tooltipWidth = 320.0;
    final maxAllowedHeight = (size.height - 16).clamp(140.0, 420.0);
    final tooltipHeight = _lastHoverTooltipSize.height.clamp(
      120.0,
      maxAllowedHeight,
    );

    final preferRight = pos.dx < size.width * 0.55;
    final preferDown = pos.dy < size.height * 0.55;

    final dxCandidate = preferRight ? pos.dx + 16 : pos.dx - tooltipWidth - 16;
    final dyCandidate = preferDown ? pos.dy + 16 : pos.dy - tooltipHeight - 16;

    final dx = dxCandidate.clamp(8.0, size.width - tooltipWidth - 8.0);
    final dy = dyCandidate.clamp(8.0, size.height - tooltipHeight - 8.0);

    _scheduleHoverTooltipMeasure();

    return Positioned(
      left: dx,
      top: dy,
      child: ConstrainedBox(
        constraints: BoxConstraints(maxHeight: maxAllowedHeight),
        child: SizedBox(
          width: tooltipWidth,
          child: KeyedSubtree(
            key: _hoverTooltipKey,
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
                      TilePreview(snapshot: snap, info: info),
                      const SizedBox(width: 12),
                      Expanded(
                        child: DefaultTextStyle(
                          style: Theme.of(context).textTheme.bodySmall!,
                          child: TileInfoTable(info: info),
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

  Widget _buildSelectedTooltip(BuildContext context, Size size) {
    final tile = _selectedTile;
    final snap = _tilemapSnapshot;
    final pos = _selectedPosition;
    if (tile == null || snap == null || pos == null) return const SizedBox();

    final info = TileInfo.compute(snap, tile);
    if (info == null) return const SizedBox();

    const tooltipWidth = 280.0;
    final maxAllowedHeight = (size.height - 16).clamp(140.0, 380.0);
    final tooltipHeight = _lastHoverTooltipSize.height.clamp(
      120.0,
      maxAllowedHeight,
    );

    final preferRight = pos.dx < size.width * 0.55;
    final preferDown = pos.dy < size.height * 0.55;

    final dxCandidate = preferRight ? pos.dx + 16 : pos.dx - tooltipWidth - 16;
    final dyCandidate = preferDown ? pos.dy + 16 : pos.dy - tooltipHeight - 16;

    final dx = dxCandidate.clamp(8.0, size.width - tooltipWidth - 8.0);
    final dy = dyCandidate.clamp(8.0, size.height - tooltipHeight - 8.0);

    return Positioned(
      left: dx,
      top: dy,
      child: ConstrainedBox(
        constraints: BoxConstraints(maxHeight: maxAllowedHeight),
        child: SizedBox(
          width: tooltipWidth,
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
                    TilePreview(snapshot: snap, info: info),
                    const SizedBox(width: 12),
                    Expanded(
                      child: DefaultTextStyle(
                        style: Theme.of(context).textTheme.bodySmall!,
                        child: TileInfoTable(info: info),
                      ),
                    ),
                  ],
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }

  void _scheduleHoverTooltipMeasure() {
    if (_hoverTooltipMeasurePending) return;
    _hoverTooltipMeasurePending = true;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _hoverTooltipMeasurePending = false;
      if (!mounted) return;
      final size = _hoverTooltipKey.currentContext?.size;
      if (size == null || size == Size.zero) return;
      if (size == _lastHoverTooltipSize) return;
      setState(() => _lastHoverTooltipSize = size);
    });
  }

  Widget _buildDesktopSidePanelWrapper(BuildContext context) {
    const panelWidth = 280.0;
    return ClipRect(
      child: TweenAnimationBuilder<double>(
        duration: const Duration(milliseconds: 180),
        curve: Curves.easeOut,
        tween: Tween<double>(end: _showSidePanel ? 1.0 : 0.0),
        builder: (context, factor, child) {
          return IgnorePointer(
            ignoring: factor == 0.0,
            child: Align(
              alignment: Alignment.centerLeft,
              widthFactor: factor,
              child: child,
            ),
          );
        },
        child: SizedBox(
          width: panelWidth,
          child: TilemapSidePanel(
            snapshot: _tilemapSnapshot,
            selectedTile: _selectedTile,
            showTileGrid: _showTileGrid,
            showAttributeGrid: _showAttributeGrid,
            showAttributeGrid32: _showAttributeGrid32,
            showNametableDelimiters: _showNametableDelimiters,
            showScrollOverlay: _showScrollOverlay,
            displayMode: _displayMode,
            captureMode: _captureMode,
            scanlineController: _scanlineController,
            dotController: _dotController,
            onShowTileGridChanged: (v) =>
                setState(() => _showTileGrid = v ?? false),
            onShowAttributeGridChanged: (v) =>
                setState(() => _showAttributeGrid = v ?? false),
            onShowAttributeGrid32Changed: (v) =>
                setState(() => _showAttributeGrid32 = v ?? false),
            onShowNametableDelimitersChanged: (v) =>
                setState(() => _showNametableDelimiters = v ?? false),
            onShowScrollOverlayChanged: (v) =>
                setState(() => _showScrollOverlay = v ?? false),
            onDisplayModeChanged: (v) {
              setState(() => _displayMode = v);
              _applyTextureRenderMode();
            },
            onCaptureModeChanged: (v) {
              setState(() => _captureMode = v);
              _applyCaptureMode();
            },
            onScanlineSubmitted: (v) {
              final value = int.tryParse(v);
              if (value != null && value >= -1 && value <= 260) {
                setState(() => _scanline = value);
                _scanlineController.text = _scanline.toString();
                _applyCaptureMode();
              }
            },
            onDotSubmitted: (v) {
              final value = int.tryParse(v);
              if (value != null && value >= 0 && value <= 340) {
                setState(() => _dot = value);
                _dotController.text = _dot.toString();
                _applyCaptureMode();
              }
            },
          ),
        ),
      ),
    );
  }

  Widget _buildSettingsButton(BuildContext context) {
    final theme = Theme.of(context);
    final l10n = AppLocalizations.of(context)!;

    return IconButton(
      icon: Container(
        padding: const EdgeInsets.all(8),
        decoration: BoxDecoration(
          color: theme.colorScheme.surfaceContainerHighest.withValues(
            alpha: 0.8,
          ),
          borderRadius: BorderRadius.circular(8),
          boxShadow: [
            BoxShadow(
              color: Colors.black.withValues(alpha: 0.1),
              blurRadius: 4,
              offset: const Offset(0, 2),
            ),
          ],
        ),
        child: Icon(
          Icons.settings,
          color: theme.colorScheme.onSurface,
          size: 20,
        ),
      ),
      tooltip: l10n.tilemapSettings,
      onPressed: () {
        final RenderBox button = context.findRenderObject() as RenderBox;
        final buttonPosition = button.localToGlobal(Offset.zero);
        showTilemapSettingsMenu(
          context: context,
          buttonPosition: buttonPosition,
          buttonSize: button.size,
          displayMode: _displayMode,
          showTileGrid: _showTileGrid,
          showAttributeGrid: _showAttributeGrid,
          showAttributeGrid32: _showAttributeGrid32,
          showNametableDelimiters: _showNametableDelimiters,
          showScrollOverlay: _showScrollOverlay,
          captureMode: _captureMode,
          scanline: _scanline,
          dot: _dot,
          onDisplayModeChanged: (v) {
            setState(() => _displayMode = v);
            _applyTextureRenderMode();
          },
          onShowTileGridChanged: (v) =>
              setState(() => _showTileGrid = v ?? false),
          onShowAttributeGridChanged: (v) =>
              setState(() => _showAttributeGrid = v ?? false),
          onShowAttributeGrid32Changed: (v) =>
              setState(() => _showAttributeGrid32 = v ?? false),
          onShowNametableDelimitersChanged: (v) =>
              setState(() => _showNametableDelimiters = v ?? false),
          onShowScrollOverlayChanged: (v) =>
              setState(() => _showScrollOverlay = v ?? false),
          onCaptureModeChanged: (v) {
            setState(() => _captureMode = v);
            _applyCaptureMode();
          },
          onScanlineDotChanged: (s, d) {
            setState(() {
              _scanline = s;
              _dot = d;
              _scanlineController.text = _scanline.toString();
              _dotController.text = _dot.toString();
            });
            _applyCaptureMode();
          },
        );
      },
    );
  }

  Widget _buildPanelToggleButton(BuildContext context) {
    final theme = Theme.of(context);
    final l10n = AppLocalizations.of(context)!;

    final icon = _showSidePanel ? Icons.chevron_right : Icons.chevron_left;
    final tooltip = _showSidePanel
        ? l10n.tilemapHidePanel
        : l10n.tilemapShowPanel;

    return IconButton(
      icon: Container(
        padding: const EdgeInsets.all(8),
        decoration: BoxDecoration(
          color: theme.colorScheme.surfaceContainerHighest.withValues(
            alpha: 0.8,
          ),
          borderRadius: BorderRadius.circular(8),
          boxShadow: [
            BoxShadow(
              color: Colors.black.withValues(alpha: 0.1),
              blurRadius: 4,
              offset: const Offset(0, 2),
            ),
          ],
        ),
        child: Icon(icon, color: theme.colorScheme.onSurface, size: 20),
      ),
      tooltip: tooltip,
      onPressed: () => setState(() => _showSidePanel = !_showSidePanel),
    );
  }

  Future<void> _applyCaptureMode() async {
    switch (_captureMode) {
      case TilemapCaptureMode.frameStart:
        await bridge.setTilemapCaptureFrameStart();
      case TilemapCaptureMode.vblankStart:
        await bridge.setTilemapCaptureVblankStart();
      case TilemapCaptureMode.scanline:
        await bridge.setTilemapCaptureScanline(scanline: _scanline, dot: _dot);
    }
  }

  Future<void> _applyTextureRenderMode() async {
    final (showTileGrid, showAttributeGrid) = switch (_displayMode) {
      TilemapDisplayMode.defaultMode => (false, false),
      TilemapDisplayMode.grayscale => (true, false),
      TilemapDisplayMode.attributeView => (false, true),
    };
    final mode = showAttributeGrid
        ? 2
        : showTileGrid
        ? 1
        : 0;
    await bridge.setTilemapDisplayMode(mode: mode);
  }

  List<Rect> _scrollOverlayRectsFromSnapshot(bridge.TilemapSnapshot snap) {
    final v = snap.tempAddr & 0x7FFF;
    final fineX = snap.fineX & 0x07;
    final coarseX = v & 0x1F;
    final coarseY = (v >> 5) & 0x1F;
    final ntX = (v >> 10) & 0x01;
    final ntY = (v >> 11) & 0x01;
    final fineY = (v >> 12) & 0x07;

    final scrollX = coarseX * 8 + fineX;
    final scrollY = coarseY * 8 + fineY;
    final baseX = ntX * 256;
    final baseY = ntY * 240;

    final x0 = (baseX + scrollX) % _width;
    final y0 = (baseY + scrollY) % _height;

    return _splitWrappedRect(
      x: x0.toDouble(),
      y: y0.toDouble(),
      w: 256.0,
      h: 240.0,
      wrapW: _width.toDouble(),
      wrapH: _height.toDouble(),
    );
  }

  List<Rect> _splitWrappedRect({
    required double x,
    required double y,
    required double w,
    required double h,
    required double wrapW,
    required double wrapH,
  }) {
    final x0 = x % wrapW;
    final y0 = y % wrapH;
    final right = x0 + w;
    final bottom = y0 + h;

    final w1 = (right <= wrapW) ? w : (wrapW - x0);
    final h1 = (bottom <= wrapH) ? h : (wrapH - y0);
    final w2 = w - w1;
    final h2 = h - h1;

    final rects = <Rect>[Rect.fromLTWH(x0, y0, w1, h1)];
    if (w2 > 0) {
      rects.add(Rect.fromLTWH(0, y0, w2, h1));
    }
    if (h2 > 0) {
      rects.add(Rect.fromLTWH(x0, 0, w1, h2));
      if (w2 > 0) {
        rects.add(Rect.fromLTWH(0, 0, w2, h2));
      }
    }
    return rects;
  }
}
