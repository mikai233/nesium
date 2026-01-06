import 'dart:async';
import 'dart:math' as math;

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
import 'package:nesium_flutter/widgets/animated_dropdown_menu.dart';

/// Tile Viewer that displays NES CHR pattern tables via a Flutter Texture.
class TileViewer extends ConsumerStatefulWidget {
  const TileViewer({super.key});

  @override
  ConsumerState<TileViewer> createState() => _TileViewerState();
}

/// Memory source for tile data (matches Rust backend)
enum _TileSource { ppu, chrRom, chrRam, prgRom }

/// Tile layout mode (matches Rust backend)
enum _TileLayout { normal, singleLine8x16, singleLine16x16 }

/// Tile background color (matches Rust backend)
enum _TileBackground {
  defaultBg,
  transparent,
  paletteColor,
  black,
  white,
  magenta,
}

/// All Mesen2-style presets (both source and palette presets)
enum _Preset { ppu, chr, rom, bg, oam }

enum _CaptureMode { frameStart, vblankStart, scanline }

class _TileViewerState extends ConsumerState<TileViewer> {
  static const int _minScanline = -1;
  static const int _maxScanline = 260;
  static const int _minDot = 0;
  static const int _maxDot = 340;

  final NesTextureService _textureService = NesTextureService();
  final ScrollController _sidePanelScrollController = ScrollController();
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

