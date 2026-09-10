import 'package:flutter/material.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/features/debugger/tile/tile_info_widgets.dart';
import 'package:nesium_flutter/features/debugger/tile/tile_preset_buttons.dart';
import 'package:nesium_flutter/features/debugger/tile/tile_viewer_controls.dart';
import 'package:nesium_flutter/features/debugger/tile/tile_viewer_models.dart';
import 'package:nesium_flutter/l10n/app_localizations.dart';
import 'package:nesium_flutter/widgets/animated_dropdown_menu.dart';
import 'package:nesium_flutter/widgets/single_position_scrollbar.dart';

class TileViewerSidePanel extends StatelessWidget {
  const TileViewerSidePanel({
    required this.snapshot,
    required this.selectedTile,
    required this.selectedPreset,
    required this.onPresetSelected,
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
    required this.source,
    required this.onSourceChanged,
    required this.startAddress,
    required this.maxAddress,
    required this.addressIncrement,
    required this.onStartAddressChanged,
    required this.columnCount,
    required this.rowCount,
    required this.layout,
    required this.onColumnsChanged,
    required this.onRowsChanged,
    required this.onLayoutChanged,
    required this.background,
    required this.onBackgroundChanged,
    required this.showTileGrid,
    required this.onShowTileGridChanged,
    required this.useGrayscale,
    required this.onUseGrayscaleChanged,
    required this.selectedPalette,
    required this.onPaletteChanged,
    super.key,
  });

  final bridge.TileSnapshot? snapshot;
  final TileCoord? selectedTile;
  final TilePreset? selectedPreset;
  final ValueChanged<TilePreset> onPresetSelected;
  final TileCaptureMode captureMode;
  final ValueChanged<TileCaptureMode> onCaptureModeChanged;
  final TextEditingController scanlineController;
  final TextEditingController dotController;
  final int minScanline;
  final int maxScanline;
  final int minDot;
  final int maxDot;
  final ValueChanged<String> onScanlineSubmitted;
  final ValueChanged<String> onDotSubmitted;
  final TileSource source;
  final ValueChanged<TileSource> onSourceChanged;
  final int startAddress;
  final int maxAddress;
  final int addressIncrement;
  final ValueChanged<int> onStartAddressChanged;
  final int columnCount;
  final int rowCount;
  final TileLayout layout;
  final ValueChanged<int> onColumnsChanged;
  final ValueChanged<int> onRowsChanged;
  final ValueChanged<TileLayout> onLayoutChanged;
  final TileBackground background;
  final ValueChanged<TileBackground> onBackgroundChanged;
  final bool showTileGrid;
  final ValueChanged<bool> onShowTileGridChanged;
  final bool useGrayscale;
  final ValueChanged<bool> onUseGrayscaleChanged;
  final int selectedPalette;
  final ValueChanged<int> onPaletteChanged;

