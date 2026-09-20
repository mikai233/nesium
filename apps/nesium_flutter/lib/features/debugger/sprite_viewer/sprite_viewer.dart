import 'dart:async';
import 'dart:typed_data';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/domain/nes_controller.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/painters/checkerboard_painter.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer_controller.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer_models.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer_state.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer_utils.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/widgets/sprite_hover_card.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/widgets/sprite_screen_preview.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/widgets/sprite_settings_menu.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/widgets/sprite_side_panel.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/widgets/sprite_thumbnail_grid.dart';
import 'package:nesium_flutter/features/debugger/viewer_skeletonizer.dart';
import 'package:nesium_flutter/l10n/app_localizations.dart';
import 'package:nesium_flutter/logging/app_logger.dart';
import 'package:nesium_flutter/platform/platform_capabilities.dart';
import 'package:nesium_flutter/widgets/single_position_scrollbar.dart';

/// Sprite Viewer - displays 64 sprite thumbnails using an auxiliary texture.
class SpriteViewer extends ConsumerStatefulWidget {
  const SpriteViewer({super.key});

  @override
  ConsumerState<SpriteViewer> createState() => _SpriteViewerState();
}

class _SpriteViewerState extends ConsumerState<SpriteViewer> {
  final SpriteViewerController _controller = SpriteViewerController();
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

  SpriteTextureHandle? _thumbTexture;
  SpriteTextureHandle? _screenTexture;
  bool _isCreating = false;
  String? _error;

  SpriteViewerDisplayOptions _displayOptions =
      const SpriteViewerDisplayOptions();
  SpriteViewerCaptureState _captureState = const SpriteViewerCaptureState();
  late final TextEditingController _scanlineController = TextEditingController(
    text: _captureState.scanline.toString(),
  );
  late final TextEditingController _dotController = TextEditingController(
    text: _captureState.dot.toString(),
  );

  // ValueNotifier for sprite overlay data - allows isolated repaint
  final ValueNotifier<SpriteOverlayData> _spriteOverlayData = ValueNotifier(
    const SpriteOverlayData(sprites: [], largeSprites: false),
  );