  /// Called when ROM is ejected - reset to PPU preset
  void _onRomEjected() {
    _applyPreset(_Preset.ppu);
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
      _tileSnapshotSub = bridge.tileStateStream().listen((snap) {
        if (!mounted) return;
        // Store snapshot for tooltip data WITHOUT triggering rebuild.
        // Texture updates automatically; setState spam kills performance.
        _tileSnapshot = snap;
      }, onError: (_) {});

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
    _sidePanelScrollController.dispose();
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

  @override
  Widget build(BuildContext context) {
    // Listen for ROM ejection to reset source to PPU
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
      return LayoutBuilder(
        builder: (context, constraints) {
          final size = constraints.biggest;
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
              // Floating hover tooltip (desktop only)
              if (_hoveredTile != null &&
                  _hoverPosition != null &&
                  _tileSnapshot != null)
                _buildHoverTooltip(context, size),
            ],
          );
        },
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

  /// Builds wrap of toggle buttons for presets with i18n labels
  Widget _buildPresetButtons(
    BuildContext context,
    List<_Preset> presets, {
    VoidCallback? onChanged,
  }) {
    final l10n = AppLocalizations.of(context)!;

    String presetLabel(_Preset preset) => switch (preset) {
      _Preset.ppu => l10n.tileViewerPresetPpu,
      _Preset.chr => l10n.tileViewerPresetChr,
      _Preset.rom => l10n.tileViewerPresetRom,
      _Preset.bg => l10n.tileViewerPresetBg,
      _Preset.oam => l10n.tileViewerPresetOam,
    };

    return Wrap(
      spacing: 4,
      runSpacing: 4,
      children: presets.map((preset) {
        final isSelected = _selectedPreset == preset;
        return FilterChip(
          label: Text(presetLabel(preset)),
          selected: isSelected,
          onSelected: (_) {
            _applyPreset(preset);
            onChanged?.call();
          },
          showCheckmark: false,
          visualDensity: VisualDensity.compact,
          labelPadding: const EdgeInsets.symmetric(horizontal: 4),
        );
      }).toList(),
    );
  }

  /// Applies a preset configuration (Mesen2-style)
  Future<void> _applyPreset(_Preset preset) async {
    final snapshot = _tileSnapshot;

    // Determine new settings based on preset
    _TileSource newSource;
    int newAddress;
    int newColumns;
    int newRows;
    _TileLayout newLayout;
    int? newPalette;

    switch (preset) {
      case _Preset.ppu:
        newSource = _TileSource.ppu;
        newAddress = 0;
        newColumns = 16;
        newRows = 32;
        newLayout = _TileLayout.normal;
        newPalette = null; // Keep current palette

      case _Preset.chr:
        // CHR ROM or CHR RAM (backend decides which is available)
        newSource = _TileSource.chrRom;
        newAddress = 0;
        newColumns = 16;
        newRows = 32;
        newLayout = _TileLayout.normal;
        newPalette = null;

      case _Preset.rom:
        newSource = _TileSource.prgRom;
        newAddress = 0;
        newColumns = 16;
        newRows = 32;
        newLayout = _TileLayout.normal;
        newPalette = null;

      case _Preset.bg:
        // BG preset: use Background Pattern Table address from PPU
        newSource = _TileSource.ppu;
        newAddress = snapshot?.bgPatternBase ?? 0;
        newColumns = 16;
        newRows = 16; // Only one pattern table (16x16)
        newLayout = _TileLayout.normal;
        // Force palette to BG range (0-3)
        newPalette = _selectedPalette >= 4 ? 0 : _selectedPalette;

      case _Preset.oam:
        // OAM preset: use Sprite Pattern Table address, handle 8x16 sprites
        newSource = _TileSource.ppu;
        final largeSprites = snapshot?.largeSprites ?? false;
        if (largeSprites) {
          // 8x16 sprite mode: show both tables with SingleLine8x16 layout
          newAddress = 0;
          newColumns = 16;
          newRows = 32;
          newLayout = _TileLayout.singleLine8x16;
        } else {
          // Normal 8x8 sprites: show sprite pattern table only
          newAddress = snapshot?.spritePatternBase ?? 0;
          newColumns = 16;
          newRows = 16;
          newLayout = _TileLayout.normal;
        }
        // Force palette to Sprite range (4-7)
        newPalette = _selectedPalette < 4 ? 4 : _selectedPalette;
    }

    // Check if texture size changed
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

    // Notify backend
    await bridge.setTileViewerSource(source: newSource.index);
    await bridge.setTileViewerStartAddress(startAddress: newAddress);
    await bridge.setTileViewerSize(columns: newColumns, rows: newRows);
    await bridge.setTileViewerLayout(layout: newLayout.index);
    if (newPalette != null) {
      await bridge.setTileViewerPalette(paletteIndex: newPalette);
    }

    // Recreate texture if size changed
    if (needsTextureRecreate) {
      await _recreateTexture();
    }
  }

  /// Clears the preset selection (called when user manually changes settings)
  void _clearPresetSelection() {
    if (_selectedPreset != null) {
      setState(() => _selectedPreset = null);
    }
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
        PopupMenuItem<void>(
          enabled: false,
          height: 32,
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: Text(
            'Presets',
            style: theme.textTheme.labelSmall?.copyWith(
              color: theme.colorScheme.onSurfaceVariant,
              fontWeight: FontWeight.w600,
            ),
          ),
        ),
        PopupMenuItem<void>(
          onTap: () {}, // Empty tap to prevent closing
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          child: StatefulBuilder(
            builder: (context, setMenuState) => _buildPresetButtons(context, [
              _Preset.ppu,
              _Preset.chr,
              _Preset.rom,
            ], onChanged: () => setMenuState(() {})),
          ),
        ),
        PopupMenuItem<void>(
          onTap: () {}, // Empty tap to prevent closing
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          child: StatefulBuilder(
            builder: (context, setMenuState) => _buildPresetButtons(context, [
              _Preset.bg,
              _Preset.oam,
            ], onChanged: () => setMenuState(() {})),
          ),
        ),
        const PopupMenuDivider(),
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
        PopupMenuItem<void>(
          onTap: () {}, // Empty tap to prevent closing
          padding: EdgeInsets.zero,
          child: StatefulBuilder(
            builder: (context, setMenuState) => CheckboxListTile(
              dense: true,
              contentPadding: const EdgeInsets.symmetric(horizontal: 16),
              title: Text(l10n.tileViewerGrayscale),
              value: _useGrayscale,
              onChanged: (v) async {
                final enabled = v ?? false;
                setState(() => _useGrayscale = enabled);
                setMenuState(() {});
                await bridge.setTileViewerDisplayMode(mode: enabled ? 1 : 0);
              },
            ),
          ),
        ),
        const PopupMenuDivider(),
        // Capture section header
        PopupMenuItem<void>(
          enabled: false,
          height: 32,
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: Text(
            l10n.tilemapCapture,
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
                RadioGroup<_CaptureMode>(
                  groupValue: _captureMode,
                  onChanged: (v) {
                    if (v == null) return;
                    setState(() => _captureMode = v);
                    setMenuState(() {});
                    unawaitedLogged(
                      _applyCaptureMode(),
                      message: 'Failed to set CHR capture point',
                    );
                  },
                  child: Column(
                    children: [
                      RadioListTile<_CaptureMode>(
                        dense: true,
                        visualDensity: VisualDensity.compact,
                        contentPadding: const EdgeInsets.symmetric(
                          horizontal: 16,
                        ),
                        title: Text(l10n.tilemapCaptureFrameStart),
                        value: _CaptureMode.frameStart,
                      ),
                      RadioListTile<_CaptureMode>(
                        dense: true,
                        visualDensity: VisualDensity.compact,
                        contentPadding: const EdgeInsets.symmetric(
                          horizontal: 16,
                        ),
                        title: Text(l10n.tilemapCaptureVblankStart),
                        value: _CaptureMode.vblankStart,
                      ),
                      RadioListTile<_CaptureMode>(
                        dense: true,
                        visualDensity: VisualDensity.compact,
                        contentPadding: const EdgeInsets.symmetric(
                          horizontal: 16,
                        ),
                        title: Text(l10n.tilemapCaptureManual),
                        value: _CaptureMode.scanline,
                      ),
                    ],
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.fromLTRB(16, 0, 16, 10),
                  child: Row(
                    children: [
                      Expanded(
                        child: _numberFieldModern(
                          label: l10n.tilemapScanline,
                          enabled: _captureMode == _CaptureMode.scanline,
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
                            setMenuState(() {});
                            _scanlineController.text = _scanline.toString();
                            unawaitedLogged(
                              _applyCaptureMode(),
                              message: 'Failed to set CHR capture point',
                            );
                          },
                        ),
                      ),
                      const SizedBox(width: 10),
                      Expanded(
                        child: _numberFieldModern(
                          label: l10n.tilemapDot,
                          enabled: _captureMode == _CaptureMode.scanline,
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
                            setMenuState(() {});
                            _dotController.text = _dot.toString();
                            unawaitedLogged(
                              _applyCaptureMode(),
                              message: 'Failed to set CHR capture point',
                            );
                          },
                        ),
                      ),
                    ],
                  ),
                ),
              ],
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
          enabled: false,
          padding: EdgeInsets.zero,
          child: StatefulBuilder(
            builder: (context, setMenuState) => Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
              child: AnimatedDropdownMenu<int>(
                density: AnimatedDropdownMenuDensity.compact,
                value: _selectedPalette,
                entries: [
                  for (var i = 0; i < 8; i++)
                    DropdownMenuEntry(
                      value: i,
                      label: i < 4
                          ? l10n.tileViewerPaletteBg(i)
                          : l10n.tileViewerPaletteSprite(i - 4),
                    ),
                ],
                onSelected: (v) async {
                  setState(() => _selectedPalette = v);
                  _clearPresetSelection(); // Manual change clears preset
                  setMenuState(() {});
                  await bridge.setTileViewerPalette(paletteIndex: v);
                },
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
      child: Container(
        decoration: BoxDecoration(
          border: Border.all(color: colorScheme.outlineVariant, width: 1),
          borderRadius: BorderRadius.circular(4),
        ),
        clipBehavior: Clip.antiAlias,
        child: LayoutBuilder(
          builder: (context, constraints) {
            final viewportSize = constraints.biggest;
            if (viewportSize.width <= 0 || viewportSize.height <= 0) {
              return const SizedBox();
            }

            final scale = math.min(
              viewportSize.width / _textureWidth,
              viewportSize.height / _textureHeight,
            );
            final contentSize = Size(
              _textureWidth * scale,
              _textureHeight * scale,
            );
            final contentOffset = Offset(
              (viewportSize.width - contentSize.width) / 2,
              (viewportSize.height - contentSize.height) / 2,
            );

            return MouseRegion(
              onHover: (event) => _handleHover(
                event.localPosition,
                contentOffset: contentOffset,
                contentSize: contentSize,
              ),
              onExit: (_) => _clearHover(),
              child: GestureDetector(
                behavior: HitTestBehavior.opaque,
                onTapDown: (details) => _handleTap(
                  details.localPosition,
                  contentOffset: contentOffset,
                  contentSize: contentSize,
                ),
                child: InteractiveViewer(
                  transformationController: _transformationController,
                  minScale: _minScale,
                  maxScale: _maxScale,
                  panEnabled: true,
                  scaleEnabled: true,
                  boundaryMargin: const EdgeInsets.all(double.infinity),
                  constrained: false,
                  child: SizedBox(
                    width: viewportSize.width,
                    height: viewportSize.height,
                    child: Stack(
                      children: [
                        Positioned(
                          left: contentOffset.dx,
                          top: contentOffset.dy,
                          width: contentSize.width,
                          height: contentSize.height,
                          child: Stack(
                            fit: StackFit.expand,
                            children: [
                              if (_flutterTextureId != null &&
                                  !ViewerSkeletonScope.enabledOf(context))
                                Texture(
                                  textureId: _flutterTextureId!,
                                  filterQuality: FilterQuality.none,
                                )
                              else
                                DecoratedBox(
                                  decoration: BoxDecoration(
                                    color: Theme.of(
                                      context,
                                    ).colorScheme.surfaceContainerHighest,
                                    borderRadius: BorderRadius.circular(12),
                                  ),
                                ),
                              if (_showTileGrid)
                                CustomPaint(painter: _TileGridPainter()),
                              if (_hoveredTile != null || _selectedTile != null)
                                CustomPaint(
                                  painter: _TileHighlightPainter(
                                    hoveredTile: _hoveredTile,
                                    selectedTile: _selectedTile,
                                    tileWidth: contentSize.width / 16,
                                    tileHeight: contentSize.height / 32,
                                  ),
                                ),
                            ],
                          ),
                        ),
                      ],
                    ),
                  ),
                ),
              ),
            );
          },
        ),
      ),
    );
  }

  void _handleHover(
    Offset position, {
    required Offset contentOffset,
    required Size contentSize,
  }) {
    // Transform screen position to child position (handles zoom/pan), then map
    // to the actual CHR texture rect within that child.
    final childPos = _transformToContent(position);
    final contentPos = childPos - contentOffset;
    final tile = _tileAtPosition(contentPos, contentSize);
    if (tile == _hoveredTile && position == _hoverPosition) return;
    setState(() {
      _hoveredTile = tile;
      _hoverPosition = position; // Keep screen position for tooltip
    });
  }

  void _clearHover() {
    if (_hoveredTile == null) return;
    setState(() {
      _hoveredTile = null;
      _hoverPosition = null;
    });
  }

  void _handleTap(
    Offset position, {
    required Offset contentOffset,
    required Size contentSize,
  }) {
    final childPos = _transformToContent(position);
    final contentPos = childPos - contentOffset;
    final tile = _tileAtPosition(contentPos, contentSize);
    setState(() {
      _selectedTile = tile;
    });
  }

  /// Transform screen coordinates to content coordinates
  /// accounting for zoom and pan transformation
  Offset _transformToContent(Offset screenPos) {
    final matrix = _transformationController.value;
    // Apply inverse transformation
    final inverted = Matrix4.inverted(matrix);
    final result = MatrixUtils.transformPoint(inverted, screenPos);
    return result;
  }

  _TileCoord? _tileAtPosition(Offset position, Size contentSize) {
    if (contentSize.width <= 0 || contentSize.height <= 0) return null;
    if (position.dx < 0 ||
        position.dy < 0 ||
        position.dx > contentSize.width ||
        position.dy > contentSize.height) {
      return null;
    }

    final tileWidth = contentSize.width / 16;
    final tileHeight = contentSize.height / 32;
    final x = (position.dx / tileWidth).floor().clamp(0, 15);
    final y = (position.dy / tileHeight).floor().clamp(0, 31);
    return _TileCoord(x, y);
  }

  _TileInfo? _computeTileInfo(_TileCoord tile) {
    final tileIndex = tile.y * 16 + tile.x;
    final patternTable = tileIndex >= 256 ? 1 : 0;
    final tileIndexInTable = tileIndex % 256;
    final chrAddress = patternTable * 0x1000 + tileIndexInTable * 16;

    return _TileInfo(
      tileIndex: tileIndex,
      patternTable: patternTable,
      tileIndexInTable: tileIndexInTable,
      chrAddress: chrAddress,
    );
  }

  /// Floating hover tooltip for desktop - positioned near cursor
  Widget _buildHoverTooltip(BuildContext context, Size size) {
    final tile = _hoveredTile;
    final snap = _tileSnapshot;
    final pos = _hoverPosition;
    if (tile == null || snap == null || pos == null) return const SizedBox();

    final info = _computeTileInfo(tile);
    if (info == null) return const SizedBox();

    const tooltipWidth = 220.0;
    const tooltipHeight = 130.0; // Estimated actual height
    const cursorOffset = 16.0;

    final preferRight = pos.dx < size.width * 0.55;
    final preferDown = pos.dy < size.height * 0.5;

    // Calculate position with consistent offset from cursor
    final dxCandidate = preferRight
        ? pos.dx + cursorOffset
        : pos.dx - tooltipWidth - cursorOffset;
    final dyCandidate = preferDown
        ? pos.dy + cursorOffset
        : pos.dy - tooltipHeight - cursorOffset;

    final dx = dxCandidate.clamp(8.0, size.width - tooltipWidth - 8.0);
    final dy = dyCandidate.clamp(8.0, size.height - tooltipHeight - 8.0);

    return Positioned(
      left: dx,
      top: dy,
      child: SizedBox(
        width: tooltipWidth,
        child: _buildTileHoverCard(context, info: info, snapshot: snap),
      ),
    );
  }

  Widget _buildTileHoverCard(
    BuildContext context, {
    required _TileInfo info,
    required bridge.TileSnapshot snapshot,
  }) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

    return Card(
      elevation: 4,
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
            _ChrTilePreview(snapshot: snapshot, info: info),
            const SizedBox(width: 12),
            Expanded(
              child: DefaultTextStyle(
                style: theme.textTheme.bodySmall!,
                child: _TileInfoTable(context: context, info: info),
              ),
            ),
          ],
        ),
      ),
    );
  }

  // ───────────────────────────── Desktop Side Panel ─────────────────────────

  Widget _buildDesktopSidePanelWrapper(BuildContext context) {
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
          child: _buildDesktopSidePanel(context),
        ),
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
        controller: _sidePanelScrollController,
        thumbVisibility: true,
        child: ListView(
          controller: _sidePanelScrollController,
          padding: const EdgeInsets.all(12),
          children: [
            _sideSection(
              context,
              title: l10n.tileViewerPresets,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  _buildPresetButtons(context, [
                    _Preset.ppu,
                    _Preset.chr,
                    _Preset.rom,
                  ]),
                  const SizedBox(height: 8),
                  _buildPresetButtons(context, [_Preset.bg, _Preset.oam]),
                ],
              ),
            ),
            _sideSection(
              context,
              title: l10n.tilemapCapture,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  RadioGroup<_CaptureMode>(
                    groupValue: _captureMode,
                    onChanged: (v) {
                      if (v == null) return;
                      setState(() => _captureMode = v);
                      unawaitedLogged(
                        _applyCaptureMode(),
                        message: 'Failed to set CHR capture point',
                      );
                    },
                    child: Column(
                      children: [
                        RadioListTile<_CaptureMode>(
                          dense: true,
                          visualDensity: VisualDensity.compact,
                          contentPadding: EdgeInsets.zero,
                          title: Text(l10n.tilemapCaptureFrameStart),
                          value: _CaptureMode.frameStart,
                        ),
                        RadioListTile<_CaptureMode>(
                          dense: true,
                          visualDensity: VisualDensity.compact,
                          contentPadding: EdgeInsets.zero,
                          title: Text(l10n.tilemapCaptureVblankStart),
                          value: _CaptureMode.vblankStart,
                        ),
                        RadioListTile<_CaptureMode>(
                          dense: true,
                          visualDensity: VisualDensity.compact,
                          contentPadding: EdgeInsets.zero,
                          title: Text(l10n.tilemapCaptureManual),
                          value: _CaptureMode.scanline,
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
                          enabled: _captureMode == _CaptureMode.scanline,
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
                            unawaitedLogged(
                              _applyCaptureMode(),
                              message: 'Failed to set CHR capture point',
                            );
                          },
                        ),
                      ),
                      const SizedBox(width: 10),
                      Expanded(
                        child: _numberFieldModern(
                          label: l10n.tilemapDot,
                          enabled: _captureMode == _CaptureMode.scanline,
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
                            unawaitedLogged(
                              _applyCaptureMode(),
                              message: 'Failed to set CHR capture point',
                            );
                          },
                        ),
                      ),
                    ],
                  ),
                ],
              ),
            ),
            // Source selector
            _sideSection(
              context,
              title: l10n.tileViewerSource,
              child: AnimatedDropdownMenu<_TileSource>(
                density: AnimatedDropdownMenuDensity.compact,
                value: _source,
                entries: [
                  DropdownMenuEntry(
                    value: _TileSource.ppu,
                    label: l10n.tileViewerSourcePpu,
                  ),
                  DropdownMenuEntry(
                    value: _TileSource.chrRom,
                    label: l10n.tileViewerSourceChrRom,
                  ),
                  DropdownMenuEntry(
                    value: _TileSource.chrRam,
                    label: l10n.tileViewerSourceChrRam,
                  ),
                  DropdownMenuEntry(
                    value: _TileSource.prgRom,
                    label: l10n.tileViewerSourcePrgRom,
                  ),
                ],
                onSelected: (v) async {
                  setState(() => _source = v);
                  await bridge.setTileViewerSource(source: v.index);
                },
              ),
            ),
            // Address input
            _sideSection(
              context,
              title: l10n.tileViewerAddress,
              child: _AddressInput(
                value: _startAddress,
                maxValue: _maxAddress,
                pageIncrement: _addressIncrement,
                byteIncrement: 1,
                onChanged: (v) async {
                  setState(() => _startAddress = v);
                  await bridge.setTileViewerStartAddress(startAddress: v);
                },
              ),
            ),
            // Size selector (columns × rows)
            _sideSection(
              context,
              title: l10n.tileViewerSize,
              child: Row(
                children: [
                  Expanded(
                    child: _SizeInput(
                      label: l10n.tileViewerColumns,
                      value: _columnCount,
                      min: 4,
                      max: 256,
                      step: _layout == _TileLayout.normal ? 1 : 2,
                      onChanged: (v) => _updateSize(columns: v),
                    ),
                  ),
                  const SizedBox(width: 8),
                  Expanded(
                    child: _SizeInput(
                      label: l10n.tileViewerRows,
                      value: _rowCount,
                      min: 4,
                      max: 256,
                      step: _layout == _TileLayout.normal ? 1 : 2,
                      onChanged: (v) => _updateSize(rows: v),
                    ),
                  ),
                ],
              ),
            ),
            // Layout selector
            _sideSection(
              context,
              title: l10n.tileViewerLayout,
              child: AnimatedDropdownMenu<_TileLayout>(
                density: AnimatedDropdownMenuDensity.compact,
                value: _layout,
                entries: [
                  DropdownMenuEntry(
                    value: _TileLayout.normal,
                    label: l10n.tileViewerLayoutNormal,
                  ),
                  DropdownMenuEntry(
                    value: _TileLayout.singleLine8x16,
                    label: l10n.tileViewerLayout8x16,
                  ),
                  DropdownMenuEntry(
                    value: _TileLayout.singleLine16x16,
                    label: l10n.tileViewerLayout16x16,
                  ),
                ],
                onSelected: (v) async {
                  setState(() => _layout = v);
                  await bridge.setTileViewerLayout(layout: v.index);
                },
              ),
            ),
            // Background selector
            _sideSection(
              context,
              title: l10n.tileViewerBackground,
              child: AnimatedDropdownMenu<_TileBackground>(
                density: AnimatedDropdownMenuDensity.compact,
                value: _background,
                entries: [
                  DropdownMenuEntry(
                    value: _TileBackground.defaultBg,
                    label: l10n.tileViewerBgDefault,
                  ),
                  DropdownMenuEntry(
                    value: _TileBackground.transparent,
                    label: l10n.tileViewerBgTransparent,
                  ),
                  DropdownMenuEntry(
                    value: _TileBackground.paletteColor,
                    label: l10n.tileViewerBgPalette,
                  ),
                  DropdownMenuEntry(
                    value: _TileBackground.black,
                    label: l10n.tileViewerBgBlack,
                  ),
                  DropdownMenuEntry(
                    value: _TileBackground.white,
                    label: l10n.tileViewerBgWhite,
                  ),
                  DropdownMenuEntry(
                    value: _TileBackground.magenta,
                    label: l10n.tileViewerBgMagenta,
                  ),
                ],
                onSelected: (v) async {
                  setState(() => _background = v);
                  await bridge.setTileViewerBackground(background: v.index);
                },
              ),
            ),
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
                  CheckboxListTile(
                    dense: true,
                    visualDensity: VisualDensity.compact,
                    controlAffinity: ListTileControlAffinity.trailing,
                    contentPadding: EdgeInsets.zero,
                    title: Text(l10n.tileViewerGrayscale),
                    value: _useGrayscale,
                    onChanged: (v) async {
                      final enabled = v ?? false;
                      setState(() => _useGrayscale = enabled);
                      await bridge.setTileViewerDisplayMode(
                        mode: enabled ? 1 : 0,
                      );
                    },
                  ),
                ],
              ),
            ),
            _sideSection(
              context,
              title: l10n.tileViewerPalette,
              child: AnimatedDropdownMenu<int>(
                density: AnimatedDropdownMenuDensity.compact,
                value: _selectedPalette,
                entries: [
                  for (var i = 0; i < 8; i++)
                    DropdownMenuEntry(
                      value: i,
                      label: i < 4
                          ? l10n.tileViewerPaletteBg(i)
                          : l10n.tileViewerPaletteSprite(i - 4),
                    ),
                ],
                onSelected: (v) async {
                  setState(() => _selectedPalette = v);
                  _clearPresetSelection(); // Manual change clears preset
                  await bridge.setTileViewerPalette(paletteIndex: v);
                },
              ),
            ),
            // Selected Tile Info (only shows on tap/click, not hover)
            if (_selectedTile != null && _tileSnapshot != null)
              _sideSection(
                context,
                title: l10n.tileViewerSelectedTile,
                child: _buildTileInfoCard(
                  context,
                  _selectedTile!,
                  _tileSnapshot!,
                ),
              ),
          ],
        ),
      ),
    );
  }

  /// Side panel tile info with preview (matching TilemapViewer TileInfoCard)
  Widget _buildTileInfoCard(
    BuildContext context,
    _TileCoord tile,
    bridge.TileSnapshot snapshot,
  ) {
    final info = _computeTileInfo(tile);
    if (info == null) return const SizedBox.shrink();

    final theme = Theme.of(context);
    final l10n = AppLocalizations.of(context)!;
    final labelStyle = theme.textTheme.bodySmall?.copyWith(
      color: theme.colorScheme.onSurfaceVariant,
    );
    final valueStyle = theme.textTheme.bodySmall?.copyWith(
      fontWeight: FontWeight.w600,
      fontFamily: 'monospace',
    );

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            SizedBox(
              width: 76,
              child: _ChrTilePreview(snapshot: snapshot, info: info),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: Column(
                children: [
                  _infoRow(
                    l10n.tileViewerPatternTable,
                    '${info.patternTable}',
                    labelStyle,
                    valueStyle,
                  ),
                  _infoRow(
                    l10n.tileViewerTileIndex,
                    '\$${info.tileIndexInTable.toRadixString(16).toUpperCase().padLeft(2, '0')}',
                    labelStyle,
                    valueStyle,
                  ),
                  _infoRow(
                    l10n.tileViewerChrAddress,
                    '\$${info.chrAddress.toRadixString(16).toUpperCase().padLeft(4, '0')}',
                    labelStyle,
                    valueStyle,
                  ),
                ],
              ),
            ),
          ],
        ),
      ],
    );
  }

  Widget _infoRow(
    String label,
    String value,
    TextStyle? labelStyle,
    TextStyle? valueStyle,
  ) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Row(
        children: [
          Expanded(
            child: Text(
              label,
              style: labelStyle,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
            ),
          ),
          const SizedBox(width: 8),
          Text(value, style: valueStyle),
        ],
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

/// Tile coordinate in the CHR grid (0-15 for x, 0-31 for y)
class _TileCoord {
  final int x;
  final int y;

  const _TileCoord(this.x, this.y);

  @override
  bool operator ==(Object other) =>
      identical(this, other) ||
      other is _TileCoord && x == other.x && y == other.y;

  @override
  int get hashCode => x.hashCode ^ y.hashCode;
}

/// Tile information for tooltip display
class _TileInfo {
  final int tileIndex; // 0-511 (global index across both pattern tables)
  final int patternTable; // 0 or 1
  final int tileIndexInTable; // 0-255 (index within the pattern table)
  final int chrAddress; // CHR address ($0000-$1FFF)

  const _TileInfo({
    required this.tileIndex,
    required this.patternTable,
    required this.tileIndexInTable,
    required this.chrAddress,
  });
}

/// Paints highlight for hovered and selected tiles
class _TileHighlightPainter extends CustomPainter {
  final _TileCoord? hoveredTile;
  final _TileCoord? selectedTile;
  final double tileWidth;
  final double tileHeight;

  _TileHighlightPainter({
    this.hoveredTile,
    this.selectedTile,
    required this.tileWidth,
    required this.tileHeight,
  });

  @override
  void paint(Canvas canvas, Size size) {
    // Draw hover highlight
    if (hoveredTile != null) {
      final rect = Rect.fromLTWH(
        hoveredTile!.x * tileWidth,
        hoveredTile!.y * tileHeight,
        tileWidth,
        tileHeight,
      );
      final paint = Paint()
        ..color = Colors.cyan.withValues(alpha: 0.3)
        ..style = PaintingStyle.fill;
      canvas.drawRect(rect, paint);

      final borderPaint = Paint()
        ..color = Colors.cyan
        ..strokeWidth = 1.5
        ..style = PaintingStyle.stroke;
      canvas.drawRect(rect, borderPaint);
    }

    // Draw selection highlight (stronger)
    if (selectedTile != null && selectedTile != hoveredTile) {
      final rect = Rect.fromLTWH(
        selectedTile!.x * tileWidth,
        selectedTile!.y * tileHeight,
        tileWidth,
        tileHeight,
      );
      final paint = Paint()
        ..color = Colors.yellow.withValues(alpha: 0.4)
        ..style = PaintingStyle.fill;
      canvas.drawRect(rect, paint);

      final borderPaint = Paint()
        ..color = Colors.yellow
        ..strokeWidth = 2.0
        ..style = PaintingStyle.stroke;
      canvas.drawRect(rect, borderPaint);
    }
  }

  @override
  bool shouldRepaint(covariant _TileHighlightPainter oldDelegate) =>
      hoveredTile != oldDelegate.hoveredTile ||
      selectedTile != oldDelegate.selectedTile;
}

// ─────────────────────────────────────────────────────────────────────────────
// Tile Preview and Info Widgets (matching TilemapViewer style)
// ─────────────────────────────────────────────────────────────────────────────

/// CHR tile preview showing zoomed 8×8 tile with palette colors
class _ChrTilePreview extends StatelessWidget {
  const _ChrTilePreview({required this.snapshot, required this.info});

  final bridge.TileSnapshot snapshot;
  final _TileInfo info;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Container(
          width: 64,
          height: 64,
          decoration: BoxDecoration(
            border: Border.all(
              color: Theme.of(context).colorScheme.outlineVariant,
            ),
            borderRadius: BorderRadius.circular(4),
          ),
          child: CustomPaint(
            painter: _ChrTilePreviewPainter(snapshot: snapshot, info: info),
          ),
        ),
        const SizedBox(height: 8),
        _PaletteStrip(snapshot: snapshot),
      ],
    );
  }
}

