part of '../tilemap_viewer.dart';

mixin _TilemapViewerUiMixin on _TilemapViewerStateBase {
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
    final tilemap = _buildTilemapView(context);

    if (isNativeDesktop) {
      return Stack(
        children: [
          Row(
            children: [
              Expanded(child: tilemap),
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

    // Mobile layout - wrap in GestureDetector to dismiss tooltip on tap outside
    return GestureDetector(
      onTap: _clearSelection,
      behavior: HitTestBehavior.translucent,
      child: Stack(
        children: [
          tilemap,
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
      ),
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

  Widget _buildTilemapView(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    // Desktop: show tooltip on hover
    final showHoverTooltip =
        isNativeDesktop && _hoveredTile != null && _tilemapSnapshot != null;
    // Mobile: show tooltip on tap selection
    final showSelectedTooltip =
        !isNativeDesktop && _selectedTile != null && _tilemapSnapshot != null;

    return Container(
      color: colorScheme.surfaceContainerLowest,
      child: Center(
        child: AspectRatio(
          aspectRatio:
              _TilemapViewerStateBase._width / _TilemapViewerStateBase._height,
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
                  minScale: _TilemapViewerStateBase._minScale,
                  maxScale: _TilemapViewerStateBase._maxScale,
                  panEnabled: true,
                  scaleEnabled: true,
                  // Allow panning beyond boundaries when zoomed in
                  boundaryMargin: const EdgeInsets.all(double.infinity),
                  constrained: false,
                  child: SizedBox(
                    width: size.width,
                    height: size.height,
                    child: MouseRegion(
                      onHover: (event) =>
                          _handleHoverWithTransform(event.localPosition, size),
                      onExit: (_) => setState(() {
                        _hoveredTile = null;
                        _hoverPosition = null;
                      }),
                      child: GestureDetector(
                        behavior: HitTestBehavior.opaque,
                        onTapDown: (details) => _handleTapWithTransform(
                          details.localPosition,
                          size,
                        ),
                        child: Stack(
                          children: [
                            if (_flutterTextureId != null &&
                                !ViewerSkeletonScope.enabledOf(context))
                              Texture(
                                textureId: _flutterTextureId!,
                                filterQuality: FilterQuality.none,
                              )
                            else
                              Positioned.fill(
                                child: DecoratedBox(
                                  decoration: BoxDecoration(
                                    color: Theme.of(
                                      context,
                                    ).colorScheme.surfaceContainerHighest,
                                    borderRadius: BorderRadius.circular(12),
                                  ),
                                ),
                              ),
                            // Use ValueListenableBuilder for scroll overlay - isolated repaint
                            ValueListenableBuilder<List<Rect>>(
                              valueListenable: _scrollOverlayRects,
                              builder: (context, scrollRects, _) {
                                return CustomPaint(
                                  painter: _TilemapGridPainter(
                                    showTileGrid: _showTileGrid,
                                    showAttributeGrid: _showAttributeGrid,
                                    showAttributeGrid32: _showAttributeGrid32,
                                    showNametableDelimiters:
                                        _showNametableDelimiters,
                                    showScrollOverlay: _showScrollOverlay,
                                    scrollOverlayRects: scrollRects,
                                    hoveredTile: _hoveredTile,
                                    selectedTile: _selectedTile,
                                  ),
                                  size: Size.infinite,
                                );
                              },
                            ),
                            if (showHoverTooltip)
                              _buildHoverTooltip(context, size),
                            if (showSelectedTooltip)
                              _buildSelectedTooltip(context, size),
                          ],
                        ),
                      ),
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

  /// Transform local position considering zoom/pan transformation
  Offset _transformPosition(Offset localPosition) {
    // The localPosition is already in the transformed (zoomed/panned) space
    // For InteractiveViewer with constrained: false, localPosition is
    // already relative to the child content
    return localPosition;
  }

  void _handleHoverWithTransform(Offset localPosition, Size size) {
    final transformedPosition = _transformPosition(localPosition);
    _handleHover(transformedPosition, size);
  }

  void _handleTapWithTransform(Offset localPosition, Size size) {
    final transformedPosition = _transformPosition(localPosition);
    _handleTap(transformedPosition, size);
  }

  void _handleHover(Offset position, Size size) {
    final tile = _tileAtPosition(position, size);
    if (tile == _hoveredTile && position == _hoverPosition) return;
    setState(() {
      _hoveredTile = tile;
      _hoverPosition = position;
    });
  }

  void _handleTap(Offset position, Size size) {
    final tile = _tileAtPosition(position, size);
    setState(() {
      _selectedTile = tile;
      _selectedPosition = position;
    });
  }

  _TileCoord? _tileAtPosition(Offset position, Size size) {
    if (size.width <= 0 || size.height <= 0) return null;
    if (position.dx < 0 ||
        position.dy < 0 ||
        position.dx > size.width ||
        position.dy > size.height) {
      return null;
    }

    final x = (position.dx / size.width) * _TilemapViewerStateBase._width;
    final y = (position.dy / size.height) * _TilemapViewerStateBase._height;
    final tileX = (x / 8).floor().clamp(0, 63);
    final tileY = (y / 8).floor().clamp(0, 59);
    return _TileCoord(tileX, tileY);
  }

  Widget _buildHoverTooltip(BuildContext context, Size size) {
    final tile = _hoveredTile;
    final snap = _tilemapSnapshot;
    final pos = _hoverPosition;
    if (tile == null || snap == null || pos == null) return const SizedBox();

    final info = _computeTileInfo(snap, tile);
    if (info == null) return const SizedBox();

    const tooltipWidth = 320.0;
    final maxAllowedHeight = (size.height - 16).clamp(140.0, 420.0);
    final tooltipHeight = _lastHoverTooltipSize.height.clamp(
      120.0,
      maxAllowedHeight,
    );

    final preferRight = pos.dx < size.width * 0.55;
    final preferDown = pos.dy < size.height * 0.55;

    final dxCandidate = preferRight ? pos.dx + 16 : pos.dx - tooltipWidth - 16;
    final dyCandidate = preferDown ? pos.dy + 16 : pos.dy - tooltipHeight - 16;

    final dx = dxCandidate.clamp(8.0, size.width - tooltipWidth - 8.0);
    final dy = dyCandidate.clamp(8.0, size.height - tooltipHeight - 8.0);

    _scheduleHoverTooltipMeasure();

    return Positioned(
      left: dx,
      top: dy,
      child: ConstrainedBox(
        constraints: BoxConstraints(maxHeight: maxAllowedHeight),
        child: SizedBox(
          width: tooltipWidth,
          child: KeyedSubtree(
            key: _hoverTooltipKey,
            child: _buildTileHoverCard(snapshot: snap, info: info),
          ),
        ),
      ),
    );
  }

  /// Tooltip for mobile - shows on tap selection
  Widget _buildSelectedTooltip(BuildContext context, Size size) {
    final tile = _selectedTile;
    final snap = _tilemapSnapshot;
    final pos = _selectedPosition;
    if (tile == null || snap == null || pos == null) return const SizedBox();

    final info = _computeTileInfo(snap, tile);
    if (info == null) return const SizedBox();

    const tooltipWidth = 280.0;
    final maxAllowedHeight = (size.height - 16).clamp(140.0, 380.0);
    final tooltipHeight = _lastHoverTooltipSize.height.clamp(
      120.0,
      maxAllowedHeight,
    );

    final preferRight = pos.dx < size.width * 0.55;
    final preferDown = pos.dy < size.height * 0.55;

    final dxCandidate = preferRight ? pos.dx + 16 : pos.dx - tooltipWidth - 16;
    final dyCandidate = preferDown ? pos.dy + 16 : pos.dy - tooltipHeight - 16;

    final dx = dxCandidate.clamp(8.0, size.width - tooltipWidth - 8.0);
    final dy = dyCandidate.clamp(8.0, size.height - tooltipHeight - 8.0);

    return Positioned(
      left: dx,
      top: dy,
      child: ConstrainedBox(
        constraints: BoxConstraints(maxHeight: maxAllowedHeight),
        child: SizedBox(
          width: tooltipWidth,
          child: _buildTileHoverCard(snapshot: snap, info: info),
        ),
      ),
    );
  }

  void _scheduleHoverTooltipMeasure() {
    if (_hoverTooltipMeasurePending) return;
    _hoverTooltipMeasurePending = true;
    WidgetsBinding.instance.addPostFrameCallback((_) {
      _hoverTooltipMeasurePending = false;
      if (!mounted) return;
      final size = _hoverTooltipKey.currentContext?.size;
      if (size == null || size == Size.zero) return;
      if (size == _lastHoverTooltipSize) return;
      setState(() => _lastHoverTooltipSize = size);
    });
  }

  Widget _buildTileHoverCard({
    required bridge.TilemapSnapshot snapshot,
    required _TileInfo info,
  }) {
    return Card(
      clipBehavior: Clip.antiAlias,
      elevation: 8,
      child: SingleChildScrollView(
        physics: const ClampingScrollPhysics(),
        child: Padding(
          padding: const EdgeInsets.all(12),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              _TilePreview(snapshot: snapshot, info: info),
              const SizedBox(width: 12),
              Expanded(
                child: DefaultTextStyle(
                  style: Theme.of(context).textTheme.bodySmall!,
                  child: _TileInfoTable(info: info),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }

  Widget _buildDesktopSidePanel(BuildContext context) {
    final snap = _tilemapSnapshot;
    final selected = _selectedTile != null && snap != null
        ? _computeTileInfo(snap, _selectedTile!)
        : null;
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
                title: l10n.tilemapPanelDisplay,
                child: Column(
                  children: [
                    _displayModeDropdown(context),
                    const SizedBox(height: 4),
                    CheckboxListTile(
                      dense: true,
                      visualDensity: VisualDensity.compact,
                      controlAffinity: ListTileControlAffinity.trailing,
                      contentPadding: EdgeInsets.zero,
                      title: Text(l10n.tilemapTileGrid),
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
                      title: Text(l10n.tilemapAttrGrid),
                      value: _showAttributeGrid,
                      onChanged: (v) {
                        setState(() => _showAttributeGrid = v ?? false);
                      },
                    ),
                    CheckboxListTile(
                      dense: true,
                      visualDensity: VisualDensity.compact,
                      controlAffinity: ListTileControlAffinity.trailing,
                      contentPadding: EdgeInsets.zero,
                      title: Text(l10n.tilemapAttrGrid32),
                      value: _showAttributeGrid32,
                      onChanged: (v) =>
                          setState(() => _showAttributeGrid32 = v ?? false),
                    ),
                    CheckboxListTile(
                      dense: true,
                      visualDensity: VisualDensity.compact,
                      controlAffinity: ListTileControlAffinity.trailing,
                      contentPadding: EdgeInsets.zero,
                      title: Text(l10n.tilemapNtBounds),
                      value: _showNametableDelimiters,
                      onChanged: (v) {
                        setState(() => _showNametableDelimiters = v ?? false);
                      },
                    ),
                    CheckboxListTile(
                      dense: true,
                      visualDensity: VisualDensity.compact,
                      controlAffinity: ListTileControlAffinity.trailing,
                      contentPadding: EdgeInsets.zero,
                      title: Text(l10n.tilemapScrollOverlay),
                      value: _showScrollOverlay,
                      onChanged: (v) =>
                          setState(() => _showScrollOverlay = v ?? false),
                    ),
                  ],
                ),
              ),
              _sideSection(
                context,
                title: l10n.tilemapCapture,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    RadioGroup<_TilemapCaptureMode>(
                      groupValue: _captureMode,
                      onChanged: (v) {
                        if (v == null) return;
                        setState(() => _captureMode = v);
                        _applyCaptureMode();
                      },
                      child: Column(
                        children: [
                          RadioListTile<_TilemapCaptureMode>(
                            dense: true,
                            visualDensity: VisualDensity.compact,
                            contentPadding: EdgeInsets.zero,
                            title: Text(l10n.tilemapCaptureFrameStart),
                            value: _TilemapCaptureMode.frameStart,
                          ),
                          RadioListTile<_TilemapCaptureMode>(
                            dense: true,
                            visualDensity: VisualDensity.compact,
                            contentPadding: EdgeInsets.zero,
                            title: Text(l10n.tilemapCaptureVblankStart),
                            value: _TilemapCaptureMode.vblankStart,
                          ),
                          RadioListTile<_TilemapCaptureMode>(
                            dense: true,
                            visualDensity: VisualDensity.compact,
                            contentPadding: EdgeInsets.zero,
                            title: Text(l10n.tilemapCaptureManual),
                            value: _TilemapCaptureMode.scanline,
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
                            enabled:
                                _captureMode == _TilemapCaptureMode.scanline,
                            controller: _scanlineController,
                            hint:
                                '${_TilemapViewerStateBase._minScanline} ~ ${_TilemapViewerStateBase._maxScanline}',
                            onSubmitted: (v) {
                              final value = int.tryParse(v);
                              if (value == null ||
                                  value <
                                      _TilemapViewerStateBase._minScanline ||
                                  value >
                                      _TilemapViewerStateBase._maxScanline) {
                                return;
                              }
                              setState(() => _scanline = value);
                              _scanlineController.text = _scanline.toString();
                              _applyCaptureMode();
                            },
                          ),
                        ),
                        const SizedBox(width: 10),
                        Expanded(
                          child: _numberFieldModern(
                            label: l10n.tilemapDot,
                            enabled:
                                _captureMode == _TilemapCaptureMode.scanline,
                            controller: _dotController,
                            hint:
                                '${_TilemapViewerStateBase._minDot} ~ ${_TilemapViewerStateBase._maxDot}',
                            onSubmitted: (v) {
                              final value = int.tryParse(v);
                              if (value == null ||
                                  value < _TilemapViewerStateBase._minDot ||
                                  value > _TilemapViewerStateBase._maxDot) {
                                return;
                              }
                              setState(() => _dot = value);
                              _dotController.text = _dot.toString();
                              _applyCaptureMode();
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
                title: l10n.tilemapPanelTilemap,
                child: Column(
                  children: [
                    _kvModern(l10n.tilemapInfoSize, '64×60'),
                    _kvModern(l10n.tilemapInfoSizePx, '512×480'),
                    _kvModern(l10n.tilemapInfoTilemapAddress, _hex(0x2000)),
                    _kvModern(
                      l10n.tilemapInfoTilesetAddress,
                      snap != null ? _hex(snap.bgPatternBase) : '—',
                    ),
                    _kvModern(
                      l10n.tilemapInfoMirroring,
                      snap != null
                          ? _mirroringLabel(l10n, snap.mirroring)
                          : '—',
                    ),
                    _kvModern(
                      l10n.tilemapInfoTileFormat,
                      l10n.tilemapInfoTileFormat2bpp,
                    ),
                  ],
                ),
              ),
              _sideSection(
                context,
                title: l10n.tilemapPanelSelectedTile,
                child: (selected == null || snap == null)
                    ? _emptyHint(colorScheme.onSurfaceVariant)
                    : _TileInfoCard(info: selected, snapshot: snap),
              ),
            ],
          );
        },
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

  Widget _kvModern(String k, String v) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          Expanded(
            child: Text(k, style: const TextStyle(color: Colors.black54)),
          ),
          Text(v, style: const TextStyle(fontWeight: FontWeight.w600)),
        ],
      ),
    );
  }

  String _hex(int value, {int width = 4}) {
    return '\$${value.toRadixString(16).toUpperCase().padLeft(width, '0')}';
  }

  String _mirroringLabel(AppLocalizations l10n, bridge.TilemapMirroring m) {
    switch (m) {
      case bridge.TilemapMirroring.horizontal:
        return l10n.tilemapMirroringHorizontal;
      case bridge.TilemapMirroring.vertical:
        return l10n.tilemapMirroringVertical;
      case bridge.TilemapMirroring.fourScreen:
        return l10n.tilemapMirroringFourScreen;
      case bridge.TilemapMirroring.singleScreenLower:
        return l10n.tilemapMirroringSingleScreenLower;
      case bridge.TilemapMirroring.singleScreenUpper:
        return l10n.tilemapMirroringSingleScreenUpper;
      case bridge.TilemapMirroring.mapperControlled:
        return l10n.tilemapMirroringMapperControlled;
    }
  }

  _TileInfo? _computeTileInfo(bridge.TilemapSnapshot snap, _TileCoord tile) {
    final tileX = tile.x;
    final tileY = tile.y;
    final ntX = tileX >= 32 ? 1 : 0;
    final ntY = tileY >= 30 ? 1 : 0;
    final ntIndex = ntY * 2 + ntX;

    final tileXInNt = tileX % 32;
    final tileYInNt = tileY % 30;

    final ntLocalAddr = tileYInNt * 32 + tileXInNt;
    final tilemapAddress = 0x2000 + ntIndex * 0x400 + ntLocalAddr;

    final ciramBase = _mirrorNametableToCiramOffset(ntIndex, snap.mirroring);
    final tileCiramAddr = ciramBase + ntLocalAddr;
    if (tileCiramAddr < 0 || tileCiramAddr >= snap.ciram.length) return null;

    final tileIndex = snap.ciram[tileCiramAddr];

    final attrLocalAddr = 0x3C0 + (tileYInNt ~/ 4) * 8 + (tileXInNt ~/ 4);
    final attrAddress = 0x2000 + ntIndex * 0x400 + attrLocalAddr;
    final attrCiramAddr = ciramBase + attrLocalAddr;
    final attrByte = attrCiramAddr >= 0 && attrCiramAddr < snap.ciram.length
        ? snap.ciram[attrCiramAddr]
        : 0;

    final shift = ((tileYInNt % 4) ~/ 2) * 4 + ((tileXInNt % 4) ~/ 2) * 2;
    final paletteIndex = (attrByte >> shift) & 0x03;
    final paletteAddress = 0x3F00 + paletteIndex * 4;

    final tileAddressPpu = snap.bgPatternBase + tileIndex * 16;

    return _TileInfo(
      tileX: tileX,
      tileY: tileY,
      ntIndex: ntIndex,
      tileIndex: tileIndex,
      tilemapAddress: tilemapAddress,
      tileAddressPpu: tileAddressPpu,
      paletteIndex: paletteIndex,
      paletteAddress: paletteAddress,
      attrAddress: attrAddress,
      attrByte: attrByte,
    );
  }

  int _mirrorNametableToCiramOffset(
    int ntIndex,
    bridge.TilemapMirroring mirroring,
  ) {
    final physicalNt = switch (mirroring) {
      bridge.TilemapMirroring.horizontal =>
        (ntIndex == 0 || ntIndex == 1) ? 0 : 1,
      bridge.TilemapMirroring.vertical =>
        (ntIndex == 0 || ntIndex == 2) ? 0 : 1,
      bridge.TilemapMirroring.fourScreen => ntIndex.clamp(0, 1),
      bridge.TilemapMirroring.singleScreenLower => 0,
      bridge.TilemapMirroring.singleScreenUpper => 1,
      bridge.TilemapMirroring.mapperControlled => ntIndex.clamp(0, 1),
    };
    return physicalNt * 0x400;
  }

  Widget _buildDesktopSidePanelWrapper(BuildContext context) {
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
          child: _buildDesktopSidePanel(context),
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
      tooltip: l10n.tilemapSettings,
      onPressed: () => _showSettingsDialog(context),
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

  void _showSettingsDialog(BuildContext context) {
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
          onTap: null, // Disable interaction but keep text color
          height: 32,
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: Text(
            l10n.tilemapOverlay,
            style: theme.textTheme.labelSmall?.copyWith(
              color: theme.colorScheme.onSurfaceVariant,
              fontWeight: FontWeight.w600,
            ),
          ),
        ),
        PopupMenuItem<void>(
          enabled: false,
          padding: EdgeInsets.zero,
          child: StatefulBuilder(
            builder: (context, setMenuState) => Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Padding(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 16,
                    vertical: 6,
                  ),
                  child: Row(
                    children: [
                      Expanded(
                        child: Text(
                          l10n.tilemapDisplayMode,
                          style: theme.textTheme.bodySmall,
                        ),
                      ),
                      SizedBox(
                        width: 180,
                        child: AnimatedDropdownMenu<_TilemapDisplayMode>(
                          density: AnimatedDropdownMenuDensity.compact,
                          value: _displayMode,
                          entries: [
                            DropdownMenuEntry(
                              value: _TilemapDisplayMode.defaultMode,
                              label: l10n.tilemapDisplayModeDefault,
                            ),
                            DropdownMenuEntry(
                              value: _TilemapDisplayMode.grayscale,
                              label: l10n.tilemapDisplayModeGrayscale,
                            ),
                            DropdownMenuEntry(
                              value: _TilemapDisplayMode.attributeView,
                              label: l10n.tilemapDisplayModeAttributeView,
                            ),
                          ],
                          onSelected: (v) {
                            setState(() => _displayMode = v);
                            setMenuState(() {});
                            _applyTextureRenderMode();
                          },
                        ),
                      ),
                    ],
                  ),
                ),
                CheckboxListTile(
                  dense: true,
                  contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                  title: Text(l10n.tilemapTileGrid),
                  value: _showTileGrid,
                  onChanged: (v) {
                    setState(() => _showTileGrid = v ?? false);
                    setMenuState(() {});
                  },
                ),
                CheckboxListTile(
                  dense: true,
                  contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                  title: Text(l10n.tilemapAttrGrid),
                  value: _showAttributeGrid,
                  onChanged: (v) {
                    setState(() => _showAttributeGrid = v ?? false);
                    setMenuState(() {});
                  },
                ),
                CheckboxListTile(
                  dense: true,
                  contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                  title: Text(l10n.tilemapAttrGrid32),
                  value: _showAttributeGrid32,
                  onChanged: (v) {
                    setState(() => _showAttributeGrid32 = v ?? false);
                    setMenuState(() {});
                  },
                ),
                CheckboxListTile(
                  dense: true,
                  contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                  title: Text(l10n.tilemapNtBounds),
                  value: _showNametableDelimiters,
                  onChanged: (v) {
                    setState(() => _showNametableDelimiters = v ?? false);
                    setMenuState(() {});
                  },
                ),
                CheckboxListTile(
                  dense: true,
                  contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                  title: Text(l10n.tilemapScrollOverlay),
                  value: _showScrollOverlay,
                  onChanged: (v) {
                    setState(() => _showScrollOverlay = v ?? false);
                    setMenuState(() {});
                  },
                ),
              ],
            ),
          ),
        ),
        const PopupMenuDivider(height: 1),
        // Capture section header
        PopupMenuItem<void>(
          onTap: null, // Disable interaction but keep text color
          height: 32,
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: Text(
            l10n.tilemapCapture,
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
            builder: (context, setMenuState) => RadioGroup<_TilemapCaptureMode>(
              groupValue: _captureMode,
              onChanged: (v) {
                if (v == null) return;
                setState(() => _captureMode = v);
                setMenuState(() {});
                _applyCaptureMode();
              },
              child: Column(
                mainAxisSize: MainAxisSize.min,
                children: [
                  RadioListTile<_TilemapCaptureMode>(
                    dense: true,
                    contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                    title: Text(l10n.tilemapCaptureFrameStart),
                    value: _TilemapCaptureMode.frameStart,
                  ),
                  RadioListTile<_TilemapCaptureMode>(
                    dense: true,
                    contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                    title: Text(l10n.tilemapCaptureVblankStart),
                    value: _TilemapCaptureMode.vblankStart,
                  ),
                  RadioListTile<_TilemapCaptureMode>(
                    dense: true,
                    contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                    title: Text(l10n.tilemapCaptureManual),
                    value: _TilemapCaptureMode.scanline,
                  ),
                  if (_captureMode == _TilemapCaptureMode.scanline) ...[
                    const PopupMenuDivider(height: 1),
                    PopupMenuItem<void>(
                      onTap: () {}, // Prevent closing
                      padding: EdgeInsets.zero,
                      child: Padding(
                        padding: const EdgeInsets.all(16),
                        child: _buildScanlineControlsForDialog(
                          theme,
                          l10n,
                          setMenuState,
                        ),
                      ),
                    ),
                  ],
                ],
              ),
            ),
          ),
        ),
      ],
    );
  }

  Widget _buildScanlineControlsForDialog(
    ThemeData theme,
    AppLocalizations l10n,
    StateSetter setDialogState,
  ) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        // Scanline input
        Text('${l10n.tilemapScanline}:', style: theme.textTheme.bodySmall),
        const SizedBox(height: 4),
        TextField(
          controller: TextEditingController(text: _scanline.toString())
            ..selection = TextSelection.fromPosition(
              TextPosition(offset: _scanline.toString().length),
            ),
          decoration: InputDecoration(
            isDense: true,
            contentPadding: const EdgeInsets.symmetric(
              horizontal: 12,
              vertical: 8,
            ),
            border: const OutlineInputBorder(),
            hintText:
                '${_TilemapViewerStateBase._minScanline} ~ ${_TilemapViewerStateBase._maxScanline}',
          ),
          keyboardType: TextInputType.number,
          onSubmitted: (v) {
            final value = int.tryParse(v);
            if (value != null &&
                value >= _TilemapViewerStateBase._minScanline &&
                value <= _TilemapViewerStateBase._maxScanline) {
              setState(() {
                _scanline = value;
                _scanlineController.text = _scanline.toString();
              });
              setDialogState(() {});
              _applyCaptureMode();
            }
          },
        ),
        const SizedBox(height: 12),
        // Dot input
        Text('${l10n.tilemapDot}:', style: theme.textTheme.bodySmall),
        const SizedBox(height: 4),
        TextField(
          controller: TextEditingController(text: _dot.toString())
            ..selection = TextSelection.fromPosition(
              TextPosition(offset: _dot.toString().length),
            ),
          decoration: InputDecoration(
            isDense: true,
            contentPadding: const EdgeInsets.symmetric(
              horizontal: 12,
              vertical: 8,
            ),
            border: const OutlineInputBorder(),
            hintText:
                '${_TilemapViewerStateBase._minDot} ~ ${_TilemapViewerStateBase._maxDot}',
          ),
          keyboardType: TextInputType.number,
          onSubmitted: (v) {
            final value = int.tryParse(v);
            if (value != null &&
                value >= _TilemapViewerStateBase._minDot &&
                value <= _TilemapViewerStateBase._maxDot) {
              setState(() {
                _dot = value;
                _dotController.text = _dot.toString();
              });
              setDialogState(() {});
              _applyCaptureMode();
            }
          },
        ),
      ],
    );
  }

  Widget _displayModeDropdown(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final theme = Theme.of(context);

    return Row(
      children: [
        Expanded(
          child: Text(
            l10n.tilemapDisplayMode,
            style: theme.textTheme.bodySmall,
          ),
        ),
        SizedBox(
          width: 180,
          child: AnimatedDropdownMenu<_TilemapDisplayMode>(
            density: AnimatedDropdownMenuDensity.compact,
            value: _displayMode,
            entries: [
              DropdownMenuEntry(
                value: _TilemapDisplayMode.defaultMode,
                label: l10n.tilemapDisplayModeDefault,
              ),
              DropdownMenuEntry(
                value: _TilemapDisplayMode.grayscale,
                label: l10n.tilemapDisplayModeGrayscale,
              ),
              DropdownMenuEntry(
                value: _TilemapDisplayMode.attributeView,
                label: l10n.tilemapDisplayModeAttributeView,
              ),
            ],
            onSelected: (v) {
              setState(() => _displayMode = v);
              _applyTextureRenderMode();
            },
          ),
        ),
      ],
    );
  }
}
