import 'package:flutter/material.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/features/debugger/tilemap/tilemap_geometry.dart';
import 'package:nesium_flutter/features/debugger/tilemap/tilemap_models.dart';
import 'package:nesium_flutter/features/debugger/tilemap/tilemap_tile_info_widgets.dart';
import 'package:nesium_flutter/l10n/app_localizations.dart';
import 'package:nesium_flutter/widgets/animated_dropdown_menu.dart';
import 'package:nesium_flutter/widgets/single_position_scrollbar.dart';

class TilemapSidePanel extends StatelessWidget {
  const TilemapSidePanel({
    required this.snapshot,
    required this.selectedTile,
    required this.displayMode,
    required this.onDisplayModeChanged,
    required this.showTileGrid,
    required this.onShowTileGridChanged,
    required this.showAttributeGrid,
    required this.onShowAttributeGridChanged,
    required this.showAttributeGrid32,
    required this.onShowAttributeGrid32Changed,
    required this.showNametableDelimiters,
    required this.onShowNametableDelimitersChanged,
    required this.showScrollOverlay,
    required this.onShowScrollOverlayChanged,
    required this.captureMode,
    required this.onCaptureModeChanged,
    required this.scanlineController,
    required this.dotController,
    required this.minScanline,
    required this.maxScanline,
    required this.minDot,
    required this.maxDot,
    required this.onScanlineSubmitted,
    required this.onDotSubmitted,
    super.key,
  });

  final bridge.TilemapSnapshot? snapshot;
  final TilemapCoord? selectedTile;
  final TilemapDisplayMode displayMode;
  final ValueChanged<TilemapDisplayMode> onDisplayModeChanged;
  final bool showTileGrid;
  final ValueChanged<bool> onShowTileGridChanged;
  final bool showAttributeGrid;
  final ValueChanged<bool> onShowAttributeGridChanged;
  final bool showAttributeGrid32;
  final ValueChanged<bool> onShowAttributeGrid32Changed;
  final bool showNametableDelimiters;
  final ValueChanged<bool> onShowNametableDelimitersChanged;
  final bool showScrollOverlay;
  final ValueChanged<bool> onShowScrollOverlayChanged;
  final TilemapCaptureMode captureMode;
  final ValueChanged<TilemapCaptureMode> onCaptureModeChanged;
  final TextEditingController scanlineController;
  final TextEditingController dotController;
  final int minScanline;
  final int maxScanline;
  final int minDot;
  final int maxDot;
  final ValueChanged<String> onScanlineSubmitted;
  final ValueChanged<String> onDotSubmitted;

