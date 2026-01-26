import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../../l10n/app_localizations.dart';
import '../../../../logging/app_logger.dart';
import '../../../../platform/nes_palette.dart' as nes_palette;
import '../../../../widgets/animated_dropdown_menu.dart';
import '../../video_settings.dart';

class PaletteSettingsCard extends ConsumerWidget {
  const PaletteSettingsCard({
    required this.pickAndApplyCustomPalette,
    super.key,
  });

  final Future<void> Function(BuildContext, VideoSettingsController)
  pickAndApplyCustomPalette;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final videoSettings = ref.watch(videoSettingsProvider);
    final videoController = ref.read(videoSettingsProvider.notifier);

    Future<void> onPaletteModeSelected(PaletteMode value) async {
      if (value == PaletteMode.builtin) {
        try {
          await videoController.setBuiltinPreset(videoSettings.builtinPreset);
        } catch (e, st) {
          logWarning(
            e,
            stackTrace: st,
            message: 'setBuiltinPreset failed',
            logger: 'palette_settings_card',
          );
        }
        return;
      }
      final hasCustom = videoSettings.customPaletteName != null;
      try {
        await videoController.setPaletteMode(PaletteMode.custom);
      } catch (e, st) {
        logWarning(
          e,
          stackTrace: st,
          message: 'setPaletteMode failed',
          logger: 'palette_settings_card',
        );
      }
      if (hasCustom) return;
      if (!context.mounted) return;
      await pickAndApplyCustomPalette(context, videoController);
    }

    Future<void> setBuiltinPalette(nes_palette.PaletteKind value) async {
      try {
        await videoController.setBuiltinPreset(value);
      } catch (e, st) {
        logWarning(
          e,
          stackTrace: st,
          message: 'setBuiltinPreset failed',
          logger: 'palette_settings_card',
        );
      }
    }

    return Column(
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 12),
          child: AnimatedDropdownMenu<PaletteMode>(
            labelText: l10n.paletteModeLabel,
            value: videoSettings.paletteMode,
            entries: [
              DropdownMenuEntry(
                value: PaletteMode.builtin,
                label: l10n.paletteModeBuiltin,
              ),
              DropdownMenuEntry(
                value: PaletteMode.custom,
                label: videoSettings.customPaletteName == null
                    ? l10n.paletteModeCustom
                    : l10n.paletteModeCustomActive(
                        videoSettings.customPaletteName!,
                      ),
              ),
            ],
            onSelected: onPaletteModeSelected,
          ),
        ),
        const SizedBox(height: 12),
        if (videoSettings.paletteMode == PaletteMode.builtin)
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 12),
            child: AnimatedDropdownMenu<nes_palette.PaletteKind>(
              labelText: l10n.builtinPaletteLabel,
              value: videoSettings.builtinPreset,
              entries: const [
                DropdownMenuEntry(
                  value: nes_palette.PaletteKind.nesdevNtsc,
                  label: 'Nesdev (NTSC)',
                ),
                DropdownMenuEntry(
                  value: nes_palette.PaletteKind.fbxCompositeDirect,
                  label: 'FirebrandX (Composite Direct)',
                ),
                DropdownMenuEntry(
                  value: nes_palette.PaletteKind.sonyCxa2025AsUs,
                  label: 'Sony CXA2025AS (US)',
                ),
                DropdownMenuEntry(
                  value: nes_palette.PaletteKind.pal2C07,
                  label: 'RP2C07 (PAL)',
                ),
                DropdownMenuEntry(
                  value: nes_palette.PaletteKind.rawLinear,
                  label: 'Raw linear',
                ),
              ],
              onSelected: setBuiltinPalette,
            ),
          )
        else
          ListTile(
            contentPadding: const EdgeInsets.symmetric(horizontal: 12),
            title: Text(l10n.customPaletteLoadTitle),
            subtitle: Text(
              videoSettings.customPaletteName == null
                  ? l10n.customPaletteLoadSubtitle
                  : l10n.paletteModeCustomActive(
                      videoSettings.customPaletteName!,
                    ),
            ),
            trailing: IconButton.filledTonal(
              tooltip: l10n.actionLoadPalette,
              icon: const Icon(Icons.folder_open),
              onPressed: () =>
                  pickAndApplyCustomPalette(context, videoController),
            ),
            onTap: () => pickAndApplyCustomPalette(context, videoController),
          ),
      ],
    );
  }
}
