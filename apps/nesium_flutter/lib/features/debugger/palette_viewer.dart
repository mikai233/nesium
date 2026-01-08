import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/domain/nes_controller.dart';
import 'package:nesium_flutter/features/debugger/viewer_skeletonizer.dart';
import 'package:nesium_flutter/l10n/app_localizations.dart';
import 'package:nesium_flutter/logging/app_logger.dart';
import 'package:nesium_flutter/platform/platform_capabilities.dart';
import 'package:nesium_flutter/widgets/single_position_scrollbar.dart';

enum _CaptureMode { frameStart, vblankStart, scanline }

class PaletteViewer extends ConsumerStatefulWidget {
  const PaletteViewer({super.key});

  @override
  ConsumerState<PaletteViewer> createState() => _PaletteViewerState();
}

class _PaletteViewerState extends ConsumerState<PaletteViewer> {
  static const int _minScanline = -1;
  static const int _maxScanline = 260;
  static const int _minDot = 0;
  static const int _maxDot = 340;
  static const double _sidePanelWidth = 280;

  StreamSubscription<bridge.PaletteSnapshot>? _subscription;
  bool _hasReceivedData = false;
  final List<int> _systemPaletteArgb = List<int>.filled(64, 0xFF000000);
  late final List<ValueNotifier<int>> _systemPaletteArgbNotifiers =
      List<ValueNotifier<int>>.generate(
        64,
        (_) => ValueNotifier<int>(0xFF000000),
      );
  late final List<ValueNotifier<_PaletteRamEntry>> _paletteRamNotifiers =
      List<ValueNotifier<_PaletteRamEntry>>.generate(
        32,
        (_) => ValueNotifier<_PaletteRamEntry>(
          const _PaletteRamEntry(value: 0, argb: 0xFF000000),
        ),
      );
  String? _error;

  bool _showSidePanel = true;

  _CaptureMode _captureMode = _CaptureMode.vblankStart;
  int _scanline = 0;
  int _dot = 0;
  late final TextEditingController _scanlineController = TextEditingController(
    text: _scanline.toString(),
  );
  late final TextEditingController _dotController = TextEditingController(
    text: _dot.toString(),
  );

  @override
  void initState() {
    super.initState();
    _startStreaming();
  }

  @override
  void dispose() {
    unawaited(_subscription?.cancel());
    unawaited(_unsubscribe());
    for (final n in _systemPaletteArgbNotifiers) {
      n.dispose();
    }
    for (final n in _paletteRamNotifiers) {
      n.dispose();
    }
    _scanlineController.dispose();
    _dotController.dispose();
    super.dispose();
  }