class _ChrTilePreviewPainter extends CustomPainter {
  _ChrTilePreviewPainter({required this.snapshot, required this.info});

  final bridge.TileSnapshot snapshot;
  final _TileInfo info;

  @override
  void paint(Canvas canvas, Size size) {
    final paint = Paint();
    final paletteIndex = snapshot.selectedPalette.clamp(0, 7);
    final palBase = paletteIndex < 4
        ? paletteIndex * 4
        : 0x10 + (paletteIndex - 4) * 4;

    // Draw 2x2 grid showing the 4 palette colors as a placeholder
    final cellW = size.width / 2;
    final cellH = size.height / 2;

    for (var i = 0; i < 4; i++) {
      final pal = snapshot.palette;
      final idx = i == 0 ? 0 : palBase + i;
      final nesColor = (idx < pal.length ? pal[idx] : 0) & 0x3F;
      paint.color = _colorFromNes(nesColor);

      final x = (i % 2) * cellW;
      final y = (i ~/ 2) * cellH;
      canvas.drawRect(Rect.fromLTWH(x, y, cellW, cellH), paint);
    }

    // Draw tile index in center
    final textPainter = TextPainter(
      text: TextSpan(
        text:
            '\$${info.tileIndexInTable.toRadixString(16).toUpperCase().padLeft(2, '0')}',
        style: TextStyle(
          color: Colors.white,
          fontSize: size.width / 4,
          fontFamily: 'monospace',
          fontWeight: FontWeight.bold,
          shadows: const [Shadow(blurRadius: 2, color: Colors.black)],
        ),
      ),
      textDirection: TextDirection.ltr,
    )..layout();
    textPainter.paint(
      canvas,
      Offset(
        (size.width - textPainter.width) / 2,
        (size.height - textPainter.height) / 2,
      ),
    );
  }

