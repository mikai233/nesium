import 'dart:async';

import 'package:desktop_multi_window/desktop_multi_window.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../logging/app_logger.dart';
import '../persistence/app_storage.dart';
import '../platform/platform_capabilities.dart';
import 'app_data_sync.dart';

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
        case AppDataSync.methodAppDataChanged:
          final args = call.arguments;
          if (args is! Map) {
            logWarning(
              'Invalid arguments for methodAppDataChanged (expected Map): $args',
              logger: 'window_message_router',
            );
            return null;
          }
          final group = args['group'];
          if (group is! String) {
            logWarning(
              'Invalid group for methodAppDataChanged (expected String): $group',
              logger: 'window_message_router',
            );
            return null;
          }

          final payload = args['payload'];

          switch (group) {
            case AppDataSync.methodSyncKV:
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
                'Unknown group for methodAppDataChanged: $group',
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
