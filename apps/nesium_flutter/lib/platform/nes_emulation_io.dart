import '../bridge/api/emulation.dart' as frb_emulation;

Future<void> setIntegerFpsMode({required bool enabled}) =>
    frb_emulation.setIntegerFpsMode(enabled: enabled);

Future<void> setRewindConfig({
  required bool enabled,
  required BigInt capacity,
}) => frb_emulation.setRewindConfig(enabled: enabled, capacity: capacity);

Future<void> setRewinding({required bool rewinding}) =>
    frb_emulation.setRewinding(rewinding: rewinding);
