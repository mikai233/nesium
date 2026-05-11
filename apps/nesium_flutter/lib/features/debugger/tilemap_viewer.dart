import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/domain/aux_texture_ids.dart';
import 'package:nesium_flutter/domain/nes_controller.dart';
import 'package:nesium_flutter/domain/nes_texture_service.dart';
import 'package:nesium_flutter/features/debugger/tilemap/tilemap_canvas.dart';
import 'package:nesium_flutter/features/debugger/tilemap/tilemap_geometry.dart';
import 'package:nesium_flutter/features/debugger/tilemap/tilemap_models.dart';
import 'package:nesium_flutter/features/debugger/tilemap/tilemap_settings_menu.dart';
import 'package:nesium_flutter/features/debugger/tilemap/tilemap_side_panel.dart';
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

class _TilemapViewerState extends ConsumerState<TilemapViewer> {
  static const int _minScanline = -1;
  static const int _maxScanline = 260;
  static const int _minDot = 0;
  static const int _maxDot = 340;
  static const double _minScale = 1.0;
  static const double _maxScale = 5.0;

  final NesTextureService _textureService = NesTextureService();
  final TransformationController _transformationController =
      TransformationController();
  final ValueNotifier<List<Rect>> _scrollOverlayRects = ValueNotifier(const []);
  final GlobalKey _hoverTooltipKey = GlobalKey();

  int? _tilemapTextureId;
  int? _flutterTextureId;
  bool _isCreating = false;
  String? _error;
  StreamSubscription<bridge.TilemapSnapshot>? _tilemapSnapshotSub;
  bridge.TilemapSnapshot? _tilemapSnapshot;

  TilemapCaptureMode _captureMode = TilemapCaptureMode.vblankStart;
  int _scanline = 0;
  int _dot = 0;
  late final TextEditingController _scanlineController = TextEditingController(
    text: _scanline.toString(),
  );
  late final TextEditingController _dotController = TextEditingController(
    text: _dot.toString(),
  );

  bool _showTileGrid = false;
  bool _showAttributeGrid = false;
  bool _showAttributeGrid32 = false;
  bool _showNametableDelimiters = true;
  bool _showScrollOverlay = false;
  TilemapDisplayMode _displayMode = TilemapDisplayMode.defaultMode;

