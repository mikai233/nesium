import 'dart:async';

class AppDataSync {
  static const String methodAppDataChanged = 'appDataChanged';
  static const String methodSyncKV = 'syncKV';

  static Future<void> broadcast({
    required String group,
    List<String> fields = const <String>[],
    Object? payload,
  }) async {
    // Web doesn't support multi-window sync via desktop_multi_window.
  }
}
