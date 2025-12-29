import 'dart:typed_data';
import 'web_cmd_sender.dart';

Future<void> setIntegerFpsMode({required bool enabled}) async {
  if (!isWebNesReady) return;
  webPostCmd('setIntegerFpsMode', {'enabled': enabled});
}

Future<void> saveState({required String path}) async {
  // handled in web_shell_web
}

Future<void> loadState({required String path}) async {
  // handled in web_shell_web
}

Future<Uint8List> saveStateToMemory() async {
  return webRequest<Uint8List>('saveState');
}

Future<void> loadStateFromMemory({required Uint8List data}) async {
  return webRequest<void>('loadState', {'data': data});
}
