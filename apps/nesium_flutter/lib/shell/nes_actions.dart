import 'package:flutter/foundation.dart';

typedef NesSlotCallback = Future<void> Function(int slot);

class NesActions {
  const NesActions({
    required this.openRom,
    this.saveState,
    this.loadState,
    this.saveStateSlot,
    this.loadStateSlot,
    this.saveStateFile,
    this.loadStateFile,
    required this.reset,
    required this.powerReset,
    required this.eject,
    required this.togglePause,
    required this.openSettings,
    required this.openAbout,
    required this.openDebugger,
    required this.openTools,
  });

  final AsyncCallback openRom;
  final AsyncCallback? saveState;
  final AsyncCallback? loadState;
  final NesSlotCallback? saveStateSlot;
  final NesSlotCallback? loadStateSlot;
  final AsyncCallback? saveStateFile;
  final AsyncCallback? loadStateFile;
  final AsyncCallback reset;
  final AsyncCallback powerReset;
  final AsyncCallback eject;
  final AsyncCallback togglePause;
  final AsyncCallback openSettings;
  final AsyncCallback openAbout;
  final AsyncCallback openDebugger;
  final AsyncCallback openTools;
}
