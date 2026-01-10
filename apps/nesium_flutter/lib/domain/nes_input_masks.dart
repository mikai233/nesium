import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../logging/app_logger.dart';
import '../platform/nes_input.dart' as nes_input;
import 'pad_button.dart';

class NesInputMasksState {
  const NesInputMasksState({required this.padMasks, required this.turboMasks});

  final Map<int, int> padMasks;
  final Map<int, int> turboMasks;

  NesInputMasksState copyWith({
    Map<int, int>? padMasks,
    Map<int, int>? turboMasks,
  }) {
    return NesInputMasksState(
      padMasks: padMasks ?? this.padMasks,
      turboMasks: turboMasks ?? this.turboMasks,
    );
  }
}

class NesInputMasksController extends Notifier<NesInputMasksState> {
  @override
  NesInputMasksState build() =>
      const NesInputMasksState(padMasks: {}, turboMasks: {});

  void flushToNative() {
    for (var i = 0; i < 2; i++) {
      final padMask = state.padMasks[i] ?? 0;
      final turboMask = state.turboMasks[i] ?? 0;

      unawaitedLogged(
        nes_input.setPadMask(pad: i, mask: padMask & 0xFF),
        message: 'setPadMask (flush) pad $i',
        logger: 'nes_input_masks',
      );
      unawaitedLogged(
        nes_input.setTurboMask(pad: i, mask: turboMask & 0xFF),
        message: 'setTurboMask (flush) pad $i',
        logger: 'nes_input_masks',
      );
    }
  }

  void setPressed(PadButton button, bool pressed, {int pad = 0}) {
    final bit = _buttonBit(button);
    final mask = 1 << bit;
    final currentMask = state.padMasks[pad] ?? 0;
    final next = pressed ? (currentMask | mask) : (currentMask & ~mask);
    if (next == currentMask) return;

    final nextMasks = Map<int, int>.from(state.padMasks);
    nextMasks[pad] = next;
    state = state.copyWith(padMasks: nextMasks);

    unawaitedLogged(
      nes_input.setPadMask(pad: pad, mask: next & 0xFF),
      message: 'setPadMask pad $pad',
      logger: 'nes_input_masks',
    );
  }

  void setTurboEnabled(PadButton button, bool enabled, {int pad = 0}) {
    final bit = _buttonBit(button);
    final mask = 1 << bit;
    final currentMask = state.turboMasks[pad] ?? 0;
    final next = enabled ? (currentMask | mask) : (currentMask & ~mask);
    if (next == currentMask) return;

    final nextMasks = Map<int, int>.from(state.turboMasks);
    nextMasks[pad] = next;
    state = state.copyWith(turboMasks: nextMasks);

    unawaitedLogged(
      nes_input.setTurboMask(pad: pad, mask: next & 0xFF),
      message: 'setTurboMask pad $pad',
      logger: 'nes_input_masks',
    );
  }

  void clearAll() {
    if (state.padMasks.isEmpty && state.turboMasks.isEmpty) return;
    state = const NesInputMasksState(padMasks: {}, turboMasks: {});
    for (var i = 0; i < 2; i++) {
      unawaitedLogged(
        nes_input.setPadMask(pad: i, mask: 0),
        message: 'setPadMask (clearAll) pad $i',
        logger: 'nes_input_masks',
      );
      unawaitedLogged(
        nes_input.setTurboMask(pad: i, mask: 0),
        message: 'setTurboMask (clearAll) pad $i',
        logger: 'nes_input_masks',
      );
    }
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
