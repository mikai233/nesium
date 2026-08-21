import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/domain/aux_texture_ids.dart';
import 'package:nesium_flutter/domain/nes_controller.dart';
import 'package:nesium_flutter/domain/nes_texture_service.dart';
import 'package:nesium_flutter/features/debugger/tile/tile_info_widgets.dart';
import 'package:nesium_flutter/features/debugger/tile/tile_viewer_canvas.dart';
import 'package:nesium_flutter/features/debugger/tile/tile_viewer_geometry.dart';
import 'package:nesium_flutter/features/debugger/tile/tile_viewer_models.dart';
import 'package:nesium_flutter/features/debugger/tile/tile_viewer_settings_menu.dart';
import 'package:nesium_flutter/features/debugger/tile/tile_viewer_side_panel.dart';
import 'package:nesium_flutter/features/debugger/viewer_skeletonizer.dart';
import 'package:nesium_flutter/l10n/app_localizations.dart';
import 'package:nesium_flutter/logging/app_logger.dart';
import 'package:nesium_flutter/platform/platform_capabilities.dart';

/// Tile Viewer that displays NES CHR pattern tables via a Flutter Texture.
class TileViewer extends ConsumerStatefulWidget {
  const TileViewer({super.key});

  @override
  ConsumerState<TileViewer> createState() => _TileViewerState();
}

class _TileViewerState extends ConsumerState<TileViewer> {
  static const int _minScanline = -1;
  static const int _maxScanline = 260;
  static const int _minDot = 0;
  static const int _maxDot = 340;
  static const double _minScale = 1.0;
  static const double _maxScale = 8.0;

  final NesTextureService _textureService = NesTextureService();
  final TransformationController _transformationController =
      TransformationController();

  int? _chrTextureId;
  int? _flutterTextureId;
  bool _isCreating = false;
  String? _error;
  StreamSubscription<bridge.TileSnapshot>? _tileSnapshotSub;
  bridge.TileSnapshot? _tileSnapshot;

  bool _showTileGrid = true;
  int _selectedPalette = 0;
  bool _useGrayscale = false;
  bool _showSidePanel = true;

  TileCaptureMode _captureMode = TileCaptureMode.vblankStart;
  int _scanline = 0;
  int _dot = 0;
  late final TextEditingController _scanlineController = TextEditingController(
    text: _scanline.toString(),
  );
  late final TextEditingController _dotController = TextEditingController(
    text: _dot.toString(),
  );

  TilePreset? _selectedPreset = TilePreset.ppu;
  TileSource _source = TileSource.ppu;
  int _startAddress = 0;
  int _columnCount = 16;
  int _rowCount = 32;
  TileLayout _layout = TileLayout.normal;
  TileBackground _background = TileBackground.defaultBg;

  TileCoord? _hoveredTile;
  TileCoord? _selectedTile;
  Offset? _hoverPosition;
  bool _isCanvasTransformed = false;

  int get _textureWidth => _columnCount * 8;
  int get _textureHeight => _rowCount * 8;
  int get _maxAddress => (_tileSnapshot?.sourceSize ?? 0x2000) - 1;
  int get _addressIncrement => _columnCount * _rowCount * 16;

  @override
  void initState() {
    super.initState();
    _transformationController.addListener(_onTransformChanged);
    _createTexture();
  }

