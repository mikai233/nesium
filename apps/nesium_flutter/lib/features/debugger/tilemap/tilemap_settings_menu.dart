import 'package:flutter/material.dart';
import 'package:nesium_flutter/features/debugger/tilemap/tilemap_models.dart';
import 'package:nesium_flutter/l10n/app_localizations.dart';
import 'package:nesium_flutter/widgets/animated_dropdown_menu.dart';

class TilemapSettingsButton extends StatelessWidget {
  const TilemapSettingsButton({
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
    required this.scanline,
    required this.dot,
    required this.minScanline,
    required this.maxScanline,
    required this.minDot,
    required this.maxDot,
    required this.onScanlineSubmitted,
    required this.onDotSubmitted,
    super.key,
  });

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
  final int scanline;
  final int dot;
  final int minScanline;
  final int maxScanline;
  final int minDot;
  final int maxDot;
  final ValueChanged<String> onScanlineSubmitted;
  final ValueChanged<String> onDotSubmitted;

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
          tooltip: l10n.tilemapSettings,
          onPressed: () => _showSettingsMenu(buttonContext),
        );
      },
    );
  }

  void _showSettingsMenu(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final theme = Theme.of(context);
    var menuDisplayMode = displayMode;
    var menuShowTileGrid = showTileGrid;
    var menuShowAttributeGrid = showAttributeGrid;
    var menuShowAttributeGrid32 = showAttributeGrid32;
    var menuShowNametableDelimiters = showNametableDelimiters;
    var menuShowScrollOverlay = showScrollOverlay;
    var menuCaptureMode = captureMode;
    var menuScanline = scanline;
    var menuDot = dot;
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
                          value: menuDisplayMode,
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
                            menuDisplayMode = v;
                            onDisplayModeChanged(v);
                            setMenuState(() {});
                          },
                        ),
                      ),
                    ],
                  ),
                ),
                _menuCheck(
                  title: l10n.tilemapTileGrid,
                  value: menuShowTileGrid,
                  onChanged: (v) {
                    menuShowTileGrid = v;
                    onShowTileGridChanged(v);
                  },
                  setMenuState: setMenuState,
                ),
                _menuCheck(
                  title: l10n.tilemapAttrGrid,
                  value: menuShowAttributeGrid,
                  onChanged: (v) {
                    menuShowAttributeGrid = v;
                    onShowAttributeGridChanged(v);
                  },
                  setMenuState: setMenuState,
                ),
                _menuCheck(
                  title: l10n.tilemapAttrGrid32,
                  value: menuShowAttributeGrid32,
                  onChanged: (v) {
                    menuShowAttributeGrid32 = v;
                    onShowAttributeGrid32Changed(v);
                  },
                  setMenuState: setMenuState,
                ),
                _menuCheck(
                  title: l10n.tilemapNtBounds,
                  value: menuShowNametableDelimiters,
                  onChanged: (v) {
                    menuShowNametableDelimiters = v;
                    onShowNametableDelimitersChanged(v);
                  },
                  setMenuState: setMenuState,
                ),
                _menuCheck(
                  title: l10n.tilemapScrollOverlay,
                  value: menuShowScrollOverlay,
                  onChanged: (v) {
                    menuShowScrollOverlay = v;
                    onShowScrollOverlayChanged(v);
                  },
                  setMenuState: setMenuState,
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
              groupValue: menuCaptureMode,
              onChanged: (v) {
                if (v == null) return;
                menuCaptureMode = v;
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
                  if (menuCaptureMode == TilemapCaptureMode.scanline) ...[
                    const PopupMenuDivider(height: 1),
                    PopupMenuItem<void>(
                      onTap: () {},
                      padding: EdgeInsets.zero,
                      child: Padding(
                        padding: const EdgeInsets.all(16),
                        child: _ScanlineControls(
                          theme: theme,
                          l10n: l10n,
                          scanline: menuScanline,
                          dot: menuDot,
                          minScanline: minScanline,
                          maxScanline: maxScanline,
                          minDot: minDot,
                          maxDot: maxDot,
                          onScanlineSubmitted: (v) {
                            final parsed = int.tryParse(v);
                            if (parsed != null &&
                                parsed >= minScanline &&
                                parsed <= maxScanline) {
                              menuScanline = parsed;
                            }
                            onScanlineSubmitted(v);
                            setMenuState(() {});
                          },
                          onDotSubmitted: (v) {
                            final parsed = int.tryParse(v);
                            if (parsed != null &&
                                parsed >= minDot &&
                                parsed <= maxDot) {
                              menuDot = parsed;
                            }
                            onDotSubmitted(v);
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

  Widget _menuCheck({
    required String title,
    required bool value,
    required ValueChanged<bool> onChanged,
    required StateSetter setMenuState,
  }) {
    return CheckboxListTile(
      dense: true,
      contentPadding: const EdgeInsets.symmetric(horizontal: 16),
      title: Text(title),
      value: value,
      onChanged: (v) {
        onChanged(v ?? false);
        setMenuState(() {});
      },
    );
  }
}

class _ScanlineControls extends StatelessWidget {
  const _ScanlineControls({
    required this.theme,
    required this.l10n,
    required this.scanline,
    required this.dot,
    required this.minScanline,
    required this.maxScanline,
    required this.minDot,
    required this.maxDot,
    required this.onScanlineSubmitted,
    required this.onDotSubmitted,
  });

  final ThemeData theme;
  final AppLocalizations l10n;
  final int scanline;
  final int dot;
  final int minScanline;
  final int maxScanline;
  final int minDot;
  final int maxDot;
  final ValueChanged<String> onScanlineSubmitted;
  final ValueChanged<String> onDotSubmitted;

  @override
  Widget build(BuildContext context) {
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
          decoration: InputDecoration(
            isDense: true,
            contentPadding: const EdgeInsets.symmetric(
              horizontal: 12,
              vertical: 8,
            ),
            border: const OutlineInputBorder(),
            hintText: '$minScanline ~ $maxScanline',
          ),
          keyboardType: TextInputType.number,
          onSubmitted: onScanlineSubmitted,
        ),
        const SizedBox(height: 12),
        Text('${l10n.tilemapDot}:', style: theme.textTheme.bodySmall),
        const SizedBox(height: 4),
        TextField(
          controller: TextEditingController(text: dot.toString())
            ..selection = TextSelection.fromPosition(
              TextPosition(offset: dot.toString().length),
            ),
          decoration: InputDecoration(
            isDense: true,
            contentPadding: const EdgeInsets.symmetric(
              horizontal: 12,
              vertical: 8,
            ),
            border: const OutlineInputBorder(),
            hintText: '$minDot ~ $maxDot',
          ),
          keyboardType: TextInputType.number,
          onSubmitted: onDotSubmitted,
        ),
      ],
    );
  }
}
