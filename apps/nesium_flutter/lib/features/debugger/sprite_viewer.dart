import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/domain/nes_texture_service.dart';
import 'package:nesium_flutter/l10n/app_localizations.dart';
import 'package:nesium_flutter/logging/app_logger.dart';
import 'package:nesium_flutter/platform/platform_capabilities.dart';

/// Sprite Viewer - displays 64 sprite thumbnails using an auxiliary texture.
class SpriteViewer extends ConsumerStatefulWidget {
  const SpriteViewer({super.key});

  @override
  ConsumerState<SpriteViewer> createState() => _SpriteViewerState();
}

class _SpriteViewerState extends ConsumerState<SpriteViewer> {
  static const int _spriteTextureId = 3;
  static const int _spriteScreenTextureId = 4;
  static const int _gridCols = 8;
  static const int _gridRows = 8;
  static const int _screenWidth = 256;
  static const int _screenHeight = 240;
  // Mesen2-style NES "offscreen region": only 16px below the 256x240 visible area.
  static const int _offscreenBottomHeight = 16;
  static const int _previewWidth = _screenWidth;
  static const int _previewHeight = _screenHeight + _offscreenBottomHeight;

  final NesTextureService _textureService = NesTextureService();
  StreamSubscription<bridge.SpriteSnapshot>? _subscription;
  bridge.SpriteSnapshot? _snapshot;
  int? _selectedIndex;
  int? _gridHoveredIndex;
  int? _previewHoveredIndex;
  bool _hasReceivedData = false;
  Offset? _gridHoverPosition;
  Offset? _previewHoverPosition;
  Offset? _previewSelectedPosition;

  // Overlay tooltip
  OverlayEntry? _tooltipOverlay;

  int? _flutterThumbTextureId;
  int? _flutterScreenTextureId;
  int _thumbTextureWidth = 0;
  int _thumbTextureHeight = 0;
  int _screenTextureWidth = 0;
  int _screenTextureHeight = 0;
  bool _isCreating = false;
  String? _error;

  bool _showGrid = true;
  bool _dimOffscreenGrid = true;
  bool _showOutline = false;
  bool _showOffscreenRegions = false;
  bool _showSidePanel = true;
  bool _showListView = false;
  _SpriteBackground _background = _SpriteBackground.gray;
  _SpriteDataSource _dataSource = _SpriteDataSource.spriteRam;

  // Zoom and pan state
  final TransformationController _previewTransformationController =
      TransformationController();
  final ScrollController _sidePanelScrollController = ScrollController();
  static const double _maxScale = 12.0;
  bool _isCanvasTransformed = false;
  Matrix4 _previewDefaultTransform = Matrix4.identity();
  Size _previewDefaultViewportSize = Size.zero;
  int _previewDefaultDisplayW = 0;
  int _previewDefaultDisplayH = 0;
  bool _previewDefaultShowOffscreenRegions = false;

  @override
  void initState() {
    super.initState();
    _previewTransformationController.addListener(_onTransformChanged);
    _startStreaming();
  }

  void _onTransformChanged() {
    final matrix = _previewTransformationController.value;
    final isTransformed = !_matrixNear(matrix, _previewDefaultTransform);
    if (_isCanvasTransformed != isTransformed) {
      setState(() => _isCanvasTransformed = isTransformed);
    }
  }

  void _resetCanvasTransform() {
    _previewTransformationController.value = Matrix4.copy(
      _previewDefaultTransform,
    );
  }

  @override
  void dispose() {
    _textureService.pauseAuxTexture(_spriteTextureId);
    _textureService.pauseAuxTexture(_spriteScreenTextureId);
    unawaited(_subscription?.cancel());
    unawaited(_unsubscribe());
    _textureService.disposeAuxTexture(_spriteTextureId);
    _textureService.disposeAuxTexture(_spriteScreenTextureId);
    _previewTransformationController.removeListener(_onTransformChanged);
    _previewTransformationController.dispose();
    _sidePanelScrollController.dispose();
    _removeTooltipOverlay();
    super.dispose();
  }

  Future<void> _startStreaming() async {
    final stream = bridge.spriteStateStream();
    _subscription = stream.listen(
      (snapshot) {
        if (mounted) {
          setState(() {
            _snapshot = snapshot;
            _hasReceivedData = true;
          });
          unawaitedLogged(
            _ensureThumbTexture(snapshot),
            message: 'Failed to create sprite aux texture',
          );
          unawaitedLogged(
            _ensureScreenTexture(),
            message: 'Failed to create sprite screen texture',
          );
        }
      },
      onError: (e) {
        // Stream error - ignore
      },
    );
  }

  Future<void> _unsubscribe() async {
    try {
      await bridge.unsubscribeSpriteState();
    } catch (_) {}
  }

  int _gridTextureWidth(bridge.SpriteSnapshot snapshot) =>
      _gridCols * snapshot.thumbnailWidth;

  int _gridTextureHeight(bridge.SpriteSnapshot snapshot) =>
      _gridRows * snapshot.thumbnailHeight;

  Future<void> _ensureThumbTexture(bridge.SpriteSnapshot snapshot) async {
    final w = _gridTextureWidth(snapshot);
    final h = _gridTextureHeight(snapshot);
    if (w <= 0 || h <= 0) return;

    // Nothing to do if current texture matches.
    if (_flutterThumbTextureId != null &&
        _thumbTextureWidth == w &&
        _thumbTextureHeight == h) {
      return;
    }
    if (_isCreating) return;

    setState(() {
      _isCreating = true;
      _error = null;
    });

    try {
      // Avoid updating a texture while it is being recreated.
      await _textureService.pauseAuxTexture(_spriteTextureId);
      await _textureService.disposeAuxTexture(_spriteTextureId);

      final textureId = await _textureService.createAuxTexture(
        id: _spriteTextureId,
        width: w,
        height: h,
      );
      if (textureId == null) {
        throw StateError('createAuxTexture returned null');
      }

      if (!mounted) return;
      setState(() {
        _flutterThumbTextureId = textureId;
        _thumbTextureWidth = w;
        _thumbTextureHeight = h;
        _isCreating = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _error = e.toString();
        _isCreating = false;
      });
    }
  }

