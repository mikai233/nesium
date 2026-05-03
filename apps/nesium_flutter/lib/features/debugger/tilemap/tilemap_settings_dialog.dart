import 'package:flutter/material.dart';
import 'package:nesium_flutter/l10n/app_localizations.dart';
import 'package:nesium_flutter/widgets/animated_dropdown_menu.dart';
import 'tilemap_models.dart';

Future<void> showTilemapSettingsMenu({
  required BuildContext context,
  required Offset buttonPosition,
  required Size buttonSize,
  required TilemapDisplayMode displayMode,
  required bool showTileGrid,
  required bool showAttributeGrid,
  required bool showAttributeGrid32,
  required bool showNametableDelimiters,
  required bool showScrollOverlay,
  required TilemapCaptureMode captureMode,
  required int scanline,
  required int dot,
  required ValueChanged<TilemapDisplayMode> onDisplayModeChanged,
  required ValueChanged<bool?> onShowTileGridChanged,
  required ValueChanged<bool?> onShowAttributeGridChanged,
  required ValueChanged<bool?> onShowAttributeGrid32Changed,
  required ValueChanged<bool?> onShowNametableDelimitersChanged,
  required ValueChanged<bool?> onShowScrollOverlayChanged,
  required ValueChanged<TilemapCaptureMode> onCaptureModeChanged,
  required void Function(int scanline, int dot) onScanlineDotChanged,
}) async {
  final l10n = AppLocalizations.of(context)!;
  final theme = Theme.of(context);
  final RenderBox overlay =
      Overlay.of(context).context.findRenderObject() as RenderBox;

  await showMenu<void>(
    context: context,
    position: RelativeRect.fromLTRB(
      buttonPosition.dx + buttonSize.width - 280,
      buttonPosition.dy + buttonSize.height + 4,
      overlay.size.width - buttonPosition.dx - buttonSize.width,
      0,
    ),
    shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
    items: [
      PopupMenuItem<void>(
        onTap: null,
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
                          onDisplayModeChanged(v);
                          setMenuState(() {});
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
                value: showTileGrid,
                onChanged: (v) {
                  onShowTileGridChanged(v);
                  setMenuState(() {});
                },
              ),
              CheckboxListTile(
                dense: true,
                contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                title: Text(l10n.tilemapAttrGrid),
                value: showAttributeGrid,
                onChanged: (v) {
                  onShowAttributeGridChanged(v);
                  setMenuState(() {});
                },
              ),
              CheckboxListTile(
                dense: true,
                contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                title: Text(l10n.tilemapAttrGrid32),
                value: showAttributeGrid32,
                onChanged: (v) {
                  onShowAttributeGrid32Changed(v);
                  setMenuState(() {});
                },
              ),
              CheckboxListTile(
                dense: true,
                contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                title: Text(l10n.tilemapNtBounds),
                value: showNametableDelimiters,
                onChanged: (v) {
                  onShowNametableDelimitersChanged(v);
                  setMenuState(() {});
                },
              ),
              CheckboxListTile(
                dense: true,
                contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                title: Text(l10n.tilemapScrollOverlay),
                value: showScrollOverlay,
                onChanged: (v) {
                  onShowScrollOverlayChanged(v);
                  setMenuState(() {});
                },
              ),
            ],
          ),
        ),
      ),
      const PopupMenuDivider(height: 1),
      PopupMenuItem<void>(
        onTap: null,
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
        onTap: () {},
        padding: EdgeInsets.zero,
        child: StatefulBuilder(
          builder: (context, setMenuState) => RadioGroup<TilemapCaptureMode>(
            groupValue: captureMode,
            onChanged: (v) {
              if (v == null) return;
              onCaptureModeChanged(v);
              setMenuState(() {});
            },
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                RadioListTile<TilemapCaptureMode>(
                  dense: true,
                  contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                  title: Text(l10n.tilemapCaptureFrameStart),
                  value: TilemapCaptureMode.frameStart,
                ),
                RadioListTile<TilemapCaptureMode>(
                  dense: true,
                  contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                  title: Text(l10n.tilemapCaptureVblankStart),
                  value: TilemapCaptureMode.vblankStart,
                ),
                RadioListTile<TilemapCaptureMode>(
                  dense: true,
                  contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                  title: Text(l10n.tilemapCaptureManual),
                  value: TilemapCaptureMode.scanline,
                ),
                if (captureMode == TilemapCaptureMode.scanline) ...[
                  const PopupMenuDivider(height: 1),
                  PopupMenuItem<void>(
                    onTap: () {},
                    padding: EdgeInsets.zero,
                    child: Padding(
                      padding: const EdgeInsets.all(16),
                      child: _ScanlineControls(
                        scanline: scanline,
                        dot: dot,
                        onChanged: (s, d) {
                          onScanlineDotChanged(s, d);
                          setMenuState(() {});
                        },
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

class _ScanlineControls extends StatelessWidget {
  const _ScanlineControls({
    required this.scanline,
    required this.dot,
    required this.onChanged,
  });

  final int scanline;
  final int dot;
  final void Function(int scanline, int dot) onChanged;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final l10n = AppLocalizations.of(context)!;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      mainAxisSize: MainAxisSize.min,
      children: [
        Text('${l10n.tilemapScanline}:', style: theme.textTheme.bodySmall),
        const SizedBox(height: 4),
        TextField(
          controller: TextEditingController(text: scanline.toString())
            ..selection = TextSelection.fromPosition(
              TextPosition(offset: scanline.toString().length),
            ),
          decoration: const InputDecoration(
            isDense: true,
            contentPadding: EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            border: OutlineInputBorder(),
            hintText: '-1 ~ 260',
          ),
          keyboardType: TextInputType.number,
          onSubmitted: (v) {
            final value = int.tryParse(v);
            if (value != null && value >= -1 && value <= 260) {
              onChanged(value, dot);
            }
          },
        ),
        const SizedBox(height: 12),
        Text('${l10n.tilemapDot}:', style: theme.textTheme.bodySmall),
        const SizedBox(height: 4),
        TextField(
          controller: TextEditingController(text: dot.toString())
            ..selection = TextSelection.fromPosition(
              TextPosition(offset: dot.toString().length),
            ),
          decoration: const InputDecoration(
            isDense: true,
            contentPadding: EdgeInsets.symmetric(horizontal: 12, vertical: 8),
            border: OutlineInputBorder(),
            hintText: '0 ~ 340',
          ),
          keyboardType: TextInputType.number,
          onSubmitted: (v) {
            final value = int.tryParse(v);
            if (value != null && value >= 0 && value <= 340) {
              onChanged(scanline, value);
            }
          },
        ),
      ],
    );
  }
}
