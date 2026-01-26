import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../../domain/nes_controller.dart';
import '../../../../l10n/app_localizations.dart';
import '../../../../platform/nes_video.dart' as nes_video;
import '../../../../widgets/animated_dropdown_menu.dart';
import '../../../../widgets/animated_settings_widgets.dart';
import '../../../../windows/current_window_kind.dart';
import '../../../../windows/window_types.dart';
import '../../../../platform/platform_capabilities.dart';
import '../../video_settings.dart';

class CpuFilterCard extends ConsumerWidget {
  const CpuFilterCard({this.useWrapper = true, super.key});

  final bool useWrapper;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final videoSettings = ref.watch(videoSettingsProvider);
    final videoController = ref.read(videoSettingsProvider.notifier);

    Future<void> setVideoFilter(nes_video.VideoFilter value) async {
      try {
        await videoController.setVideoFilter(value);

        final isNtsc =
            value == nes_video.VideoFilter.ntscComposite ||
            value == nes_video.VideoFilter.ntscSVideo ||
            value == nes_video.VideoFilter.ntscRgb ||
            value == nes_video.VideoFilter.ntscMonochrome;
        if (isNtsc) {
          final options = ref.read(videoSettingsProvider).ntscOptions;
          await nes_video.setNtscOptions(options: options);
        }

        final isNtscBisqwit =
            value == nes_video.VideoFilter.ntscBisqwit2X ||
            value == nes_video.VideoFilter.ntscBisqwit4X ||
            value == nes_video.VideoFilter.ntscBisqwit8X;
        if (isNtscBisqwit) {
          final options = ref.read(videoSettingsProvider).ntscBisqwitOptions;
          await nes_video.setNtscBisqwitOptions(options: options);
        }

        if (value == nes_video.VideoFilter.lcdGrid) {
          final strength = ref.read(videoSettingsProvider).lcdGridStrength;
          await nes_video.setLcdGridOptions(
            options: nes_video.LcdGridOptions(strength: strength),
          );
        }

        if (value == nes_video.VideoFilter.scanlines) {
          final intensity = ref.read(videoSettingsProvider).scanlineIntensity;
          await nes_video.setScanlineOptions(
            options: nes_video.ScanlineOptions(intensity: intensity),
          );
        }

        final kind = ref.read(currentWindowKindProvider);
        final applyInThisEngine = !isNativeDesktop || kind == WindowKind.main;

        if (applyInThisEngine) {
          await ref.read(nesControllerProvider.notifier).setVideoFilter(value);
        }
      } catch (e) {
        // Log error here if needed, but keeping it simple for now
      }
    }

