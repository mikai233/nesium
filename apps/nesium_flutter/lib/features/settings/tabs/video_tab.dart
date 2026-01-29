import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../l10n/app_localizations.dart';
import '../../../widgets/animated_settings_widgets.dart';
import '../video_settings.dart';
import 'video/cpu_filter_card.dart';
import 'video/gpu_shader_card.dart';
import 'video/ntsc_settings_card.dart';
import 'video/palette_settings_card.dart';
import 'video/general_video_settings_card.dart';
import 'video/shader_settings_card.dart';

class VideoTab extends ConsumerWidget {
  const VideoTab({required this.pickAndApplyCustomPalette, super.key});

  final Future<void> Function(BuildContext, VideoSettingsController)
  pickAndApplyCustomPalette;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;

    return ListView(
      padding: const EdgeInsets.all(20),
      children: [
        AnimatedSectionHeader(
          title: l10n.videoTitle,
          icon: Icons.videocam,
          delay: const Duration(milliseconds: 100),
        ),
        const SizedBox(height: 8),

        // Group CPU filters, NTSC settings, Palettes, and GPU shaders in one card
        // to match original grouping but with modular components
        AnimatedSettingsCard(
          index: 0,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              // We'll use the components but they shouldn't have their own card wrappers
              // if we want to keep them in one card.
              // Wait, I already added a card wrapper to CpuFilterCard.
              // I'll adjust the components to be more flexible.
              const CpuFilterCardContent(),
              const NtscSettingsCard(),
              PaletteSettingsCard(
                pickAndApplyCustomPalette: pickAndApplyCustomPalette,
              ),
              const GpuShaderCard(),
              const ShaderSettingsCard(),
              const SizedBox(height: 12),
            ],
          ),
        ),
        const SizedBox(height: 12),
        const GeneralVideoSettingsCard(),
      ],
    );
  }
}

// Internal version of CpuFilterCard that only provides the content
class CpuFilterCardContent extends ConsumerWidget {
  const CpuFilterCardContent({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    // This is essentially the same as CpuFilterCard but without the AnimatedSettingsCard wrapper
    // I will refactor CpuFilterCard to use this or vice versa.
    // For now, I'll just use the logic from CpuFilterCard.
    return const CpuFilterCard(useWrapper: false);
  }
}
