import 'dart:async';
import 'dart:typed_data';

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

/// Tilemap Viewer that displays NES nametables via a Flutter Texture.
class TilemapViewer extends ConsumerStatefulWidget {
  const TilemapViewer({super.key});

  @override
  ConsumerState<TilemapViewer> createState() => _TilemapViewerState();
}

enum _TilemapDisplayMode { defaultMode, grayscale, attributeView }

class _TilemapViewerState extends ConsumerState<TilemapViewer> {
  static const int _width = 512;
  static const int _height = 480;
  static const int _minScanline = -1;
  static const int _maxScanline = 260;
  static const int _minDot = 0;
  static const int _maxDot = 340;

  final NesTextureService _textureService = NesTextureService();
  int? _tilemapTextureId;
  int? _flutterTextureId;
  bool _isCreating = false;
  String? _error;
  StreamSubscription<bridge.TilemapSnapshot>? _tilemapSnapshotSub;
  bridge.TilemapSnapshot? _tilemapSnapshot;

  // Capture mode state
  _TilemapCaptureMode _captureMode = _TilemapCaptureMode.vblankStart;
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
  _TilemapDisplayMode _displayMode = _TilemapDisplayMode.defaultMode;

  // Hover/selection
  _TileCoord? _hoveredTile;
  _TileCoord? _selectedTile;
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
    // Check if transformation is not identity (default state)
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
      _tilemapSnapshotSub = bridge.tilemapStateStream().listen((snap) {
        if (!mounted) return;
        // Store snapshot for tooltip data access.
        _tilemapSnapshot = snap;
        // Update scroll overlay rects via ValueNotifier (isolated repaint).
        if (_showScrollOverlay) {
          _scrollOverlayRects.value = _scrollOverlayRectsFromSnapshot(snap);
        }
      }, onError: (_) {});
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
    final base = ViewerSkeletonizer(
      enabled: loading,
      child: _buildMainLayout(context),
    );
    return base;
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

