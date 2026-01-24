import 'dart:async';

import 'package:desktop_multi_window/desktop_multi_window.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../domain/nes_controller.dart';
import '../domain/nes_input_masks.dart';
import '../logging/app_logger.dart';
import '../platform/nes_video.dart' as nes_video;
import '../platform/platform_capabilities.dart';
import '../features/controls/input_settings.dart';
import '../features/settings/language_settings.dart';
import '../features/settings/emulation_settings.dart';
import '../features/settings/theme_settings.dart';
import '../features/settings/video_settings.dart';
import 'current_window_kind.dart';
import 'settings_sync.dart';

import 'window_types.dart';

final windowMessageRouterProvider = Provider<void>((ref) {
  if (!isNativeDesktop) return;

  var disposed = false;
  Timer? ntscApplyTimer;
  ref.onDispose(() => disposed = true);
  ref.onDispose(() => ntscApplyTimer?.cancel());

  Future<void> handleVideoSettingsChanged({
    required bool isMain,
    required List<String> fields,
    Object? payload,
  }) async {
    // Ignore events that cannot affect the main window presentation.
    // (For example, NTSC tuning is applied directly to the shared Rust pipeline.)
    const relevant = <String>{
      'videoFilter',
      'aspectRatio',
      'integerScaling',
      'screenVerticalOffset',
    };
    if (fields.isNotEmpty && !fields.any(relevant.contains)) {
      return;
    }

    final before = ref.read(videoSettingsProvider);
    VideoSettings? next;
    if (payload is Map) {
      try {
        next = VideoSettings.fromJson(Map<String, dynamic>.from(payload));
      } catch (_) {
        next = null;
      }
    }

    if (next != null) {
      ref.read(videoSettingsProvider.notifier).applySynced(next);
    } else {
      await ref
          .read(videoSettingsProvider.notifier)
          .reloadFromStorage(applyPalette: false);
    }

    if (!isMain) return;

    final after = ref.read(videoSettingsProvider);
    final filter = after.videoFilter;
    final isNtsc =
        filter == nes_video.VideoFilter.ntscComposite ||
        filter == nes_video.VideoFilter.ntscSVideo ||
        filter == nes_video.VideoFilter.ntscRgb ||
        filter == nes_video.VideoFilter.ntscMonochrome;

    try {
      if (before.videoFilter != after.videoFilter) {
        ntscApplyTimer?.cancel();
        ntscApplyTimer = null;

        if (isNtsc) {
          await nes_video.setNtscOptions(options: after.ntscOptions);
        }
        await ref.read(nesControllerProvider.notifier).setVideoFilter(filter);
        return;
      }

      if (isNtsc && before.ntscOptions != after.ntscOptions) {
        // Debounce slider drag updates; only apply the latest values.
        ntscApplyTimer?.cancel();
        ntscApplyTimer = Timer(const Duration(milliseconds: 120), () {
          if (disposed) return;
          final latest = ref.read(videoSettingsProvider);
          unawaited(nes_video.setNtscOptions(options: latest.ntscOptions));
        });
      }
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'Failed to apply video settings change in main window',
        logger: 'window_message_router',
      );
    }
  }

  scheduleMicrotask(() async {
    if (disposed) return;

    late final WindowController controller;
    try {
      controller = await WindowController.fromCurrentEngine();
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'Failed to get WindowController for message router',
        logger: 'window_message_router',
      );
      return;
    }

    await controller.setWindowMethodHandler((call) async {
      switch (call.method) {
        case 'setLanguage':
          final arg = call.arguments;
          final languageCode = arg is String ? arg : null;
          ref
              .read(appLanguageProvider.notifier)
              .applyIncomingLanguageFromWindow(languageCode);
          return null;
        case SettingsSync.methodSettingsChanged:
          final args = call.arguments;
          if (args is! Map) return null;
          final group = args['group'];
          if (group is! String) return null;
          final fieldsRaw = args['fields'];
          final fields = fieldsRaw is List
              ? fieldsRaw.whereType<String>().toList(growable: false)
              : const <String>[];
          final payload = args['payload'];

          final kind = ref.read(currentWindowKindProvider);
          switch (group) {
            case 'video':
              await handleVideoSettingsChanged(
                isMain: kind == WindowKind.main,
                fields: fields,
                payload: payload,
              );
              break;
            case 'theme':
              final next = payload is Map
                  ? ThemeSettings.fromJson(Map<String, dynamic>.from(payload))
                  : null;
              if (next != null) {
                ref.read(themeSettingsProvider.notifier).applySynced(next);
              } else {
                ref.invalidate(themeSettingsProvider);
              }
              break;
            case 'input':
              ref.read(inputSettingsProvider.notifier).applySynced(payload);
              if (kind == WindowKind.main) {
                ref.read(nesInputMasksProvider.notifier).clearAll();
              }
              break;
            case 'emulation':
              final next = payload is Map
                  ? EmulationSettings.fromJson(
                      Map<String, dynamic>.from(payload),
                    )
                  : null;
              if (next != null) {
                ref.read(emulationSettingsProvider.notifier).applySynced(next);
              } else {
                ref.invalidate(emulationSettingsProvider);
              }
              break;

            default:
              break;
          }
          return null;
        default:
          return null;
      }
    });
  });
});
