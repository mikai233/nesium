import 'package:flutter/material.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer_models.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer_state.dart';
import 'package:nesium_flutter/l10n/app_localizations.dart';
import 'package:nesium_flutter/platform/platform_capabilities.dart';
import 'package:nesium_flutter/widgets/animated_dropdown_menu.dart';

Future<void> showSpriteViewerSettingsMenu({
  required BuildContext context,
  required SpriteViewerDisplayOptions displayOptions,
  required SpriteViewerCaptureState captureState,
  required TextEditingController scanlineController,
  required TextEditingController dotController,
  required ValueChanged<SpriteViewerDisplayOptions> onDisplayOptionsChanged,
  required ValueChanged<SpriteViewerCaptureState> onCaptureStateChanged,
  required VoidCallback onApplyCaptureMode,
}) {
  // The popup has its own route; retain the latest values between edits.
  void updateDisplayOptions(SpriteViewerDisplayOptions value) {
    displayOptions = value;
    onDisplayOptionsChanged(value);
  }

  void updateCaptureState(SpriteViewerCaptureState value) {
    captureState = value;
    onCaptureStateChanged(value);
  }

  final theme = Theme.of(context);
  final l10n = AppLocalizations.of(context)!;

  final RenderBox button = context.findRenderObject() as RenderBox;
  final RenderBox overlay =
      Overlay.of(context).context.findRenderObject() as RenderBox;
  final buttonPosition = button.localToGlobal(Offset.zero, ancestor: overlay);

  return showMenu<void>(
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
          l10n.tilemapPanelDisplay,
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
              CheckboxListTile(
                dense: true,
                contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                title: Text(l10n.spriteViewerShowGrid),
                value: displayOptions.showGrid,
                onChanged: (value) {
                  updateDisplayOptions(
                    displayOptions.copyWith(showGrid: value ?? false),
                  );
                  setMenuState(() {});
                },
              ),
              CheckboxListTile(
                dense: true,
                contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                title: Text(l10n.spriteViewerShowOutline),
                value: displayOptions.showOutline,
                onChanged: (value) {
                  updateDisplayOptions(
                    displayOptions.copyWith(showOutline: value ?? false),
                  );
                  setMenuState(() {});
                },
              ),
              CheckboxListTile(
                dense: true,
                contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                title: Text(l10n.spriteViewerShowOffscreenRegions),
                value: displayOptions.showOffscreenRegions,
                onChanged: (value) {
                  updateDisplayOptions(
                    displayOptions.copyWith(
                      showOffscreenRegions: value ?? false,
                    ),
                  );
                  setMenuState(() {});
                },
              ),
              CheckboxListTile(
                dense: true,
                contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                title: Text(l10n.spriteViewerDimOffscreenSpritesGrid),
                value: displayOptions.dimOffscreenGrid,
                onChanged: (value) {
                  updateDisplayOptions(
                    displayOptions.copyWith(dimOffscreenGrid: value ?? false),
                  );
                  setMenuState(() {});
                },
              ),
              Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: 16,
                  vertical: 6,
                ),
                child: Row(
                  children: [
                    Expanded(child: Text(l10n.tileViewerBackground)),
                    SizedBox(
                      width: 180,
                      child: AnimatedDropdownMenu<SpriteViewerBackground>(
                        density: AnimatedDropdownMenuDensity.compact,
                        value: displayOptions.background,
                        entries: [
                          for (final background
                              in SpriteViewerBackground.values)
                            DropdownMenuEntry(
                              value: background,
                              label: background.label(l10n),
                            ),
                        ],
                        onSelected: (value) {
                          updateDisplayOptions(
                            displayOptions.copyWith(background: value),
                          );
                          setMenuState(() {});
                        },
                      ),
                    ),
                  ],
                ),
              ),
              if (isNativeDesktop)
                CheckboxListTile(
                  dense: true,
                  contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                  title: Text(l10n.spriteViewerShowListView),
                  value: displayOptions.showListView,
                  onChanged: (value) {
                    updateDisplayOptions(
                      displayOptions.copyWith(showListView: value ?? false),
                    );
                    setMenuState(() {});
                  },
                ),
            ],
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
              RadioGroup<SpriteCaptureMode>(
                groupValue: captureState.mode,
                onChanged: (value) {
                  if (value == null) return;
                  updateCaptureState(captureState.copyWith(mode: value));
                  setMenuState(() {});
                  onApplyCaptureMode();
                },
                child: Column(
                  children: [
                    RadioListTile<SpriteCaptureMode>(
                      dense: true,
                      visualDensity: VisualDensity.compact,
                      contentPadding: const EdgeInsets.symmetric(
                        horizontal: 16,
                      ),
                      title: Text(l10n.tilemapCaptureFrameStart),
                      value: SpriteCaptureMode.frameStart,
                    ),
                    RadioListTile<SpriteCaptureMode>(
                      dense: true,
                      visualDensity: VisualDensity.compact,
                      contentPadding: const EdgeInsets.symmetric(
                        horizontal: 16,
                      ),
                      title: Text(l10n.tilemapCaptureVblankStart),
                      value: SpriteCaptureMode.vblankStart,
                    ),
                    RadioListTile<SpriteCaptureMode>(
                      dense: true,
                      visualDensity: VisualDensity.compact,
                      contentPadding: const EdgeInsets.symmetric(
                        horizontal: 16,
                      ),
                      title: Text(l10n.tilemapCaptureManual),
                      value: SpriteCaptureMode.scanline,
                    ),
                  ],
                ),
              ),
              Padding(
                padding: const EdgeInsets.fromLTRB(16, 0, 16, 10),
                child: Row(
                  children: [
                    Expanded(
                      child: _SpriteNumberField(
                        label: l10n.tilemapScanline,
                        enabled:
                            captureState.mode == SpriteCaptureMode.scanline,
                        controller: scanlineController,
                        hint:
                            '${SpriteViewerLayout.minScanline} ~ ${SpriteViewerLayout.maxScanline}',
                        onSubmitted: (value) {
                          final parsed = int.tryParse(value);
                          if (parsed == null ||
                              parsed < SpriteViewerLayout.minScanline ||
                              parsed > SpriteViewerLayout.maxScanline) {
                            return;
                          }
                          updateCaptureState(
                            captureState.copyWith(scanline: parsed),
                          );
                          scanlineController.text = parsed.toString();
                          setMenuState(() {});
                          onApplyCaptureMode();
                        },
                      ),
                    ),
                    const SizedBox(width: 10),
                    Expanded(
                      child: _SpriteNumberField(
                        label: l10n.tilemapDot,
                        enabled:
                            captureState.mode == SpriteCaptureMode.scanline,
                        controller: dotController,
                        hint:
                            '${SpriteViewerLayout.minDot} ~ ${SpriteViewerLayout.maxDot}',
                        onSubmitted: (value) {
                          final parsed = int.tryParse(value);
                          if (parsed == null ||
                              parsed < SpriteViewerLayout.minDot ||
                              parsed > SpriteViewerLayout.maxDot) {
                            return;
                          }
                          updateCaptureState(
                            captureState.copyWith(dot: parsed),
                          );
                          dotController.text = parsed.toString();
                          setMenuState(() {});
                          onApplyCaptureMode();
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
    ],
  );
}

class _SpriteNumberField extends StatelessWidget {
  const _SpriteNumberField({
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
