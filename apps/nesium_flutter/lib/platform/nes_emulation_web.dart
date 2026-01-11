import 'dart:typed_data';
import 'web_cmd_sender.dart';

Future<void> setIntegerFpsMode({required bool enabled}) async {
  if (!isWebNesReady) return;
  webPostCmd('setIntegerFpsMode', {'enabled': enabled});
}

Future<void> setRewindConfig({
  required bool enabled,
  required BigInt capacity,
}) async {
  if (!isWebNesReady) return;
  webPostCmd('setRewindConfig', {'enabled': enabled, 'capacity': capacity});
}

Future<void> setRewinding({required bool rewinding}) async {
  if (!isWebNesReady) return;
  webPostCmd('setRewinding', {'rewinding': rewinding});
}

Future<void> setFastForwarding({required bool fastForwarding}) async {
  if (!isWebNesReady) return;
  webPostCmd('setFastForwarding', {'fastForwarding': fastForwarding});
}

Future<void> setFastForwardSpeed({required int speedPercent}) async {
  if (!isWebNesReady) return;
  webPostCmd('setFastForwardSpeed', {'speedPercent': speedPercent});
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

Future<void> loadTasMovie({required String data}) async {
  if (!isWebNesReady) return;
  webPostCmd('loadTasMovie', {'data': data});
}
