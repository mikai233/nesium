import 'dart:async';

import 'package:desktop_multi_window/desktop_multi_window.dart';

import '../platform/platform_capabilities.dart';

class SettingsSync {
  static const String methodSettingsChanged = 'settingsChanged';

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
    } catch (_) {
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
    } catch (_) {
      // Best-effort.
    }
  }
}
