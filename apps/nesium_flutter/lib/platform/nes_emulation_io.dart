import '../bridge/api/emulation.dart' as frb_emulation;

Future<void> setIntegerFpsMode({required bool enabled}) =>
    frb_emulation.setIntegerFpsMode(enabled: enabled);
