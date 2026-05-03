part of '../tilemap_viewer.dart';

abstract class _TilemapViewerStateBase extends ConsumerState<TilemapViewer> {
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
      _tilemapSnapshotSub = bridge.tilemapStateStream().listen(
        (snap) {
          if (!mounted) return;
          // Store snapshot for tooltip data access.
          _tilemapSnapshot = snap;
          // Update scroll overlay rects via ValueNotifier (isolated repaint).
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