  Future<void> _unsubscribe() async {
    try {
      await bridge.unsubscribePaletteState();
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'Failed to unsubscribe palette state',
        logger: 'palette_viewer',
      );
    }
  }

  Future<void> _applyCaptureMode() async {
    switch (_captureMode) {
      case _CaptureMode.frameStart:
        await bridge.setPaletteCaptureFrameStart();
      case _CaptureMode.vblankStart:
        await bridge.setPaletteCaptureVblankStart();
      case _CaptureMode.scanline:
        await bridge.setPaletteCaptureScanline(scanline: _scanline, dot: _dot);
    }
  }

  Future<void> _startStreaming() async {
    try {
      setState(() => _error = null);
      await _applyCaptureMode();
      await _subscription?.cancel();
      _subscription = bridge.paletteStateStream().listen(
        (snap) {
          if (!mounted) return;
          final firstData = !_hasReceivedData;
          if (firstData) {
            setState(() => _hasReceivedData = true);
          }
          _updateSystemPalette(snap.bgraPalette);
          _updatePaletteRam(snap.palette);
        },
        onError: (e) {
          if (!mounted) return;
          setState(() => _error = e.toString());
        },
      );
    } catch (e) {
      if (!mounted) return;
      setState(() => _error = e.toString());
    }
  }

  bool _updateSystemPalette(Uint8List bgraPalette) {
    if (bgraPalette.length < 64 * 4) return false;

    var anyChanged = false;
    for (var i = 0; i < 64; i++) {
      final base = i * 4;
      final b = bgraPalette[base];
      final g = bgraPalette[base + 1];
      final r = bgraPalette[base + 2];
      final a = bgraPalette[base + 3];
      final argb = (a << 24) | (r << 16) | (g << 8) | b;
      if (_systemPaletteArgb[i] == argb) continue;

      _systemPaletteArgb[i] = argb;
      _systemPaletteArgbNotifiers[i].value = argb;
      anyChanged = true;
    }
    return anyChanged;
  }

  void _updatePaletteRam(Uint8List paletteRam) {
    if (paletteRam.length < 32) return;

    for (var i = 0; i < 32; i++) {
      final value = paletteRam[i];
      final color = _systemPaletteArgb[value & 0x3F];
      final notifier = _paletteRamNotifiers[i];
      final current = notifier.value;
      if (current.value == value && current.argb == color) continue;
      notifier.value = _PaletteRamEntry(value: value, argb: color);
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    if (_error != null) {
      return Center(
        child: Column(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            const Icon(Icons.error_outline, size: 48, color: Colors.red),
            const SizedBox(height: 16),
            Text(l10n.tilemapError(_error ?? ''), textAlign: TextAlign.center),
            const SizedBox(height: 16),
            FilledButton.tonal(
              onPressed: _startStreaming,
              child: Text(l10n.tilemapRetry),
            ),
          ],
        ),
      );
    }

    final hasRom = ref.watch(nesControllerProvider).romHash != null;
    final loading = !hasRom || !_hasReceivedData;
    final content = _buildPaletteContent(context);

    if (!isNativeDesktop) {
      final base = ViewerSkeletonizer(
        enabled: loading,
        child: Stack(
          children: [
            content,
            Positioned(
              top: 12,
              right: 12,
              child: _buildSettingsButton(context),
            ),
          ],
        ),
      );
      return base;
    }

    final base = ViewerSkeletonizer(
      enabled: loading,
      child: Stack(
        children: [
          Row(
            children: [
              Expanded(child: content),
              _buildDesktopSidePanelWrapper(context),
            ],
          ),
          Positioned(
            top: 12,
            right: 12,
            child: _buildPanelToggleButton(context),
          ),
        ],
      ),
    );
    return base;
  }

  Widget _buildPaletteContent(BuildContext context) {
    final theme = Theme.of(context);
    final l10n = AppLocalizations.of(context)!;

    return SingleChildScrollView(
      padding: const EdgeInsets.all(16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            l10n.paletteViewerPaletteRamTitle,
            style: theme.textTheme.titleMedium,
          ),
          const SizedBox(height: 8),
          _ColorGrid(
            columns: 16,
            itemCount: 32,
            cellBuilder: (context, i) =>
                _PaletteRamCell(index: i, entry: _paletteRamNotifiers[i]),
          ),
          const SizedBox(height: 24),
          Text(
            l10n.paletteViewerSystemPaletteTitle,
            style: theme.textTheme.titleMedium,
          ),
          const SizedBox(height: 8),
          _ColorGrid(
            columns: 16,
            itemCount: 64,
            cellBuilder: (context, i) => _SystemPaletteCell(
              index: i,
              argb: _systemPaletteArgbNotifiers[i],
            ),
          ),
        ],
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
      tooltip: l10n.paletteViewerSettingsTooltip,
      onPressed: () => _showSettingsSheet(context),
    );
  }

  Future<void> _showSettingsSheet(BuildContext context) async {
    final l10n = AppLocalizations.of(context)!;
    await showModalBottomSheet<void>(
      context: context,
      showDragHandle: true,
      builder: (context) => SafeArea(
        child: SingleChildScrollView(
          padding: const EdgeInsets.all(16),
          child: _buildCaptureControls(
            context,
            l10n,
            dense: true,
            showTitle: true,
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

  Widget _buildDesktopSidePanelWrapper(BuildContext context) {
    final panelWidth = _sidePanelWidth;
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
    final theme = Theme.of(context);
    final l10n = AppLocalizations.of(context)!;
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
                title: l10n.tilemapCapture,
                child: _buildCaptureControls(
                  context,
                  l10n,
                  dense: true,
                  showTitle: false,
                ),
              ),
            ],
          );
        },
      ),
    );
  }

  Widget _buildCaptureControls(
    BuildContext context,
    AppLocalizations l10n, {
    required bool dense,
    required bool showTitle,
  }) {
    final theme = Theme.of(context);
    final textTheme = theme.textTheme;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        if (showTitle) ...[
          Text(l10n.tilemapCapture, style: dense ? textTheme.titleSmall : null),
          const SizedBox(height: 8),
        ],
        RadioGroup<_CaptureMode>(
          groupValue: _captureMode,
          onChanged: (v) {
            if (v == null) return;
            setState(() => _captureMode = v);
            unawaitedLogged(
              _applyCaptureMode(),
              message: 'Failed to set palette capture point',
              logger: 'palette_viewer',
            );
          },
          child: Column(
            children: [
              RadioListTile<_CaptureMode>(
                dense: dense,
                value: _CaptureMode.frameStart,
                title: Text(l10n.tilemapCaptureFrameStart),
                visualDensity: VisualDensity.compact,
                contentPadding: EdgeInsets.zero,
              ),
              RadioListTile<_CaptureMode>(
                dense: dense,
                value: _CaptureMode.vblankStart,
                title: Text(l10n.tilemapCaptureVblankStart),
                visualDensity: VisualDensity.compact,
                contentPadding: EdgeInsets.zero,
              ),
              RadioListTile<_CaptureMode>(
                dense: dense,
                value: _CaptureMode.scanline,
                title: Text(l10n.tilemapCaptureManual),
                visualDensity: VisualDensity.compact,
                contentPadding: EdgeInsets.zero,
              ),
            ],
          ),
        ),
        const SizedBox(height: 8),
        Row(
          children: [
            Expanded(
              child: TextField(
                controller: _scanlineController,
                enabled: _captureMode == _CaptureMode.scanline,
                keyboardType: TextInputType.number,
                decoration: InputDecoration(
                  labelText:
                      '${l10n.tilemapScanline} ($_minScanline..$_maxScanline)',
                  isDense: dense,
                  filled: true,
                  fillColor: theme.colorScheme.surfaceContainerLowest,
                  border: OutlineInputBorder(
                    borderRadius: BorderRadius.circular(10),
                  ),
                ),
                onChanged: (v) {
                  final parsed = int.tryParse(v);
                  if (parsed == null) return;
                  _scanline = parsed.clamp(_minScanline, _maxScanline);
                },
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
                    message: 'Failed to set palette capture point',
                    logger: 'palette_viewer',
                  );
                },
              ),
            ),
            const SizedBox(width: 12),
            Expanded(
              child: TextField(
                controller: _dotController,
                enabled: _captureMode == _CaptureMode.scanline,
                keyboardType: TextInputType.number,
                decoration: InputDecoration(
                  labelText: '${l10n.tilemapDot} ($_minDot..$_maxDot)',
                  isDense: dense,
                  filled: true,
                  fillColor: theme.colorScheme.surfaceContainerLowest,
                  border: OutlineInputBorder(
                    borderRadius: BorderRadius.circular(10),
                  ),
                ),
                onChanged: (v) {
                  final parsed = int.tryParse(v);
                  if (parsed == null) return;
                  _dot = parsed.clamp(_minDot, _maxDot);
                },
                onSubmitted: (v) {
                  final value = int.tryParse(v);
                  if (value == null || value < _minDot || value > _maxDot) {
                    return;
                  }
                  setState(() => _dot = value);
                  _dotController.text = _dot.toString();
                  unawaitedLogged(
                    _applyCaptureMode(),
                    message: 'Failed to set palette capture point',
                    logger: 'palette_viewer',
                  );
                },
              ),
            ),
          ],
        ),
      ],
    );
  }

  Widget _sideSection(
    BuildContext context, {
    required String title,
    required Widget child,
  }) {
    final theme = Theme.of(context);
    return Card(
      elevation: 0,
      color: theme.colorScheme.surface,
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(
              title,
              style: theme.textTheme.titleSmall?.copyWith(
                fontWeight: FontWeight.w600,
              ),
            ),
            const SizedBox(height: 8),
            child,
          ],
        ),
      ),
    );
  }
}

