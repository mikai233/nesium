import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/bridge/api/input.dart' as nes_input;
import 'package:nesium_flutter/bridge/lib.dart' show PadButton;

class NesInputMasksState {
  const NesInputMasksState({required this.padMask, required this.turboMask});

  final int padMask;
  final int turboMask;

  NesInputMasksState copyWith({int? padMask, int? turboMask}) {
    return NesInputMasksState(
      padMask: padMask ?? this.padMask,
      turboMask: turboMask ?? this.turboMask,
    );
  }
}

class NesInputMasksController extends Notifier<NesInputMasksState> {
  @override
  NesInputMasksState build() =>
      const NesInputMasksState(padMask: 0, turboMask: 0);

  void setPressed(PadButton button, bool pressed) {
    final bit = _buttonBit(button);
    final mask = 1 << bit;
    final next = pressed ? (state.padMask | mask) : (state.padMask & ~mask);
    if (next == state.padMask) return;
    state = state.copyWith(padMask: next);
    unawaited(
      nes_input.setPadMask(pad: 0, mask: next & 0xFF).catchError((_) {}),
    );
  }

  void setTurboEnabled(PadButton button, bool enabled) {
    final bit = _buttonBit(button);
    final mask = 1 << bit;
    final next = enabled ? (state.turboMask | mask) : (state.turboMask & ~mask);
    if (next == state.turboMask) return;
    state = state.copyWith(turboMask: next);
    unawaited(
      nes_input.setTurboMask(pad: 0, mask: next & 0xFF).catchError((_) {}),
    );
  }

  void clearAll() {
    if (state.padMask == 0 && state.turboMask == 0) return;
    state = const NesInputMasksState(padMask: 0, turboMask: 0);
    unawaited(nes_input.setPadMask(pad: 0, mask: 0).catchError((_) {}));
    unawaited(nes_input.setTurboMask(pad: 0, mask: 0).catchError((_) {}));
  }
}

int _buttonBit(PadButton button) {
  switch (button) {
    case PadButton.a:
      return 0;
    case PadButton.b:
      return 1;
    case PadButton.select:
      return 2;
    case PadButton.start:
      return 3;
    case PadButton.up:
      return 4;
    case PadButton.down:
      return 5;
    case PadButton.left:
      return 6;
    case PadButton.right:
      return 7;
  }
}

final nesInputMasksProvider =
    NotifierProvider<NesInputMasksController, NesInputMasksState>(
      NesInputMasksController.new,
    );
