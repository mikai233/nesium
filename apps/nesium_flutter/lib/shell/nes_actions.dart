import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

typedef NesSlotCallback = Future<void> Function(int slot);

final nesActionsProvider = Provider<NesActions>((ref) {
  throw UnimplementedError('nesActionsProvider must be overridden');
});

class NesActions {
  const NesActions({
    this.openRom,
    this.saveState,
    this.loadState,
    this.openAutoSave,
    this.saveStateSlot,
    this.loadStateSlot,
    this.saveStateFile,
    this.loadStateFile,
    this.loadTasMovie,
    this.reset,
    this.powerReset,
    this.powerOff,
    this.togglePause,
    this.openSettings,
    this.openAbout,
    this.openDebugger,
    this.openTools,
    this.openTilemapViewer,
    this.openTileViewer,
    this.openSpriteViewer,
    this.openPaletteViewer,
    this.openHistoryViewer,
    this.openNetplay,
  });

  final AsyncCallback? openRom;
  final AsyncCallback? saveState;
  final AsyncCallback? loadState;
  final AsyncCallback? openAutoSave;
  final NesSlotCallback? saveStateSlot;
  final NesSlotCallback? loadStateSlot;
  final AsyncCallback? saveStateFile;
  final AsyncCallback? loadStateFile;
  final AsyncCallback? loadTasMovie;
  final AsyncCallback? reset;
  final AsyncCallback? powerReset;
  final AsyncCallback? powerOff;
  final AsyncCallback? togglePause;
  final AsyncCallback? openSettings;
  final AsyncCallback? openAbout;
  final AsyncCallback? openDebugger;
  final AsyncCallback? openTools;
  final AsyncCallback? openTilemapViewer;
  final AsyncCallback? openTileViewer;
  final AsyncCallback? openSpriteViewer;
  final AsyncCallback? openPaletteViewer;
  final AsyncCallback? openHistoryViewer;
  final AsyncCallback? openNetplay;
}
