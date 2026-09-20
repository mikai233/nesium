import 'package:flutter/material.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as bridge;
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer_models.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer_state.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/sprite_viewer_utils.dart';
import 'package:nesium_flutter/features/debugger/sprite_viewer/widgets/sprite_info_card.dart';
import 'package:nesium_flutter/l10n/app_localizations.dart';
import 'package:nesium_flutter/widgets/animated_dropdown_menu.dart';
import 'package:nesium_flutter/widgets/single_position_scrollbar.dart';

class SpriteSidePanel extends StatelessWidget {
  const SpriteSidePanel({
    super.key,
    required this.snapshot,
    required this.grid,
    required this.displayOptions,
    required this.captureState,
    required this.scanlineController,
    required this.dotController,
    required this.selectedIndex,
    required this.thumbTextureId,
    required this.onDisplayOptionsChanged,
    required this.onCaptureStateChanged,
    required this.onApplyCaptureMode,
  });

  final bridge.SpriteSnapshot snapshot;
  final Widget grid;
  final SpriteViewerDisplayOptions displayOptions;
  final SpriteViewerCaptureState captureState;
  final TextEditingController scanlineController;
  final TextEditingController dotController;
  final int? selectedIndex;
  final int? thumbTextureId;
  final ValueChanged<SpriteViewerDisplayOptions> onDisplayOptionsChanged;
  final ValueChanged<SpriteViewerCaptureState> onCaptureStateChanged;
  final VoidCallback onApplyCaptureMode;

