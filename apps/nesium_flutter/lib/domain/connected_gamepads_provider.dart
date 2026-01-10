import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../logging/app_logger.dart';
import '../platform/nes_gamepad.dart' as nes_gamepad;

/// Provider for the list of connected gamepads.
/// Refreshes automatically every 2 seconds.
final connectedGamepadsProvider =
    StreamProvider.autoDispose<List<nes_gamepad.GamepadInfo>>((ref) async* {
      // Initialize gamepad if not already done
      if (nes_gamepad.isGamepadSupported) {
        await nes_gamepad.initGamepad();
      }

      // Initial list
      yield await _fetchGamepads();

      // Periodic refresh
      await for (final _ in Stream.periodic(const Duration(seconds: 2))) {
        yield await _fetchGamepads();
      }
    });

Future<List<nes_gamepad.GamepadInfo>> _fetchGamepads() async {
  if (!nes_gamepad.isGamepadSupported) return [];
  try {
    return await nes_gamepad.listGamepads();
  } catch (e, st) {
    logError(e, stackTrace: st, message: 'Failed to list gamepads');
    return [];
  }
}
