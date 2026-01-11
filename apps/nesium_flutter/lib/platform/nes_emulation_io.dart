import 'dart:typed_data';
import '../bridge/api/emulation.dart' as frb_emulation;

Future<void> setIntegerFpsMode({required bool enabled}) =>
    frb_emulation.setIntegerFpsMode(enabled: enabled);

Future<void> setRewindConfig({
  required bool enabled,
  required BigInt capacity,
}) => frb_emulation.setRewindConfig(enabled: enabled, capacity: capacity);

Future<void> setRewinding({required bool rewinding}) =>
    frb_emulation.setRewinding(rewinding: rewinding);

Future<void> setFastForwarding({required bool fastForwarding}) =>
    frb_emulation.setFastForwarding(fastForwarding: fastForwarding);

Future<void> setFastForwardSpeed({required int speedPercent}) =>
    frb_emulation.setFastForwardSpeed(speedPercent: speedPercent);

Future<void> loadTasMovie({required String data}) =>
    frb_emulation.loadTasMovie(data: data);

Future<Uint8List> saveStateToMemory() => frb_emulation.saveStateToMemory();

Future<void> loadStateFromMemory({required Uint8List data}) =>
    frb_emulation.loadStateFromMemory(data: data);