@immutable
class _PaletteRamEntry {
  final int value;
  final int argb;

  const _PaletteRamEntry({required this.value, required this.argb});
}

class _ColorGrid extends StatefulWidget {
  final int columns;
  final int itemCount;
  final Widget Function(BuildContext context, int index) cellBuilder;

  const _ColorGrid({
    required this.columns,
    required this.itemCount,
    required this.cellBuilder,
  });

  @override
  State<_ColorGrid> createState() => _ColorGridState();
}

class _ColorGridState extends State<_ColorGrid> {
  static const double _cellSize = 32.0;
  static const double _gap = 6.0;
  static const _duration = Duration(milliseconds: 180);

  int _columnsForWidth(double maxWidth) {
    final maxColumns = widget.columns.clamp(1, 999);
    if (!maxWidth.isFinite) return maxColumns;
    final fit = ((maxWidth + _gap) / (_cellSize + _gap)).floor();
    return fit.clamp(1, maxColumns);
  }

  @override
  Widget build(BuildContext context) {
    return LayoutBuilder(
      builder: (context, constraints) {
        final cols = _columnsForWidth(constraints.maxWidth);
        final rows = (widget.itemCount / cols).ceil();

        final gridWidth = (cols * _cellSize) + ((cols - 1) * _gap);
        final gridHeight = rows == 0
            ? 0.0
            : (rows * _cellSize) + ((rows - 1) * _gap);

        return ClipRect(
          child: AnimatedContainer(
            duration: _duration,
            curve: Curves.easeOut,
            width: gridWidth,
            height: gridHeight,
            child: Stack(
              children: [
                for (var i = 0; i < widget.itemCount; i++)
                  _buildAnimatedCell(context, i, cols),
              ],
            ),
          ),
        );
      },
    );
  }

