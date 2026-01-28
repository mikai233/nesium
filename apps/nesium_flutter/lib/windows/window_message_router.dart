import 'dart:async';

import 'package:desktop_multi_window/desktop_multi_window.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../logging/app_logger.dart';
import '../platform/platform_capabilities.dart';
import '../persistence/app_storage.dart';
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
        case SettingsSync.methodSettingsChanged:
          final args = call.arguments;
          if (args is! Map) {
            logWarning(
              'Invalid arguments for methodSettingsChanged (expected Map): $args',
              logger: 'window_message_router',
            );
            return null;
          }
          final group = args['group'];
          if (group is! String) {
            logWarning(
              'Invalid group for methodSettingsChanged (expected String): $group',
              logger: 'window_message_router',
            );
            return null;
          }

          final payload = args['payload'];

          switch (group) {
            case SettingsSync.methodRequestFullSync:
              if (payload != null) {
                ref.read(appStorageProvider).handleRequestFullSync(payload);
              } else {
                logWarning(
                  'Invalid payload for methodRequestFullSync (expected ID): $payload',
                  logger: 'window_message_router',
                );
              }
              break;
            case SettingsSync.methodSyncKV:
              if (payload is Map) {
                final key = payload['key'];
                final value = payload['value'];
                if (key is String) {
                  ref.read(appStorageProvider).handleSyncUpdate(key, value);
                } else {
                  logWarning(
                    'Invalid key for methodSyncKV (expected String): $key',
                    logger: 'window_message_router',
                  );
                }
              } else {
                logWarning(
                  'Invalid payload for methodSyncKV (expected Map): $payload',
                  logger: 'window_message_router',
                );
              }
              break;
            default:
              logWarning(
                'Unknown group for methodSettingsChanged: $group',
                logger: 'window_message_router',
              );
              break;
          }
          return null;
        default:
          logWarning(
            'Unknown method call: ${call.method}',
            logger: 'window_message_router',
          );
          return null;
      }
    });
  });
});
