import 'package:flutter/foundation.dart';

class NesActions {
  const NesActions({
    required this.openRom,
    required this.reset,
    required this.togglePause,
    required this.openSettings,
    required this.openDebugger,
    required this.openTools,
  });

  final AsyncCallback openRom;
  final AsyncCallback reset;
  final AsyncCallback togglePause;
  final AsyncCallback openSettings;
  final AsyncCallback openDebugger;
  final AsyncCallback openTools;
}
