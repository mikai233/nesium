import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/domain/nes_texture_service.dart';
import 'package:nesium_flutter/l10n/app_localizations.dart';
import 'package:nesium_flutter/platform/platform_capabilities.dart';

/// Tile Viewer that displays NES CHR pattern tables via a Flutter Texture.
class TileViewer extends ConsumerStatefulWidget {
  const TileViewer({super.key});

  @override
  ConsumerState<TileViewer> createState() => _TileViewerState();
}

class _TileViewerState extends ConsumerState<TileViewer> {
  static const int _chrTextureId = 2;
  static const int _width = 128; // 16 tiles × 8 pixels
  static const int _height = 256; // 32 tiles × 8 pixels

  final NesTextureService _textureService = NesTextureService();
  int? _flutterTextureId;
  bool _isCreating = false;
  String? _error;
  StreamSubscription<bridge.ChrSnapshot>? _chrSnapshotSub;

  // Display options
  bool _showTileGrid = true;
  int _selectedPalette = 0; // 0-7: 0-3 BG, 4-7 Sprite
  bool _showSidePanel = true; // Desktop side panel visibility

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

  Future<void> _createTexture() async {
    if (_isCreating) return;
    setState(() {
      _isCreating = true;
      _error = null;
    });

    try {
      final textureId = await _textureService.createAuxTexture(
        id: _chrTextureId,
        width: _width,
        height: _height,
      );

      await _chrSnapshotSub?.cancel();
      _chrSnapshotSub = bridge.chrStateStream().listen((snap) {
        if (!mounted) return;
        // We don't need to setState since the texture is updated directly
      }, onError: (_) {});

      await bridge.setChrPalette(paletteIndex: _selectedPalette);

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
    _textureService.pauseAuxTexture(_chrTextureId);
    unawaited(_chrSnapshotSub?.cancel());
    bridge.unsubscribeChrState();
    _textureService.disposeAuxTexture(_chrTextureId);
    _transformationController.removeListener(_onTransformChanged);
    _transformationController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (_error != null) {
      return _buildErrorState();
    }
    if (_isCreating || _flutterTextureId == null) {
      return const Center(child: CircularProgressIndicator());
    }
    return _buildMainLayout(context);
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
    final tileView = _buildTileView(context);

    if (isNativeDesktop) {
      // Desktop layout with side panel
      return Stack(
        children: [
          Row(
            children: [
              Expanded(child: tileView),
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

    // Mobile layout with popup menu
    return Stack(
      children: [
        tileView,
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
      tooltip: l10n.tileViewerSettings,
      onPressed: () => _showSettingsMenu(context),
    );
  }

  void _showSettingsMenu(BuildContext context) {
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
          enabled: false,
          height: 32,
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: Text(
            l10n.tileViewerOverlays,
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
            builder: (context, setMenuState) => CheckboxListTile(
              dense: true,
              contentPadding: const EdgeInsets.symmetric(horizontal: 16),
              title: Text(l10n.tileViewerShowGrid),
              value: _showTileGrid,
              onChanged: (v) {
                setState(() => _showTileGrid = v ?? false);
                setMenuState(() {});
              },
            ),
          ),
        ),
        const PopupMenuDivider(height: 1),
        // Palette section header
        PopupMenuItem<void>(
          enabled: false,
          height: 32,
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: Text(
            l10n.tileViewerPalette,
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
            builder: (context, setMenuState) => Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
              child: DropdownButtonHideUnderline(
                child: DropdownButton<int>(
                  isDense: true,
                  isExpanded: true,
                  value: _selectedPalette,
                  items: List.generate(8, (i) {
                    final label = i < 4
                        ? l10n.tileViewerPaletteBg(i)
                        : l10n.tileViewerPaletteSprite(i - 4);
                    return DropdownMenuItem(value: i, child: Text(label));
                  }),
                  onChanged: (v) async {
                    if (v == null) return;
                    setState(() => _selectedPalette = v);
                    setMenuState(() {});
                    await bridge.setChrPalette(paletteIndex: v);
                  },
                ),
              ),
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildTileView(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

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
                    child: Stack(
                      children: [
                        Texture(
                          textureId: _flutterTextureId!,
                          filterQuality: FilterQuality.none,
                        ),
                        if (_showTileGrid)
                          CustomPaint(
                            painter: _TileGridPainter(),
                            size: Size.infinite,
                          ),
                      ],
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

  // ───────────────────────────── Desktop Side Panel ─────────────────────────

  Widget _buildDesktopSidePanelWrapper(BuildContext context) {
    return ClipRect(
      child: AnimatedContainer(
        duration: const Duration(milliseconds: 180),
        curve: Curves.easeOut,
        width: _showSidePanel ? 240 : 0,
        child: _showSidePanel ? _buildDesktopSidePanel(context) : null,
      ),
    );
  }

  Widget _buildDesktopSidePanel(BuildContext context) {
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
              title: l10n.tileViewerOverlays,
              child: Column(
                children: [
                  CheckboxListTile(
                    dense: true,
                    visualDensity: VisualDensity.compact,
                    controlAffinity: ListTileControlAffinity.trailing,
                    contentPadding: EdgeInsets.zero,
                    title: Text(l10n.tileViewerShowGrid),
                    value: _showTileGrid,
                    onChanged: (v) {
                      setState(() => _showTileGrid = v ?? false);
                    },
                  ),
                ],
              ),
            ),
            _sideSection(
              context,
              title: l10n.tileViewerPalette,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  DropdownButton<int>(
                    isExpanded: true,
                    value: _selectedPalette,
                    items: List.generate(8, (i) {
                      final label = i < 4
                          ? l10n.tileViewerPaletteBg(i)
                          : l10n.tileViewerPaletteSprite(i - 4);
                      return DropdownMenuItem(value: i, child: Text(label));
                    }),
                    onChanged: (v) async {
                      if (v == null) return;
                      setState(() => _selectedPalette = v);
                      await bridge.setChrPalette(paletteIndex: v);
                    },
                  ),
                ],
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
}

class _TileGridPainter extends CustomPainter {
  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint()
      ..color = Colors.white.withValues(alpha: 0.2)
      ..strokeWidth = 1.0
      ..style = PaintingStyle.stroke;

    // 16 tiles wide × 32 tiles tall
    const tilesX = 16;
    const tilesY = 32;

    final tileWidth = size.width / tilesX;
    final tileHeight = size.height / tilesY;

    // Draw vertical lines
    for (int i = 0; i <= tilesX; i++) {
      final x = i * tileWidth;
      canvas.drawLine(Offset(x, 0), Offset(x, size.height), paint);
    }

    // Draw horizontal lines
    for (int i = 0; i <= tilesY; i++) {
      final y = i * tileHeight;
      canvas.drawLine(Offset(0, y), Offset(size.width, y), paint);
    }

    // Draw separator between pattern tables (after row 16)
    final separatorY = 16 * tileHeight;
    final separatorPaint = Paint()
      ..color = Colors.yellow.withValues(alpha: 0.5)
      ..strokeWidth = 2.0;
    canvas.drawLine(
      Offset(0, separatorY),
      Offset(size.width, separatorY),
      separatorPaint,
    );
  }

  @override
  bool shouldRepaint(covariant CustomPainter oldDelegate) => false;
}