  @override
  Widget build(BuildContext context) {
    final snap = snapshot;
    final selected = selectedTile != null && snap != null
        ? computeTilemapTileInfo(snap, selectedTile!)
        : null;
    final l10n = AppLocalizations.of(context)!;
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

    return Container(
      decoration: BoxDecoration(
        color: colorScheme.surfaceContainerLowest,
        border: Border(left: BorderSide(color: colorScheme.outlineVariant)),
      ),
      child: SinglePositionScrollbar(
        thumbVisibility: true,
        builder: (context, controller) {
          return ListView(
            controller: controller,
            primary: false,
            padding: const EdgeInsets.all(12),
            children: [
              _SideSection(
                title: l10n.tilemapPanelDisplay,
                child: Column(
                  children: [
                    _DisplayModeDropdown(
                      displayMode: displayMode,
                      onChanged: onDisplayModeChanged,
                    ),
                    const SizedBox(height: 4),
                    _CheckOption(
                      title: l10n.tilemapTileGrid,
                      value: showTileGrid,
                      onChanged: onShowTileGridChanged,
                    ),
                    _CheckOption(
                      title: l10n.tilemapAttrGrid,
                      value: showAttributeGrid,
                      onChanged: onShowAttributeGridChanged,
                    ),
                    _CheckOption(
                      title: l10n.tilemapAttrGrid32,
                      value: showAttributeGrid32,
                      onChanged: onShowAttributeGrid32Changed,
                    ),
                    _CheckOption(
                      title: l10n.tilemapNtBounds,
                      value: showNametableDelimiters,
                      onChanged: onShowNametableDelimitersChanged,
                    ),
                    _CheckOption(
                      title: l10n.tilemapScrollOverlay,
                      value: showScrollOverlay,
                      onChanged: onShowScrollOverlayChanged,
                    ),
                  ],
                ),
              ),
              _SideSection(
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
                          child: _NumberField(
                            label: l10n.tilemapScanline,
                            enabled: captureMode == TilemapCaptureMode.scanline,
                            controller: scanlineController,
                            hint: '$minScanline ~ $maxScanline',
                            onSubmitted: onScanlineSubmitted,
                          ),
                        ),
                        const SizedBox(width: 10),
                        Expanded(
                          child: _NumberField(
                            label: l10n.tilemapDot,
                            enabled: captureMode == TilemapCaptureMode.scanline,
                            controller: dotController,
                            hint: '$minDot ~ $maxDot',
                            onSubmitted: onDotSubmitted,
                          ),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
              _SideSection(
                title: l10n.tilemapPanelTilemap,
                child: Column(
                  children: [
                    _kv(l10n.tilemapInfoSize, '64×60'),
                    _kv(l10n.tilemapInfoSizePx, '512×480'),
                    _kv(l10n.tilemapInfoTilemapAddress, tilemapHex(0x2000)),
                    _kv(
                      l10n.tilemapInfoTilesetAddress,
                      snap != null ? tilemapHex(snap.bgPatternBase) : '—',
                    ),
                    _kv(
                      l10n.tilemapInfoMirroring,
                      snap != null
                          ? _mirroringLabel(l10n, snap.mirroring)
                          : '—',
                    ),
                    _kv(
                      l10n.tilemapInfoTileFormat,
                      l10n.tilemapInfoTileFormat2bpp,
                    ),
                  ],
                ),
              ),
              _SideSection(
                title: l10n.tilemapPanelSelectedTile,
                child: (selected == null || snap == null)
                    ? Text(
                        '—',
                        style: TextStyle(color: colorScheme.onSurfaceVariant),
                      )
                    : TilemapTileInfoCard(info: selected, snapshot: snap),
              ),
            ],
          );
        },
      ),
    );
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
}

class _DisplayModeDropdown extends StatelessWidget {
  const _DisplayModeDropdown({
    required this.displayMode,
    required this.onChanged,
  });

  final TilemapDisplayMode displayMode;
  final ValueChanged<TilemapDisplayMode> onChanged;

  @override
  Widget build(BuildContext context) {
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
            onSelected: (v) {
              onChanged(v);
            },
          ),
        ),
      ],
    );
  }
}

class _CheckOption extends StatelessWidget {
  const _CheckOption({
    required this.title,
    required this.value,
    required this.onChanged,
  });

  final String title;
  final bool value;
  final ValueChanged<bool> onChanged;

  @override
  Widget build(BuildContext context) {
    return CheckboxListTile(
      dense: true,
      visualDensity: VisualDensity.compact,
      controlAffinity: ListTileControlAffinity.trailing,
      contentPadding: EdgeInsets.zero,
      title: Text(title),
      value: value,
      onChanged: (v) => onChanged(v ?? false),
    );
  }
}

class _SideSection extends StatelessWidget {
  const _SideSection({required this.title, required this.child});

  final String title;
  final Widget child;

  @override
  Widget build(BuildContext context) {
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
}

class _NumberField extends StatelessWidget {
  const _NumberField({
    required this.label,
    required this.enabled,
    required this.controller,
    required this.hint,
    required this.onSubmitted,
  });

  final String label;
  final bool enabled;
  final TextEditingController controller;
  final String hint;
  final ValueChanged<String> onSubmitted;

  @override
  Widget build(BuildContext context) {
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
}

Widget _kv(String k, String v) {
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
