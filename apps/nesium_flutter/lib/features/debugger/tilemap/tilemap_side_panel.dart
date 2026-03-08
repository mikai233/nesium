import 'package:flutter/material.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/l10n/app_localizations.dart';
import 'package:nesium_flutter/widgets/animated_dropdown_menu.dart';
import 'package:nesium_flutter/widgets/single_position_scrollbar.dart';
import 'tilemap_models.dart';
import 'tilemap_widgets.dart';

class TilemapSidePanel extends StatelessWidget {
  const TilemapSidePanel({
    super.key,
    required this.snapshot,
    required this.selectedTile,
    required this.showTileGrid,
    required this.showAttributeGrid,
    required this.showAttributeGrid32,
    required this.showNametableDelimiters,
    required this.showScrollOverlay,
    required this.displayMode,
    required this.captureMode,
    required this.scanlineController,
    required this.dotController,
    required this.onShowTileGridChanged,
    required this.onShowAttributeGridChanged,
    required this.onShowAttributeGrid32Changed,
    required this.onShowNametableDelimitersChanged,
    required this.onShowScrollOverlayChanged,
    required this.onDisplayModeChanged,
    required this.onCaptureModeChanged,
    required this.onScanlineSubmitted,
    required this.onDotSubmitted,
  });

  final bridge.TilemapSnapshot? snapshot;
  final TileCoord? selectedTile;
  final bool showTileGrid;
  final bool showAttributeGrid;
  final bool showAttributeGrid32;
  final bool showNametableDelimiters;
  final bool showScrollOverlay;
  final TilemapDisplayMode displayMode;
  final TilemapCaptureMode captureMode;
  final TextEditingController scanlineController;
  final TextEditingController dotController;

  final ValueChanged<bool?> onShowTileGridChanged;
  final ValueChanged<bool?> onShowAttributeGridChanged;
  final ValueChanged<bool?> onShowAttributeGrid32Changed;
  final ValueChanged<bool?> onShowNametableDelimitersChanged;
  final ValueChanged<bool?> onShowScrollOverlayChanged;
  final ValueChanged<TilemapDisplayMode> onDisplayModeChanged;
  final ValueChanged<TilemapCaptureMode> onCaptureModeChanged;
  final ValueChanged<String> onScanlineSubmitted;
  final ValueChanged<String> onDotSubmitted;

  @override
  Widget build(BuildContext context) {
    final snap = snapshot;
    final selected = selectedTile != null && snap != null
        ? TileInfo.compute(snap, selectedTile!)
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
                      value: showTileGrid,
                      onChanged: onShowTileGridChanged,
                    ),
                    CheckboxListTile(
                      dense: true,
                      visualDensity: VisualDensity.compact,
                      controlAffinity: ListTileControlAffinity.trailing,
                      contentPadding: EdgeInsets.zero,
                      title: Text(l10n.tilemapAttrGrid),
                      value: showAttributeGrid,
                      onChanged: onShowAttributeGridChanged,
                    ),
                    CheckboxListTile(
                      dense: true,
                      visualDensity: VisualDensity.compact,
                      controlAffinity: ListTileControlAffinity.trailing,
                      contentPadding: EdgeInsets.zero,
                      title: Text(l10n.tilemapAttrGrid32),
                      value: showAttributeGrid32,
                      onChanged: onShowAttributeGrid32Changed,
                    ),
                    CheckboxListTile(
                      dense: true,
                      visualDensity: VisualDensity.compact,
                      controlAffinity: ListTileControlAffinity.trailing,
                      contentPadding: EdgeInsets.zero,
                      title: Text(l10n.tilemapNtBounds),
                      value: showNametableDelimiters,
                      onChanged: onShowNametableDelimitersChanged,
                    ),
                    CheckboxListTile(
                      dense: true,
                      visualDensity: VisualDensity.compact,
                      controlAffinity: ListTileControlAffinity.trailing,
                      contentPadding: EdgeInsets.zero,
                      title: Text(l10n.tilemapScrollOverlay),
                      value: showScrollOverlay,
                      onChanged: onShowScrollOverlayChanged,
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
                    RadioGroup<TilemapCaptureMode>(
                      groupValue: captureMode,
                      onChanged: (v) {
                        if (v == null) return;
                        onCaptureModeChanged(v);
                      },
                      child: Column(
                        children: [
                          RadioListTile<TilemapCaptureMode>(
                            dense: true,
                            visualDensity: VisualDensity.compact,
                            contentPadding: EdgeInsets.zero,
                            title: Text(l10n.tilemapCaptureFrameStart),
                            value: TilemapCaptureMode.frameStart,
                          ),
                          RadioListTile<TilemapCaptureMode>(
                            dense: true,
                            visualDensity: VisualDensity.compact,
                            contentPadding: EdgeInsets.zero,
                            title: Text(l10n.tilemapCaptureVblankStart),
                            value: TilemapCaptureMode.vblankStart,
                          ),
                          RadioListTile<TilemapCaptureMode>(
                            dense: true,
                            visualDensity: VisualDensity.compact,
                            contentPadding: EdgeInsets.zero,
                            title: Text(l10n.tilemapCaptureManual),
                            value: TilemapCaptureMode.scanline,
                          ),
                        ],
                      ),
                    ),
                    const SizedBox(height: 10),
                    Row(
                      children: [
                        Expanded(
                          child: _numberFieldModern(
                            context,
                            label: l10n.tilemapScanline,
                            enabled: captureMode == TilemapCaptureMode.scanline,
                            controller: scanlineController,
                            hint: '-1 ~ 260',
                            onSubmitted: onScanlineSubmitted,
                          ),
                        ),
                        const SizedBox(width: 10),
                        Expanded(
                          child: _numberFieldModern(
                            context,
                            label: l10n.tilemapDot,
                            enabled: captureMode == TilemapCaptureMode.scanline,
                            controller: dotController,
                            hint: '0 ~ 340',
                            onSubmitted: onDotSubmitted,
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
                    _kvModern(
                      l10n.tilemapInfoTilemapAddress,
                      formatHex(0x2000),
                    ),
                    _kvModern(
                      l10n.tilemapInfoTilesetAddress,
                      snap != null ? formatHex(snap.bgPatternBase) : '—',
                    ),
                    _kvModern(
                      l10n.tilemapInfoMirroring,
                      snap != null ? mirroringLabel(l10n, snap.mirroring) : '—',
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
                    : TileInfoCard(info: selected, snapshot: snap),
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

  Widget _numberFieldModern(
    BuildContext context, {
    required String label,
    required bool enabled,
    required TextEditingController controller,
    required String hint,
    ValueChanged<String>? onSubmitted,
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
          child: AnimatedDropdownMenu<TilemapDisplayMode>(
            density: AnimatedDropdownMenuDensity.compact,
            value: displayMode,
            entries: [
              DropdownMenuEntry(
                value: TilemapDisplayMode.defaultMode,
                label: l10n.tilemapDisplayModeDefault,
              ),
              DropdownMenuEntry(
                value: TilemapDisplayMode.grayscale,
                label: l10n.tilemapDisplayModeGrayscale,
              ),
              DropdownMenuEntry(
                value: TilemapDisplayMode.attributeView,
                label: l10n.tilemapDisplayModeAttributeView,
              ),
            ],
            onSelected: onDisplayModeChanged,
          ),
        ),
      ],
    );
  }
}