  Future<void> _ensureScreenTexture() async {
    final desiredW = _previewWidth;
    final desiredH = _previewHeight;

    if (_flutterScreenTextureId != null &&
        _screenTextureWidth == desiredW &&
        _screenTextureHeight == desiredH) {
      return;
    }
    if (_isCreating) return;

    setState(() {
      _isCreating = true;
      _error = null;
    });

    try {
      await _textureService.pauseAuxTexture(_spriteScreenTextureId);
      await _textureService.disposeAuxTexture(_spriteScreenTextureId);

      final textureId = await _textureService.createAuxTexture(
        id: _spriteScreenTextureId,
        width: desiredW,
        height: desiredH,
      );
      if (textureId == null) {
        throw StateError('createAuxTexture returned null');
      }

      if (!mounted) return;
      setState(() {
        _flutterScreenTextureId = textureId;
        _screenTextureWidth = desiredW;
        _screenTextureHeight = desiredH;
        _isCreating = false;
      });
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _error = e.toString();
        _isCreating = false;
      });
    }
  }

  Future<void> _retry() async {
    await _subscription?.cancel();
    await _unsubscribe();
    if (!mounted) return;
    setState(() {
      _error = null;
      _snapshot = null;
      _hasReceivedData = false;
      _flutterThumbTextureId = null;
      _flutterScreenTextureId = null;
      _thumbTextureWidth = 0;
      _thumbTextureHeight = 0;
      _screenTextureWidth = 0;
      _screenTextureHeight = 0;
    });
    await _startStreaming();
  }

  @override
  Widget build(BuildContext context) {
    final snapshot = _snapshot;

    if (_error != null) return _buildErrorState(context);
    if (!_hasReceivedData ||
        snapshot == null ||
        _flutterThumbTextureId == null ||
        _flutterScreenTextureId == null) {
      return const Center(child: CircularProgressIndicator());
    }

    return _buildMainLayout(context, snapshot);
  }

  Widget _buildErrorState(BuildContext context) {
    final theme = Theme.of(context);
    final l10n = AppLocalizations.of(context)!;

    return Center(
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          const Icon(Icons.error_outline, size: 48, color: Colors.red),
          const SizedBox(height: 16),
          Text(l10n.spriteViewerError(_error ?? '')),
          const SizedBox(height: 16),
          FilledButton.tonal(
            onPressed: _retry,
            child: Text(
              l10n.tileViewerRetry,
              style: theme.textTheme.labelLarge,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildMainLayout(
    BuildContext context,
    bridge.SpriteSnapshot snapshot,
  ) {
    final grid = _buildThumbnailGrid(context, snapshot);
    final preview = _buildScreenPreview(context, snapshot);

    if (isNativeDesktop) {
      return Column(
        children: [
          Expanded(
            child: Stack(
              children: [
                Row(
                  children: [
                    Expanded(child: preview),
                    _buildDesktopSidePanelWrapper(context, snapshot, grid),
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
            ),
          ),
          if (_showListView) _buildSpriteListView(context, snapshot),
        ],
      );
    }

    // Mobile layout
    return GestureDetector(
      onTap: _clearSelection,
      behavior: HitTestBehavior.translucent,
      child: Stack(
        children: [
          Column(
            children: [
              Expanded(child: preview),
              SizedBox(height: 220, child: grid),
            ],
          ),
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
    if (_selectedIndex == null) return;
    setState(() {
      _selectedIndex = null;
      _previewSelectedPosition = null;
    });
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
          color: theme.colorScheme.surfaceContainerHighest.withValues(
            alpha: 0.9,
          ),
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
      tooltip: l10n.spriteViewerSettingsTooltip,
      onPressed: () => _showSettingsMenu(context),
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

  void _showSettingsMenu(BuildContext context) {
    final theme = Theme.of(context);
    final l10n = AppLocalizations.of(context)!;

    final RenderBox button = context.findRenderObject() as RenderBox;
    final RenderBox overlay =
        Overlay.of(context).context.findRenderObject() as RenderBox;
    final buttonPosition = button.localToGlobal(Offset.zero, ancestor: overlay);

    showMenu<void>(
      context: context,
      position: RelativeRect.fromLTRB(
        buttonPosition.dx + button.size.width - 280,
        buttonPosition.dy + button.size.height + 4,
        overlay.size.width - buttonPosition.dx - button.size.width,
        0,
      ),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
      items: [
        PopupMenuItem<void>(
          enabled: false,
          height: 32,
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: Text(
            l10n.tilemapPanelDisplay,
            style: theme.textTheme.labelSmall?.copyWith(
              color: theme.colorScheme.onSurfaceVariant,
              fontWeight: FontWeight.w600,
            ),
          ),
        ),
        PopupMenuItem<void>(
          onTap: () {},
          padding: EdgeInsets.zero,
          child: StatefulBuilder(
            builder: (context, setMenuState) => Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                CheckboxListTile(
                  dense: true,
                  contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                  title: Text(l10n.spriteViewerShowGrid),
                  value: _showGrid,
                  onChanged: (v) {
                    setState(() => _showGrid = v ?? false);
                    setMenuState(() {});
                  },
                ),
                CheckboxListTile(
                  dense: true,
                  contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                  title: Text(l10n.spriteViewerShowOutline),
                  value: _showOutline,
                  onChanged: (v) {
                    setState(() => _showOutline = v ?? false);
                    setMenuState(() {});
                  },
                ),
                CheckboxListTile(
                  dense: true,
                  contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                  title: Text(l10n.spriteViewerShowOffscreenRegions),
                  value: _showOffscreenRegions,
                  onChanged: (v) {
                    setState(() => _showOffscreenRegions = v ?? false);
                    setMenuState(() {});
                  },
                ),
                CheckboxListTile(
                  dense: true,
                  contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                  title: Text(l10n.spriteViewerDimOffscreenSpritesGrid),
                  value: _dimOffscreenGrid,
                  onChanged: (v) {
                    setState(() => _dimOffscreenGrid = v ?? false);
                    setMenuState(() {});
                  },
                ),
                Padding(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 16,
                    vertical: 6,
                  ),
                  child: Row(
                    children: [
                      Expanded(child: Text(l10n.tileViewerBackground)),
                      DropdownButtonHideUnderline(
                        child: DropdownButton<_SpriteBackground>(
                          isDense: true,
                          value: _background,
                          items: _SpriteBackground.values
                              .map(
                                (b) => DropdownMenuItem(
                                  value: b,
                                  child: Text(b.label(l10n)),
                                ),
                              )
                              .toList(),
                          onChanged: (v) {
                            if (v == null) return;
                            setState(() => _background = v);
                            setMenuState(() {});
                          },
                        ),
                      ),
                    ],
                  ),
                ),
                if (isNativeDesktop)
                  CheckboxListTile(
                    dense: true,
                    contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                    title: Text(l10n.spriteViewerShowListView),
                    value: _showListView,
                    onChanged: (v) {
                      setState(() => _showListView = v ?? false);
                      setMenuState(() {});
                    },
                  ),
              ],
            ),
          ),
        ),
      ],
    );
  }

  Offset _transformToPreviewContent(Offset screenPos) {
    final matrix = _previewTransformationController.value;
    final inverted = Matrix4.inverted(matrix);
    return MatrixUtils.transformPoint(inverted, screenPos);
  }

  int? _gridIndexAtPosition(
    Offset localPosition,
    Size viewportSize,
    bridge.SpriteSnapshot snapshot,
  ) {
    if (viewportSize.width <= 0 || viewportSize.height <= 0) return null;
    if (localPosition.dx < 0 ||
        localPosition.dy < 0 ||
        localPosition.dx > viewportSize.width ||
        localPosition.dy > viewportSize.height) {
      return null;
    }

    final totalW = _gridTextureWidth(snapshot).toDouble();
    final totalH = _gridTextureHeight(snapshot).toDouble();
    final x = (localPosition.dx / viewportSize.width) * totalW;
    final y = (localPosition.dy / viewportSize.height) * totalH;
    final col = (x / snapshot.thumbnailWidth).floor().clamp(0, _gridCols - 1);
    final row = (y / snapshot.thumbnailHeight).floor().clamp(0, _gridRows - 1);
    return row * _gridCols + col;
  }

  void _handleGridHover(
    Offset localPosition,
    Size viewportSize,
    bridge.SpriteSnapshot snapshot,
    BuildContext gridContext,
  ) {
    final idx = _gridIndexAtPosition(localPosition, viewportSize, snapshot);
    if (idx == _gridHoveredIndex && localPosition == _gridHoverPosition) return;

    _gridHoveredIndex = idx;
    _gridHoverPosition = localPosition;

    if (idx != null && idx < snapshot.sprites.length) {
      // Get global position for tooltip
      final RenderBox? box = gridContext.findRenderObject() as RenderBox?;
      if (box != null) {
        final globalPos = box.localToGlobal(localPosition);
        _showTooltipOverlay(globalPos, snapshot, idx);
      }
    } else {
      _removeTooltipOverlay();
    }
  }

  void _clearGridHover() {
    if (_gridHoveredIndex == null) return;
    _gridHoveredIndex = null;
    _gridHoverPosition = null;
    _removeTooltipOverlay();
  }

  void _showTooltipOverlay(
    Offset globalPosition,
    bridge.SpriteSnapshot snapshot,
    int index,
  ) {
    _removeTooltipOverlay();

    final overlay = Overlay.of(context);

    _tooltipOverlay = OverlayEntry(
      builder: (context) {
        final screenSize = MediaQuery.of(context).size;
        const tooltipWidth = 280.0;
        const tooltipHeight = 200.0;
        const cursorGap = 16.0;
        const screenPadding = 8.0;

        final bounds = Rect.fromLTWH(
          screenPadding,
          screenPadding,
          (screenSize.width - screenPadding * 2).clamp(0.0, double.infinity),
          (screenSize.height - screenPadding * 2).clamp(0.0, double.infinity),
        );

        Rect rectFor(double left, double top) =>
            Rect.fromLTWH(left, top, tooltipWidth, tooltipHeight);

        bool fits(Rect r) =>
            bounds.contains(r.topLeft) && bounds.contains(r.bottomRight);

        final cx = globalPosition.dx;
        final cy = globalPosition.dy;

        final candidates = <Rect>[
          // Prefer right-bottom, then left-bottom, then right-top, then left-top.
          rectFor(cx + cursorGap, cy + cursorGap),
          rectFor(cx - cursorGap - tooltipWidth, cy + cursorGap),
          rectFor(cx + cursorGap, cy - cursorGap - tooltipHeight),
          rectFor(
            cx - cursorGap - tooltipWidth,
            cy - cursorGap - tooltipHeight,
          ),
        ];

        Rect rect = candidates.firstWhere(
          fits,
          orElse: () {
            // Fall back to clamped right-bottom.
            final left = (cx + cursorGap).clamp(
              bounds.left,
              (bounds.right - tooltipWidth).clamp(bounds.left, bounds.right),
            );
            final top = (cy + cursorGap).clamp(
              bounds.top,
              (bounds.bottom - tooltipHeight).clamp(bounds.top, bounds.bottom),
            );
            return rectFor(left, top);
          },
        );

        // Ensure the tooltip never overlaps the cursor position. If it does, push it away.
        if (rect.contains(Offset(cx, cy))) {
          final tryRects =
              <Rect>[
                    rectFor(cx + cursorGap, rect.top),
                    rectFor(cx - cursorGap - tooltipWidth, rect.top),
                    rectFor(rect.left, cy + cursorGap),
                    rectFor(rect.left, cy - cursorGap - tooltipHeight),
                  ]
                  .map(
                    (r) => rectFor(
                      r.left.clamp(
                        bounds.left,
                        (bounds.right - tooltipWidth).clamp(
                          bounds.left,
                          bounds.right,
                        ),
                      ),
                      r.top.clamp(
                        bounds.top,
                        (bounds.bottom - tooltipHeight).clamp(
                          bounds.top,
                          bounds.bottom,
                        ),
                      ),
                    ),
                  )
                  .toList();

          rect = tryRects.firstWhere(
            (r) => !r.contains(Offset(cx, cy)) && fits(r),
            orElse: () => rect,
          );
        }

        return Positioned(
          left: rect.left,
          top: rect.top,
          child: IgnorePointer(
            child: Material(
              type: MaterialType.transparency,
              child: SizedBox(
                width: tooltipWidth,
                child: _SpriteHoverCard(
                  snapshot: snapshot,
                  textureId: _flutterThumbTextureId!,
                  index: index,
                ),
              ),
            ),
          ),
        );
      },
    );

    overlay.insert(_tooltipOverlay!);
  }

  void _removeTooltipOverlay() {
    _tooltipOverlay?.remove();
    _tooltipOverlay = null;
  }

  void _handleGridTap(
    Offset localPosition,
    Size viewportSize,
    bridge.SpriteSnapshot snapshot,
  ) {
    final idx = _gridIndexAtPosition(localPosition, viewportSize, snapshot);
    setState(() {
      _selectedIndex = idx;
      _previewSelectedPosition = null;
    });
  }

  Offset _previewScreenCoordAtContentPosition(Offset contentPos) {
    return contentPos;
  }

  int? _hitTestSprite(bridge.SpriteSnapshot snapshot, Offset screenCoord) {
    final x = screenCoord.dx;
    final y = screenCoord.dy;
    final spriteH = snapshot.largeSprites ? 16.0 : 8.0;

    for (var i = 0; i < snapshot.sprites.length; i++) {
      final s = snapshot.sprites[i];
      final sx = s.x.toDouble();
      final sy = (s.y + 1).toDouble();
      if (x >= sx && x < sx + 8 && y >= sy && y < sy + spriteH) {
        return i;
      }
    }
    return null;
  }

  void _handlePreviewHover(
    Offset localPosition,
    bridge.SpriteSnapshot snapshot,
    BuildContext previewContext,
  ) {
    final contentPos = _transformToPreviewContent(localPosition);
    final screenCoord = _previewScreenCoordAtContentPosition(contentPos);
    final idx = _hitTestSprite(snapshot, screenCoord);
    if (idx == _previewHoveredIndex && localPosition == _previewHoverPosition) {
      return;
    }
    setState(() {
      _previewHoveredIndex = idx;
      _previewHoverPosition = localPosition;
    });

    if (idx != null && idx < snapshot.sprites.length) {
      final RenderBox? box = previewContext.findRenderObject() as RenderBox?;
      if (box != null) {
        final globalPos = box.localToGlobal(localPosition);
        _showTooltipOverlay(globalPos, snapshot, idx);
      }
    } else {
      _removeTooltipOverlay();
    }
  }

  void _clearPreviewHover() {
    if (_previewHoveredIndex == null) return;
    setState(() {
      _previewHoveredIndex = null;
      _previewHoverPosition = null;
    });
    _removeTooltipOverlay();
  }

  void _handlePreviewTap(Offset localPosition, bridge.SpriteSnapshot snapshot) {
    final contentPos = _transformToPreviewContent(localPosition);
    final screenCoord = _previewScreenCoordAtContentPosition(contentPos);
    final idx = _hitTestSprite(snapshot, screenCoord);
    setState(() {
      _selectedIndex = idx;
      _previewSelectedPosition = localPosition;
    });
  }

  Color _backgroundColor(ThemeData theme) => _background.color(theme);

  Widget _backgroundWidget(ThemeData theme) {
    if (_background == _SpriteBackground.transparent) {
      return const SizedBox.expand(
        child: CustomPaint(painter: _CheckerboardPainter()),
      );
    }
    return SizedBox.expand(child: ColoredBox(color: _backgroundColor(theme)));
  }

  bool _matrixNear(Matrix4 a, Matrix4 b, {double eps = 1e-3}) {
    final as = a.storage;
    final bs = b.storage;
    for (var i = 0; i < 16; i++) {
      if ((as[i] - bs[i]).abs() > eps) return false;
    }
    return true;
  }

  Matrix4 _computePreviewDefaultTransform({
    required Size viewportSize,
    required double contentW,
    required double contentH,
  }) {
    final fitScale = viewportSize.width <= 0
        ? 1.0
        : viewportSize.width / contentW;
    final scale = fitScale > _maxScale ? _maxScale : fitScale;
    final dx = (viewportSize.width - contentW * scale) / 2.0;
    final dy = (viewportSize.height - contentH * scale) / 2.0;
    return Matrix4.identity()
      ..translateByDouble(dx, dy, 0, 1)
      ..scaleByDouble(scale, scale, 1, 1);
  }

  void _maybeUpdatePreviewDefaultTransform({
    required Size viewportSize,
    required int displayW,
    required int displayH,
  }) {
    if (viewportSize == Size.zero) return;
    final needsUpdate =
        viewportSize != _previewDefaultViewportSize ||
        displayW != _previewDefaultDisplayW ||
        displayH != _previewDefaultDisplayH ||
        _showOffscreenRegions != _previewDefaultShowOffscreenRegions;
    if (!needsUpdate) return;

    final wasAtDefault =
        _previewDefaultDisplayW == 0 ||
        _matrixNear(
          _previewTransformationController.value,
          _previewDefaultTransform,
        );

    final nextDefault = _computePreviewDefaultTransform(
      viewportSize: viewportSize,
      contentW: displayW.toDouble(),
      contentH: displayH.toDouble(),
    );

    _previewDefaultViewportSize = viewportSize;
    _previewDefaultDisplayW = displayW;
    _previewDefaultDisplayH = displayH;
    _previewDefaultShowOffscreenRegions = _showOffscreenRegions;
    _previewDefaultTransform = nextDefault;

    if (!wasAtDefault) return;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!mounted) return;
      _previewTransformationController.value = Matrix4.copy(nextDefault);
    });
  }

  Widget _buildThumbnailGrid(
    BuildContext context,
    bridge.SpriteSnapshot snapshot,
  ) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

    final gridW = _gridTextureWidth(snapshot);
    final gridH = _gridTextureHeight(snapshot);
    final aspectRatio = gridH == 0 ? 1.0 : gridW / gridH;

    final visibleMask = List<bool>.generate(_gridCols * _gridRows, (i) {
      if (i < snapshot.sprites.length) return snapshot.sprites[i].visible;
      return false;
    });

    return Center(
      child: AspectRatio(
        aspectRatio: aspectRatio,
        child: LayoutBuilder(
          builder: (context, constraints) {
            final viewportSize = constraints.biggest;

            return Stack(
              clipBehavior: Clip.none,
              children: [
                // Grid with clipping
                Positioned.fill(
                  child: Container(
                    decoration: BoxDecoration(
                      border: Border.all(
                        color: colorScheme.outlineVariant,
                        width: 1,
                      ),
                      borderRadius: BorderRadius.circular(4),
                    ),
                    clipBehavior: Clip.antiAlias,
                    child: MouseRegion(
                      onHover: (event) => _handleGridHover(
                        event.localPosition,
                        viewportSize,
                        snapshot,
                        context,
                      ),
                      onExit: (_) => _clearGridHover(),
                      child: GestureDetector(
                        behavior: HitTestBehavior.opaque,
                        onTapDown: (details) => _handleGridTap(
                          details.localPosition,
                          viewportSize,
                          snapshot,
                        ),
                        child: Stack(
                          children: [
                            _backgroundWidget(theme),
                            Texture(
                              textureId: _flutterThumbTextureId!,
                              filterQuality: FilterQuality.none,
                            ),
                            CustomPaint(
                              painter: _SpriteGridPainter(
                                showGrid: _showGrid,
                                dimOffscreen: _dimOffscreenGrid,
                                visibleMask: visibleMask,
                                hoveredIndex: _gridHoveredIndex,
                                selectedIndex: _selectedIndex,
                              ),
                              size: Size.infinite,
                            ),
                          ],
                        ),
                      ),
                    ),
                  ),
                ),
              ],
            );
          },
        ),
      ),
    );
  }

  Widget _buildScreenPreview(
    BuildContext context,
    bridge.SpriteSnapshot snapshot,
  ) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

    final displayW = _previewWidth;
    final displayH = _showOffscreenRegions ? _previewHeight : _screenHeight;
    final aspectRatio = displayH == 0 ? 1.0 : displayW / displayH;

    final showSelectedTooltip =
        !isNativeDesktop &&
        _selectedIndex != null &&
        _previewSelectedPosition != null &&
        _selectedIndex! < snapshot.sprites.length;

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
                _maybeUpdatePreviewDefaultTransform(
                  viewportSize: viewportSize,
                  displayW: displayW,
                  displayH: displayH,
                );

                final defaultScale = _previewDefaultTransform
                    .getMaxScaleOnAxis();
                final minScale = defaultScale < 1.0 ? defaultScale : 1.0;

                final content = SizedBox(
                  width: displayW.toDouble(),
                  height: displayH.toDouble(),
                  child: Stack(
                    children: [
                      _backgroundWidget(theme),
                      Positioned.fill(
                        child: _showOffscreenRegions
                            ? Texture(
                                textureId: _flutterScreenTextureId!,
                                filterQuality: FilterQuality.none,
                              )
                            : ClipRect(
                                child: OverflowBox(
                                  alignment: Alignment.topLeft,
                                  minWidth: _previewWidth.toDouble(),
                                  maxWidth: _previewWidth.toDouble(),
                                  minHeight: _previewHeight.toDouble(),
                                  maxHeight: _previewHeight.toDouble(),
                                  child: SizedBox(
                                    width: _previewWidth.toDouble(),
                                    height: _previewHeight.toDouble(),
                                    child: Texture(
                                      textureId: _flutterScreenTextureId!,
                                      filterQuality: FilterQuality.none,
                                    ),
                                  ),
                                ),
                              ),
                      ),
                      // CustomPaint overlay - same coordinate origin as the preview (0,0 at top-left)
                      CustomPaint(
                        painter: _SpritePreviewOverlayPainter(
                          sprites: snapshot.sprites,
                          largeSprites: snapshot.largeSprites,
                          showOutline: _showOutline,
                          showOffscreenRegions: _showOffscreenRegions,
                          hoveredIndex: _previewHoveredIndex,
                          selectedIndex: _selectedIndex,
                        ),
                        size: Size.infinite,
                      ),
                    ],
                  ),
                );

                return Builder(
                  builder: (previewContext) {
                    return MouseRegion(
                      onHover: (event) => _handlePreviewHover(
                        event.localPosition,
                        snapshot,
                        previewContext,
                      ),
                      onExit: (_) => _clearPreviewHover(),
                      child: GestureDetector(
                        behavior: HitTestBehavior.opaque,
                        onTapDown: (details) =>
                            _handlePreviewTap(details.localPosition, snapshot),
                        child: Stack(
                          children: [
                            Positioned.fill(
                              child: InteractiveViewer(
                                transformationController:
                                    _previewTransformationController,
                                minScale: minScale,
                                maxScale: _maxScale,
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
                                child: _buildTooltipFor(
                                  snapshot: snapshot,
                                  index: _selectedIndex!,
                                  position: _previewSelectedPosition!,
                                  viewportSize: viewportSize,
                                  tooltipWidth: 300,
                                  tooltipHeight: 170,
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

  Widget _buildTooltipFor({
    required bridge.SpriteSnapshot snapshot,
    required int index,
    required Offset position,
    required Size viewportSize,
    required double tooltipWidth,
    required double tooltipHeight,
  }) {
    // Offset from mouse cursor - tooltip should never overlap mouse position
    const double cursorOffset = 20.0;

    // Calculate available space in each direction from cursor
    final spaceRight = viewportSize.width - position.dx - cursorOffset;
    final spaceLeft = position.dx - cursorOffset;
    final spaceBottom = viewportSize.height - position.dy - cursorOffset;
    final spaceTop = position.dy - cursorOffset;

    double dx, dy;

    // Horizontal positioning: prefer right, fallback to left
    if (spaceRight >= tooltipWidth) {
      // Position to the right of cursor
      dx = position.dx + cursorOffset;
    } else if (spaceLeft >= tooltipWidth) {
      // Position to the left of cursor
      dx = position.dx - cursorOffset - tooltipWidth;
    } else {
      // Not enough space on either side, align to right edge
      dx = (viewportSize.width - tooltipWidth).clamp(0.0, double.infinity);
    }

    // Vertical positioning: prefer bottom, fallback to top
    if (spaceBottom >= tooltipHeight) {
      // Position below cursor
      dy = position.dy + cursorOffset;
    } else if (spaceTop >= tooltipHeight) {
      // Position above cursor
      dy = position.dy - cursorOffset - tooltipHeight;
    } else {
      // Not enough space, position at top
      dy = 0.0;
    }

    // Ensure tooltip stays within viewport bounds
    dx = dx.clamp(
      0.0,
      (viewportSize.width - tooltipWidth).clamp(0.0, double.infinity),
    );
    dy = dy.clamp(
      0.0,
      (viewportSize.height - tooltipHeight).clamp(0.0, double.infinity),
    );

    return Positioned(
      left: dx,
      top: dy,
      child: SizedBox(
        width: tooltipWidth,
        child: _SpriteHoverCard(
          snapshot: snapshot,
          textureId: _flutterThumbTextureId!,
          index: index,
        ),
      ),
    );
  }

  Widget _buildDesktopSidePanelWrapper(
    BuildContext context,
    bridge.SpriteSnapshot snapshot,
    Widget grid,
  ) {
    const panelWidth = 320.0;
    return ClipRect(
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 180),
        curve: Curves.easeOut,
        width: _showSidePanel ? panelWidth : 0,
        child: _showSidePanel
            ? SizedBox(
                width: panelWidth,
                child: _buildDesktopSidePanel(context, snapshot, grid),
              )
            : null,
      ),
    );
  }

  Widget _buildDesktopSidePanel(
    BuildContext context,
    bridge.SpriteSnapshot snapshot,
    Widget grid,
  ) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final l10n = AppLocalizations.of(context)!;

    final selected =
        _selectedIndex != null &&
            _selectedIndex! >= 0 &&
            _selectedIndex! < snapshot.sprites.length
        ? snapshot.sprites[_selectedIndex!]
        : null;

    return Container(
      decoration: BoxDecoration(
        color: colorScheme.surfaceContainerLowest,
        border: Border(
          left: BorderSide(color: colorScheme.outlineVariant, width: 1),
        ),
      ),
      child: Scrollbar(
        controller: _sidePanelScrollController,
        thumbVisibility: true,
        child: ListView(
          controller: _sidePanelScrollController,
          padding: const EdgeInsets.all(12),
          children: [
            // Sprite Grid
            _sideSection(
              context,
              title: l10n.spriteViewerPanelSprites,
              child: LayoutBuilder(
                builder: (context, constraints) {
                  final gridW = _gridTextureWidth(snapshot);
                  final gridH = _gridTextureHeight(snapshot);
                  final aspectRatio = gridH == 0 ? 1.0 : gridW / gridH;
                  final availableWidth = constraints.maxWidth;
                  final calculatedHeight = availableWidth / aspectRatio;

                  return SizedBox(
                    height: calculatedHeight.clamp(150.0, 300.0),
                    child: grid,
                  );
                },
              ),
            ),
            // List View Toggle
            CheckboxListTile(
              dense: true,
              visualDensity: VisualDensity.compact,
              controlAffinity: ListTileControlAffinity.trailing,
              contentPadding: EdgeInsets.zero,
              title: Text(l10n.spriteViewerShowListView),
              value: _showListView,
              onChanged: (v) => setState(() => _showListView = v ?? false),
            ),
            const SizedBox(height: 8),
            _sideSection(
              context,
              title: l10n.tilemapPanelDisplay,
              child: Column(
                children: [
                  CheckboxListTile(
                    dense: true,
                    visualDensity: VisualDensity.compact,
                    controlAffinity: ListTileControlAffinity.trailing,
                    contentPadding: EdgeInsets.zero,
                    title: Text(l10n.spriteViewerShowGrid),
                    value: _showGrid,
                    onChanged: (v) => setState(() => _showGrid = v ?? false),
                  ),
                  CheckboxListTile(
                    dense: true,
                    visualDensity: VisualDensity.compact,
                    controlAffinity: ListTileControlAffinity.trailing,
                    contentPadding: EdgeInsets.zero,
                    title: Text(l10n.spriteViewerShowOutline),
                    value: _showOutline,
                    onChanged: (v) => setState(() => _showOutline = v ?? false),
                  ),
                  CheckboxListTile(
                    dense: true,
                    visualDensity: VisualDensity.compact,
                    controlAffinity: ListTileControlAffinity.trailing,
                    contentPadding: EdgeInsets.zero,
                    title: Text(l10n.spriteViewerShowOffscreenRegions),
                    value: _showOffscreenRegions,
                    onChanged: (v) =>
                        setState(() => _showOffscreenRegions = v ?? false),
                  ),
                  CheckboxListTile(
                    dense: true,
                    visualDensity: VisualDensity.compact,
                    controlAffinity: ListTileControlAffinity.trailing,
                    contentPadding: EdgeInsets.zero,
                    title: Text(l10n.spriteViewerDimOffscreenSpritesGrid),
                    value: _dimOffscreenGrid,
                    onChanged: (v) =>
                        setState(() => _dimOffscreenGrid = v ?? false),
                  ),
                  const SizedBox(height: 6),
                  Row(
                    children: [
                      Expanded(
                        child: Text(
                          l10n.tileViewerBackground,
                          style: theme.textTheme.bodySmall,
                        ),
                      ),
                      DropdownButtonHideUnderline(
                        child: DropdownButton<_SpriteBackground>(
                          isDense: true,
                          value: _background,
                          items: _SpriteBackground.values
                              .map(
                                (b) => DropdownMenuItem(
                                  value: b,
                                  child: Text(b.label(l10n)),
                                ),
                              )
                              .toList(),
                          onChanged: (v) {
                            if (v == null) return;
                            setState(() => _background = v);
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
              title: l10n.spriteViewerPanelDataSource,
              child: Row(
                children: [
                  Expanded(
                    child: Text(
                      l10n.tileViewerSource,
                      style: theme.textTheme.bodySmall,
                    ),
                  ),
                  DropdownButtonHideUnderline(
                    child: DropdownButton<_SpriteDataSource>(
                      isDense: true,
                      value: _dataSource,
                      items: _SpriteDataSource.values
                          .map(
                            (s) => DropdownMenuItem(
                              value: s,
                              enabled: s == _SpriteDataSource.spriteRam,
                              child: Text(s.label(l10n)),
                            ),
                          )
                          .toList(),
                      onChanged: (v) {
                        if (v == null) return;
                        if (v != _SpriteDataSource.spriteRam) return;
                        setState(() => _dataSource = v);
                      },
                    ),
                  ),
                ],
              ),
            ),
            _sideSection(
              context,
              title: l10n.spriteViewerPanelSprite,
              child: Column(
                children: [
                  _kvModern(
                    context,
                    l10n.spriteViewerLabelMode,
                    snapshot.largeSprites ? '8×16' : '8×8',
                  ),
                  _kvModern(
                    context,
                    l10n.spriteViewerLabelPatternBase,
                    _hex(snapshot.patternBase, width: 4),
                  ),
                  _kvModern(
                    context,
                    l10n.spriteViewerLabelThumbnailSize,
                    '${snapshot.thumbnailWidth}×${snapshot.thumbnailHeight}',
                  ),
                ],
              ),
            ),
            _sideSection(
              context,
              title: l10n.spriteViewerPanelSelectedSprite,
              child: selected == null
                  ? _emptyHint(colorScheme.onSurfaceVariant)
                  : _SpriteInfoCard(
                      sprite: selected,
                      snapshot: snapshot,
                      textureId: _flutterThumbTextureId!,
                    ),
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

  Widget _kvModern(BuildContext context, String k, String v) {
    final theme = Theme.of(context);
    final labelStyle = theme.textTheme.bodySmall?.copyWith(
      color: theme.colorScheme.onSurfaceVariant,
    );
    final valueStyle = theme.textTheme.bodySmall?.copyWith(
      fontWeight: FontWeight.w600,
      fontFamily: 'monospace',
    );

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          Expanded(child: Text(k, style: labelStyle)),
          Text(v, style: valueStyle),
        ],
      ),
    );
  }

  String _hex(int value, {int width = 2}) =>
      '\$${value.toRadixString(16).toUpperCase().padLeft(width, '0')}';

  Widget _buildSpriteListView(
    BuildContext context,
    bridge.SpriteSnapshot snapshot,
  ) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final l10n = AppLocalizations.of(context)!;

    return Container(
      height: 240,
      decoration: BoxDecoration(
        color: colorScheme.surfaceContainerLowest,
        border: Border(top: BorderSide(color: colorScheme.outlineVariant)),
      ),
      child: Scrollbar(
        thumbVisibility: true,
        child: ListView.builder(
          padding: const EdgeInsets.symmetric(vertical: 8),
          itemCount: snapshot.sprites.length,
          itemBuilder: (context, i) {
            final s = snapshot.sprites[i];
            final selected = _selectedIndex == i;
            final yActual = (s.y + 1) & 0xFF;
            final mono = theme.textTheme.bodySmall?.copyWith(
              fontFamily: 'monospace',
            );

            String flags() =>
                '${s.flipH ? 'H' : '-'}${s.flipV ? 'V' : '-'}${s.behindBg ? 'B' : 'F'}';

            Widget cell(String text, {double? width}) {
              return SizedBox(
                width: width,
                child: Text(
                  text,
                  style: mono,
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
              );
            }

            return InkWell(
              onTap: () => setState(() => _selectedIndex = i),
              child: Container(
                padding: const EdgeInsets.symmetric(
                  horizontal: 12,
                  vertical: 8,
                ),
                color: selected
                    ? colorScheme.primaryContainer
                    : Colors.transparent,
                child: Row(
                  children: [
                    cell('#${s.index.toString().padLeft(2, '0')}', width: 44),
                    cell(
                      'X ${_hex(s.x)} (${s.x.toString().padLeft(3)})',
                      width: 110,
                    ),
                    cell(
                      'Y ${_hex(s.y)} (${yActual.toString().padLeft(3)})',
                      width: 110,
                    ),
                    cell('T ${_hex(s.tileIndex)}', width: 54),
                    cell('P ${s.palette}', width: 40),
                    cell(flags(), width: 44),
                    const Spacer(),
                    Text(
                      s.visible
                          ? l10n.spriteViewerVisibleStatusVisible
                          : l10n.spriteViewerVisibleStatusOffscreen,
                      style: theme.textTheme.bodySmall?.copyWith(
                        color: s.visible
                            ? colorScheme.onSurfaceVariant
                            : colorScheme.error,
                        fontWeight: FontWeight.w600,
                      ),
                    ),
                  ],
                ),
              ),
            );
          },
        ),
      ),
    );
  }
}

class _SpriteGridPainter extends CustomPainter {
  _SpriteGridPainter({
    required this.showGrid,
    required this.dimOffscreen,
    required this.visibleMask,
    required this.hoveredIndex,
    required this.selectedIndex,
  });

  final bool showGrid;
  final bool dimOffscreen;
  final List<bool> visibleMask;
  final int? hoveredIndex;
  final int? selectedIndex;

  @override
  void paint(Canvas canvas, Size size) {
    if (size.width <= 0 || size.height <= 0) return;
    final cellW = size.width / _SpriteViewerState._gridCols;
    final cellH = size.height / _SpriteViewerState._gridRows;

    if (dimOffscreen && visibleMask.isNotEmpty) {
      final paint = Paint()..color = Colors.black.withValues(alpha: 0.35);
      for (var i = 0; i < visibleMask.length; i++) {
        final visible = visibleMask[i];
        if (visible) continue;
        final col = i % _SpriteViewerState._gridCols;
        final row = i ~/ _SpriteViewerState._gridCols;
        final rect = Rect.fromLTWH(col * cellW, row * cellH, cellW, cellH);
        canvas.drawRect(rect, paint);
      }
    }

    if (showGrid) {
      final paint = Paint()
        ..color = Colors.white.withValues(alpha: 0.55)
        ..strokeWidth = 1.0
        ..style = PaintingStyle.stroke;

      for (var r = 0; r <= _SpriteViewerState._gridRows; r++) {
        final y = r * cellH;
        canvas.drawLine(Offset(0, y), Offset(size.width, y), paint);
      }
      for (var c = 0; c <= _SpriteViewerState._gridCols; c++) {
        final x = c * cellW;
        canvas.drawLine(Offset(x, 0), Offset(x, size.height), paint);
      }
    }

    if (hoveredIndex != null) {
      final paint = Paint()
        ..color = Colors.white.withValues(alpha: 0.9)
        ..strokeWidth = 2.0
        ..style = PaintingStyle.stroke;
      final col = hoveredIndex! % _SpriteViewerState._gridCols;
      final row = hoveredIndex! ~/ _SpriteViewerState._gridCols;
      canvas.drawRect(
        Rect.fromLTWH(col * cellW, row * cellH, cellW, cellH),
        paint,
      );
    }

    if (selectedIndex != null) {
      final paint = Paint()
        ..color = Colors.yellow.withValues(alpha: 0.95)
        ..strokeWidth = 2.5
        ..style = PaintingStyle.stroke;
      final col = selectedIndex! % _SpriteViewerState._gridCols;
      final row = selectedIndex! ~/ _SpriteViewerState._gridCols;
      canvas.drawRect(
        Rect.fromLTWH(col * cellW, row * cellH, cellW, cellH),
        paint,
      );
    }
  }

  @override
  bool shouldRepaint(_SpriteGridPainter oldDelegate) {
    return showGrid != oldDelegate.showGrid ||
        dimOffscreen != oldDelegate.dimOffscreen ||
        hoveredIndex != oldDelegate.hoveredIndex ||
        selectedIndex != oldDelegate.selectedIndex ||
        visibleMask != oldDelegate.visibleMask;
  }
}

class _SpriteHoverCard extends StatelessWidget {
  const _SpriteHoverCard({
    required this.snapshot,
    required this.textureId,
    required this.index,
  });

  final bridge.SpriteSnapshot snapshot;
  final int textureId;
  final int index;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final l10n = AppLocalizations.of(context)!;
    final s = snapshot.sprites[index];
    final yActual = (s.y + 1) & 0xFF;

    final spriteSize = snapshot.largeSprites ? '8×16' : '8×8';

    int spriteTileBaseAddr() {
      if (!snapshot.largeSprites) {
        return snapshot.patternBase + s.tileIndex * 16;
      }
      final tableBase = (s.tileIndex & 0x01) != 0 ? 0x1000 : 0x0000;
      final baseTile = s.tileIndex & 0xFE;
      return tableBase + baseTile * 16;
    }

    final tileAddr = spriteTileBaseAddr();
    final tileAddr2 = snapshot.largeSprites ? tileAddr + 16 : null;
    final paletteAddr = 0x3F10 + s.palette * 4;

    String hex(int v, {int width = 2}) =>
        '\$${v.toRadixString(16).toUpperCase().padLeft(width, '0')}';

    return Card(
      elevation: 8,
      shadowColor: colorScheme.shadow.withValues(alpha: 0.3),
      color: colorScheme.surfaceContainerHigh,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(12),
        side: BorderSide(color: colorScheme.outlineVariant),
      ),
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            _SpriteThumbnailPreview(
              textureId: textureId,
              index: index,
              thumbWidth: snapshot.thumbnailWidth,
              thumbHeight: snapshot.thumbnailHeight,
              scale: 6,
            ),
            const SizedBox(width: 12),
            Expanded(
              child: DefaultTextStyle(
                style: theme.textTheme.bodySmall!,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      l10n.spriteViewerTooltipTitle(s.index),
                      style: theme.textTheme.titleSmall?.copyWith(
                        fontWeight: FontWeight.w700,
                      ),
                    ),
                    const SizedBox(height: 8),
                    _kv(theme, l10n.spriteViewerLabelPos, '${s.x}, $yActual'),
                    _kv(theme, l10n.spriteViewerLabelSize, spriteSize),
                    _kv(theme, l10n.spriteViewerLabelTile, hex(s.tileIndex)),
                    _kv(
                      theme,
                      l10n.spriteViewerLabelTileAddr,
                      tileAddr2 == null
                          ? hex(tileAddr, width: 4)
                          : '${hex(tileAddr, width: 4)} / ${hex(tileAddr2, width: 4)}',
                    ),
                    _kv(theme, l10n.spriteViewerLabelPalette, '${s.palette}'),
                    _kv(
                      theme,
                      l10n.spriteViewerLabelPaletteAddr,
                      hex(paletteAddr, width: 4),
                    ),
                    _kv(
                      theme,
                      l10n.spriteViewerLabelFlip,
                      '${s.flipH ? 'H' : '-'}${s.flipV ? 'V' : '-'}',
                    ),
                    _kv(
                      theme,
                      l10n.spriteViewerLabelPriority,
                      s.behindBg
                          ? l10n.spriteViewerPriorityBehindBg
                          : l10n.spriteViewerPriorityInFront,
                    ),
                    _kv(
                      theme,
                      l10n.spriteViewerLabelVisible,
                      s.visible
                          ? l10n.spriteViewerValueYes
                          : l10n.spriteViewerValueNoOffscreen,
                    ),
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _kv(ThemeData theme, String k, String v) {
    final labelStyle = theme.textTheme.bodySmall?.copyWith(
      color: theme.colorScheme.onSurfaceVariant,
    );
    final valueStyle = theme.textTheme.bodySmall?.copyWith(
      fontWeight: FontWeight.w600,
      fontFamily: 'monospace',
    );

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        children: [
          Expanded(child: Text(k, style: labelStyle)),
          Text(v, style: valueStyle),
        ],
      ),
    );
  }
}

class _SpriteInfoCard extends StatelessWidget {
  const _SpriteInfoCard({
    required this.sprite,
    required this.snapshot,
    required this.textureId,
  });

  final bridge.SpriteInfo sprite;
  final bridge.SpriteSnapshot snapshot;
  final int textureId;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final l10n = AppLocalizations.of(context)!;
    final labelStyle = theme.textTheme.bodySmall?.copyWith(
      color: colorScheme.onSurfaceVariant,
    );
    final valueStyle = theme.textTheme.bodySmall?.copyWith(
      fontWeight: FontWeight.w600,
      fontFamily: 'monospace',
    );
    final yActual = (sprite.y + 1) & 0xFF;
    final spriteSize = snapshot.largeSprites ? '8×16' : '8×8';

    int spriteTileBaseAddr() {
      if (!snapshot.largeSprites) {
        return snapshot.patternBase + sprite.tileIndex * 16;
      }
      final tableBase = (sprite.tileIndex & 0x01) != 0 ? 0x1000 : 0x0000;
      final baseTile = sprite.tileIndex & 0xFE;
      return tableBase + baseTile * 16;
    }

    final tileAddr = spriteTileBaseAddr();
    final tileAddr2 = snapshot.largeSprites ? tileAddr + 16 : null;
    final paletteAddr = 0x3F10 + sprite.palette * 4;

    String hex(int v, {int width = 2}) =>
        '\$${v.toRadixString(16).toUpperCase().padLeft(width, '0')}';

    Widget kv(String k, String v) {
      return Padding(
        padding: const EdgeInsets.symmetric(vertical: 4),
        child: Row(
          children: [
            Expanded(child: Text(k, style: labelStyle)),
            Text(v, style: valueStyle),
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
            _SpriteThumbnailPreview(
              textureId: textureId,
              index: sprite.index,
              thumbWidth: snapshot.thumbnailWidth,
              thumbHeight: snapshot.thumbnailHeight,
              scale: 7,
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Column(
                children: [
                  kv(l10n.spriteViewerLabelIndex, '#${sprite.index}'),
                  kv(l10n.spriteViewerLabelPos, '${sprite.x}, $yActual'),
                  kv(l10n.spriteViewerLabelSize, spriteSize),
                  kv(l10n.spriteViewerLabelTile, hex(sprite.tileIndex)),
                  kv(
                    l10n.spriteViewerLabelTileAddr,
                    tileAddr2 == null
                        ? hex(tileAddr, width: 4)
                        : '${hex(tileAddr, width: 4)} / ${hex(tileAddr2, width: 4)}',
                  ),
                ],
              ),
            ),
          ],
        ),
        const SizedBox(height: 10),
        const Divider(height: 1),
        const SizedBox(height: 10),
        kv(l10n.spriteViewerLabelPalette, '${sprite.palette}'),
        kv(l10n.spriteViewerLabelPaletteAddr, hex(paletteAddr, width: 4)),
        kv(
          l10n.spriteViewerLabelFlip,
          '${sprite.flipH ? 'H' : '-'}${sprite.flipV ? 'V' : '-'}',
        ),
        kv(
          l10n.spriteViewerLabelPriority,
          sprite.behindBg
              ? l10n.spriteViewerPriorityBehindBg
              : l10n.spriteViewerPriorityInFront,
        ),
        kv(
          l10n.spriteViewerLabelVisible,
          sprite.visible
              ? l10n.spriteViewerValueYes
              : l10n.spriteViewerValueNoOffscreen,
        ),
      ],
    );
  }
}

class _SpriteThumbnailPreview extends StatelessWidget {
  const _SpriteThumbnailPreview({
    required this.textureId,
    required this.index,
    required this.thumbWidth,
    required this.thumbHeight,
    required this.scale,
  });

  final int textureId;
  final int index;
  final int thumbWidth;
  final int thumbHeight;
  final double scale;

  @override
  Widget build(BuildContext context) {
    final gridX = index % _SpriteViewerState._gridCols;
    final gridY = index ~/ _SpriteViewerState._gridCols;

    final srcX = (gridX * thumbWidth).toDouble();
    final srcY = (gridY * thumbHeight).toDouble();

    final dstW = thumbWidth * scale;
    final dstH = thumbHeight * scale;

    final totalW = (_SpriteViewerState._gridCols * thumbWidth * scale);
    final totalH = (_SpriteViewerState._gridRows * thumbHeight * scale);

    // Use UnconstrainedBox + OverflowBox to allow the texture to render at full size,
    // then clip and position it correctly
    return ClipRRect(
      borderRadius: BorderRadius.circular(8),
      child: SizedBox(
        width: dstW,
        height: dstH,
        child: OverflowBox(
          alignment: Alignment.topLeft,
          minWidth: totalW,
          maxWidth: totalW,
          minHeight: totalH,
          maxHeight: totalH,
          child: Transform.translate(
            offset: Offset(-srcX * scale, -srcY * scale),
            child: SizedBox(
              width: totalW,
              height: totalH,
              child: Texture(
                textureId: textureId,
                filterQuality: FilterQuality.none,
              ),
            ),
          ),
        ),
      ),
    );
  }
}

enum _SpriteBackground { gray, black, white, magenta, transparent }

extension on _SpriteBackground {
  String label(AppLocalizations l10n) => switch (this) {
    _SpriteBackground.gray => l10n.spriteViewerBgGray,
    _SpriteBackground.black => l10n.tileViewerBgBlack,
    _SpriteBackground.white => l10n.tileViewerBgWhite,
    _SpriteBackground.magenta => l10n.tileViewerBgMagenta,
    _SpriteBackground.transparent => l10n.tileViewerBgTransparent,
  };

  Color color(ThemeData theme) => switch (this) {
    _SpriteBackground.gray => const Color(0xFF808080),
    _SpriteBackground.black => Colors.black,
    _SpriteBackground.white => Colors.white,
    _SpriteBackground.magenta => const Color(0xFFFF00FF),
    _SpriteBackground.transparent => const Color(0x00000000),
  };
}

enum _SpriteDataSource { spriteRam, cpuMemory }

extension on _SpriteDataSource {
  String label(AppLocalizations l10n) => switch (this) {
    _SpriteDataSource.spriteRam => l10n.spriteViewerDataSourceSpriteRam,
    _SpriteDataSource.cpuMemory => l10n.spriteViewerDataSourceCpuMemory,
  };
}

class _CheckerboardPainter extends CustomPainter {
  const _CheckerboardPainter();

  @override
  void paint(Canvas canvas, Size size) {
    const cell = 12.0;
    final light = Paint()..color = const Color(0xFFE6E6E6);
    final dark = Paint()..color = const Color(0xFFCBCBCB);

    for (var y = 0.0; y < size.height; y += cell) {
      for (var x = 0.0; x < size.width; x += cell) {
        final isDark = ((x / cell).floor() + (y / cell).floor()) % 2 == 0;
        canvas.drawRect(Rect.fromLTWH(x, y, cell, cell), isDark ? dark : light);
      }
    }
  }

  @override
  bool shouldRepaint(covariant _CheckerboardPainter oldDelegate) => false;
}

class _SpritePreviewOverlayPainter extends CustomPainter {
  const _SpritePreviewOverlayPainter({
    required this.sprites,
    required this.largeSprites,
    required this.showOutline,
    required this.showOffscreenRegions,
    required this.hoveredIndex,
    required this.selectedIndex,
  });

  final List<bridge.SpriteInfo> sprites;
  final bool largeSprites;
  final bool showOutline;
  final bool showOffscreenRegions;
  final int? hoveredIndex;
  final int? selectedIndex;

  @override
  void paint(Canvas canvas, Size size) {
    if (size.width <= 0 || size.height <= 0) return;

    final baseW = _SpriteViewerState._screenWidth.toDouble();
    final baseH =
        (showOffscreenRegions
                ? _SpriteViewerState._previewHeight
                : _SpriteViewerState._screenHeight)
            .toDouble();

    final sx = size.width / baseW;
    final sy = size.height / baseH;
    final spriteH = (largeSprites ? 16 : 8).toDouble();

    if (showOffscreenRegions) {
      final paint = Paint()
        ..color = Colors.white.withValues(alpha: 0.5)
        ..strokeWidth = 1.0
        ..style = PaintingStyle.stroke;

      final rect = Rect.fromLTWH(
        0,
        0,
        baseW * sx,
        _SpriteViewerState._screenHeight.toDouble() * sy,
      );
      canvas.drawRect(rect, paint);
    }

    if (showOutline) {
      final paint = Paint()
        ..color = Colors.white.withValues(alpha: 0.35)
        ..strokeWidth = 1.0
        ..style = PaintingStyle.stroke;

      for (final s in sprites) {
        final x = s.x.toDouble() * sx;
        final y = (s.y + 1).toDouble() * sy;
        final rect = Rect.fromLTWH(x, y, 8 * sx, spriteH * sy);
        canvas.drawRect(rect, paint);
      }
    }

    if (hoveredIndex != null &&
        hoveredIndex! >= 0 &&
        hoveredIndex! < sprites.length) {
      final s = sprites[hoveredIndex!];
      final paint = Paint()
        ..color = Colors.white.withValues(alpha: 0.9)
        ..strokeWidth = 1.0
        ..style = PaintingStyle.stroke;
      final x = s.x.toDouble() * sx;
      final y = (s.y + 1).toDouble() * sy;
      canvas.drawRect(Rect.fromLTWH(x, y, 8 * sx, spriteH * sy), paint);
    }

    if (selectedIndex != null &&
        selectedIndex! >= 0 &&
        selectedIndex! < sprites.length) {
      final s = sprites[selectedIndex!];
      final paint = Paint()
        ..color = Colors.yellow.withValues(alpha: 0.95)
        ..strokeWidth = 1.0
        ..style = PaintingStyle.stroke;
      final x = s.x.toDouble() * sx;
      final y = (s.y + 1).toDouble() * sy;
      canvas.drawRect(Rect.fromLTWH(x, y, 8 * sx, spriteH * sy), paint);
    }
  }

  @override
  bool shouldRepaint(covariant _SpritePreviewOverlayPainter oldDelegate) {
    return sprites != oldDelegate.sprites ||
        largeSprites != oldDelegate.largeSprites ||
        showOutline != oldDelegate.showOutline ||
        showOffscreenRegions != oldDelegate.showOffscreenRegions ||
        hoveredIndex != oldDelegate.hoveredIndex ||
        selectedIndex != oldDelegate.selectedIndex;
  }
}