  Color _colorFromNes(int nesColor) {
    final rgba = snapshot.rgbaPalette;
    final base = (nesColor & 0x3F) * 4;
    if (base + 3 >= rgba.length) return Colors.black;
    return Color.fromARGB(
      rgba[base + 3],
      rgba[base],
      rgba[base + 1],
      rgba[base + 2],
    );
  }

  @override
  bool shouldRepaint(covariant _ChrTilePreviewPainter old) =>
      info.tileIndex != old.info.tileIndex ||
      snapshot.selectedPalette != old.snapshot.selectedPalette;
}

/// Shows the 4 colors of the selected palette
class _PaletteStrip extends StatelessWidget {
  const _PaletteStrip({required this.snapshot});

  final bridge.TileSnapshot snapshot;

  @override
  Widget build(BuildContext context) {
    final paletteIndex = snapshot.selectedPalette.clamp(0, 7);
    final palBase = paletteIndex < 4
        ? paletteIndex * 4
        : 0x10 + (paletteIndex - 4) * 4;

    return Row(
      children: List.generate(4, (i) {
        final pal = snapshot.palette;
        final idx = palBase + i;
        final nesColor =
            (i == 0
                ? (pal.isNotEmpty ? pal[0] : 0)
                : (idx < pal.length ? pal[idx] : 0)) &
            0x3F;
        final rgba = snapshot.rgbaPalette;
        final base = nesColor * 4;
        final color = base + 3 < rgba.length
            ? Color.fromARGB(
                rgba[base + 3],
                rgba[base],
                rgba[base + 1],
                rgba[base + 2],
              )
            : Colors.black;

        return Container(width: 16, height: 8, color: color);
      }),
    );
  }
}

