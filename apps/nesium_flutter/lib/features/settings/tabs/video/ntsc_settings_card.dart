import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../../l10n/app_localizations.dart';
import '../../../../logging/app_logger.dart';
import '../../../../platform/nes_video.dart' as nes_video;
import '../../../../widgets/animated_settings_widgets.dart';
import '../../video_settings.dart';

class NtscSettingsCard extends ConsumerStatefulWidget {
  const NtscSettingsCard({super.key});

  @override
  ConsumerState<NtscSettingsCard> createState() => _NtscSettingsCardState();
}

class _NtscSettingsCardState extends ConsumerState<NtscSettingsCard> {
  Timer? _ntscApplyTimer;
  Timer? _ntscBisqwitApplyTimer;

  @override
  void dispose() {
    _ntscApplyTimer?.cancel();
    _ntscBisqwitApplyTimer?.cancel();
    super.dispose();
  }

  void _scheduleApplyNtscOptions(nes_video.NtscOptions options) {
    _ntscApplyTimer?.cancel();
    _ntscApplyTimer = Timer(const Duration(milliseconds: 120), () async {
      try {
        await nes_video.setNtscOptions(options: options);
      } catch (e, st) {
        logWarning(
          e,
          stackTrace: st,
          message: 'setNtscOptions failed',
          logger: 'ntsc_settings_card',
        );
      }
    });
  }