  @override
  Widget build(BuildContext context) {
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
              SideSection(
                title: l10n.tileViewerPresets,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    TilePresetButtons(
                      presets: const [
                        TilePreset.ppu,
                        TilePreset.chr,
                        TilePreset.rom,
                      ],
                      selectedPreset: selectedPreset,
                      onSelected: onPresetSelected,
                    ),
                    const SizedBox(height: 8),
                    TilePresetButtons(
                      presets: const [TilePreset.bg, TilePreset.oam],
                      selectedPreset: selectedPreset,
                      onSelected: onPresetSelected,
                    ),
                  ],
                ),
              ),
              SideSection(
                title: l10n.tilemapCapture,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    RadioGroup<TileCaptureMode>(
                      groupValue: captureMode,
                      onChanged: (v) {
                        if (v == null) return;
                        onCaptureModeChanged(v);
                      },
                      child: Column(
                        children: [
                          RadioListTile<TileCaptureMode>(
                            dense: true,
                            visualDensity: VisualDensity.compact,
                            contentPadding: EdgeInsets.zero,
                            title: Text(l10n.tilemapCaptureFrameStart),
                            value: TileCaptureMode.frameStart,
                          ),
                          RadioListTile<TileCaptureMode>(
                            dense: true,
                            visualDensity: VisualDensity.compact,
                            contentPadding: EdgeInsets.zero,
                            title: Text(l10n.tilemapCaptureVblankStart),
                            value: TileCaptureMode.vblankStart,
                          ),
                          RadioListTile<TileCaptureMode>(
                            dense: true,
                            visualDensity: VisualDensity.compact,
                            contentPadding: EdgeInsets.zero,
                            title: Text(l10n.tilemapCaptureManual),
                            value: TileCaptureMode.scanline,
                          ),
                        ],
                      ),
                    ),
                    const SizedBox(height: 10),
                    Row(
                      children: [
                        Expanded(
                          child: NumberField(
                            label: l10n.tilemapScanline,
                            enabled: captureMode == TileCaptureMode.scanline,
                            controller: scanlineController,
                            hint: '$minScanline ~ $maxScanline',
                            onSubmitted: onScanlineSubmitted,
                          ),
                        ),
                        const SizedBox(width: 10),
                        Expanded(
                          child: NumberField(
                            label: l10n.tilemapDot,
                            enabled: captureMode == TileCaptureMode.scanline,
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
              SideSection(
                title: l10n.tileViewerSource,
                child: AnimatedDropdownMenu<TileSource>(
                  density: AnimatedDropdownMenuDensity.compact,
                  value: source,
                  entries: [
                    DropdownMenuEntry(
                      value: TileSource.ppu,
                      label: l10n.tileViewerSourcePpu,
                    ),
                    DropdownMenuEntry(
                      value: TileSource.chrRom,
                      label: l10n.tileViewerSourceChrRom,
                    ),
                    DropdownMenuEntry(
                      value: TileSource.chrRam,
                      label: l10n.tileViewerSourceChrRam,
                    ),
                    DropdownMenuEntry(
                      value: TileSource.prgRom,
                      label: l10n.tileViewerSourcePrgRom,
                    ),
                  ],
                  onSelected: onSourceChanged,
                ),
              ),
              SideSection(
                title: l10n.tileViewerAddress,
                child: AddressInput(
                  value: startAddress,
                  maxValue: maxAddress,
                  pageIncrement: addressIncrement,
                  byteIncrement: 1,
                  onChanged: onStartAddressChanged,
                ),
              ),
              SideSection(
                title: l10n.tileViewerSize,
                child: Row(
                  children: [
                    Expanded(
                      child: SizeInput(
                        label: l10n.tileViewerColumns,
                        value: columnCount,
                        min: 4,
                        max: 256,
                        step: layout == TileLayout.normal ? 1 : 2,
                        onChanged: onColumnsChanged,
                      ),
                    ),
                    const SizedBox(width: 8),
                    Expanded(
                      child: SizeInput(
                        label: l10n.tileViewerRows,
                        value: rowCount,
                        min: 4,
                        max: 256,
                        step: layout == TileLayout.normal ? 1 : 2,
                        onChanged: onRowsChanged,
                      ),
                    ),
                  ],
                ),
              ),
              SideSection(
                title: l10n.tileViewerLayout,
                child: AnimatedDropdownMenu<TileLayout>(
                  density: AnimatedDropdownMenuDensity.compact,
                  value: layout,
                  entries: [
                    DropdownMenuEntry(
                      value: TileLayout.normal,
                      label: l10n.tileViewerLayoutNormal,
                    ),
                    DropdownMenuEntry(
                      value: TileLayout.singleLine8x16,
                      label: l10n.tileViewerLayout8x16,
                    ),
                    DropdownMenuEntry(
                      value: TileLayout.singleLine16x16,
                      label: l10n.tileViewerLayout16x16,
                    ),
                  ],
                  onSelected: onLayoutChanged,
                ),
              ),
              SideSection(
                title: l10n.tileViewerBackground,
                child: AnimatedDropdownMenu<TileBackground>(
                  density: AnimatedDropdownMenuDensity.compact,
                  value: background,
                  entries: [
                    DropdownMenuEntry(
                      value: TileBackground.defaultBg,
                      label: l10n.tileViewerBgDefault,
                    ),
                    DropdownMenuEntry(
                      value: TileBackground.transparent,
                      label: l10n.tileViewerBgTransparent,
                    ),
                    DropdownMenuEntry(
                      value: TileBackground.paletteColor,
                      label: l10n.tileViewerBgPalette,
                    ),
                    DropdownMenuEntry(
                      value: TileBackground.black,
                      label: l10n.tileViewerBgBlack,
                    ),
                    DropdownMenuEntry(
                      value: TileBackground.white,
                      label: l10n.tileViewerBgWhite,
                    ),
                    DropdownMenuEntry(
                      value: TileBackground.magenta,
                      label: l10n.tileViewerBgMagenta,
                    ),
                  ],
                  onSelected: onBackgroundChanged,
                ),
              ),
              SideSection(
                title: l10n.tileViewerOverlays,
                child: Column(
                  children: [
                    CheckboxListTile(
                      dense: true,
                      visualDensity: VisualDensity.compact,
                      controlAffinity: ListTileControlAffinity.trailing,
                      contentPadding: EdgeInsets.zero,
                      title: Text(l10n.tileViewerShowGrid),
                      value: showTileGrid,
                      onChanged: (v) => onShowTileGridChanged(v ?? false),
                    ),
                    CheckboxListTile(
                      dense: true,
                      visualDensity: VisualDensity.compact,
                      controlAffinity: ListTileControlAffinity.trailing,
                      contentPadding: EdgeInsets.zero,
                      title: Text(l10n.tileViewerGrayscale),
                      value: useGrayscale,
                      onChanged: (v) => onUseGrayscaleChanged(v ?? false),
                    ),
                  ],
                ),
              ),
              SideSection(
                title: l10n.tileViewerPalette,
                child: _PaletteDropdown(
                  selectedPalette: selectedPalette,
                  onPaletteChanged: onPaletteChanged,
                ),
              ),
              if (selectedTile != null && snapshot != null)
                SideSection(
                  title: l10n.tileViewerSelectedTile,
                  child: TileInfoCard(tile: selectedTile!, snapshot: snapshot!),
                ),
            ],
          );
        },
      ),
    );
  }
}

class _PaletteDropdown extends StatelessWidget {
  const _PaletteDropdown({
    required this.selectedPalette,
    required this.onPaletteChanged,
  });

  final int selectedPalette;
  final ValueChanged<int> onPaletteChanged;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    return AnimatedDropdownMenu<int>(
      density: AnimatedDropdownMenuDensity.compact,
      value: selectedPalette,
      entries: [
        for (var i = 0; i < 8; i++)
          DropdownMenuEntry(
            value: i,
            label: i < 4
                ? l10n.tileViewerPaletteBg(i)
                : l10n.tileViewerPaletteSprite(i - 4),
          ),
      ],
      onSelected: onPaletteChanged,
    );
  }
}