  @override
  Widget build(BuildContext context) {
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;
    final l10n = AppLocalizations.of(context)!;

    final selected =
        selectedIndex != null &&
            selectedIndex! >= 0 &&
            selectedIndex! < snapshot.sprites.length
        ? snapshot.sprites[selectedIndex!]
        : null;

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
              _SideSection(
                title: l10n.spriteViewerPanelSprites,
                child: LayoutBuilder(
                  builder: (context, constraints) {
                    final gridW = SpriteViewerLayout.gridTextureWidth(snapshot);
                    final gridH = SpriteViewerLayout.gridTextureHeight(
                      snapshot,
                    );
                    final aspectRatio = gridH == 0 ? 1.0 : gridW / gridH;
                    final height = (constraints.maxWidth / aspectRatio).clamp(
                      150.0,
                      300.0,
                    );
                    return SizedBox(height: height, child: grid);
                  },
                ),
              ),
              CheckboxListTile(
                dense: true,
                visualDensity: VisualDensity.compact,
                controlAffinity: ListTileControlAffinity.trailing,
                contentPadding: EdgeInsets.zero,
                title: Text(l10n.spriteViewerShowListView),
                value: displayOptions.showListView,
                onChanged: (value) => onDisplayOptionsChanged(
                  displayOptions.copyWith(showListView: value ?? false),
                ),
              ),
              const SizedBox(height: 8),
              _SideSection(
                title: l10n.tilemapPanelDisplay,
                child: Column(
                  children: [
                    _panelToggle(
                      title: l10n.spriteViewerShowGrid,
                      value: displayOptions.showGrid,
                      onChanged: (value) => onDisplayOptionsChanged(
                        displayOptions.copyWith(showGrid: value ?? false),
                      ),
                    ),
                    _panelToggle(
                      title: l10n.spriteViewerShowOutline,
                      value: displayOptions.showOutline,
                      onChanged: (value) => onDisplayOptionsChanged(
                        displayOptions.copyWith(showOutline: value ?? false),
                      ),
                    ),
                    _panelToggle(
                      title: l10n.spriteViewerShowOffscreenRegions,
                      value: displayOptions.showOffscreenRegions,
                      onChanged: (value) => onDisplayOptionsChanged(
                        displayOptions.copyWith(
                          showOffscreenRegions: value ?? false,
                        ),
                      ),
                    ),
                    _panelToggle(
                      title: l10n.spriteViewerDimOffscreenSpritesGrid,
                      value: displayOptions.dimOffscreenGrid,
                      onChanged: (value) => onDisplayOptionsChanged(
                        displayOptions.copyWith(
                          dimOffscreenGrid: value ?? false,
                        ),
                      ),
                    ),
                    const SizedBox(height: 6),
                    Row(
                      children: [
                        Expanded(
                          child: Text(
                            l10n.tileViewerBackground,
                            style: theme.textTheme.bodySmall,
                          ),
                        ),
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
                              onDisplayOptionsChanged(
                                displayOptions.copyWith(background: value),
                              );
                            },
                          ),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
              _SideSection(
                title: l10n.spriteViewerPanelDataSource,
                child: Row(
                  children: [
                    Expanded(
                      child: Text(
                        l10n.tileViewerSource,
                        style: theme.textTheme.bodySmall,
                      ),
                    ),
                    SizedBox(
                      width: 180,
                      child: AnimatedDropdownMenu<SpriteDataSource>(
                        density: AnimatedDropdownMenuDensity.compact,
                        value: displayOptions.dataSource,
                        entries: [
                          for (final source in SpriteDataSource.values)
                            DropdownMenuEntry(
                              value: source,
                              enabled: source == SpriteDataSource.spriteRam,
                              label: source.label(l10n),
                            ),
                        ],
                        onSelected: (value) {
                          if (value != SpriteDataSource.spriteRam) return;
                          onDisplayOptionsChanged(
                            displayOptions.copyWith(dataSource: value),
                          );
                        },
                      ),
                    ),
                  ],
                ),
              ),
              _SideSection(
                title: l10n.tilemapCapture,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    RadioGroup<SpriteCaptureMode>(
                      groupValue: captureState.mode,
                      onChanged: (value) {
                        if (value == null) return;
                        onCaptureStateChanged(
                          captureState.copyWith(mode: value),
                        );
                        onApplyCaptureMode();
                      },
                      child: Column(
                        children: [
                          RadioListTile<SpriteCaptureMode>(
                            dense: true,
                            visualDensity: VisualDensity.compact,
                            contentPadding: EdgeInsets.zero,
                            title: Text(l10n.tilemapCaptureFrameStart),
                            value: SpriteCaptureMode.frameStart,
                          ),
                          RadioListTile<SpriteCaptureMode>(
                            dense: true,
                            visualDensity: VisualDensity.compact,
                            contentPadding: EdgeInsets.zero,
                            title: Text(l10n.tilemapCaptureVblankStart),
                            value: SpriteCaptureMode.vblankStart,
                          ),
                          RadioListTile<SpriteCaptureMode>(
                            dense: true,
                            visualDensity: VisualDensity.compact,
                            contentPadding: EdgeInsets.zero,
                            title: Text(l10n.tilemapCaptureManual),
                            value: SpriteCaptureMode.scanline,
                          ),
                        ],
                      ),
                    ),
                    const SizedBox(height: 10),
                    Row(
                      children: [
                        Expanded(
                          child: _SpriteSideNumberField(
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
                              onCaptureStateChanged(
                                captureState.copyWith(scanline: parsed),
                              );
                              scanlineController.text = parsed.toString();
                              onApplyCaptureMode();
                            },
                          ),
                        ),
                        const SizedBox(width: 10),
                        Expanded(
                          child: _SpriteSideNumberField(
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
                              onCaptureStateChanged(
                                captureState.copyWith(dot: parsed),
                              );
                              dotController.text = parsed.toString();
                              onApplyCaptureMode();
                            },
                          ),
                        ),
                      ],
                    ),
                  ],
                ),
              ),
              _SideSection(
                title: l10n.spriteViewerPanelSprite,
                child: Column(
                  children: [
                    _kv(
                      context,
                      l10n.spriteViewerLabelMode,
                      spriteSizeLabel(snapshot),
                    ),
                    _kv(
                      context,
                      l10n.spriteViewerLabelPatternBase,
                      formatHexValue(snapshot.patternBase, width: 4),
                    ),
                    _kv(
                      context,
                      l10n.spriteViewerLabelThumbnailSize,
                      '${snapshot.thumbnailWidth}×${snapshot.thumbnailHeight}',
                    ),
                  ],
                ),
              ),
              _SideSection(
                title: l10n.spriteViewerPanelSelectedSprite,
                child: selected == null || thumbTextureId == null
                    ? Text(
                        '—',
                        style: TextStyle(color: colorScheme.onSurfaceVariant),
                      )
                    : SpriteInfoCard(
                        sprite: selected,
                        snapshot: snapshot,
                        textureId: thumbTextureId!,
                      ),
              ),
            ],
          );
        },
      ),
    );
  }

  Widget _panelToggle({
    required String title,
    required bool value,
    required ValueChanged<bool?> onChanged,
  }) {
    return CheckboxListTile(
      dense: true,
      visualDensity: VisualDensity.compact,
      controlAffinity: ListTileControlAffinity.trailing,
      contentPadding: EdgeInsets.zero,
      title: Text(title),
      value: value,
      onChanged: onChanged,
    );
  }

  Widget _kv(BuildContext context, String label, String value) {
    final theme = Theme.of(context);
    final labelStyle = theme.textTheme.bodySmall?.copyWith(
      color: theme.colorScheme.onSurfaceVariant,
    );
    final valueStyle = theme.textTheme.bodySmall?.copyWith(
      fontWeight: FontWeight.w600,
      fontFamily: 'monospace',
    );

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          Expanded(child: Text(label, style: labelStyle)),
          Text(value, style: valueStyle),
        ],
      ),
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

class _SpriteSideNumberField extends StatelessWidget {
  const _SpriteSideNumberField({
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
