import 'package:flutter_riverpod/flutter_riverpod.dart';

class RemapLocation {
  final Object action;
  final int port;
  const RemapLocation(this.action, this.port);
}

class RemappingNotifier extends Notifier<RemapLocation?> {
  @override
  RemapLocation? build() => null;
  void update(RemapLocation? val) => state = val;
}

/// State for the remapping process.
final remappingStateProvider =
    NotifierProvider<RemappingNotifier, RemapLocation?>(RemappingNotifier.new);

enum NesButtonAction {
  a,
  b,
  select,
  start,
  up,
  down,
  left,
  right,
  turboA,
  turboB,
  rewind,
  fastForward,
  saveState,
  loadState,
  pause,
  fullScreen,
}

extension NesButtonActionExt on NesButtonAction {
  bool get isCore => index <= NesButtonAction.turboB.index;
  bool get isExtended => index > NesButtonAction.turboB.index;
}
