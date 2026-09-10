part of '../tile_viewer.dart';

mixin _TileViewerUiMixin on _TileViewerStateBase {
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
                          hint:
                              '${_TileViewerStateBase._minScanline} ~ ${_TileViewerStateBase._maxScanline}',
                          onSubmitted: (v) {
                            final value = int.tryParse(v);
                            if (value == null ||
                                value < _TileViewerStateBase._minScanline ||
                                value > _TileViewerStateBase._maxScanline) {
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
                          hint:
                              '${_TileViewerStateBase._minDot} ~ ${_TileViewerStateBase._maxDot}',
                          onSubmitted: (v) {
                            final value = int.tryParse(v);
                            if (value == null ||
                                value < _TileViewerStateBase._minDot ||
                                value > _TileViewerStateBase._maxDot) {
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
                  minScale: _TileViewerStateBase._minScale,
                  maxScale: _TileViewerStateBase._maxScale,
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
      child: SinglePositionScrollbar(
        thumbVisibility: true,
        builder: (context, controller) {
          return ListView(
            controller: controller,
            primary: false,
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
                            hint:
                                '${_TileViewerStateBase._minScanline} ~ ${_TileViewerStateBase._maxScanline}',
                            onSubmitted: (v) {
                              final value = int.tryParse(v);
                              if (value == null ||
                                  value < _TileViewerStateBase._minScanline ||
                                  value > _TileViewerStateBase._maxScanline) {
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
                            hint:
                                '${_TileViewerStateBase._minDot} ~ ${_TileViewerStateBase._maxDot}',
                            onSubmitted: (v) {
                              final value = int.tryParse(v);
                              if (value == null ||
                                  value < _TileViewerStateBase._minDot ||
                                  value > _TileViewerStateBase._maxDot) {
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
          );
        },
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
