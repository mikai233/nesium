part of '../tile_viewer.dart';

abstract class _TileViewerStateBase extends ConsumerState<TileViewer> {
  static const int _minScanline = -1;
  static const int _maxScanline = 260;
  static const int _minDot = 0;
  static const int _maxDot = 340;

  final NesTextureService _textureService = NesTextureService();
  int? _chrTextureId;
  int? _flutterTextureId;
  bool _isCreating = false;
  String? _error;
  StreamSubscription<bridge.TileSnapshot>? _tileSnapshotSub;

  // Display options
  bool _showTileGrid = true;
  int _selectedPalette = 0; // 0-7: 0-3 BG, 4-7 Sprite
  bool _useGrayscale = false;
  bool _showSidePanel = true; // Desktop side panel visibility

  // Capture mode state
  _CaptureMode _captureMode = _CaptureMode.vblankStart;
  int _scanline = 0;
  int _dot = 0;
  late final TextEditingController _scanlineController = TextEditingController(
    text: _scanline.toString(),
  );
  late final TextEditingController _dotController = TextEditingController(
    text: _dot.toString(),
  );

  // Selected preset (null = no preset selected, manual config)
  _Preset? _selectedPreset = _Preset.ppu;

  // Configurable tile viewer options (synced with TileSnapshot)
  _TileSource _source = _TileSource.ppu;
  int _startAddress = 0;
  int _columnCount = 16;
  int _rowCount = 32;
  _TileLayout _layout = _TileLayout.normal;
  _TileBackground _background = _TileBackground.defaultBg;

  // Dynamic texture dimensions
  int get _textureWidth => _columnCount * 8;
  int get _textureHeight => _rowCount * 8;

  // Max address for current source
  int get _maxAddress => (_tileSnapshot?.sourceSize ?? 0x2000) - 1;

  // Address increment for page navigation
  int get _addressIncrement => _columnCount * _rowCount * 16; // 16 bytes/tile

  // Hover/selection for tile info tooltip
  _TileCoord? _hoveredTile;
  _TileCoord? _selectedTile;
  Offset? _hoverPosition; // For floating tooltip positioning
  bridge.TileSnapshot? _tileSnapshot; // Current snapshot for tile preview

  // Zoom and pan state
  final TransformationController _transformationController =
      TransformationController();
  static const double _minScale = 1.0;
  static const double _maxScale = 8.0;
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

  /// Updates tile viewer size (columns and/or rows) and recreates texture if needed
  Future<void> _updateSize({int? columns, int? rows}) async {
    final newColumns = columns ?? _columnCount;
    final newRows = rows ?? _rowCount;

    // Apply restrictions for non-normal layouts
    final adjustedColumns = _layout == _TileLayout.normal
        ? newColumns
        : (newColumns ~/ 2) * 2; // Force even for 8x16/16x16
    final adjustedRows = _layout == _TileLayout.normal
        ? newRows
        : (newRows ~/ 2) * 2; // Force even for 8x16/16x16

    if (adjustedColumns == _columnCount && adjustedRows == _rowCount) return;

    setState(() {
      _columnCount = adjustedColumns;
      _rowCount = adjustedRows;
    });

    // Notify backend and recreate texture
    await bridge.setTileViewerSize(
      columns: adjustedColumns,
      rows: adjustedRows,
    );
    await _recreateTexture();
  }

  /// Recreates the texture with current dimensions
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
          // Store snapshot for tooltip data WITHOUT triggering rebuild.
          // Texture updates automatically; setState spam kills performance.
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

  Future<void> _applyCaptureMode() async {
    switch (_captureMode) {
      case _CaptureMode.frameStart:
        await bridge.setTileViewerCaptureFrameStart();
      case _CaptureMode.vblankStart:
        await bridge.setTileViewerCaptureVblankStart();
      case _CaptureMode.scanline:
        await bridge.setTileViewerCaptureScanline(
          scanline: _scanline,
          dot: _dot,
        );
    }
  }
}
