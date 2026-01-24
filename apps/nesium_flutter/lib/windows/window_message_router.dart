import 'dart:async';

import 'package:desktop_multi_window/desktop_multi_window.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../logging/app_logger.dart';
import '../platform/platform_capabilities.dart';
import '../features/settings/language_settings.dart';
import '../features/settings/theme_settings.dart';
import 'settings_sync.dart';

final windowMessageRouterProvider = Provider<void>((ref) {
  if (!isNativeDesktop) return;

  var disposed = false;
  ref.onDispose(() => disposed = true);

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
          final payload = args['payload'];

          switch (group) {
            case 'language':
              if (payload is String) {
                ref.read(appLanguageProvider.notifier).applySynced(payload);
              }
              break;
            case 'theme':
              final next = payload is Map
                  ? ThemeSettings.fromJson(Map<String, dynamic>.from(payload))
                  : null;
              if (next != null) {
                ref.read(themeSettingsProvider.notifier).applySynced(next);
              }
              break;
          }
          return null;
        default:
          return null;
      }
    });
  });
});
