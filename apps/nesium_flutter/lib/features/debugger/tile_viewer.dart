import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/domain/nes_texture_service.dart';
import 'package:nesium_flutter/l10n/app_localizations.dart';

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
    return IconButton.filled(
      onPressed: () => _showSettingsDialog(context),
      icon: const Icon(Icons.settings),
      tooltip: AppLocalizations.of(context)!.tileViewerSettings,
    );
  }

  Future<void> _showSettingsDialog(BuildContext context) async {
    await showDialog(
      context: context,
      builder: (context) => _TileViewerSettingsDialog(
        showTileGrid: _showTileGrid,
        selectedPalette: _selectedPalette,
        onShowTileGridChanged: (value) {
          setState(() => _showTileGrid = value);
        },
        onPaletteChanged: (value) async {
          setState(() => _selectedPalette = value);
          await bridge.setChrPalette(paletteIndex: value);
        },
      ),
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
                        Texture(textureId: _flutterTextureId!),
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
}

class _TileViewerSettingsDialog extends StatelessWidget {
  final bool showTileGrid;
  final int selectedPalette;
  final ValueChanged<bool> onShowTileGridChanged;
  final ValueChanged<int> onPaletteChanged;

  const _TileViewerSettingsDialog({
    required this.showTileGrid,
    required this.selectedPalette,
    required this.onShowTileGridChanged,
    required this.onPaletteChanged,
  });

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final theme = Theme.of(context);

    return AlertDialog(
      title: Text(l10n.tileViewerSettings),
      content: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(l10n.tileViewerOverlays, style: theme.textTheme.titleSmall),
          const SizedBox(height: 8),
          CheckboxListTile(
            title: Text(l10n.tileViewerShowGrid),
            value: showTileGrid,
            onChanged: (value) => onShowTileGridChanged(value ?? false),
          ),
          const SizedBox(height: 16),
          Text(l10n.tileViewerPalette, style: theme.textTheme.titleSmall),
          const SizedBox(height: 8),
          DropdownButton<int>(
            value: selectedPalette,
            isExpanded: true,
            items: List.generate(8, (i) {
              final label = i < 4
                  ? l10n.tileViewerPaletteBg(i)
                  : l10n.tileViewerPaletteSprite(i - 4);
              return DropdownMenuItem(value: i, child: Text(label));
            }),
            onChanged: (value) {
              if (value != null) onPaletteChanged(value);
            },
          ),
        ],
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: Text(l10n.tileViewerClose),
        ),
      ],
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