  void _scheduleApplyNtscBisqwitOptions(nes_video.NtscBisqwitOptions options) {
    _ntscBisqwitApplyTimer?.cancel();
    _ntscBisqwitApplyTimer = Timer(const Duration(milliseconds: 120), () async {
      try {
        await nes_video.setNtscBisqwitOptions(options: options);
      } catch (e, st) {
        logWarning(
          e,
          stackTrace: st,
          message: 'setNtscBisqwitOptions failed',
          logger: 'ntsc_settings_card',
        );
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final videoSettings = ref.watch(videoSettingsProvider);
    final videoController = ref.read(videoSettingsProvider.notifier);

    final isNtsc =
        videoSettings.videoFilter == nes_video.VideoFilter.ntscComposite ||
        videoSettings.videoFilter == nes_video.VideoFilter.ntscSVideo ||
        videoSettings.videoFilter == nes_video.VideoFilter.ntscRgb ||
        videoSettings.videoFilter == nes_video.VideoFilter.ntscMonochrome;

    final isNtscBisqwit =
        videoSettings.videoFilter == nes_video.VideoFilter.ntscBisqwit2X ||
        videoSettings.videoFilter == nes_video.VideoFilter.ntscBisqwit4X ||
        videoSettings.videoFilter == nes_video.VideoFilter.ntscBisqwit8X;

    if (!isNtsc && !isNtscBisqwit) return const SizedBox.shrink();

    if (isNtscBisqwit) {
      return Padding(
        padding: const EdgeInsets.symmetric(horizontal: 12),
        child: Column(
          children: [
            AnimatedExpansionTile(
              labelText: l10n.videoNtscBisqwitSettingsTitle,
              title: Text(l10n.keyboardPresetCustom),
              initiallyExpanded: false,
              trailing: IconButton(
                icon: const Icon(Icons.refresh_rounded, size: 20),
                tooltip: l10n.videoResetToDefault,
                onPressed: () => videoController.resetNtscBisqwitOptions(),
                constraints: const BoxConstraints(),
                padding: EdgeInsets.zero,
                visualDensity: VisualDensity.compact,
              ),
              children: [
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: AnimatedSliderTile(
                    label: l10n.videoNtscHueLabel,
                    value: videoSettings.ntscBisqwitOptions.hue,
                    min: -1,
                    max: 1,
                    divisions: 200,
                    valueLabel: videoSettings.ntscBisqwitOptions.hue
                        .toStringAsFixed(2),
                    onChanged: (value) {
                      final o = videoSettings.ntscBisqwitOptions;
                      final next = nes_video.NtscBisqwitOptions(
                        brightness: o.brightness,
                        contrast: o.contrast,
                        hue: value,
                        saturation: o.saturation,
                        yFilterLength: o.yFilterLength,
                        iFilterLength: o.iFilterLength,
                        qFilterLength: o.qFilterLength,
                      );
                      unawaited(videoController.setNtscBisqwitOptions(next));
                      _scheduleApplyNtscBisqwitOptions(next);
                    },
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: AnimatedSliderTile(
                    label: l10n.videoNtscSaturationLabel,
                    value: videoSettings.ntscBisqwitOptions.saturation,
                    min: -1,
                    max: 1,
                    divisions: 200,
                    valueLabel: videoSettings.ntscBisqwitOptions.saturation
                        .toStringAsFixed(2),
                    onChanged: (value) {
                      final o = videoSettings.ntscBisqwitOptions;
                      final next = nes_video.NtscBisqwitOptions(
                        brightness: o.brightness,
                        contrast: o.contrast,
                        hue: o.hue,
                        saturation: value,
                        yFilterLength: o.yFilterLength,
                        iFilterLength: o.iFilterLength,
                        qFilterLength: o.qFilterLength,
                      );
                      unawaited(videoController.setNtscBisqwitOptions(next));
                      _scheduleApplyNtscBisqwitOptions(next);
                    },
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: AnimatedSliderTile(
                    label: l10n.videoNtscContrastLabel,
                    value: videoSettings.ntscBisqwitOptions.contrast,
                    min: -1,
                    max: 1,
                    divisions: 200,
                    valueLabel: videoSettings.ntscBisqwitOptions.contrast
                        .toStringAsFixed(2),
                    onChanged: (value) {
                      final o = videoSettings.ntscBisqwitOptions;
                      final next = nes_video.NtscBisqwitOptions(
                        brightness: o.brightness,
                        contrast: value,
                        hue: o.hue,
                        saturation: o.saturation,
                        yFilterLength: o.yFilterLength,
                        iFilterLength: o.iFilterLength,
                        qFilterLength: o.qFilterLength,
                      );
                      unawaited(videoController.setNtscBisqwitOptions(next));
                      _scheduleApplyNtscBisqwitOptions(next);
                    },
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: AnimatedSliderTile(
                    label: l10n.videoNtscBrightnessLabel,
                    value: videoSettings.ntscBisqwitOptions.brightness,
                    min: -1,
                    max: 1,
                    divisions: 200,
                    valueLabel: videoSettings.ntscBisqwitOptions.brightness
                        .toStringAsFixed(2),
                    onChanged: (value) {
                      final o = videoSettings.ntscBisqwitOptions;
                      final next = nes_video.NtscBisqwitOptions(
                        brightness: value,
                        contrast: o.contrast,
                        hue: o.hue,
                        saturation: o.saturation,
                        yFilterLength: o.yFilterLength,
                        iFilterLength: o.iFilterLength,
                        qFilterLength: o.qFilterLength,
                      );
                      unawaited(videoController.setNtscBisqwitOptions(next));
                      _scheduleApplyNtscBisqwitOptions(next);
                    },
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: AnimatedSliderTile(
                    label: l10n.videoNtscBisqwitYFilterLengthLabel,
                    value: videoSettings.ntscBisqwitOptions.yFilterLength,
                    min: -0.46,
                    max: 4,
                    divisions: 446,
                    valueLabel:
                        (videoSettings.ntscBisqwitOptions.yFilterLength * 100)
                            .round()
                            .toString(),
                    onChanged: (value) {
                      final o = videoSettings.ntscBisqwitOptions;
                      final next = nes_video.NtscBisqwitOptions(
                        brightness: o.brightness,
                        contrast: o.contrast,
                        hue: o.hue,
                        saturation: o.saturation,
                        yFilterLength: value,
                        iFilterLength: o.iFilterLength,
                        qFilterLength: o.qFilterLength,
                      );
                      unawaited(videoController.setNtscBisqwitOptions(next));
                      _scheduleApplyNtscBisqwitOptions(next);
                    },
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: AnimatedSliderTile(
                    label: l10n.videoNtscBisqwitIFilterLengthLabel,
                    value: videoSettings.ntscBisqwitOptions.iFilterLength,
                    min: 0,
                    max: 4,
                    divisions: 400,
                    valueLabel:
                        (videoSettings.ntscBisqwitOptions.iFilterLength * 100)
                            .round()
                            .toString(),
                    onChanged: (value) {
                      final o = videoSettings.ntscBisqwitOptions;
                      final next = nes_video.NtscBisqwitOptions(
                        brightness: o.brightness,
                        contrast: o.contrast,
                        hue: o.hue,
                        saturation: o.saturation,
                        yFilterLength: o.yFilterLength,
                        iFilterLength: value,
                        qFilterLength: o.qFilterLength,
                      );
                      unawaited(videoController.setNtscBisqwitOptions(next));
                      _scheduleApplyNtscBisqwitOptions(next);
                    },
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: AnimatedSliderTile(
                    label: l10n.videoNtscBisqwitQFilterLengthLabel,
                    value: videoSettings.ntscBisqwitOptions.qFilterLength,
                    min: 0,
                    max: 4,
                    divisions: 400,
                    valueLabel:
                        (videoSettings.ntscBisqwitOptions.qFilterLength * 100)
                            .round()
                            .toString(),
                    onChanged: (value) {
                      final o = videoSettings.ntscBisqwitOptions;
                      final next = nes_video.NtscBisqwitOptions(
                        brightness: o.brightness,
                        contrast: o.contrast,
                        hue: o.hue,
                        saturation: o.saturation,
                        yFilterLength: o.yFilterLength,
                        iFilterLength: o.iFilterLength,
                        qFilterLength: value,
                      );
                      unawaited(videoController.setNtscBisqwitOptions(next));
                      _scheduleApplyNtscBisqwitOptions(next);
                    },
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
          ],
        ),
      );
    } else {
      return Padding(
        padding: const EdgeInsets.symmetric(horizontal: 12),
        child: Column(
          children: [
            AnimatedExpansionTile(
              labelText: l10n.videoNtscAdvancedTitle,
              title: Text(l10n.keyboardPresetCustom),
              initiallyExpanded: false,
              trailing: IconButton(
                icon: const Icon(Icons.refresh_rounded, size: 20),
                tooltip: l10n.videoResetToDefault,
                onPressed: () => videoController.resetNtscOptions(),
                constraints: const BoxConstraints(),
                padding: EdgeInsets.zero,
                visualDensity: VisualDensity.compact,
              ),
              children: [
                Padding(
                  padding: const EdgeInsets.symmetric(
                    horizontal: 16,
                    vertical: 8,
                  ),
                  child: SwitchListTile(
                    contentPadding: EdgeInsets.zero,
                    title: Text(l10n.videoNtscMergeFieldsLabel),
                    value: videoSettings.ntscOptions.mergeFields,
                    onChanged: (value) async {
                      final o = videoSettings.ntscOptions;
                      final next = nes_video.NtscOptions(
                        hue: o.hue,
                        saturation: o.saturation,
                        contrast: o.contrast,
                        brightness: o.brightness,
                        sharpness: o.sharpness,
                        gamma: o.gamma,
                        resolution: o.resolution,
                        artifacts: o.artifacts,
                        fringing: o.fringing,
                        bleed: o.bleed,
                        mergeFields: value,
                      );
                      unawaited(videoController.setNtscOptions(next));
                      _scheduleApplyNtscOptions(next);
                    },
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: AnimatedSliderTile(
                    label: l10n.videoNtscHueLabel,
                    value: videoSettings.ntscOptions.hue,
                    min: -1,
                    max: 1,
                    divisions: 200,
                    valueLabel: videoSettings.ntscOptions.hue.toStringAsFixed(
                      2,
                    ),
                    onChanged: (value) async {
                      final o = videoSettings.ntscOptions;
                      final next = nes_video.NtscOptions(
                        hue: value,
                        saturation: o.saturation,
                        contrast: o.contrast,
                        brightness: o.brightness,
                        sharpness: o.sharpness,
                        gamma: o.gamma,
                        resolution: o.resolution,
                        artifacts: o.artifacts,
                        fringing: o.fringing,
                        bleed: o.bleed,
                        mergeFields: o.mergeFields,
                      );
                      unawaited(videoController.setNtscOptions(next));
                      _scheduleApplyNtscOptions(next);
                    },
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: AnimatedSliderTile(
                    label: l10n.videoNtscSaturationLabel,
                    value: videoSettings.ntscOptions.saturation,
                    min: -1,
                    max: 1,
                    divisions: 200,
                    valueLabel: videoSettings.ntscOptions.saturation
                        .toStringAsFixed(2),
                    onChanged: (value) async {
                      final o = videoSettings.ntscOptions;
                      final next = nes_video.NtscOptions(
                        hue: o.hue,
                        saturation: value,
                        contrast: o.contrast,
                        brightness: o.brightness,
                        sharpness: o.sharpness,
                        gamma: o.gamma,
                        resolution: o.resolution,
                        artifacts: o.artifacts,
                        fringing: o.fringing,
                        bleed: o.bleed,
                        mergeFields: o.mergeFields,
                      );
                      unawaited(videoController.setNtscOptions(next));
                      _scheduleApplyNtscOptions(next);
                    },
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: AnimatedSliderTile(
                    label: l10n.videoNtscContrastLabel,
                    value: videoSettings.ntscOptions.contrast,
                    min: -1,
                    max: 1,
                    divisions: 200,
                    valueLabel: videoSettings.ntscOptions.contrast
                        .toStringAsFixed(2),
                    onChanged: (value) async {
                      final o = videoSettings.ntscOptions;
                      final next = nes_video.NtscOptions(
                        hue: o.hue,
                        saturation: o.saturation,
                        contrast: value,
                        brightness: o.brightness,
                        sharpness: o.sharpness,
                        gamma: o.gamma,
                        resolution: o.resolution,
                        artifacts: o.artifacts,
                        fringing: o.fringing,
                        bleed: o.bleed,
                        mergeFields: o.mergeFields,
                      );
                      unawaited(videoController.setNtscOptions(next));
                      _scheduleApplyNtscOptions(next);
                    },
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: AnimatedSliderTile(
                    label: l10n.videoNtscBrightnessLabel,
                    value: videoSettings.ntscOptions.brightness,
                    min: -1,
                    max: 1,
                    divisions: 200,
                    valueLabel: videoSettings.ntscOptions.brightness
                        .toStringAsFixed(2),
                    onChanged: (value) async {
                      final o = videoSettings.ntscOptions;
                      final next = nes_video.NtscOptions(
                        hue: o.hue,
                        saturation: o.saturation,
                        contrast: o.contrast,
                        brightness: value,
                        sharpness: o.sharpness,
                        gamma: o.gamma,
                        resolution: o.resolution,
                        artifacts: o.artifacts,
                        fringing: o.fringing,
                        bleed: o.bleed,
                        mergeFields: o.mergeFields,
                      );
                      unawaited(videoController.setNtscOptions(next));
                      _scheduleApplyNtscOptions(next);
                    },
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: AnimatedSliderTile(
                    label: l10n.videoNtscSharpnessLabel,
                    value: videoSettings.ntscOptions.sharpness,
                    min: -1,
                    max: 1,
                    divisions: 200,
                    valueLabel: videoSettings.ntscOptions.sharpness
                        .toStringAsFixed(2),
                    onChanged: (value) async {
                      final o = videoSettings.ntscOptions;
                      final next = nes_video.NtscOptions(
                        hue: o.hue,
                        saturation: o.saturation,
                        contrast: o.contrast,
                        brightness: o.brightness,
                        sharpness: value,
                        gamma: o.gamma,
                        resolution: o.resolution,
                        artifacts: o.artifacts,
                        fringing: o.fringing,
                        bleed: o.bleed,
                        mergeFields: o.mergeFields,
                      );
                      unawaited(videoController.setNtscOptions(next));
                      _scheduleApplyNtscOptions(next);
                    },
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: AnimatedSliderTile(
                    label: l10n.videoNtscGammaLabel,
                    value: videoSettings.ntscOptions.gamma,
                    min: -1,
                    max: 1,
                    divisions: 200,
                    valueLabel: videoSettings.ntscOptions.gamma.toStringAsFixed(
                      2,
                    ),
                    onChanged: (value) async {
                      final o = videoSettings.ntscOptions;
                      final next = nes_video.NtscOptions(
                        hue: o.hue,
                        saturation: o.saturation,
                        contrast: o.contrast,
                        brightness: o.brightness,
                        sharpness: o.sharpness,
                        gamma: value,
                        resolution: o.resolution,
                        artifacts: o.artifacts,
                        fringing: o.fringing,
                        bleed: o.bleed,
                        mergeFields: o.mergeFields,
                      );
                      unawaited(videoController.setNtscOptions(next));
                      _scheduleApplyNtscOptions(next);
                    },
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: AnimatedSliderTile(
                    label: l10n.videoNtscResolutionLabel,
                    value: videoSettings.ntscOptions.resolution,
                    min: -1,
                    max: 1,
                    divisions: 200,
                    valueLabel: videoSettings.ntscOptions.resolution
                        .toStringAsFixed(2),
                    onChanged: (value) async {
                      final o = videoSettings.ntscOptions;
                      final next = nes_video.NtscOptions(
                        hue: o.hue,
                        saturation: o.saturation,
                        contrast: o.contrast,
                        brightness: o.brightness,
                        sharpness: o.sharpness,
                        gamma: o.gamma,
                        resolution: value,
                        artifacts: o.artifacts,
                        fringing: o.fringing,
                        bleed: o.bleed,
                        mergeFields: o.mergeFields,
                      );
                      unawaited(videoController.setNtscOptions(next));
                      _scheduleApplyNtscOptions(next);
                    },
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: AnimatedSliderTile(
                    label: l10n.videoNtscArtifactsLabel,
                    value: videoSettings.ntscOptions.artifacts,
                    min: -1,
                    max: 1,
                    divisions: 200,
                    valueLabel: videoSettings.ntscOptions.artifacts
                        .toStringAsFixed(2),
                    onChanged: (value) async {
                      final o = videoSettings.ntscOptions;
                      final next = nes_video.NtscOptions(
                        hue: o.hue,
                        saturation: o.saturation,
                        contrast: o.contrast,
                        brightness: o.brightness,
                        sharpness: o.sharpness,
                        gamma: o.gamma,
                        resolution: o.resolution,
                        artifacts: value,
                        fringing: o.fringing,
                        bleed: o.bleed,
                        mergeFields: o.mergeFields,
                      );
                      unawaited(videoController.setNtscOptions(next));
                      _scheduleApplyNtscOptions(next);
                    },
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: AnimatedSliderTile(
                    label: l10n.videoNtscFringingLabel,
                    value: videoSettings.ntscOptions.fringing,
                    min: -1,
                    max: 1,
                    divisions: 200,
                    valueLabel: videoSettings.ntscOptions.fringing
                        .toStringAsFixed(2),
                    onChanged: (value) async {
                      final o = videoSettings.ntscOptions;
                      final next = nes_video.NtscOptions(
                        hue: o.hue,
                        saturation: o.saturation,
                        contrast: o.contrast,
                        brightness: o.brightness,
                        sharpness: o.sharpness,
                        gamma: o.gamma,
                        resolution: o.resolution,
                        artifacts: o.artifacts,
                        fringing: value,
                        bleed: o.bleed,
                        mergeFields: o.mergeFields,
                      );
                      unawaited(videoController.setNtscOptions(next));
                      _scheduleApplyNtscOptions(next);
                    },
                  ),
                ),
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: AnimatedSliderTile(
                    label: l10n.videoNtscBleedLabel,
                    value: videoSettings.ntscOptions.bleed,
                    min: -1,
                    max: 1,
                    divisions: 200,
                    valueLabel: videoSettings.ntscOptions.bleed.toStringAsFixed(
                      2,
                    ),
                    onChanged: (value) async {
                      final o = videoSettings.ntscOptions;
                      final next = nes_video.NtscOptions(
                        hue: o.hue,
                        saturation: o.saturation,
                        contrast: o.contrast,
                        brightness: o.brightness,
                        sharpness: o.sharpness,
                        gamma: o.gamma,
                        resolution: o.resolution,
                        artifacts: o.artifacts,
                        fringing: o.fringing,
                        bleed: value,
                        mergeFields: o.mergeFields,
                      );
                      unawaited(videoController.setNtscOptions(next));
                      _scheduleApplyNtscOptions(next);
                    },
                  ),
                ),
                const SizedBox(height: 8),
              ],
            ),
            const SizedBox(height: 12),
          ],
        ),
      );
    }
  }
}
