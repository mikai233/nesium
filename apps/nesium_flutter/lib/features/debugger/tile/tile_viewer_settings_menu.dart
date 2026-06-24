import 'package:flutter/material.dart';
import 'package:nesium_flutter/features/debugger/tile/tile_preset_buttons.dart';
import 'package:nesium_flutter/features/debugger/tile/tile_viewer_controls.dart';
import 'package:nesium_flutter/features/debugger/tile/tile_viewer_models.dart';
import 'package:nesium_flutter/l10n/app_localizations.dart';
import 'package:nesium_flutter/widgets/animated_dropdown_menu.dart';

class TileViewerSettingsButton extends StatelessWidget {
  const TileViewerSettingsButton({
    required this.selectedPreset,
    required this.onPresetSelected,
    required this.showTileGrid,
    required this.onShowTileGridChanged,
    required this.useGrayscale,
    required this.onUseGrayscaleChanged,
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
    required this.selectedPalette,
    required this.onPaletteChanged,
    super.key,
  });

  final TilePreset? selectedPreset;
  final ValueChanged<TilePreset> onPresetSelected;
  final bool showTileGrid;
  final ValueChanged<bool> onShowTileGridChanged;
  final bool useGrayscale;
  final ValueChanged<bool> onUseGrayscaleChanged;
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
  final int selectedPalette;
  final ValueChanged<int> onPaletteChanged;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final l10n = AppLocalizations.of(context)!;

    return Builder(
      builder: (buttonContext) {
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
          onPressed: () => _showSettingsMenu(buttonContext),
        );
      },
    );
  }

  void _showSettingsMenu(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final theme = Theme.of(context);
    var menuPreset = selectedPreset;
    var menuShowTileGrid = showTileGrid;
    var menuUseGrayscale = useGrayscale;
    var menuCaptureMode = captureMode;
    var menuSelectedPalette = selectedPalette;

    final button = context.findRenderObject()! as RenderBox;
    final overlay =
        Overlay.of(context).context.findRenderObject()! as RenderBox;
    final buttonPosition = button.localToGlobal(Offset.zero, ancestor: overlay);

    showMenu<void>(
      context: context,
      position: RelativeRect.fromLTRB(
        buttonPosition.dx + button.size.width - 280,
        buttonPosition.dy + button.size.height + 4,
        overlay.size.width - buttonPosition.dx - button.size.width,
        0,
      ),
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
      items: [
        PopupMenuItem<void>(
          enabled: false,
          height: 32,
          padding: const EdgeInsets.symmetric(horizontal: 16),
          child: Text(
            l10n.tileViewerPresets,
            style: theme.textTheme.labelSmall?.copyWith(
              color: theme.colorScheme.onSurfaceVariant,
              fontWeight: FontWeight.w600,
            ),
          ),
        ),
        PopupMenuItem<void>(
          onTap: () {},
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          child: StatefulBuilder(
            builder: (context, setMenuState) => TilePresetButtons(
              presets: const [TilePreset.ppu, TilePreset.chr, TilePreset.rom],
              selectedPreset: menuPreset,
              onSelected: (preset) {
                menuPreset = preset;
                onPresetSelected(preset);
                setMenuState(() {});
              },
            ),
          ),
        ),
        PopupMenuItem<void>(
          onTap: () {},
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
          child: StatefulBuilder(
            builder: (context, setMenuState) => TilePresetButtons(
              presets: const [TilePreset.bg, TilePreset.oam],
              selectedPreset: menuPreset,
              onSelected: (preset) {
                menuPreset = preset;
                onPresetSelected(preset);
                setMenuState(() {});
              },
            ),
          ),
        ),
        const PopupMenuDivider(),
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
          onTap: () {},
          padding: EdgeInsets.zero,
          child: StatefulBuilder(
            builder: (context, setMenuState) => CheckboxListTile(
              dense: true,
              contentPadding: const EdgeInsets.symmetric(horizontal: 16),
              title: Text(l10n.tileViewerShowGrid),
              value: menuShowTileGrid,
              onChanged: (v) {
                menuShowTileGrid = v ?? false;
                onShowTileGridChanged(menuShowTileGrid);
                setMenuState(() {});
              },
            ),
          ),
        ),
        PopupMenuItem<void>(
          onTap: () {},
          padding: EdgeInsets.zero,
          child: StatefulBuilder(
            builder: (context, setMenuState) => CheckboxListTile(
              dense: true,
              contentPadding: const EdgeInsets.symmetric(horizontal: 16),
              title: Text(l10n.tileViewerGrayscale),
              value: menuUseGrayscale,
              onChanged: (v) {
                menuUseGrayscale = v ?? false;
                onUseGrayscaleChanged(menuUseGrayscale);
                setMenuState(() {});
              },
            ),
          ),
        ),
        const PopupMenuDivider(),
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
          onTap: () {},
          padding: EdgeInsets.zero,
          child: StatefulBuilder(
            builder: (context, setMenuState) => Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                RadioGroup<TileCaptureMode>(
                  groupValue: menuCaptureMode,
                  onChanged: (v) {
                    if (v == null) return;
                    menuCaptureMode = v;
                    onCaptureModeChanged(v);
                    setMenuState(() {});
                  },
                  child: Column(
                    children: [
                      RadioListTile<TileCaptureMode>(
                        dense: true,
                        visualDensity: VisualDensity.compact,
                        contentPadding: const EdgeInsets.symmetric(
                          horizontal: 16,
                        ),
                        title: Text(l10n.tilemapCaptureFrameStart),
                        value: TileCaptureMode.frameStart,
                      ),
                      RadioListTile<TileCaptureMode>(
                        dense: true,
                        visualDensity: VisualDensity.compact,
                        contentPadding: const EdgeInsets.symmetric(
                          horizontal: 16,
                        ),
                        title: Text(l10n.tilemapCaptureVblankStart),
                        value: TileCaptureMode.vblankStart,
                      ),
                      RadioListTile<TileCaptureMode>(
                        dense: true,
                        visualDensity: VisualDensity.compact,
                        contentPadding: const EdgeInsets.symmetric(
                          horizontal: 16,
                        ),
                        title: Text(l10n.tilemapCaptureManual),
                        value: TileCaptureMode.scanline,
                      ),
                    ],
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.fromLTRB(16, 0, 16, 10),
                  child: Row(
                    children: [
                      Expanded(
                        child: NumberField(
                          label: l10n.tilemapScanline,
                          enabled: menuCaptureMode == TileCaptureMode.scanline,
                          controller: scanlineController,
                          hint: '$minScanline ~ $maxScanline',
                          onSubmitted: (v) {
                            onScanlineSubmitted(v);
                            setMenuState(() {});
                          },
                        ),
                      ),
                      const SizedBox(width: 10),
                      Expanded(
                        child: NumberField(
                          label: l10n.tilemapDot,
                          enabled: menuCaptureMode == TileCaptureMode.scanline,
                          controller: dotController,
                          hint: '$minDot ~ $maxDot',
                          onSubmitted: (v) {
                            onDotSubmitted(v);
                            setMenuState(() {});
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
                value: menuSelectedPalette,
                entries: [
                  for (var i = 0; i < 8; i++)
                    DropdownMenuEntry(
                      value: i,
                      label: i < 4
                          ? l10n.tileViewerPaletteBg(i)
                          : l10n.tileViewerPaletteSprite(i - 4),
                    ),
                ],
                onSelected: (v) {
                  menuSelectedPalette = v;
                  menuPreset = null;
                  onPaletteChanged(v);
                  setMenuState(() {});
                },
              ),
            ),
          ),
        ),
      ],
    );
  }
}