  TilemapCoord? _hoveredTile;
  TilemapCoord? _selectedTile;
  Offset? _hoverPosition;
  Offset? _selectedPosition;
  Size _lastHoverTooltipSize = const Size(320, 240);
  bool _hoverTooltipMeasurePending = false;
  bool _showSidePanel = true;
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
        width: tilemapLogicalWidth,
        height: tilemapLogicalHeight,
      );

      await _tilemapSnapshotSub?.cancel();
      _tilemapSnapshotSub = bridge.tilemapStateStream().listen(
        (snap) {
          if (!mounted) return;
          _tilemapSnapshot = snap;
          if (_showScrollOverlay) {
            _scrollOverlayRects.value = scrollOverlayRectsFromTilemapSnapshot(
              snap,
            );
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
    final tilemap = _buildTilemapView();

    if (isNativeDesktop) {
      return Stack(
        children: [
          Row(
            children: [
              Expanded(child: tilemap),
              _buildDesktopSidePanelWrapper(),
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
          Positioned(top: 12, right: 12, child: _buildSettingsButton()),
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

  Widget _buildTilemapView() {
    return TilemapCanvas(
      textureId: _flutterTextureId,
      snapshot: _tilemapSnapshot,
      transformationController: _transformationController,
      minScale: _minScale,
      maxScale: _maxScale,
      showTileGrid: _showTileGrid,
      showAttributeGrid: _showAttributeGrid,
      showAttributeGrid32: _showAttributeGrid32,
      showNametableDelimiters: _showNametableDelimiters,
      showScrollOverlay: _showScrollOverlay,
      scrollOverlayRects: _scrollOverlayRects,
      hoveredTile: _hoveredTile,
      selectedTile: _selectedTile,
      hoverPosition: _hoverPosition,
      selectedPosition: _selectedPosition,
      lastHoverTooltipSize: _lastHoverTooltipSize,
      hoverTooltipKey: _hoverTooltipKey,
      onHover: _handleHover,
      onTap: _handleTap,
      onHoverExit: _clearHover,
      onScheduleHoverTooltipMeasure: _scheduleHoverTooltipMeasure,
    );
  }

  Widget _buildDesktopSidePanelWrapper() {
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
            displayMode: _displayMode,
            onDisplayModeChanged: _setDisplayMode,
            showTileGrid: _showTileGrid,
            onShowTileGridChanged: (v) => setState(() => _showTileGrid = v),
            showAttributeGrid: _showAttributeGrid,
            onShowAttributeGridChanged: (v) =>
                setState(() => _showAttributeGrid = v),
            showAttributeGrid32: _showAttributeGrid32,
            onShowAttributeGrid32Changed: (v) =>
                setState(() => _showAttributeGrid32 = v),
            showNametableDelimiters: _showNametableDelimiters,
            onShowNametableDelimitersChanged: (v) =>
                setState(() => _showNametableDelimiters = v),
            showScrollOverlay: _showScrollOverlay,
            onShowScrollOverlayChanged: _setShowScrollOverlay,
            captureMode: _captureMode,
            onCaptureModeChanged: _setCaptureMode,
            scanlineController: _scanlineController,
            dotController: _dotController,
            minScanline: _minScanline,
            maxScanline: _maxScanline,
            minDot: _minDot,
            maxDot: _maxDot,
            onScanlineSubmitted: _submitScanline,
            onDotSubmitted: _submitDot,
          ),
        ),
      ),
    );
  }

  Widget _buildSettingsButton() {
    return TilemapSettingsButton(
      displayMode: _displayMode,
      onDisplayModeChanged: _setDisplayMode,
      showTileGrid: _showTileGrid,
      onShowTileGridChanged: (v) => setState(() => _showTileGrid = v),
      showAttributeGrid: _showAttributeGrid,
      onShowAttributeGridChanged: (v) => setState(() => _showAttributeGrid = v),
      showAttributeGrid32: _showAttributeGrid32,
      onShowAttributeGrid32Changed: (v) =>
          setState(() => _showAttributeGrid32 = v),
      showNametableDelimiters: _showNametableDelimiters,
      onShowNametableDelimitersChanged: (v) =>
          setState(() => _showNametableDelimiters = v),
      showScrollOverlay: _showScrollOverlay,
      onShowScrollOverlayChanged: _setShowScrollOverlay,
      captureMode: _captureMode,
      onCaptureModeChanged: _setCaptureMode,
      scanline: _scanline,
      dot: _dot,
      minScanline: _minScanline,
      maxScanline: _maxScanline,
      minDot: _minDot,
      maxDot: _maxDot,
      onScanlineSubmitted: _submitScanline,
      onDotSubmitted: _submitDot,
    );
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

  void _clearSelection() {
    if (_selectedTile != null) {
      setState(() {
        _selectedTile = null;
        _selectedPosition = null;
      });
    }
  }

  void _clearHover() {
    setState(() {
      _hoveredTile = null;
      _hoverPosition = null;
    });
  }

  void _handleHover(Offset position, Size size) {
    final tile = tilemapTileAtPosition(position, size);
    if (tile == _hoveredTile && position == _hoverPosition) return;
    setState(() {
      _hoveredTile = tile;
      _hoverPosition = position;
    });
  }

  void _handleTap(Offset position, Size size) {
    final tile = tilemapTileAtPosition(position, size);
    setState(() {
      _selectedTile = tile;
      _selectedPosition = position;
    });
  }

  void _scheduleHoverTooltipMeasure() {
    if (_hoverTooltipMeasurePending) return;
    _hoverTooltipMeasurePending = true;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _hoverTooltipMeasurePending = false;
      if (!mounted) return;
      final size = _hoverTooltipKey.currentContext?.size;
      if (size == null || size == Size.zero || size == _lastHoverTooltipSize) {
        return;
      }
      setState(() => _lastHoverTooltipSize = size);
    });
  }

  void _setDisplayMode(TilemapDisplayMode value) {
    setState(() => _displayMode = value);
    unawaitedLogged(
      _applyTextureRenderMode(),
      message: 'Failed to set tilemap display mode',
    );
  }

  void _setCaptureMode(TilemapCaptureMode value) {
    setState(() => _captureMode = value);
    unawaitedLogged(
      _applyCaptureMode(),
      message: 'Failed to set tilemap capture point',
    );
  }

  void _setShowScrollOverlay(bool value) {
    setState(() => _showScrollOverlay = value);
    final snap = _tilemapSnapshot;
    if (value && snap != null) {
      _scrollOverlayRects.value = scrollOverlayRectsFromTilemapSnapshot(snap);
    } else if (!value) {
      _scrollOverlayRects.value = const [];
    }
  }

  void _submitScanline(String value) {
    final parsed = int.tryParse(value);
    if (parsed == null || parsed < _minScanline || parsed > _maxScanline) {
      return;
    }
    setState(() => _scanline = parsed);
    _scanlineController.text = _scanline.toString();
    unawaitedLogged(
      _applyCaptureMode(),
      message: 'Failed to set tilemap capture point',
    );
  }

  void _submitDot(String value) {
    final parsed = int.tryParse(value);
    if (parsed == null || parsed < _minDot || parsed > _maxDot) return;
    setState(() => _dot = parsed);
    _dotController.text = _dot.toString();
    unawaitedLogged(
      _applyCaptureMode(),
      message: 'Failed to set tilemap capture point',
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
}