    // Mobile layout - wrap in GestureDetector to dismiss tooltip on tap outside
    return GestureDetector(
      onTap: _clearSelection,
      behavior: HitTestBehavior.translucent,
      child: Stack(
        children: [
          tilemap,
          // Settings button (top-right)
          Positioned(top: 12, right: 12, child: _buildSettingsButton(context)),
          // Reset zoom button (bottom-left, only when transformed)
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
    // Desktop: show tooltip on hover
    final showHoverTooltip =
        isNativeDesktop && _hoveredTile != null && _tilemapSnapshot != null;
    // Mobile: show tooltip on tap selection
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
                  // Allow panning beyond boundaries when zoomed in
                  boundaryMargin: const EdgeInsets.all(double.infinity),
                  constrained: false,
                  child: SizedBox(
                    width: size.width,
                    height: size.height,
                    child: MouseRegion(
                      onHover: (event) =>
                          _handleHoverWithTransform(event.localPosition, size),
                      onExit: (_) => setState(() {
                        _hoveredTile = null;
                        _hoverPosition = null;
                      }),
                      child: GestureDetector(
                        behavior: HitTestBehavior.opaque,
                        onTapDown: (details) => _handleTapWithTransform(
                          details.localPosition,
                          size,
                        ),
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
                            // Use ValueListenableBuilder for scroll overlay - isolated repaint
                            ValueListenableBuilder<List<Rect>>(
                              valueListenable: _scrollOverlayRects,
                              builder: (context, scrollRects, _) {
                                return CustomPaint(
                                  painter: _TilemapGridPainter(
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

  /// Transform local position considering zoom/pan transformation
  Offset _transformPosition(Offset localPosition) {
    // The localPosition is already in the transformed (zoomed/panned) space
    // For InteractiveViewer with constrained: false, localPosition is
    // already relative to the child content
    return localPosition;
  }

  void _handleHoverWithTransform(Offset localPosition, Size size) {
    final transformedPosition = _transformPosition(localPosition);
    _handleHover(transformedPosition, size);
  }

  void _handleTapWithTransform(Offset localPosition, Size size) {
    final transformedPosition = _transformPosition(localPosition);
    _handleTap(transformedPosition, size);
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

  _TileCoord? _tileAtPosition(Offset position, Size size) {
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
    return _TileCoord(tileX, tileY);
  }

  Widget _buildHoverTooltip(BuildContext context, Size size) {
    final tile = _hoveredTile;
    final snap = _tilemapSnapshot;
    final pos = _hoverPosition;
    if (tile == null || snap == null || pos == null) return const SizedBox();

    final info = _computeTileInfo(snap, tile);
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
            child: _buildTileHoverCard(snapshot: snap, info: info),
          ),
        ),
      ),
    );
  }

  /// Tooltip for mobile - shows on tap selection
  Widget _buildSelectedTooltip(BuildContext context, Size size) {
    final tile = _selectedTile;
    final snap = _tilemapSnapshot;
    final pos = _selectedPosition;
    if (tile == null || snap == null || pos == null) return const SizedBox();

    final info = _computeTileInfo(snap, tile);
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
          child: _buildTileHoverCard(snapshot: snap, info: info),
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

  Widget _buildTileHoverCard({
    required bridge.TilemapSnapshot snapshot,
    required _TileInfo info,
  }) {
    return Card(
      clipBehavior: Clip.antiAlias,
      elevation: 8,
      child: SingleChildScrollView(
        physics: const ClampingScrollPhysics(),
        child: Padding(
          padding: const EdgeInsets.all(12),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _TilePreview(snapshot: snapshot, info: info),
              const SizedBox(width: 12),
              Expanded(
                child: DefaultTextStyle(
                  style: Theme.of(context).textTheme.bodySmall!,
                  child: _TileInfoTable(info: info),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildDesktopSidePanel(BuildContext context) {
    final snap = _tilemapSnapshot;
    final selected = _selectedTile != null && snap != null
        ? _computeTileInfo(snap, _selectedTile!)
        : null;
    final l10n = AppLocalizations.of(context)!;
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

    return Container(
      decoration: BoxDecoration(
        color: colorScheme.surfaceContainerLowest,
        border: Border(
          left: BorderSide(color: colorScheme.outlineVariant, width: 1),
        ),
      ),
      child: Scrollbar(
        thumbVisibility: true,
        child: ListView(
          padding: const EdgeInsets.all(12),
          children: [
            _sideSection(
              context,
              title: l10n.tilemapPanelDisplay,
              child: Column(
                children: [
                  _displayModeDropdown(context),
                  const SizedBox(height: 4),
                  CheckboxListTile(
                    dense: true,
                    visualDensity: VisualDensity.compact,
                    controlAffinity: ListTileControlAffinity.trailing,
                    contentPadding: EdgeInsets.zero,
                    title: Text(l10n.tilemapTileGrid),
                    value: _showTileGrid,
                    onChanged: (v) {
                      setState(() => _showTileGrid = v ?? false);
                    },
                  ),
                  CheckboxListTile(
                    dense: true,
                    visualDensity: VisualDensity.compact,
                    controlAffinity: ListTileControlAffinity.trailing,
                    contentPadding: EdgeInsets.zero,
                    title: Text(l10n.tilemapAttrGrid),
                    value: _showAttributeGrid,
                    onChanged: (v) {
                      setState(() => _showAttributeGrid = v ?? false);
                    },
                  ),
                  CheckboxListTile(
                    dense: true,
                    visualDensity: VisualDensity.compact,
                    controlAffinity: ListTileControlAffinity.trailing,
                    contentPadding: EdgeInsets.zero,
                    title: Text(l10n.tilemapAttrGrid32),
                    value: _showAttributeGrid32,
                    onChanged: (v) =>
                        setState(() => _showAttributeGrid32 = v ?? false),
                  ),
                  CheckboxListTile(
                    dense: true,
                    visualDensity: VisualDensity.compact,
                    controlAffinity: ListTileControlAffinity.trailing,
                    contentPadding: EdgeInsets.zero,
                    title: Text(l10n.tilemapNtBounds),
                    value: _showNametableDelimiters,
                    onChanged: (v) {
                      setState(() => _showNametableDelimiters = v ?? false);
                    },
                  ),
                  CheckboxListTile(
                    dense: true,
                    visualDensity: VisualDensity.compact,
                    controlAffinity: ListTileControlAffinity.trailing,
                    contentPadding: EdgeInsets.zero,
                    title: Text(l10n.tilemapScrollOverlay),
                    value: _showScrollOverlay,
                    onChanged: (v) =>
                        setState(() => _showScrollOverlay = v ?? false),
                  ),
                ],
              ),
            ),
            _sideSection(
              context,
              title: l10n.tilemapCapture,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  RadioGroup<_TilemapCaptureMode>(
                    groupValue: _captureMode,
                    onChanged: (v) {
                      if (v == null) return;
                      setState(() => _captureMode = v);
                      _applyCaptureMode();
                    },
                    child: Column(
                      children: [
                        RadioListTile<_TilemapCaptureMode>(
                          dense: true,
                          visualDensity: VisualDensity.compact,
                          contentPadding: EdgeInsets.zero,
                          title: Text(l10n.tilemapCaptureFrameStart),
                          value: _TilemapCaptureMode.frameStart,
                        ),
                        RadioListTile<_TilemapCaptureMode>(
                          dense: true,
                          visualDensity: VisualDensity.compact,
                          contentPadding: EdgeInsets.zero,
                          title: Text(l10n.tilemapCaptureVblankStart),
                          value: _TilemapCaptureMode.vblankStart,
                        ),
                        RadioListTile<_TilemapCaptureMode>(
                          dense: true,
                          visualDensity: VisualDensity.compact,
                          contentPadding: EdgeInsets.zero,
                          title: Text(l10n.tilemapCaptureManual),
                          value: _TilemapCaptureMode.scanline,
                        ),
                      ],
                    ),
                  ),
                  const SizedBox(height: 10),
                  Row(
                    children: [
                      Expanded(
                        child: _numberFieldModern(
                          label: l10n.tilemapScanline,
                          enabled: _captureMode == _TilemapCaptureMode.scanline,
                          controller: _scanlineController,
                          hint: '$_minScanline ~ $_maxScanline',
                          onSubmitted: (v) {
                            final value = int.tryParse(v);
                            if (value == null ||
                                value < _minScanline ||
                                value > _maxScanline) {
                              return;
                            }
                            setState(() => _scanline = value);
                            _scanlineController.text = _scanline.toString();
                            _applyCaptureMode();
                          },
                        ),
                      ),
                      const SizedBox(width: 10),
                      Expanded(
                        child: _numberFieldModern(
                          label: l10n.tilemapDot,
                          enabled: _captureMode == _TilemapCaptureMode.scanline,
                          controller: _dotController,
                          hint: '$_minDot ~ $_maxDot',
                          onSubmitted: (v) {
                            final value = int.tryParse(v);
                            if (value == null ||
                                value < _minDot ||
                                value > _maxDot) {
                              return;
                            }
                            setState(() => _dot = value);
                            _dotController.text = _dot.toString();
                            _applyCaptureMode();
                          },
                        ),
                      ),
                    ],
                  ),
                ],
              ),
            ),
            _sideSection(
              context,
              title: l10n.tilemapPanelTilemap,
              child: Column(
                children: [
                  _kvModern(l10n.tilemapInfoSize, '64×60'),
                  _kvModern(l10n.tilemapInfoSizePx, '512×480'),
                  _kvModern(l10n.tilemapInfoTilemapAddress, _hex(0x2000)),
                  _kvModern(
                    l10n.tilemapInfoTilesetAddress,
                    snap != null ? _hex(snap.bgPatternBase) : '—',
                  ),
                  _kvModern(
                    l10n.tilemapInfoMirroring,
                    snap != null ? _mirroringLabel(l10n, snap.mirroring) : '—',
                  ),
                  _kvModern(
                    l10n.tilemapInfoTileFormat,
                    l10n.tilemapInfoTileFormat2bpp,
                  ),
                ],
              ),
            ),
            _sideSection(
              context,
              title: l10n.tilemapPanelSelectedTile,
              child: (selected == null || snap == null)
                  ? _emptyHint(colorScheme.onSurfaceVariant)
                  : _TileInfoCard(info: selected, snapshot: snap),
            ),
          ],
        ),
      ),
    );
  }

  Widget _sideSection(
    BuildContext context, {
    required String title,
    required Widget child,
  }) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Card(
        elevation: 0,
        color: colorScheme.surface,
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(12),
          side: BorderSide(color: colorScheme.outlineVariant),
        ),
        child: Padding(
          padding: const EdgeInsets.all(10),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                title,
                style: theme.textTheme.titleSmall?.copyWith(
                  fontWeight: FontWeight.w700,
                ),
              ),
              const SizedBox(height: 10),
              child,
            ],
          ),
        ),
      ),
    );
  }

  Widget _emptyHint(Color color) {
    return Text('—', style: TextStyle(color: color));
  }

  Widget _numberFieldModern({
    required String label,
    required bool enabled,
    required TextEditingController controller,
    required String hint,
    required ValueChanged<String> onSubmitted,
  }) {
    return TextField(
      enabled: enabled,
      controller: controller
        ..selection = TextSelection.fromPosition(
          TextPosition(offset: controller.text.length),
        ),
      decoration: InputDecoration(
        labelText: label,
        hintText: hint,
        isDense: true,
        filled: true,
        fillColor: Theme.of(context).colorScheme.surfaceContainerLowest,
        border: OutlineInputBorder(borderRadius: BorderRadius.circular(10)),
      ),
      keyboardType: TextInputType.number,
      onSubmitted: onSubmitted,
    );
  }

  Widget _kvModern(String k, String v) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          Expanded(
            child: Text(k, style: const TextStyle(color: Colors.black54)),
          ),
          Text(v, style: const TextStyle(fontWeight: FontWeight.w600)),
        ],
      ),
    );
  }

  String _hex(int value, {int width = 4}) {
    return '\$${value.toRadixString(16).toUpperCase().padLeft(width, '0')}';
  }

  String _mirroringLabel(AppLocalizations l10n, bridge.TilemapMirroring m) {
    switch (m) {
      case bridge.TilemapMirroring.horizontal:
        return l10n.tilemapMirroringHorizontal;
      case bridge.TilemapMirroring.vertical:
        return l10n.tilemapMirroringVertical;
      case bridge.TilemapMirroring.fourScreen:
        return l10n.tilemapMirroringFourScreen;
      case bridge.TilemapMirroring.singleScreenLower:
        return l10n.tilemapMirroringSingleScreenLower;
      case bridge.TilemapMirroring.singleScreenUpper:
        return l10n.tilemapMirroringSingleScreenUpper;
      case bridge.TilemapMirroring.mapperControlled:
        return l10n.tilemapMirroringMapperControlled;
    }
  }

  _TileInfo? _computeTileInfo(bridge.TilemapSnapshot snap, _TileCoord tile) {
    final tileX = tile.x;
    final tileY = tile.y;
    final ntX = tileX >= 32 ? 1 : 0;
    final ntY = tileY >= 30 ? 1 : 0;
    final ntIndex = ntY * 2 + ntX;

    final tileXInNt = tileX % 32;
    final tileYInNt = tileY % 30;

    final ntLocalAddr = tileYInNt * 32 + tileXInNt;
    final tilemapAddress = 0x2000 + ntIndex * 0x400 + ntLocalAddr;

    final ciramBase = _mirrorNametableToCiramOffset(ntIndex, snap.mirroring);
    final tileCiramAddr = ciramBase + ntLocalAddr;
    if (tileCiramAddr < 0 || tileCiramAddr >= snap.ciram.length) return null;

    final tileIndex = snap.ciram[tileCiramAddr];

    final attrLocalAddr = 0x3C0 + (tileYInNt ~/ 4) * 8 + (tileXInNt ~/ 4);
    final attrAddress = 0x2000 + ntIndex * 0x400 + attrLocalAddr;
    final attrCiramAddr = ciramBase + attrLocalAddr;
    final attrByte = attrCiramAddr >= 0 && attrCiramAddr < snap.ciram.length
        ? snap.ciram[attrCiramAddr]
        : 0;

    final shift = ((tileYInNt % 4) ~/ 2) * 4 + ((tileXInNt % 4) ~/ 2) * 2;
    final paletteIndex = (attrByte >> shift) & 0x03;
    final paletteAddress = 0x3F00 + paletteIndex * 4;

    final tileAddressPpu = snap.bgPatternBase + tileIndex * 16;

    return _TileInfo(
      tileX: tileX,
      tileY: tileY,
      ntIndex: ntIndex,
      tileIndex: tileIndex,
      tilemapAddress: tilemapAddress,
      tileAddressPpu: tileAddressPpu,
      paletteIndex: paletteIndex,
      paletteAddress: paletteAddress,
      attrAddress: attrAddress,
      attrByte: attrByte,
    );
  }

  int _mirrorNametableToCiramOffset(
    int ntIndex,
    bridge.TilemapMirroring mirroring,
  ) {
    final physicalNt = switch (mirroring) {
      bridge.TilemapMirroring.horizontal =>
        (ntIndex == 0 || ntIndex == 1) ? 0 : 1,
      bridge.TilemapMirroring.vertical =>
        (ntIndex == 0 || ntIndex == 2) ? 0 : 1,
      bridge.TilemapMirroring.fourScreen => ntIndex.clamp(0, 1),
      bridge.TilemapMirroring.singleScreenLower => 0,
      bridge.TilemapMirroring.singleScreenUpper => 1,
      bridge.TilemapMirroring.mapperControlled => ntIndex.clamp(0, 1),
    };
    return physicalNt * 0x400;
  }

  Widget _buildDesktopSidePanelWrapper(BuildContext context) {
    return ClipRect(
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 180),
        curve: Curves.easeOut,
        width: _showSidePanel ? 280 : 0,
        child: _showSidePanel ? _buildDesktopSidePanel(context) : null,
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
      onPressed: () => _showSettingsDialog(context),
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

  void _showSettingsDialog(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final theme = Theme.of(context);

    // Get button position for menu placement (positioned at top-right)
    final RenderBox button = context.findRenderObject() as RenderBox;
    final RenderBox overlay =
        Overlay.of(context).context.findRenderObject() as RenderBox;
    final buttonPosition = button.localToGlobal(Offset.zero, ancestor: overlay);

    showMenu<void>(
      context: context,
      position: RelativeRect.fromLTRB(
        buttonPosition.dx +
            button.size.width -
            280, // 280px menu width, aligned to button right
        buttonPosition.dy +
            button.size.height +
            4, // Just below button with small gap
        overlay.size.width -
            buttonPosition.dx -
            button.size.width, // Right edge
        0,
      ),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
      items: [
        // Overlay section header
        PopupMenuItem<void>(
          onTap: null, // Disable interaction but keep text color
          height: 32,
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: Text(
            l10n.tilemapOverlay,
            style: theme.textTheme.labelSmall?.copyWith(
              color: theme.colorScheme.onSurfaceVariant,
              fontWeight: FontWeight.w600,
            ),
          ),
        ),
        PopupMenuItem<void>(
          onTap: () {}, // Empty tap to prevent closing
          padding: EdgeInsets.zero,
          child: StatefulBuilder(
            builder: (context, setMenuState) => Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Padding(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 16,
                    vertical: 6,
                  ),
                  child: Row(
                    children: [
                      Expanded(
                        child: Text(
                          l10n.tilemapDisplayMode,
                          style: theme.textTheme.bodySmall,
                        ),
                      ),
                      DropdownButtonHideUnderline(
                        child: DropdownButton<_TilemapDisplayMode>(
                          isDense: true,
                          value: _displayMode,
                          items: [
                            DropdownMenuItem(
                              value: _TilemapDisplayMode.defaultMode,
                              child: Text(l10n.tilemapDisplayModeDefault),
                            ),
                            DropdownMenuItem(
                              value: _TilemapDisplayMode.grayscale,
                              child: Text(l10n.tilemapDisplayModeGrayscale),
                            ),
                            DropdownMenuItem(
                              value: _TilemapDisplayMode.attributeView,
                              child: Text(l10n.tilemapDisplayModeAttributeView),
                            ),
                          ],
                          onChanged: (v) {
                            if (v == null) return;
                            setState(() => _displayMode = v);
                            setMenuState(() {});
                            _applyTextureRenderMode();
                          },
                        ),
                      ),
                    ],
                  ),
                ),
                CheckboxListTile(
                  dense: true,
                  contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                  title: Text(l10n.tilemapTileGrid),
                  value: _showTileGrid,
                  onChanged: (v) {
                    setState(() => _showTileGrid = v ?? false);
                    setMenuState(() {});
                  },
                ),
                CheckboxListTile(
                  dense: true,
                  contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                  title: Text(l10n.tilemapAttrGrid),
                  value: _showAttributeGrid,
                  onChanged: (v) {
                    setState(() => _showAttributeGrid = v ?? false);
                    setMenuState(() {});
                  },
                ),
                CheckboxListTile(
                  dense: true,
                  contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                  title: Text(l10n.tilemapAttrGrid32),
                  value: _showAttributeGrid32,
                  onChanged: (v) {
                    setState(() => _showAttributeGrid32 = v ?? false);
                    setMenuState(() {});
                  },
                ),
                CheckboxListTile(
                  dense: true,
                  contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                  title: Text(l10n.tilemapNtBounds),
                  value: _showNametableDelimiters,
                  onChanged: (v) {
                    setState(() => _showNametableDelimiters = v ?? false);
                    setMenuState(() {});
                  },
                ),
                CheckboxListTile(
                  dense: true,
                  contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                  title: Text(l10n.tilemapScrollOverlay),
                  value: _showScrollOverlay,
                  onChanged: (v) {
                    setState(() => _showScrollOverlay = v ?? false);
                    setMenuState(() {});
                  },
                ),
              ],
            ),
          ),
        ),
        const PopupMenuDivider(height: 1),
        // Capture section header
        PopupMenuItem<void>(
          onTap: null, // Disable interaction but keep text color
          height: 32,
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: Text(
            l10n.tilemapCapture,
            style: theme.textTheme.labelSmall?.copyWith(
              color: theme.colorScheme.primary,
              fontWeight: FontWeight.w600,
            ),
          ),
        ),
        PopupMenuItem<void>(
          onTap: () {}, // Empty tap to prevent closing
          padding: EdgeInsets.zero,
          child: StatefulBuilder(
            builder: (context, setMenuState) => RadioGroup<_TilemapCaptureMode>(
              groupValue: _captureMode,
              onChanged: (v) {
                if (v == null) return;
                setState(() => _captureMode = v);
                setMenuState(() {});
                _applyCaptureMode();
              },
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  RadioListTile<_TilemapCaptureMode>(
                    dense: true,
                    contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                    title: Text(l10n.tilemapCaptureFrameStart),
                    value: _TilemapCaptureMode.frameStart,
                  ),
                  RadioListTile<_TilemapCaptureMode>(
                    dense: true,
                    contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                    title: Text(l10n.tilemapCaptureVblankStart),
                    value: _TilemapCaptureMode.vblankStart,
                  ),
                  RadioListTile<_TilemapCaptureMode>(
                    dense: true,
                    contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                    title: Text(l10n.tilemapCaptureManual),
                    value: _TilemapCaptureMode.scanline,
                  ),
                  if (_captureMode == _TilemapCaptureMode.scanline) ...[
                    const PopupMenuDivider(height: 1),
                    PopupMenuItem<void>(
                      onTap: () {}, // Prevent closing
                      padding: EdgeInsets.zero,
                      child: Padding(
                        padding: const EdgeInsets.all(16),
                        child: _buildScanlineControlsForDialog(
                          theme,
                          l10n,
                          setMenuState,
                        ),
                      ),
                    ),
                  ],
                ],
              ),
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildScanlineControlsForDialog(
    ThemeData theme,
    AppLocalizations l10n,
    StateSetter setDialogState,
  ) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        // Scanline input
        Text('${l10n.tilemapScanline}:', style: theme.textTheme.bodySmall),
        const SizedBox(height: 4),
        TextField(
          controller: TextEditingController(text: _scanline.toString())
            ..selection = TextSelection.fromPosition(
              TextPosition(offset: _scanline.toString().length),
            ),
          decoration: InputDecoration(
            isDense: true,
            contentPadding: const EdgeInsets.symmetric(
              horizontal: 12,
              vertical: 8,
            ),
            border: const OutlineInputBorder(),
            hintText: '$_minScanline ~ $_maxScanline',
          ),
          keyboardType: TextInputType.number,
          onSubmitted: (v) {
            final value = int.tryParse(v);
            if (value != null &&
                value >= _minScanline &&
                value <= _maxScanline) {
              setState(() {
                _scanline = value;
                _scanlineController.text = _scanline.toString();
              });
              setDialogState(() {});
              _applyCaptureMode();
            }
          },
        ),
        const SizedBox(height: 12),
        // Dot input
        Text('${l10n.tilemapDot}:', style: theme.textTheme.bodySmall),
        const SizedBox(height: 4),
        TextField(
          controller: TextEditingController(text: _dot.toString())
            ..selection = TextSelection.fromPosition(
              TextPosition(offset: _dot.toString().length),
            ),
          decoration: InputDecoration(
            isDense: true,
            contentPadding: const EdgeInsets.symmetric(
              horizontal: 12,
              vertical: 8,
            ),
            border: const OutlineInputBorder(),
            hintText: '$_minDot ~ $_maxDot',
          ),
          keyboardType: TextInputType.number,
          onSubmitted: (v) {
            final value = int.tryParse(v);
            if (value != null && value >= _minDot && value <= _maxDot) {
              setState(() {
                _dot = value;
                _dotController.text = _dot.toString();
              });
              setDialogState(() {});
              _applyCaptureMode();
            }
          },
        ),
      ],
    );
  }

  Future<void> _applyCaptureMode() async {
    switch (_captureMode) {
      case _TilemapCaptureMode.frameStart:
        await bridge.setTilemapCaptureFrameStart();
      case _TilemapCaptureMode.vblankStart:
        await bridge.setTilemapCaptureVblankStart();
      case _TilemapCaptureMode.scanline:
        await bridge.setTilemapCaptureScanline(scanline: _scanline, dot: _dot);
    }
  }

  Future<void> _applyTextureRenderMode() async {
    final (showTileGrid, showAttributeGrid) = switch (_displayMode) {
      _TilemapDisplayMode.defaultMode => (false, false),
      _TilemapDisplayMode.grayscale => (true, false),
      _TilemapDisplayMode.attributeView => (false, true),
    };
    final mode = showAttributeGrid
        ? 2
        : showTileGrid
        ? 1
        : 0;
    await bridge.setTilemapDisplayMode(mode: mode);
  }

  Widget _displayModeDropdown(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final theme = Theme.of(context);

    return Row(
      children: [
        Expanded(
          child: Text(
            l10n.tilemapDisplayMode,
            style: theme.textTheme.bodySmall,
          ),
        ),
        DropdownButtonHideUnderline(
          child: DropdownButton<_TilemapDisplayMode>(
            isDense: true,
            value: _displayMode,
            items: [
              DropdownMenuItem(
                value: _TilemapDisplayMode.defaultMode,
                child: Text(l10n.tilemapDisplayModeDefault),
              ),
              DropdownMenuItem(
                value: _TilemapDisplayMode.grayscale,
                child: Text(l10n.tilemapDisplayModeGrayscale),
              ),
              DropdownMenuItem(
                value: _TilemapDisplayMode.attributeView,
                child: Text(l10n.tilemapDisplayModeAttributeView),
              ),
            ],
            onChanged: (v) {
              if (v == null) return;
              setState(() => _displayMode = v);
              _applyTextureRenderMode();
            },
          ),
        ),
      ],
    );
  }

  List<Rect> _scrollOverlayRectsFromSnapshot(bridge.TilemapSnapshot snap) {
    // Use the PPU `t` (temp) address for scroll origin. The `v` address
    // is advanced by background fetches/pipeline and is not stable for viewport math.
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

enum _TilemapCaptureMode { frameStart, vblankStart, scanline }

@immutable
class _TileCoord {
  const _TileCoord(this.x, this.y);

  final int x;
  final int y;

  @override
  bool operator ==(Object other) =>
      other is _TileCoord && other.x == x && other.y == y;

  @override
  int get hashCode => Object.hash(x, y);
}

@immutable
class _TileInfo {
  const _TileInfo({
    required this.tileX,
    required this.tileY,
    required this.ntIndex,
    required this.tileIndex,
    required this.tilemapAddress,
    required this.tileAddressPpu,
    required this.paletteIndex,
    required this.paletteAddress,
    required this.attrAddress,
    required this.attrByte,
  });

  final int tileX;
  final int tileY;
  final int ntIndex;
  final int tileIndex;
  final int tilemapAddress;
  final int tileAddressPpu;
  final int paletteIndex;
  final int paletteAddress;
  final int attrAddress;
  final int attrByte;
}

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

class _TilePreviewPainter extends CustomPainter {
  _TilePreviewPainter({required this.snapshot, required this.info});

  final bridge.TilemapSnapshot snapshot;
  final _TileInfo info;

  @override
  void paint(Canvas canvas, Size size) {
    final sx = size.width / 8;
    final sy = size.height / 8;
    final paint = Paint();

    final base = snapshot.bgPatternBase + info.tileIndex * 16;
    for (var py = 0; py < 8; py++) {
      final o = base + py;
      if (o + 8 >= snapshot.chr.length) break;
      final plane0 = snapshot.chr[o];
      final plane1 = snapshot.chr[o + 8];
      for (var px = 0; px < 8; px++) {
        final bit = 7 - px;
        final lo = (plane0 >> bit) & 1;
        final hi = (plane1 >> bit) & 1;
        final colorIndex = (hi << 1) | lo;
        final nesColor = _nesColorIndex(
          snapshot,
          paletteIndex: info.paletteIndex,
          colorIndex: colorIndex,
        );
        paint.color = _colorFromNes(snapshot.rgbaPalette, nesColor);
        canvas.drawRect(Rect.fromLTWH(px * sx, py * sy, sx, sy), paint);
      }
    }
  }

  int _nesColorIndex(
    bridge.TilemapSnapshot snap, {
    required int paletteIndex,
    required int colorIndex,
  }) {
    final pal = snap.palette;
    if (pal.isEmpty) return 0;
    if (colorIndex == 0) return pal[0] & 0x3F;
    final idx = paletteIndex * 4 + colorIndex;
    if (idx < 0 || idx >= pal.length) return 0;
    return pal[idx] & 0x3F;
  }

  Color _colorFromNes(Uint8List rgbaPalette, int nesColor) {
    final base = (nesColor & 0x3F) * 4;
    if (base + 3 >= rgbaPalette.length) return const Color(0xFF000000);
    final r = rgbaPalette[base];
    final g = rgbaPalette[base + 1];
    final b = rgbaPalette[base + 2];
    final a = rgbaPalette[base + 3];
    return Color.fromARGB(a, r, g, b);
  }

  @override
  bool shouldRepaint(_TilePreviewPainter oldDelegate) {
    return snapshot != oldDelegate.snapshot || info != oldDelegate.info;
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

/// CustomPainter for drawing grid overlays on tilemap texture.
/// Uses vector graphics for resolution-independent sharp rendering.
class _TilemapGridPainter extends CustomPainter {
  final bool showTileGrid;
  final bool showAttributeGrid;
  final bool showAttributeGrid32;
  final bool showNametableDelimiters;
  final bool showScrollOverlay;
  final List<Rect> scrollOverlayRects;
  final _TileCoord? hoveredTile;
  final _TileCoord? selectedTile;

  _TilemapGridPainter({
    required this.showTileGrid,
    required this.showAttributeGrid,
    required this.showAttributeGrid32,
    required this.showNametableDelimiters,
    required this.showScrollOverlay,
    required this.scrollOverlayRects,
    required this.hoveredTile,
    required this.selectedTile,
  });

  @override
  void paint(Canvas canvas, Size size) {
    // Tilemap is 512x480 (2x2 nametables of 256x240 each)
    const logicalWidth = 512.0;
    const logicalHeight = 480.0;

    // Calculate scaling factor
    final scaleX = size.width / logicalWidth;
    final scaleY = size.height / logicalHeight;

    // Draw 8×8 tile grid
    if (showTileGrid) {
      final paint = Paint()
        ..color = Colors.white.withValues(alpha: 0.5)
        ..strokeWidth =
            1.0 // Clear 1px lines
        ..style = PaintingStyle.stroke;

      for (var y = 0; y <= logicalHeight; y += 8) {
        final scaledY = y * scaleY;
        canvas.drawLine(Offset(0, scaledY), Offset(size.width, scaledY), paint);
      }
      for (var x = 0; x <= logicalWidth; x += 8) {
        final scaledX = x * scaleX;
        canvas.drawLine(
          Offset(scaledX, 0),
          Offset(scaledX, size.height),
          paint,
        );
      }
    }

    // Draw 16×16 attribute grid
    if (showAttributeGrid) {
      final paint = Paint()
        ..color = Colors.cyan
            .withValues(alpha: 0.7) // Increased visibility
        ..strokeWidth =
            0.8 // Increased from 0.6
        ..style = PaintingStyle.stroke;

      for (var y = 0; y <= logicalHeight; y += 16) {
        final scaledY = y * scaleY;
        canvas.drawLine(Offset(0, scaledY), Offset(size.width, scaledY), paint);
      }
      for (var x = 0; x <= logicalWidth; x += 16) {
        final scaledX = x * scaleX;
        canvas.drawLine(
          Offset(scaledX, 0),
          Offset(scaledX, size.height),
          paint,
        );
      }
    }

    // Draw 32×32 attribute grid (attribute bytes boundaries)
    if (showAttributeGrid32) {
      final paint = Paint()
        ..color = Colors.orange.withValues(alpha: 0.55)
        ..strokeWidth = 1.0
        ..style = PaintingStyle.stroke;

      for (var y = 0; y <= logicalHeight; y += 32) {
        final scaledY = y * scaleY;
        canvas.drawLine(Offset(0, scaledY), Offset(size.width, scaledY), paint);
      }
      for (var x = 0; x <= logicalWidth; x += 32) {
        final scaledX = x * scaleX;
        canvas.drawLine(
          Offset(scaledX, 0),
          Offset(scaledX, size.height),
          paint,
        );
      }
    }

    // Draw nametable delimiters (256×240 boundaries)
    if (showNametableDelimiters) {
      final paint = Paint()
        ..color = Colors.white.withValues(alpha: 0.9)
        ..strokeWidth =
            1.0 // Crisp single-pixel line
        ..style = PaintingStyle.stroke;

      // Vertical delimiter at x=256
      final verticalX = 256.0 * scaleX;
      canvas.drawLine(
        Offset(verticalX, 0),
        Offset(verticalX, size.height),
        paint,
      );

      // Horizontal delimiter at y=240
      final horizontalY = 240.0 * scaleY;
      canvas.drawLine(
        Offset(0, horizontalY),
        Offset(size.width, horizontalY),
        paint,
      );
    }

    if (showScrollOverlay && scrollOverlayRects.isNotEmpty) {
      final outline = Paint()
        ..color = Colors.pinkAccent.withValues(alpha: 0.9)
        ..strokeWidth = 2.0
        ..style = PaintingStyle.stroke;
      final fill = Paint()
        ..color = Colors.pinkAccent.withValues(alpha: 0.12)
        ..style = PaintingStyle.fill;

      for (final r in scrollOverlayRects) {
        final scaled = Rect.fromLTWH(
          r.left * scaleX,
          r.top * scaleY,
          r.width * scaleX,
          r.height * scaleY,
        );
        canvas.drawRect(scaled, fill);
        canvas.drawRect(scaled, outline);
      }
    }

    // Hovered tile outline
    if (hoveredTile != null) {
      final paint = Paint()
        ..color = Colors.white.withValues(alpha: 0.9)
        ..strokeWidth = 2.0
        ..style = PaintingStyle.stroke;

      final rect = Rect.fromLTWH(
        hoveredTile!.x * 8.0 * scaleX,
        hoveredTile!.y * 8.0 * scaleY,
        8.0 * scaleX,
        8.0 * scaleY,
      );
      canvas.drawRect(rect, paint);
    }

    // Selected tile outline
    if (selectedTile != null) {
      final paint = Paint()
        ..color = Colors.yellow.withValues(alpha: 0.95)
        ..strokeWidth = 2.5
        ..style = PaintingStyle.stroke;

      final rect = Rect.fromLTWH(
        selectedTile!.x * 8.0 * scaleX,
        selectedTile!.y * 8.0 * scaleY,
        8.0 * scaleX,
        8.0 * scaleY,
      );
      canvas.drawRect(rect, paint);
    }
  }

  @override
  bool shouldRepaint(_TilemapGridPainter oldDelegate) {
    return showTileGrid != oldDelegate.showTileGrid ||
        showAttributeGrid != oldDelegate.showAttributeGrid ||
        showAttributeGrid32 != oldDelegate.showAttributeGrid32 ||
        hoveredTile != oldDelegate.hoveredTile ||
        selectedTile != oldDelegate.selectedTile ||
        showNametableDelimiters != oldDelegate.showNametableDelimiters ||
        showScrollOverlay != oldDelegate.showScrollOverlay ||
        !_rectListEquals(scrollOverlayRects, oldDelegate.scrollOverlayRects);
  }

  bool _rectListEquals(List<Rect> a, List<Rect> b) {
    if (identical(a, b)) return true;
    if (a.length != b.length) return false;
    for (var i = 0; i < a.length; i++) {
      if (a[i] != b[i]) return false;
    }
    return true;
  }
}
