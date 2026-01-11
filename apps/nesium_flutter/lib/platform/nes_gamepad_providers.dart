import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'nes_gamepad.dart' as nes_gamepad;

/// Provider for the button mapping for a specific NES port.
final gamepadMappingProvider =
    FutureProvider.family<nes_gamepad.GamepadMapping?, int>((ref, port) async {
      return nes_gamepad.getGamepadMapping(port);
    });

/// Provider for raw pressed buttons on a specific gamepad ID.
final gamepadPressedButtonsProvider =
    StreamProvider.family<List<nes_gamepad.GamepadButton>, int>((
      ref,
      gamepadId,
    ) async* {
      while (true) {
        yield await nes_gamepad.getGamepadPressedButtons(gamepadId);
        await Future.delayed(const Duration(milliseconds: 50));
      }
    });