  void _onRomEjected() {
    unawaitedLogged(
      _applyPreset(TilePreset.ppu),
      message: 'Failed to apply PPU tile preset',
    );
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

  Future<void> _updateSize({int? columns, int? rows}) async {
    final newColumns = columns ?? _columnCount;
    final newRows = rows ?? _rowCount;
    final adjustedColumns = _layout == TileLayout.normal
        ? newColumns
        : (newColumns ~/ 2) * 2;
    final adjustedRows = _layout == TileLayout.normal
        ? newRows
        : (newRows ~/ 2) * 2;

    if (adjustedColumns == _columnCount && adjustedRows == _rowCount) return;

    setState(() {
      _columnCount = adjustedColumns;
      _rowCount = adjustedRows;
    });

    await bridge.setTileViewerSize(
      columns: adjustedColumns,
      rows: adjustedRows,
    );
    await _recreateTexture();
  }

  Future<void> _recreateTexture() async {
    final ids = await AuxTextureIdsCache.get();
    _chrTextureId ??= ids.tile;

    await _textureService.disposeAuxTexture(_chrTextureId!);
    final textureId = await _textureService.createAuxTexture(
      id: _chrTextureId!,
      width: _textureWidth,
      height: _textureHeight,
    );
    if (mounted) {
      setState(() => _flutterTextureId = textureId);
    }
  }

  Future<void> _createTexture() async {
    if (_isCreating) return;
    setState(() {
      _isCreating = true;
      _error = null;
    });

    try {
      final ids = await AuxTextureIdsCache.get();
      _chrTextureId ??= ids.tile;

      final textureId = await _textureService.createAuxTexture(
        id: _chrTextureId!,
        width: _textureWidth,
        height: _textureHeight,
      );

      await _tileSnapshotSub?.cancel();
      _tileSnapshotSub = bridge.tileStateStream().listen(
        (snap) {
          if (!mounted) return;
          _tileSnapshot = snap;
        },
        onError: (e, st) {
          logError(
            e,
            stackTrace: st,
            message: 'Tile state stream error',
            logger: 'tile_viewer',
          );
        },
      );

      unawaitedLogged(
        _applyCaptureMode(),
        message: 'Failed to set CHR capture point',
      );
      await bridge.setTileViewerPalette(paletteIndex: _selectedPalette);

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
    final textureId = _chrTextureId;
    if (textureId != null) {
      _textureService.pauseAuxTexture(textureId);
    }
    unawaited(_tileSnapshotSub?.cancel());
    bridge.unsubscribeTileState();
    if (textureId != null) {
      _textureService.disposeAuxTexture(textureId);
    }
    _scanlineController.dispose();
    _dotController.dispose();
    _transformationController.removeListener(_onTransformChanged);
    _transformationController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    ref.listen(nesControllerProvider, (prev, next) {
      final prevHasRom = prev?.romHash != null;
      final nextHasRom = next.romHash != null;
      if (prevHasRom && !nextHasRom) {
        _onRomEjected();
      }
    });

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
          Text(l10n.tileViewerError(_error ?? '')),
          const SizedBox(height: 16),
          FilledButton.tonal(
            onPressed: _createTexture,
            child: Text(l10n.tileViewerRetry),
          ),
        ],
      ),
    );
  }

  Widget _buildMainLayout(BuildContext context) {
    final tileView = _buildTileView();

    if (isNativeDesktop) {
      return LayoutBuilder(
        builder: (context, constraints) {
          final size = constraints.biggest;
          return Stack(
            children: [
              Row(
                children: [
                  Expanded(child: tileView),
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
              if (_hoveredTile != null &&
                  _hoverPosition != null &&
                  _tileSnapshot != null)
                TileHoverTooltip(
                  tile: _hoveredTile!,
                  snapshot: _tileSnapshot!,
                  position: _hoverPosition!,
                  viewportSize: size,
                ),
            ],
          );
        },
      );
    }

    return Stack(
      children: [
        tileView,
        Positioned(top: 12, right: 12, child: _buildSettingsButton()),
        if (_isCanvasTransformed)
          Positioned(
            bottom: 12,
            left: 12,
            child: _buildResetZoomButton(context),
          ),
      ],
    );
  }

  Widget _buildTileView() {
    return TileViewerCanvas(
      textureId: _flutterTextureId,
      textureWidth: _textureWidth,
      textureHeight: _textureHeight,
      transformationController: _transformationController,
      minScale: _minScale,
      maxScale: _maxScale,
      showTileGrid: _showTileGrid,
      hoveredTile: _hoveredTile,
      selectedTile: _selectedTile,
      onHover: _handleHover,
      onTap: _handleTap,
      onHoverExit: _clearHover,
    );
  }

  Widget _buildDesktopSidePanelWrapper() {
    const panelWidth = 240.0;
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
          child: TileViewerSidePanel(
            snapshot: _tileSnapshot,
            selectedTile: _selectedTile,
            selectedPreset: _selectedPreset,
            onPresetSelected: _setPreset,
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
            source: _source,
            onSourceChanged: _setSource,
            startAddress: _startAddress,
            maxAddress: _maxAddress,
            addressIncrement: _addressIncrement,
            onStartAddressChanged: _setStartAddress,
            columnCount: _columnCount,
            rowCount: _rowCount,
            layout: _layout,
            onColumnsChanged: (v) => unawaitedLogged(_updateSize(columns: v)),
            onRowsChanged: (v) => unawaitedLogged(_updateSize(rows: v)),
            onLayoutChanged: _setLayout,
            background: _background,
            onBackgroundChanged: _setBackground,
            showTileGrid: _showTileGrid,
            onShowTileGridChanged: (v) => setState(() => _showTileGrid = v),
            useGrayscale: _useGrayscale,
            onUseGrayscaleChanged: _setUseGrayscale,
            selectedPalette: _selectedPalette,
            onPaletteChanged: _setPalette,
          ),
        ),
      ),
    );
  }

  Widget _buildSettingsButton() {
    return TileViewerSettingsButton(
      selectedPreset: _selectedPreset,
      onPresetSelected: _setPreset,
      showTileGrid: _showTileGrid,
      onShowTileGridChanged: (v) => setState(() => _showTileGrid = v),
      useGrayscale: _useGrayscale,
      onUseGrayscaleChanged: _setUseGrayscale,
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
      selectedPalette: _selectedPalette,
      onPaletteChanged: _setPalette,
    );
  }

  Widget _buildResetZoomButton(BuildContext context) {
    final theme = Theme.of(context);
    final l10n = AppLocalizations.of(context)!;

    return AnimatedOpacity(
      opacity: _isCanvasTransformed ? 1.0 : 0.0,
      duration: const Duration(milliseconds: 200),
      child: IgnorePointer(
        ignoring: !_isCanvasTransformed,
        child: Material(
          color: theme.colorScheme.surfaceContainer,
          borderRadius: BorderRadius.circular(8),
          elevation: 2,
          child: InkWell(
            onTap: _resetCanvasTransform,
            borderRadius: BorderRadius.circular(8),
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  Icon(
                    Icons.zoom_out_map,
                    size: 20,
                    color: theme.colorScheme.onSurface,
                  ),
                  const SizedBox(width: 8),
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

  void _handleHover({
    required Offset position,
    required Offset contentOffset,
    required Size contentSize,
  }) {
    final childPos = _transformToContent(position);
    final contentPos = childPos - contentOffset;
    final tile = tileAtPosition(contentPos, contentSize);
    if (tile == _hoveredTile && position == _hoverPosition) return;
    setState(() {
      _hoveredTile = tile;
      _hoverPosition = position;
    });
  }

  void _clearHover() {
    if (_hoveredTile == null) return;
    setState(() {
      _hoveredTile = null;
      _hoverPosition = null;
    });
  }

  void _handleTap({
    required Offset position,
    required Offset contentOffset,
    required Size contentSize,
  }) {
    final childPos = _transformToContent(position);
    final contentPos = childPos - contentOffset;
    final tile = tileAtPosition(contentPos, contentSize);
    setState(() => _selectedTile = tile);
  }

  Offset _transformToContent(Offset screenPos) {
    final matrix = _transformationController.value;
    final inverted = Matrix4.inverted(matrix);
    return MatrixUtils.transformPoint(inverted, screenPos);
  }

  Future<void> _applyCaptureMode() async {
    switch (_captureMode) {
      case TileCaptureMode.frameStart:
        await bridge.setTileViewerCaptureFrameStart();
      case TileCaptureMode.vblankStart:
        await bridge.setTileViewerCaptureVblankStart();
      case TileCaptureMode.scanline:
        await bridge.setTileViewerCaptureScanline(
          scanline: _scanline,
          dot: _dot,
        );
    }
  }

  void _setCaptureMode(TileCaptureMode value) {
    setState(() => _captureMode = value);
    unawaitedLogged(
      _applyCaptureMode(),
      message: 'Failed to set CHR capture point',
    );
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
      message: 'Failed to set CHR capture point',
    );
  }

  void _submitDot(String value) {
    final parsed = int.tryParse(value);
    if (parsed == null || parsed < _minDot || parsed > _maxDot) return;
    setState(() => _dot = parsed);
    _dotController.text = _dot.toString();
    unawaitedLogged(
      _applyCaptureMode(),
      message: 'Failed to set CHR capture point',
    );
  }

  void _setPreset(TilePreset preset) {
    unawaitedLogged(
      _applyPreset(preset),
      message: 'Failed to apply tile viewer preset',
    );
  }

  Future<void> _applyPreset(TilePreset preset) async {
    final snapshot = _tileSnapshot;
    late final TileSource newSource;
    late final int newAddress;
    late final int newColumns;
    late final int newRows;
    late final TileLayout newLayout;
    int? newPalette;

    switch (preset) {
      case TilePreset.ppu:
        newSource = TileSource.ppu;
        newAddress = 0;
        newColumns = 16;
        newRows = 32;
        newLayout = TileLayout.normal;
        newPalette = null;
      case TilePreset.chr:
        newSource = TileSource.chrRom;
        newAddress = 0;
        newColumns = 16;
        newRows = 32;
        newLayout = TileLayout.normal;
        newPalette = null;
      case TilePreset.rom:
        newSource = TileSource.prgRom;
        newAddress = 0;
        newColumns = 16;
        newRows = 32;
        newLayout = TileLayout.normal;
        newPalette = null;
      case TilePreset.bg:
        newSource = TileSource.ppu;
        newAddress = snapshot?.bgPatternBase ?? 0;
        newColumns = 16;
        newRows = 16;
        newLayout = TileLayout.normal;
        newPalette = _selectedPalette >= 4 ? 0 : _selectedPalette;
      case TilePreset.oam:
        newSource = TileSource.ppu;
        final largeSprites = snapshot?.largeSprites ?? false;
        if (largeSprites) {
          newAddress = 0;
          newColumns = 16;
          newRows = 32;
          newLayout = TileLayout.singleLine8x16;
        } else {
          newAddress = snapshot?.spritePatternBase ?? 0;
          newColumns = 16;
          newRows = 16;
          newLayout = TileLayout.normal;
        }
        newPalette = _selectedPalette < 4 ? 4 : _selectedPalette;
    }

    final needsTextureRecreate =
        newColumns != _columnCount || newRows != _rowCount;

    setState(() {
      _selectedPreset = preset;
      _source = newSource;
      _startAddress = newAddress;
      _columnCount = newColumns;
      _rowCount = newRows;
      _layout = newLayout;
      if (newPalette != null) {
        _selectedPalette = newPalette;
      }
    });

    await bridge.setTileViewerSource(source: newSource.index);
    await bridge.setTileViewerStartAddress(startAddress: newAddress);
    await bridge.setTileViewerSize(columns: newColumns, rows: newRows);
    await bridge.setTileViewerLayout(layout: newLayout.index);
    if (newPalette != null) {
      await bridge.setTileViewerPalette(paletteIndex: newPalette);
    }

    if (needsTextureRecreate) {
      await _recreateTexture();
    }
  }

  void _clearPresetSelection() {
    if (_selectedPreset != null) {
      setState(() => _selectedPreset = null);
    }
  }

  void _setSource(TileSource value) {
    setState(() => _source = value);
    unawaitedLogged(
      bridge.setTileViewerSource(source: value.index),
      message: 'Failed to set tile viewer source',
    );
  }

  void _setStartAddress(int value) {
    setState(() => _startAddress = value);
    unawaitedLogged(
      bridge.setTileViewerStartAddress(startAddress: value),
      message: 'Failed to set tile viewer start address',
    );
  }

  void _setLayout(TileLayout value) {
    setState(() => _layout = value);
    unawaitedLogged(
      bridge.setTileViewerLayout(layout: value.index),
      message: 'Failed to set tile viewer layout',
    );
  }

  void _setBackground(TileBackground value) {
    setState(() => _background = value);
    unawaitedLogged(
      bridge.setTileViewerBackground(background: value.index),
      message: 'Failed to set tile viewer background',
    );
  }

  void _setUseGrayscale(bool enabled) {
    setState(() => _useGrayscale = enabled);
    unawaitedLogged(
      bridge.setTileViewerDisplayMode(mode: enabled ? 1 : 0),
      message: 'Failed to set tile viewer display mode',
    );
  }

  void _setPalette(int value) {
    setState(() => _selectedPalette = value);
    _clearPresetSelection();
    unawaitedLogged(
      bridge.setTileViewerPalette(paletteIndex: value),
      message: 'Failed to set tile viewer palette',
    );
  }
}
