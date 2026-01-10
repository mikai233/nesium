import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../logging/app_logger.dart';
import '../platform/nes_input.dart' as nes_input;
import 'pad_button.dart';

class NesInputMasksState {
  const NesInputMasksState({
    required this.padMasks,
    required this.turboMasks,
    this.gamepadMasks = const {},
    this.gamepadTurboMasks = const {},
  });

  /// Masks from keyboard/virtual controls.
  final Map<int, int> padMasks;
  final Map<int, int> turboMasks;

  /// Masks specifically from gamepads.
  final Map<int, int> gamepadMasks;
  final Map<int, int> gamepadTurboMasks;

  NesInputMasksState copyWith({
    Map<int, int>? padMasks,
    Map<int, int>? turboMasks,
    Map<int, int>? gamepadMasks,
    Map<int, int>? gamepadTurboMasks,
  }) {
    return NesInputMasksState(
      padMasks: padMasks ?? this.padMasks,
      turboMasks: turboMasks ?? this.turboMasks,
      gamepadMasks: gamepadMasks ?? this.gamepadMasks,
      gamepadTurboMasks: gamepadTurboMasks ?? this.gamepadTurboMasks,
    );
  }
}

class NesInputMasksController extends Notifier<NesInputMasksState> {
  @override
  NesInputMasksState build() =>
      const NesInputMasksState(padMasks: {}, turboMasks: {});

  void flushToNative() {
    for (var i = 0; i < 2; i++) {
      final padMask = (state.padMasks[i] ?? 0) | (state.gamepadMasks[i] ?? 0);
      final turboMask =
          (state.turboMasks[i] ?? 0) | (state.gamepadTurboMasks[i] ?? 0);

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

  /// Updates gamepad-specific masks. This is called from the polling loop
  /// (on Web) or when state changes.
  void updateGamepadMasks(int port, int mask, int turboMask) {
    if (port >= 2) return; // Only 2 ports for now

    final oldMask = state.gamepadMasks[port] ?? 0;
    final oldTurbo = state.gamepadTurboMasks[port] ?? 0;

    if (oldMask == mask && oldTurbo == turboMask) return;

    final nextMasks = Map<int, int>.from(state.gamepadMasks);
    final nextTurboMasks = Map<int, int>.from(state.gamepadTurboMasks);
    nextMasks[port] = mask;
    nextTurboMasks[port] = turboMask;

    state = state.copyWith(
      gamepadMasks: nextMasks,
      gamepadTurboMasks: nextTurboMasks,
    );

    // Flush merged masks to native
    _flushPort(port);
  }

  void _flushPort(int port) {
    final mergedPad =
        (state.padMasks[port] ?? 0) | (state.gamepadMasks[port] ?? 0);
    final mergedTurbo =
        (state.turboMasks[port] ?? 0) | (state.gamepadTurboMasks[port] ?? 0);

    nes_input.setPadMask(pad: port, mask: mergedPad & 0xFF);
    nes_input.setTurboMask(pad: port, mask: mergedTurbo & 0xFF);
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

    _flushPort(pad);
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

    _flushPort(pad);
  }

  void clearAll() {
    if (state.padMasks.isEmpty &&
        state.turboMasks.isEmpty &&
        state.gamepadMasks.isEmpty &&
        state.gamepadTurboMasks.isEmpty) {
      return;
    }

    state = const NesInputMasksState(
      padMasks: {},
      turboMasks: {},
      gamepadMasks: {},
      gamepadTurboMasks: {},
    );
    for (var i = 0; i < 2; i++) {
      _flushPort(i);
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
