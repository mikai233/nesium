import 'web_cmd_sender.dart';

Future<void> setIntegerFpsMode({required bool enabled}) async {
  if (!isWebNesReady) return;
  webPostCmd('setIntegerFpsMode', {'enabled': enabled});
}
