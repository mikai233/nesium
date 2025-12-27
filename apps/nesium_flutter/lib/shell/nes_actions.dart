import 'package:flutter/foundation.dart';

class NesActions {
  const NesActions({
    required this.openRom,
    required this.reset,
    required this.powerReset,
    required this.eject,
    required this.togglePause,
    required this.openSettings,
    required this.openDebugger,
    required this.openTools,
  });

  final AsyncCallback openRom;
  final AsyncCallback reset;
  final AsyncCallback powerReset;
  final AsyncCallback eject;
  final AsyncCallback togglePause;
  final AsyncCallback openSettings;
  final AsyncCallback openDebugger;
  final AsyncCallback openTools;
}
