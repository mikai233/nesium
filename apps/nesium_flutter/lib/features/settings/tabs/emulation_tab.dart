import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../l10n/app_localizations.dart';
import '../../../widgets/animated_dropdown_menu.dart';
import '../../../widgets/animated_settings_widgets.dart';
import '../emulation_settings.dart';

class EmulationTab extends ConsumerWidget {
  const EmulationTab({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final colorScheme = Theme.of(context).colorScheme;
    final emulationSettings = ref.watch(emulationSettingsProvider);
    final emulationController = ref.read(emulationSettingsProvider.notifier);

    return ListView(
      padding: const EdgeInsets.all(20),
      children: [
        AnimatedSectionHeader(
          title: l10n.emulationTitle,
          icon: Icons.developer_board,
          delay: const Duration(milliseconds: 100),
        ),
        const SizedBox(height: 8),
        AnimatedSettingsCard(
          index: 0,
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: Column(
              children: [
                SwitchListTile(
                  contentPadding: EdgeInsets.zero,
                  secondary: const Icon(Icons.speed),
                  value: emulationSettings.integerFpsMode,
                  title: Text(l10n.integerFpsTitle),
                  subtitle: Text(l10n.integerFpsSubtitle),
                  onChanged: emulationController.setIntegerFpsMode,
                ),
                const Divider(height: 1),
                SwitchListTile(
                  contentPadding: EdgeInsets.zero,
                  secondary: const Icon(Icons.pause_circle_outline),
                  title: Text(l10n.pauseInBackgroundTitle),
                  subtitle: Text(l10n.pauseInBackgroundSubtitle),
                  value: emulationSettings.pauseInBackground,
                  onChanged: emulationController.setPauseInBackground,
                ),
                const Divider(height: 1),
                SwitchListTile(
                  contentPadding: EdgeInsets.zero,
                  secondary: const Icon(Icons.visibility),
                  title: Text(l10n.showOverlayTitle),
                  subtitle: Text(l10n.showOverlaySubtitle),
                  value: emulationSettings.showEmulationStatusOverlay,
                  onChanged: emulationController.setShowEmulationStatusOverlay,
                ),
              ],
            ),
          ),
        ),
        const SizedBox(height: 12),
        AnimatedSettingsCard(
          index: 1,
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: Column(
              children: [
                SwitchListTile(
                  contentPadding: EdgeInsets.zero,
                  secondary: const Icon(Icons.save_outlined),
                  title: Text(l10n.autoSaveEnabledTitle),
                  subtitle: Text(l10n.autoSaveEnabledSubtitle),
                  value: emulationSettings.autoSaveEnabled,
                  onChanged: emulationController.setAutoSaveEnabled,
                ),
                ClipRect(
                  child: AnimatedSwitcher(
                    duration: const Duration(milliseconds: 220),
                    reverseDuration: const Duration(milliseconds: 180),
                    switchInCurve: Curves.easeOutCubic,
                    switchOutCurve: Curves.easeInCubic,
                    transitionBuilder: (child, animation) {
                      return FadeTransition(
                        opacity: animation,
                        child: SizeTransition(
                          sizeFactor: animation,
                          axisAlignment: -1,
                          child: child,
                        ),
                      );
                    },
                    child: emulationSettings.autoSaveEnabled
                        ? Padding(
                            key: const ValueKey('autoSaveInterval'),
                            padding: const EdgeInsets.fromLTRB(56, 0, 0, 4),
                            child: AnimatedSliderTile(
                              label: l10n.autoSaveIntervalTitle,
                              value: emulationSettings.autoSaveIntervalInMinutes
                                  .toDouble(),
                              min: 1,
                              max: 60,
                              divisions: 59,
                              onChanged: (v) => emulationController
                                  .setAutoSaveIntervalInMinutes(v.toInt()),
                              valueLabel: l10n.autoSaveIntervalValue(
                                emulationSettings.autoSaveIntervalInMinutes,
                              ),
                            ),
                          )
                        : const SizedBox.shrink(
                            key: ValueKey('autoSaveIntervalEmpty'),
                          ),
                  ),
                ),
              ],
            ),
          ),
        ),
        const SizedBox(height: 12),
        AnimatedSettingsCard(
          index: 2,
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: AnimatedDropdownMenu<int>(
              labelText: l10n.quickSaveSlotTitle,
              helperText: l10n.quickSaveSlotSubtitle,
              value: emulationSettings.quickSaveSlot,
              entries: [
                for (int i = 1; i <= 10; i++)
                  DropdownMenuEntry(
                    value: i,
                    label: l10n.quickSaveSlotValue(i),
                  ),
              ],
              onSelected: (value) =>
                  emulationController.setQuickSaveSlot(value),
            ),
          ),
        ),
        const SizedBox(height: 12),
        AnimatedSettingsCard(
          index: 3,
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                AnimatedSliderTile(
                  label: l10n.fastForwardSpeedTitle,
                  value: emulationSettings.fastForwardSpeedPercent.toDouble(),
                  min: 100,
                  max: 1000,
                  divisions: 9,
                  onChanged: (v) =>
                      emulationController.setFastForwardSpeedPercent(v.round()),
                  valueLabel: l10n.fastForwardSpeedValue(
                    emulationSettings.fastForwardSpeedPercent,
                  ),
                ),
                const SizedBox(height: 4),
                Text(
                  l10n.fastForwardSpeedSubtitle,
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: colorScheme.onSurfaceVariant,
                  ),
                ),
              ],
            ),
          ),
        ),
        const SizedBox(height: 12),
        AnimatedSettingsCard(
          index: 4,
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: Column(
              children: [
                SwitchListTile(
                  contentPadding: EdgeInsets.zero,
                  secondary: const Icon(Icons.history_toggle_off),
                  title: Text(l10n.rewindEnabledTitle),
                  subtitle: Text(l10n.rewindEnabledSubtitle),
                  value: emulationSettings.rewindEnabled,
                  onChanged: emulationController.setRewindEnabled,
                ),
                ClipRect(
                  child: AnimatedSwitcher(
                    duration: const Duration(milliseconds: 220),
                    reverseDuration: const Duration(milliseconds: 180),
                    switchInCurve: Curves.easeOutCubic,
                    switchOutCurve: Curves.easeInCubic,
                    transitionBuilder: (child, animation) {
                      return FadeTransition(
                        opacity: animation,
                        child: SizeTransition(
                          sizeFactor: animation,
                          axisAlignment: -1,
                          child: child,
                        ),
                      );
                    },
                    child: emulationSettings.rewindEnabled
                        ? Padding(
                            key: const ValueKey('rewindSettings'),
                            padding: const EdgeInsets.fromLTRB(56, 0, 0, 4),
                            child: Column(
                              children: [
                                AnimatedSliderTile(
                                  label: l10n.rewindMinutesTitle,
                                  value: emulationSettings.rewindSeconds
                                      .toDouble(),
                                  min: 60,
                                  max: 3600,
                                  divisions: 59,
                                  onChanged: (v) => emulationController
                                      .setRewindSeconds(v.toInt()),
                                  valueLabel: l10n.rewindMinutesValue(
                                    emulationSettings.rewindSeconds ~/ 60,
                                  ),
                                ),
                                const SizedBox(height: 12),
                                AnimatedSliderTile(
                                  label: l10n.rewindSpeedTitle,
                                  value: emulationSettings.rewindSpeedPercent
                                      .toDouble(),
                                  min: 100,
                                  max: 1000,
                                  divisions: 9,
                                  onChanged: (v) => emulationController
                                      .setRewindSpeedPercent(v.round()),
                                  valueLabel: l10n.rewindSpeedValue(
                                    emulationSettings.rewindSpeedPercent,
                                  ),
                                ),
                                const SizedBox(height: 4),
                                Align(
                                  alignment: Alignment.centerLeft,
                                  child: Text(
                                    l10n.rewindSpeedSubtitle,
                                    style: Theme.of(context).textTheme.bodySmall
                                        ?.copyWith(
                                          color: colorScheme.onSurfaceVariant,
                                        ),
                                  ),
                                ),
                              ],
                            ),
                          )
                        : const SizedBox.shrink(
                            key: ValueKey('rewindSettingsEmpty'),
                          ),
                  ),
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }
}
