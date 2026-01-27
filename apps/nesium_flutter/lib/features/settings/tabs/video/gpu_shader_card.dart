import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../../l10n/app_localizations.dart';
import '../../../../logging/app_logger.dart';
import '../../android_shader_settings.dart';
import '../../android_video_backend_settings.dart';
import '../../macos_shader_settings.dart';
import '../../windows_shader_settings.dart';
import '../../windows_video_backend_settings.dart';
import '../../../shaders/shader_browser_page.dart';

class GpuShaderCard extends ConsumerWidget {
  const GpuShaderCard({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final isAndroid =
        !kIsWeb && defaultTargetPlatform == TargetPlatform.android;
    final isWindows =
        !kIsWeb && defaultTargetPlatform == TargetPlatform.windows;
    final isMacos = !kIsWeb && defaultTargetPlatform == TargetPlatform.macOS;

    if (!isAndroid && !isWindows && !isMacos) return const SizedBox.shrink();

    final androidBackend = ref.watch(androidVideoBackendSettingsProvider);
    final androidShaderSettings = isAndroid
        ? ref.watch(androidShaderSettingsProvider)
        : null;
    final androidShaderController = isAndroid
        ? ref.read(androidShaderSettingsProvider.notifier)
        : null;

    final windowsBackend = isWindows
        ? ref.watch(windowsVideoBackendSettingsProvider)
        : null;
    final windowsShaderSettings = isWindows
        ? ref.watch(windowsShaderSettingsProvider)
        : null;
    final windowsShaderController = isWindows
        ? ref.read(windowsShaderSettingsProvider.notifier)
        : null;

    final macosShaderSettings = isMacos
        ? ref.watch(macosShaderSettingsProvider)
        : null;
    final macosShaderController = isMacos
        ? ref.read(macosShaderSettingsProvider.notifier)
        : null;

    Future<void> pickAndSetShaderPreset() async {
      if (isAndroid) {
        if (androidShaderController == null) return;
        if (androidBackend.backend != AndroidVideoBackend.hardware) return;
      }
      if (isWindows) {
        if (windowsShaderController == null) return;
        if (windowsBackend?.backend != WindowsVideoBackend.d3d11Gpu) return;
      }
      if (isMacos) {
        if (macosShaderController == null) return;
      }

      if (!context.mounted) return;
      Navigator.of(context).push(
        MaterialPageRoute(builder: (context) => const ShaderBrowserPage()),
      );
    }

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        const Padding(
          padding: EdgeInsets.symmetric(horizontal: 16),
          child: Divider(height: 24),
        ),
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 0, 16, 8),
          child: Text(
            l10n.videoFilterCategoryGpu,
            style: Theme.of(context).textTheme.titleSmall?.copyWith(
              color: Theme.of(context).colorScheme.primary,
            ),
          ),
        ),
        if (isWindows &&
            windowsShaderSettings != null &&
            windowsBackend != null)
          Column(
            children: [
              SwitchListTile(
                contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                secondary: const Icon(Icons.auto_fix_high),
                title: Text(l10n.videoShaderLibrashaderTitle),
                subtitle: Text(
                  windowsBackend.backend == WindowsVideoBackend.d3d11Gpu
                      ? l10n.videoShaderLibrashaderSubtitleWindows
                      : l10n.videoShaderLibrashaderSubtitleDisabledWindows,
                ),
                value: windowsShaderSettings.enabled,
                onChanged:
                    windowsBackend.backend == WindowsVideoBackend.d3d11Gpu
                    ? (value) async {
                        try {
                          await windowsShaderController?.setEnabled(value);
                        } catch (e, st) {
                          logWarning(
                            e,
                            stackTrace: st,
                            message: 'setEnabled failed',
                            logger: 'gpu_shader_card',
                          );
                        }
                      }
                    : null,
              ),
              const SizedBox(height: 1),
              ListTile(
                enabled: windowsBackend.backend == WindowsVideoBackend.d3d11Gpu,
                contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                leading: const Icon(Icons.description_outlined),
                title: Text(l10n.videoShaderPresetLabel),
                subtitle: Text(
                  windowsShaderSettings.presetPath ??
                      l10n.videoShaderPresetNotSet,
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                ),
                trailing: const Icon(Icons.folder_open),
                onTap: windowsBackend.backend == WindowsVideoBackend.d3d11Gpu
                    ? pickAndSetShaderPreset
                    : null,
                onLongPress: windowsShaderSettings.presetPath == null
                    ? null
                    : () async {
                        try {
                          await windowsShaderController?.setPresetPath(null);
                        } catch (e, st) {
                          logWarning(
                            e,
                            stackTrace: st,
                            message: 'clear preset failed',
                            logger: 'gpu_shader_card',
                          );
                        }
                      },
              ),
            ],
          ),
        if (isAndroid && androidShaderSettings != null)
          Column(
            children: [
              SwitchListTile(
                contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                secondary: const Icon(Icons.auto_fix_high),
                title: Text(l10n.videoShaderLibrashaderTitle),
                subtitle: Text(
                  androidBackend.backend == AndroidVideoBackend.hardware
                      ? l10n.videoShaderLibrashaderSubtitle
                      : l10n.videoShaderLibrashaderSubtitleDisabled,
                ),
                value: androidShaderSettings.enabled,
                onChanged:
                    androidBackend.backend == AndroidVideoBackend.hardware
                    ? (value) async {
                        try {
                          await androidShaderController?.setEnabled(value);
                        } catch (e, st) {
                          logWarning(
                            e,
                            stackTrace: st,
                            message: 'setEnabled failed',
                            logger: 'gpu_shader_card',
                          );
                        }
                      }
                    : null,
              ),
              const SizedBox(height: 1),
              ListTile(
                enabled: androidBackend.backend == AndroidVideoBackend.hardware,
                contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                leading: const Icon(Icons.description_outlined),
                title: Text(l10n.videoShaderPresetLabel),
                subtitle: Text(
                  androidShaderSettings.presetPath ??
                      l10n.videoShaderPresetNotSet,
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                ),
                trailing: const Icon(Icons.folder_open),
                onTap: androidBackend.backend == AndroidVideoBackend.hardware
                    ? pickAndSetShaderPreset
                    : null,
                onLongPress: androidShaderSettings.presetPath == null
                    ? null
                    : () async {
                        try {
                          await androidShaderController?.setPresetPath(null);
                        } catch (e, st) {
                          logWarning(
                            e,
                            stackTrace: st,
                            message: 'clear preset failed',
                            logger: 'gpu_shader_card',
                          );
                        }
                      },
              ),
            ],
          ),
        if (isMacos && macosShaderSettings != null)
          Column(
            children: [
              SwitchListTile(
                contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                secondary: const Icon(Icons.auto_fix_high),
                title: Text(l10n.videoShaderLibrashaderTitle),
                subtitle: Text(l10n.videoShaderLibrashaderSubtitleMacos),
                value: macosShaderSettings.enabled,
                onChanged: (value) async {
                  try {
                    await macosShaderController?.setEnabled(value);
                  } catch (e, st) {
                    logWarning(
                      e,
                      stackTrace: st,
                      message: 'setEnabled failed',
                      logger: 'gpu_shader_card',
                    );
                  }
                },
              ),
              const SizedBox(height: 1),
              ListTile(
                contentPadding: const EdgeInsets.symmetric(horizontal: 16),
                leading: const Icon(Icons.description_outlined),
                title: Text(l10n.videoShaderPresetLabel),
                subtitle: Text(
                  macosShaderSettings.presetPath ??
                      l10n.videoShaderPresetNotSet,
                  maxLines: 2,
                  overflow: TextOverflow.ellipsis,
                ),
                trailing: const Icon(Icons.folder_open),
                onTap: pickAndSetShaderPreset,
                onLongPress: macosShaderSettings.presetPath == null
                    ? null
                    : () async {
                        try {
                          await macosShaderController?.setPresetPath(null);
                        } catch (e, st) {
                          logWarning(
                            e,
                            stackTrace: st,
                            message: 'clear preset failed',
                            logger: 'gpu_shader_card',
                          );
                        }
                      },
              ),
            ],
          ),
      ],
    );
  }
}