  // Zoom and pan state
  final TransformationController _previewTransformationController =
      TransformationController();
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
    final isTransformed = !matrixNear(matrix, _previewDefaultTransform);
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
    unawaitedLogged(
      _controller.dispose(),
      message: 'Failed to dispose sprite viewer',
    );
    _scanlineController.dispose();
    _dotController.dispose();
    _previewTransformationController.removeListener(_onTransformChanged);
    _previewTransformationController.dispose();
    _removeTooltipOverlay();
    _spriteOverlayData.dispose();
    super.dispose();
  }

  Future<void> _applyCaptureMode() async {
    await _controller.applyCaptureMode(_captureState);
  }

  Future<void> _startStreaming() async {
    await _controller.startStreaming(
      captureState: _captureState,
      onSnapshot: (snapshot) {
        if (mounted) {
          final firstData = !_hasReceivedData;
          _snapshot = snapshot;
          _hasReceivedData = true;

          if (firstData) {
            setState(() {});
          }
          if (_displayOptions.showOutline) {
            _spriteOverlayData.value = SpriteOverlayData(
              sprites: snapshot.sprites,
              largeSprites: snapshot.largeSprites,
            );
          }

          unawaitedLogged(
            _ensureTextures(snapshot),
            message: 'Failed to create sprite textures',
          );
        }
      },
      onError: (_) {},
    );
  }

  Future<void> _ensureTextures(bridge.SpriteSnapshot snapshot) async {
    if (_isCreating || _error != null) return;
    final width = SpriteViewerLayout.gridTextureWidth(snapshot);
    final height = SpriteViewerLayout.gridTextureHeight(snapshot);
    if (_thumbTexture?.width == width &&
        _thumbTexture?.height == height &&
        _screenTexture != null) {
      return;
    }

    setState(() {
      _isCreating = true;
      _error = null;
    });

    try {
      final thumbTexture = await _controller.ensureThumbTexture(
        snapshot: snapshot,
        currentHandle: _thumbTexture,
      );
      if (!mounted) return;
      _thumbTexture = thumbTexture;
      final screenTexture = await _controller.ensureScreenTexture(
        currentHandle: _screenTexture,
      );
      if (!mounted) return;
      setState(() {
        _screenTexture = screenTexture;
        _isCreating = false;
      });
      // A size change may have arrived while the textures were being created.
      final latestSnapshot = _snapshot;
      if (latestSnapshot != null) {
        await _ensureTextures(latestSnapshot);
      }
    } catch (e) {
      if (!mounted) return;
      setState(() {
        _error = e.toString();
        _isCreating = false;
      });
    }
  }

  Future<void> _retry() async {
    await _controller.stopStreaming();
    if (!mounted) return;
    setState(() {
      _error = null;
      _snapshot = null;
      _hasReceivedData = false;
      _thumbTexture = null;
      _screenTexture = null;
    });
    await _startStreaming();
  }

  @override
  Widget build(BuildContext context) {
    final snapshot = _snapshot;
    final hasRom = ref.watch(nesControllerProvider).romHash != null;
    final loading =
        !hasRom ||
        !_hasReceivedData ||
        snapshot == null ||
        _thumbTexture == null ||
        _screenTexture == null;
    final effectiveSnapshot = snapshot ?? _loadingSnapshot();

    if (_error != null) return _buildErrorState(context);
    final base = ViewerSkeletonizer(
      enabled: loading,
      child: _buildMainLayout(context, effectiveSnapshot),
    );
    return base;
  }

  bridge.SpriteSnapshot _loadingSnapshot() {
    final sprites = List<bridge.SpriteInfo>.generate(
      64,
      (i) => bridge.SpriteInfo(
        index: i,
        x: 0,
        y: 0,
        tileIndex: 0,
        palette: 0,
        flipH: false,
        flipV: false,
        behindBg: false,
        visible: true,
      ),
    );
    return bridge.SpriteSnapshot(
      sprites: sprites,
      thumbnailWidth: 8,
      thumbnailHeight: 8,
      largeSprites: false,
      patternBase: 0,
      rgbaPalette: Uint8List(0),
    );
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
          if (_displayOptions.showListView)
            _buildSpriteListView(context, snapshot),
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

    return Builder(
      builder: (buttonContext) => IconButton(
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
        onPressed: () => showSpriteViewerSettingsMenu(
          context: buttonContext,
          displayOptions: _displayOptions,
          captureState: _captureState,
          scanlineController: _scanlineController,
          dotController: _dotController,
          onDisplayOptionsChanged: (value) {
            setState(() => _displayOptions = value);
          },
          onCaptureStateChanged: (value) {
            setState(() => _captureState = value);
          },
          onApplyCaptureMode: () {
            unawaitedLogged(
              _applyCaptureMode(),
              message: 'Failed to set sprite capture point',
            );
          },
        ),
      ),
    );
  }

  Widget _buildPanelToggleButton(BuildContext context) {
    final theme = Theme.of(context);
    final l10n = AppLocalizations.of(context)!;

    final icon = _displayOptions.showSidePanel
        ? Icons.chevron_right
        : Icons.chevron_left;
    final tooltip = _displayOptions.showSidePanel
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
      onPressed: () => setState(
        () => _displayOptions = _displayOptions.copyWith(
          showSidePanel: !_displayOptions.showSidePanel,
        ),
      ),
    );
  }

  void _clearGridHover() {
    if (_gridHoveredIndex == null) return;
    setState(() {
      _gridHoveredIndex = null;
      _gridHoverPosition = null;
    });
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
        final rect = computeOverlayTooltipRect(
          globalPosition: globalPosition,
          screenSize: screenSize,
          tooltipWidth: tooltipWidth,
          tooltipHeight: tooltipHeight,
        );

        return Positioned(
          left: rect.left,
          top: rect.top,
          child: IgnorePointer(
            child: Material(
              type: MaterialType.transparency,
              child: SizedBox(
                width: tooltipWidth,
                child: SpriteHoverCard(
                  snapshot: snapshot,
                  textureId: _thumbTexture!.textureId,
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

  void _clearPreviewHover() {
    if (_previewHoveredIndex == null) return;
    setState(() {
      _previewHoveredIndex = null;
      _previewHoverPosition = null;
    });
    _removeTooltipOverlay();
  }

  Color _backgroundColor(ThemeData theme) =>
      _displayOptions.background.color(theme);

  Widget _backgroundWidget(ThemeData theme) {
    if (_displayOptions.background == SpriteViewerBackground.transparent) {
      return const SizedBox.expand(
        child: CustomPaint(painter: CheckerboardPainter()),
      );
    }
    return SizedBox.expand(child: ColoredBox(color: _backgroundColor(theme)));
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
        _displayOptions.showOffscreenRegions !=
            _previewDefaultShowOffscreenRegions;
    if (!needsUpdate) return;

    final wasAtDefault =
        _previewDefaultDisplayW == 0 ||
        matrixNear(
          _previewTransformationController.value,
          _previewDefaultTransform,
        );

    final nextDefault = computePreviewDefaultTransform(
      viewportSize: viewportSize,
      contentW: displayW.toDouble(),
      contentH: displayH.toDouble(),
      maxScale: _maxScale,
    );

    _previewDefaultViewportSize = viewportSize;
    _previewDefaultDisplayW = displayW;
    _previewDefaultDisplayH = displayH;
    _previewDefaultShowOffscreenRegions = _displayOptions.showOffscreenRegions;
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
    return SpriteThumbnailGrid(
      snapshot: snapshot,
      background: _displayOptions.background,
      backgroundBuilder: _backgroundWidget(Theme.of(context)),
      textureId: _thumbTexture?.textureId,
      showGrid: _displayOptions.showGrid,
      dimOffscreen: _displayOptions.dimOffscreenGrid,
      hoveredIndex: _gridHoveredIndex,
      selectedIndex: _selectedIndex,
      onHover: (index, localPosition, globalPosition) {
        if (index == _gridHoveredIndex && localPosition == _gridHoverPosition) {
          return;
        }
        setState(() {
          _gridHoveredIndex = index;
          _gridHoverPosition = localPosition;
        });
        if (index != null &&
            index < snapshot.sprites.length &&
            globalPosition != null) {
          _showTooltipOverlay(globalPosition, snapshot, index);
        } else {
          _removeTooltipOverlay();
        }
      },
      onHoverExit: _clearGridHover,
      onTap: (index) {
        setState(() {
          _selectedIndex = index;
          _previewSelectedPosition = null;
        });
      },
    );
  }

  Widget _buildScreenPreview(
    BuildContext context,
    bridge.SpriteSnapshot snapshot,
  ) {
    return SpriteScreenPreview(
      snapshot: snapshot,
      screenTextureId: _screenTexture?.textureId,
      thumbTextureId: _thumbTexture?.textureId,
      overlayDataListenable: _spriteOverlayData,
      backgroundBuilder: _backgroundWidget(Theme.of(context)),
      showOutline: _displayOptions.showOutline,
      showOffscreenRegions: _displayOptions.showOffscreenRegions,
      hoveredIndex: _previewHoveredIndex,
      selectedIndex: _selectedIndex,
      selectedPosition: _previewSelectedPosition,
      transformationController: _previewTransformationController,
      maxScale: _maxScale,
      onViewportChanged: (viewportSize, displayW, displayH) {
        _maybeUpdatePreviewDefaultTransform(
          viewportSize: viewportSize,
          displayW: displayW,
          displayH: displayH,
        );
      },
      onHover: (index, localPosition, globalPosition) {
        if (index == _previewHoveredIndex &&
            localPosition == _previewHoverPosition) {
          return;
        }
        setState(() {
          _previewHoveredIndex = index;
          _previewHoverPosition = localPosition;
        });
        if (index != null &&
            index < snapshot.sprites.length &&
            globalPosition != null) {
          _showTooltipOverlay(globalPosition, snapshot, index);
        } else {
          _removeTooltipOverlay();
        }
      },
      onHoverExit: _clearPreviewHover,
      onTap: (index, localPosition) {
        setState(() {
          _selectedIndex = index;
          _previewSelectedPosition = localPosition;
        });
      },
    );
  }

  Widget _buildDesktopSidePanelWrapper(
    BuildContext context,
    bridge.SpriteSnapshot snapshot,
    Widget grid,
  ) {
    const panelWidth = 320.0;
    return ClipRect(
      child: TweenAnimationBuilder<double>(
        duration: const Duration(milliseconds: 180),
        curve: Curves.easeOut,
        tween: Tween<double>(end: _displayOptions.showSidePanel ? 1.0 : 0.0),
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
          child: SpriteSidePanel(
            snapshot: snapshot,
            grid: grid,
            displayOptions: _displayOptions,
            captureState: _captureState,
            scanlineController: _scanlineController,
            dotController: _dotController,
            selectedIndex: _selectedIndex,
            thumbTextureId: _thumbTexture?.textureId,
            onDisplayOptionsChanged: (value) {
              setState(() => _displayOptions = value);
            },
            onCaptureStateChanged: (value) {
              setState(() => _captureState = value);
            },
            onApplyCaptureMode: () {
              unawaitedLogged(
                _applyCaptureMode(),
                message: 'Failed to set sprite capture point',
              );
            },
          ),
        ),
      ),
    );
  }

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
      child: SinglePositionScrollbar(
        thumbVisibility: true,
        builder: (context, controller) {
          return ListView.builder(
            controller: controller,
            primary: false,
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
                        'X ${formatHexValue(s.x)} (${s.x.toString().padLeft(3)})',
                        width: 110,
                      ),
                      cell(
                        'Y ${formatHexValue(s.y)} (${yActual.toString().padLeft(3)})',
                        width: 110,
                      ),
                      cell('T ${formatHexValue(s.tileIndex)}', width: 54),
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
          );
        },
      ),
    );
  }
}