  Widget _buildAnimatedCell(BuildContext context, int index, int columns) {
    final col = index % columns;
    final row = index ~/ columns;

    final left = col * (_cellSize + _gap);
    final top = row * (_cellSize + _gap);

    return AnimatedPositioned(
      key: ValueKey(index),
      duration: _duration,
      curve: Curves.easeOut,
      left: left,
      top: top,
      width: _cellSize,
      height: _cellSize,
      child: RepaintBoundary(child: widget.cellBuilder(context, index)),
    );
  }
}

class _PaletteRamCell extends StatelessWidget {
  const _PaletteRamCell({required this.index, required this.entry});

  final int index;
  final ValueListenable<_PaletteRamEntry> entry;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return ValueListenableBuilder<_PaletteRamEntry>(
      valueListenable: entry,
      builder: (context, e, _) {
        final v = e.value & 0x3F;
        final addr = 0x3F00 + index;
        final addrStr = addr.toRadixString(16).toUpperCase().padLeft(4, '0');
        final valueStr = v.toRadixString(16).toUpperCase().padLeft(2, '0');
        return _ColorSwatchCell(
          color: Color(e.argb),
          label: index.toString().padLeft(2, '0'),
          tooltip: l10n.paletteViewerTooltipPaletteRam('\$$addrStr', valueStr),
        );
      },
    );
  }
}

class _SystemPaletteCell extends StatelessWidget {
  const _SystemPaletteCell({required this.index, required this.argb});

  final int index;
  final ValueListenable<int> argb;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return ValueListenableBuilder<int>(
      valueListenable: argb,
      builder: (context, v, _) {
        return _ColorSwatchCell(
          color: Color(v),
          label: index.toString().padLeft(2, '0'),
          tooltip: l10n.paletteViewerTooltipSystemIndex(index),
        );
      },
    );
  }
}

class _ColorSwatchCell extends StatelessWidget {
  const _ColorSwatchCell({
    required this.color,
    required this.label,
    required this.tooltip,
  });

  final Color color;
  final String label;
  final String tooltip;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    return Tooltip(
      message: tooltip,
      child: DecoratedBox(
        decoration: BoxDecoration(
          color: color,
          borderRadius: BorderRadius.circular(8),
          border: Border.all(color: theme.colorScheme.outlineVariant),
        ),
        child: Center(
          child: Text(
            label,
            style: theme.textTheme.labelSmall?.copyWith(
              color: color.computeLuminance() > 0.55
                  ? Colors.black
                  : Colors.white,
              fontFeatures: const [FontFeature.tabularFigures()],
            ),
          ),
        ),
      ),
    );
  }
}
