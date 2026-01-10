import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../logging/app_logger.dart';
import '../features/settings/gamepad_settings.dart';
import 'connected_gamepads_provider.dart';
import '../platform/nes_gamepad.dart' as nes_gamepad;

/// Provider for gamepad state that polls and merges with keyboard input.
final gamepadServiceProvider = NotifierProvider<GamepadService, void>(
  GamepadService.new,
);

/// Service that manages gamepad initialization and state.
class GamepadService extends Notifier<void> {
  bool _initialized = false;

  @override
  void build() {
    // Keep settings loaded
    ref.listen(gamepadSettingsProvider, (_, _) {});

    // Restore mappings when gamepads list updates
    ref.listen(connectedGamepadsProvider, (_, next) {
      next.whenData((list) {
        ref.read(gamepadSettingsProvider.notifier).restoreMappings(list);
      });
    });

    // Initialize only once
    if (!_initialized) {
      _init();
    }

    ref.onDispose(() {
      if (_initialized) {
        nes_gamepad.shutdownGamepad();
      }
    });
  }

  Future<void> _init() async {
    if (_initialized) return;
    if (!nes_gamepad.isGamepadSupported) return;

    try {
      await nes_gamepad.initGamepad();
      _initialized = true;
      appLog.info('Gamepad Service initialized (polling moved to Rust)');
    } catch (e, st) {
      logError(
        e,
        stackTrace: st,
        message: 'Failed to initialize Gamepad Service',
      );
    }
  }

  /// Triggers vibration on the gamepad assigned to the given port.
  Future<void> rumble({
    required int port,
    double strength = 1.0,
    int durationMs = 200,
  }) async {
    if (!_initialized) return;
    try {
      await nes_gamepad.rumbleGamepad(
        port: port,
        strength: strength,
        durationMs: durationMs,
      );
    } catch (e, st) {
      logError(e, stackTrace: st, message: 'Failed to rumble gamepad $port');
    }
  }

  /// Lists connected gamepads.
  Future<List<nes_gamepad.GamepadInfo>> listGamepads() async {
    if (!_initialized) return [];
    return nes_gamepad.listGamepads();
  }

  /// Shuts down the gamepad subsystem.
  Future<void> shutdown() async {
    if (_initialized) {
      await nes_gamepad.shutdownGamepad();
      _initialized = false;
    }
  }
}