/// Table showing tile metadata
class _TileInfoTable extends StatelessWidget {
  const _TileInfoTable({required this.context, required this.info});

  final BuildContext context;
  final _TileInfo info;

  @override
  Widget build(BuildContext ctx) {
    final l10n = AppLocalizations.of(context)!;
    final theme = Theme.of(context);
    final labelStyle = theme.textTheme.bodySmall?.copyWith(
      color: theme.colorScheme.onSurfaceVariant,
    );
    final valueStyle = theme.textTheme.bodySmall?.copyWith(
      fontWeight: FontWeight.w600,
      fontFamily: 'monospace',
    );

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        _kv(
          l10n.tileViewerPatternTable,
          '${info.patternTable}',
          labelStyle,
          valueStyle,
        ),
        _kv(
          l10n.tileViewerTileIndex,
          '\$${info.tileIndexInTable.toRadixString(16).toUpperCase().padLeft(2, '0')}',
          labelStyle,
          valueStyle,
        ),
        _kv(
          l10n.tileViewerChrAddress,
          '\$${info.chrAddress.toRadixString(16).toUpperCase().padLeft(4, '0')}',
          labelStyle,
          valueStyle,
        ),
      ],
    );
  }

  Widget _kv(
    String label,
    String value,
    TextStyle? labelStyle,
    TextStyle? valueStyle,
  ) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 3),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: labelStyle),
          Text(value, style: valueStyle),
        ],
      ),
    );
  }
}

