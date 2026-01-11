import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../logging/app_logger.dart';
import '../features/settings/gamepad_settings.dart';
import '../features/settings/emulation_settings.dart';
import 'connected_gamepads_provider.dart';
import 'nes_input_masks.dart';
import '../platform/nes_gamepad.dart' as nes_gamepad;
import '../shell/nes_actions.dart';
import '../features/controls/input_settings.dart';

/// Provider for gamepad state that polls and merges with keyboard input.
final gamepadServiceProvider = NotifierProvider<GamepadService, void>(
  GamepadService.new,
);

/// Service that manages gamepad initialization and state.
class GamepadService extends Notifier<void> {
  bool _initialized = false;
  Timer? _pollTimer;
  int _quickSaveSlot = 1;

  @override
  void build() {
    // Keep settings loaded
    ref.listen(gamepadSettingsProvider, (_, _) {});
    _quickSaveSlot = ref.read(emulationSettingsProvider).quickSaveSlot;

    // Restore mappings when gamepads list updates
    ref.listen(connectedGamepadsProvider, (_, next) {
      next.whenData((list) {
        ref.read(gamepadSettingsProvider.notifier).restoreMappings(list);
      });
    });
    ref.listen(emulationSettingsProvider, (_, next) {
      _quickSaveSlot = next.quickSaveSlot;
    });

    // Initialize only once
    if (!_initialized) {
      _init();
    }

    ref.onDispose(() {
      _pollTimer?.cancel();
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
      appLog.info('Gamepad Service initialized');

      // On Web, we need a Dart-side polling loop to bridge input to the WASM core.
      // On Desktop, this is handled by a dedicated Rust thread.
      if (kIsWeb) {
        _startWebPolling();
      }
    } catch (e, st) {
      logError(
        e,
        stackTrace: st,
        message: 'Failed to initialize Gamepad Service',
      );
    }
  }

  void _startWebPolling() {
    _pollTimer?.cancel();
    // Poll at ~120Hz (8ms) for responsiveness
    _pollTimer = Timer.periodic(const Duration(milliseconds: 8), (_) async {
      final result = await nes_gamepad.pollGamepads();
      if (result != null) {
        final masks = result.padMasks;
        final turboMasks = result.turboMasks;

        // Port 0 and 1 only for now
        for (var i = 0; i < 2; i++) {
          ref
              .read(nesInputMasksProvider.notifier)
              .updateGamepadMasks(i, masks[i], turboMasks[i]);
        }

        // Trigger extended actions
        final actions = ref.read(nesActionsProvider);
        final pollActions = result.actions;

        actions.setRewinding?.call(pollActions.rewind);
        actions.setFastForwarding?.call(pollActions.fastForward);
        if (pollActions.saveState) {
          final slot = _quickSaveSlot;
          if (actions.saveStateSlot != null) {
            unawaited(actions.saveStateSlot!.call(slot));
          } else {
            actions.saveState?.call();
          }
        }
        if (pollActions.loadState) {
          final slot = _quickSaveSlot;
          if (actions.loadStateSlot != null) {
            unawaited(actions.loadStateSlot!.call(slot));
          } else {
            actions.loadState?.call();
          }
        }
        if (pollActions.pause) actions.togglePause?.call();

        // Update last input method if there is any activity
        final hasAnyActivity =
            masks.any((m) => m != 0) ||
            turboMasks.any((m) => m != 0) ||
            pollActions.rewind ||
            pollActions.fastForward ||
            pollActions.saveState ||
            pollActions.loadState ||
            pollActions.pause;

        if (hasAnyActivity) {
          ref.read(lastInputMethodProvider.notifier).set(InputMethod.gamepad);
        }
      }
    });
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
    _pollTimer?.cancel();
    if (_initialized) {
      await nes_gamepad.shutdownGamepad();
      _initialized = false;
    }
  }
}
