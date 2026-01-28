import 'dart:async';
import 'dart:convert';

import 'package:desktop_multi_window/desktop_multi_window.dart';

import '../logging/app_logger.dart';
import '../platform/platform_capabilities.dart';

class AppDataSync {
  static const String methodAppDataChanged = 'appDataChanged';
  static const String methodSyncKV = 'syncKV';

  static Future<void> broadcast({
    required String group,
    List<String> fields = const <String>[],
    Object? payload,
  }) async {
    if (!isNativeDesktop) return;

    late final String currentId;
    String? currentArgs;
    try {
      final controller = await WindowController.fromCurrentEngine();
      currentId = controller.windowId;
      currentArgs = controller.arguments;
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'AppDataSync.broadcast: failed to get current window info',
        logger: 'app_data_sync',
      );
      return;
    }

    try {
      // Resolve Main Window ID from arguments if we are a sub-window.
      String? targetMainId;
      if (currentArgs.isNotEmpty) {
        try {
          final data = jsonDecode(currentArgs);
          if (data is Map && data['mainId'] is String) {
            targetMainId = data['mainId'] as String;
          }
        } catch (e, st) {
          logError(
            e,
            stackTrace: st,
            message: 'AppDataSync.broadcast: failed to decode window arguments',
            logger: 'app_data_sync',
          );
        }
      }

      final windows = await WindowController.getAll();
      final targetIds = <String>{};

      // 1. If we are a sub-window, we must ensure Main Window gets the update.
      if (targetMainId != null && targetMainId != currentId) {
        targetIds.add(targetMainId);
      }

      // 2. Add all other existing windows except ourselves.
      for (final window in windows) {
        if (window.windowId != currentId) {
          targetIds.add(window.windowId);
        }
      }

      // 3. Dispatch to all target IDs.
      for (final id in targetIds) {
        try {
          final window = WindowController.fromWindowId(id);
          final args = <String, Object?>{'group': group, 'fields': fields};
          if (payload != null) {
            args['payload'] = payload;
          }
          unawaited(window.invokeMethod<void>(methodAppDataChanged, args));
        } catch (e, st) {
          logWarning(
            e,
            stackTrace: st,
            message: 'AppDataSync.broadcast: failed to send to window $id',
            logger: 'app_data_sync',
          );
        }
      }
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'AppDataSync.broadcast: failed to send to other windows',
        logger: 'app_data_sync',
      );
    }
  }
}