/// Hex address input with 4-button navigation (Mesen2 style)
/// Buttons: << (prev page), < (prev byte), [value], > (next byte), >> (next page)
class _AddressInput extends StatelessWidget {
  const _AddressInput({
    required this.value,
    required this.maxValue,
    required this.pageIncrement,
    required this.onChanged,
    this.byteIncrement = 1,
  });

  final int value;
  final int maxValue;
  final int pageIncrement; // Large step (page)
  final int byteIncrement; // Small step (byte/tile)
  final ValueChanged<int> onChanged;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final hexValue =
        '\$${value.toRadixString(16).toUpperCase().padLeft(4, '0')}';

    Widget navButton(String label, int delta, {bool enabled = true}) {
      return InkWell(
        onTap: enabled
            ? () => onChanged((value + delta).clamp(0, maxValue))
            : null,
        borderRadius: BorderRadius.circular(4),
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 6, vertical: 4),
          child: Text(
            label,
            style: theme.textTheme.bodyMedium?.copyWith(
              fontFamily: 'monospace',
              fontWeight: FontWeight.bold,
              color: enabled
                  ? theme.colorScheme.primary
                  : theme.colorScheme.onSurface.withValues(alpha: 0.3),
            ),
          ),
        ),
      );
    }

    return Row(
      children: [
        navButton('«', -pageIncrement, enabled: value > 0),
        navButton('<', -byteIncrement, enabled: value > 0),
        Expanded(
          child: Container(
            padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
            decoration: BoxDecoration(
              border: Border.all(color: theme.colorScheme.outlineVariant),
              borderRadius: BorderRadius.circular(4),
            ),
            child: Text(
              hexValue,
              textAlign: TextAlign.center,
              style: theme.textTheme.bodyMedium?.copyWith(
                fontFamily: 'monospace',
                fontWeight: FontWeight.w500,
              ),
            ),
          ),
        ),
        // Next byte > (Mesen2: CanIncrementSmall = Value < Maximum)
        navButton('>', byteIncrement, enabled: value < maxValue),
        // Next page >> (Mesen2: CanIncrementLarge = Value < Maximum - LargeIncrement + 1)
        navButton(
          '»',
          pageIncrement,
          enabled: value < maxValue - pageIncrement + 1,
        ),
      ],
    );
  }
}

