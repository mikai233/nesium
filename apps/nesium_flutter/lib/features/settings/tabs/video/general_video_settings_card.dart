import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../../l10n/app_localizations.dart';
import '../../../../logging/app_logger.dart';
import '../../../../widgets/animated_dropdown_menu.dart';
import '../../../../widgets/animated_settings_widgets.dart';
import '../../android_performance_settings.dart';
import '../../android_video_backend_settings.dart';
import '../../macos_performance_settings.dart';
import '../../windows_performance_settings.dart';
import '../../windows_video_backend_settings.dart';
import '../../video_settings.dart';

class GeneralVideoSettingsCard extends ConsumerWidget {
  const GeneralVideoSettingsCard({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final videoSettings = ref.watch(videoSettingsProvider);
    final videoController = ref.read(videoSettingsProvider.notifier);
    final isAndroid =
        !kIsWeb && defaultTargetPlatform == TargetPlatform.android;
    final isWindows =
        !kIsWeb && defaultTargetPlatform == TargetPlatform.windows;
    final isMacos = !kIsWeb && defaultTargetPlatform == TargetPlatform.macOS;

    final androidBackend = ref.watch(androidVideoBackendSettingsProvider);
    final androidBackendController = ref.read(
      androidVideoBackendSettingsProvider.notifier,
    );
    final androidPerformance = isAndroid
        ? ref.watch(androidPerformanceSettingsControllerProvider)
        : AndroidPerformanceSettings(highPerformance: false);
    final androidPerformanceController = isAndroid
        ? ref.read(androidPerformanceSettingsControllerProvider.notifier)
        : null;

    final windowsBackend = isWindows
        ? ref.watch(windowsVideoBackendSettingsProvider)
        : const WindowsVideoBackendSettings(
            backend: WindowsVideoBackend.d3d11Gpu,
          );
    final windowsBackendController = isWindows
        ? ref.read(windowsVideoBackendSettingsProvider.notifier)
        : null;
    final windowsPerformance = isWindows
        ? ref.watch(windowsPerformanceSettingsControllerProvider)
        : WindowsPerformanceSettings(highPerformance: false);
    final windowsPerformanceController = isWindows
        ? ref.read(windowsPerformanceSettingsControllerProvider.notifier)
        : null;
    final macosPerformance = isMacos
        ? ref.watch(macosPerformanceSettingsControllerProvider)
        : MacosPerformanceSettings(highPerformance: false);
    final macosPerformanceController = isMacos
        ? ref.read(macosPerformanceSettingsControllerProvider.notifier)
        : null;

    Future<void> setAspectRatio(NesAspectRatio value) async {
      try {
        await videoController.setAspectRatio(value);
      } catch (e, st) {
        logWarning(
          e,
          stackTrace: st,
          message: 'setAspectRatio failed',
          logger: 'general_video_settings_card',
        );
      }
    }

    Future<void> setAndroidBackend(AndroidVideoBackend value) async {
      try {
        await androidBackendController.setBackend(value);
      } catch (e, st) {
        logWarning(
          e,
          stackTrace: st,
          message: 'setAndroidBackend failed',
          logger: 'general_video_settings_card',
        );
      }
    }

    Future<void> setWindowsBackend(WindowsVideoBackend value) async {
      if (windowsBackendController == null) return;
      try {
        await windowsBackendController.setBackend(value);
      } catch (e, st) {
        logWarning(
          e,
          stackTrace: st,
          message: 'setWindowsBackend failed',
          logger: 'general_video_settings_card',
        );
      }
    }

    return AnimatedSettingsCard(
      index: 1,
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          children: [
            SwitchListTile(
              contentPadding: EdgeInsets.zero,
              value: videoSettings.integerScaling,
              title: Text(l10n.videoIntegerScalingTitle),
              subtitle: Text(l10n.videoIntegerScalingSubtitle),
              secondary: const Icon(Icons.grid_on),
              onChanged: (value) async {
                try {
                  await videoController.setIntegerScaling(value);
                } catch (e, st) {
                  logWarning(
                    e,
                    stackTrace: st,
                    message: 'setIntegerScaling failed',
                    logger: 'general_video_settings_card',
                  );
                }
              },
            ),
            const SizedBox(height: 12),
            AnimatedDropdownMenu<NesAspectRatio>(
              labelText: l10n.videoAspectRatio,
              value: videoSettings.aspectRatio,
              entries: [
                DropdownMenuEntry(
                  value: NesAspectRatio.square,
                  label: l10n.videoAspectRatioSquare,
                ),
                DropdownMenuEntry(
                  value: NesAspectRatio.ntsc,
                  label: l10n.videoAspectRatioNtsc,
                ),
                DropdownMenuEntry(
                  value: NesAspectRatio.stretch,
                  label: l10n.videoAspectRatioStretch,
                ),
              ],
              onSelected: setAspectRatio,
            ),
            AnimatedSliderTile(
              label: l10n.videoScreenVerticalOffset,
              value: videoSettings.screenVerticalOffset,
              min: -240,
              max: 240,
              divisions: 96,
              onChanged: (v) =>
                  videoController.setScreenVerticalOffset(v.roundToDouble()),
              valueLabel:
                  '${videoSettings.screenVerticalOffset.toStringAsFixed(0)} px',
            ),
            if (isAndroid) ...[
              const SizedBox(height: 12),
              AnimatedDropdownMenu<AndroidVideoBackend>(
                labelText: l10n.videoBackendAndroidLabel,
                helperText: l10n.videoBackendRestartHint,
                value: androidBackend.backend,
                entries: [
                  DropdownMenuEntry(
                    value: AndroidVideoBackend.hardware,
                    label: l10n.videoBackendHardware,
                  ),
                  DropdownMenuEntry(
                    value: AndroidVideoBackend.upload,
                    label: l10n.videoBackendUpload,
                  ),
                ],
                onSelected: setAndroidBackend,
              ),
              const SizedBox(height: 16),
              SwitchListTile(
                contentPadding: EdgeInsets.zero,
                secondary: const Icon(Icons.rocket_launch),
                title: Text(l10n.highPerformanceModeLabel),
                subtitle: Text(l10n.highPerformanceModeDescription),
                value: androidPerformance.highPerformance,
                onChanged: androidPerformanceController == null
                    ? null
                    : (value) => androidPerformanceController
                          .setHighPerformance(value),
              ),
            ],
            if (isWindows) ...[
              const SizedBox(height: 12),
              AnimatedDropdownMenu<WindowsVideoBackend>(
                labelText: l10n.videoBackendWindowsLabel,
                value: windowsBackend.backend,
                entries: [
                  DropdownMenuEntry(
                    value: WindowsVideoBackend.d3d11Gpu,
                    label: l10n.videoBackendD3D11,
                  ),
                  DropdownMenuEntry(
                    value: WindowsVideoBackend.softwareCpu,
                    label: l10n.videoBackendSoftware,
                  ),
                ],
                onSelected: setWindowsBackend,
              ),
              const SizedBox(height: 16),
              SwitchListTile(
                contentPadding: EdgeInsets.zero,
                secondary: const Icon(Icons.rocket_launch),
                title: Text(l10n.highPerformanceModeLabel),
                subtitle: Text(l10n.highPerformanceModeDescription),
                value: windowsPerformance.highPerformance,
                onChanged: windowsPerformanceController == null
                    ? null
                    : (value) => windowsPerformanceController
                          .setHighPerformance(value),
              ),
              const SizedBox(height: 12),
              SwitchListTile(
                contentPadding: EdgeInsets.zero,
                secondary: const Icon(Icons.layers),
                title: Text(l10n.windowsNativeOverlayTitle),
                subtitle: Text(l10n.windowsNativeOverlaySubtitle),
                value: windowsBackend.useNativeOverlay,
                onChanged: windowsBackendController == null
                    ? null
                    : (value) =>
                          windowsBackendController.setNativeOverlay(value),
              ),
            ],
            if (isMacos) ...[
              const SizedBox(height: 16),
              SwitchListTile(
                contentPadding: EdgeInsets.zero,
                secondary: const Icon(Icons.rocket_launch),
                title: Text(l10n.highPerformanceModeLabel),
                subtitle: Text(l10n.highPerformanceModeDescription),
                value: macosPerformance.highPerformance,
                onChanged: macosPerformanceController == null
                    ? null
                    : (value) =>
                          macosPerformanceController.setHighPerformance(value),
              ),
            ],
          ],
        ),
      ),
    );
  }
}