    final content = Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(12, 12, 12, 4),
          child: Text(
            l10n.videoFilterCategoryCpu,
            style: Theme.of(context).textTheme.titleSmall?.copyWith(
              color: Theme.of(context).colorScheme.primary,
            ),
          ),
        ),
        Padding(
          padding: const EdgeInsets.fromLTRB(12, 0, 12, 12),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              AnimatedDropdownMenu<nes_video.VideoFilter>(
                labelText: l10n.videoFilterLabel,
                value: videoSettings.videoFilter,
                onSelected: setVideoFilter,
                entries: [
                  DropdownMenuEntry(
                    value: nes_video.VideoFilter.none,
                    label: l10n.videoFilterNone,
                  ),
                  DropdownMenuEntry(
                    value: nes_video.VideoFilter.prescale2X,
                    label: l10n.videoFilterPrescale2x,
                  ),
                  DropdownMenuEntry(
                    value: nes_video.VideoFilter.prescale3X,
                    label: l10n.videoFilterPrescale3x,
                  ),
                  DropdownMenuEntry(
                    value: nes_video.VideoFilter.prescale4X,
                    label: l10n.videoFilterPrescale4x,
                  ),
                  DropdownMenuEntry(
                    value: nes_video.VideoFilter.sai2X,
                    label: l10n.videoFilter2xSai,
                  ),
                  DropdownMenuEntry(
                    value: nes_video.VideoFilter.super2XSai,
                    label: l10n.videoFilterSuper2xSai,
                  ),
                  DropdownMenuEntry(
                    value: nes_video.VideoFilter.superEagle,
                    label: l10n.videoFilterSuperEagle,
                  ),
                  DropdownMenuEntry(
                    value: nes_video.VideoFilter.lcdGrid,
                    label: l10n.videoFilterLcdGrid,
                  ),
                  DropdownMenuEntry(
                    value: nes_video.VideoFilter.scanlines,
                    label: l10n.videoFilterScanlines,
                  ),
                  if (!kIsWeb) ...[
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.hq2X,
                      label: l10n.videoFilterHq2x,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.hq3X,
                      label: l10n.videoFilterHq3x,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.hq4X,
                      label: l10n.videoFilterHq4x,
                    ),
                  ],
                  DropdownMenuEntry(
                    value: nes_video.VideoFilter.xbrz2X,
                    label: l10n.videoFilterXbrz2x,
                  ),
                  DropdownMenuEntry(
                    value: nes_video.VideoFilter.xbrz3X,
                    label: l10n.videoFilterXbrz3x,
                  ),
                  DropdownMenuEntry(
                    value: nes_video.VideoFilter.xbrz4X,
                    label: l10n.videoFilterXbrz4x,
                  ),
                  DropdownMenuEntry(
                    value: nes_video.VideoFilter.xbrz5X,
                    label: l10n.videoFilterXbrz5x,
                  ),
                  DropdownMenuEntry(
                    value: nes_video.VideoFilter.xbrz6X,
                    label: l10n.videoFilterXbrz6x,
                  ),
                  if (!kIsWeb) ...[
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.ntscComposite,
                      label: l10n.videoFilterNtscComposite,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.ntscSVideo,
                      label: l10n.videoFilterNtscSvideo,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.ntscRgb,
                      label: l10n.videoFilterNtscRgb,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.ntscMonochrome,
                      label: l10n.videoFilterNtscMonochrome,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.ntscBisqwit2X,
                      label: l10n.videoFilterNtscBisqwit2x,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.ntscBisqwit4X,
                      label: l10n.videoFilterNtscBisqwit4x,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.ntscBisqwit8X,
                      label: l10n.videoFilterNtscBisqwit8x,
                    ),
                  ],
                ],
              ),
              const SizedBox(height: 12),
              if (videoSettings.videoFilter == nes_video.VideoFilter.lcdGrid)
                AnimatedSliderTile(
                  label: l10n.videoLcdGridStrengthLabel,
                  value: videoSettings.lcdGridStrength,
                  min: 0,
                  max: 1,
                  divisions: 100,
                  valueLabel:
                      '${(videoSettings.lcdGridStrength * 100).round()}%',
                  onChanged: (value) {
                    unawaited(videoController.setLcdGridStrength(value));
                  },
                ),
              if (videoSettings.videoFilter == nes_video.VideoFilter.lcdGrid)
                const SizedBox(height: 12),
              if (videoSettings.videoFilter == nes_video.VideoFilter.scanlines)
                AnimatedSliderTile(
                  label: l10n.videoScanlinesIntensityLabel,
                  value: videoSettings.scanlineIntensity,
                  min: 0,
                  max: 1,
                  divisions: 100,
                  valueLabel:
                      '${(videoSettings.scanlineIntensity * 100).round()}%',
                  onChanged: (value) {
                    unawaited(videoController.setScanlineIntensity(value));
                  },
                ),
              if (videoSettings.videoFilter == nes_video.VideoFilter.scanlines)
                const SizedBox(height: 12),
            ],
          ),
        ),
      ],
    );

    if (useWrapper) {
      return AnimatedSettingsCard(index: 0, child: content);
    }
    return content;
  }
}