/// Compact numeric input with label for size controls
class _SizeInput extends StatelessWidget {
  const _SizeInput({
    required this.label,
    required this.value,
    required this.min,
    required this.max,
    required this.step,
    required this.onChanged,
  });

  final String label;
  final int value;
  final int min;
  final int max;
  final int step;
  final ValueChanged<int> onChanged;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        Text(
          label,
          style: theme.textTheme.labelSmall?.copyWith(
            color: theme.colorScheme.onSurfaceVariant,
          ),
        ),
        const SizedBox(height: 4),
        Row(
          children: [
            InkWell(
              onTap: value > min
                  ? () => onChanged((value - step).clamp(min, max))
                  : null,
              borderRadius: BorderRadius.circular(4),
              child: Container(
                padding: const EdgeInsets.all(4),
                child: Icon(
                  Icons.remove,
                  size: 16,
                  color: value > min
                      ? theme.colorScheme.onSurface
                      : theme.colorScheme.onSurface.withValues(alpha: 0.3),
                ),
              ),
            ),
            Expanded(
              child: Text(
                '$value',
                textAlign: TextAlign.center,
                style: theme.textTheme.bodyMedium?.copyWith(
                  fontWeight: FontWeight.w600,
                ),
              ),
            ),
            InkWell(
              onTap: value < max
                  ? () => onChanged((value + step).clamp(min, max))
                  : null,
              borderRadius: BorderRadius.circular(4),
              child: Container(
                padding: const EdgeInsets.all(4),
                child: Icon(
                  Icons.add,
                  size: 16,
                  color: value < max
                      ? theme.colorScheme.onSurface
                      : theme.colorScheme.onSurface.withValues(alpha: 0.3),
                ),
              ),
            ),
          ],
        ),
      ],
    );
  }
}
