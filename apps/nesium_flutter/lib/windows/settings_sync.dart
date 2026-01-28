import 'dart:async';

import 'package:desktop_multi_window/desktop_multi_window.dart';

import '../logging/app_logger.dart';
import '../platform/platform_capabilities.dart';

class SettingsSync {
  static const String methodSettingsChanged = 'settingsChanged';
  static const String methodSyncKV = 'syncKV';
  static const String methodRequestFullSync = 'requestFullSync';

  static Future<void> broadcast({
    required String group,
    List<String> fields = const <String>[],
    Object? payload,
  }) async {
    if (!isNativeDesktop) return;

    late final String currentId;
    try {
      final controller = await WindowController.fromCurrentEngine();
      currentId = controller.windowId;
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'SettingsSync.broadcast: failed to get current window id',
        logger: 'settings_sync',
      );
      return;
    }

    try {
      final windows = await WindowController.getAll();
      for (final window in windows) {
        if (window.windowId == currentId) continue;
        final args = <String, Object?>{'group': group, 'fields': fields};
        if (payload != null) {
          args['payload'] = payload;
        }
        unawaited(window.invokeMethod<void>(methodSettingsChanged, args));
      }
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'SettingsSync.broadcast: failed to send to other windows',
        logger: 'settings_sync',
      );
    }
  }
}
