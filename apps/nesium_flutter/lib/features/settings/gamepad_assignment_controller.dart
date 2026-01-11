import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../platform/nes_gamepad.dart' as nes_gamepad;
import '../../persistence/app_storage.dart';
import '../../persistence/keys.dart';
import '../../logging/app_logger.dart';
import '../../domain/connected_gamepads_provider.dart';

class GamepadAssignmentController extends Notifier<Map<String, int>> {
  @override
  Map<String, int> build() {
    final storage = ref.read(appStorageProvider);
    final saved = storage.get(StorageKeys.settingsGamepadAssignments);
    final state = _fromStorage(saved);

    // Initial restoration logic: check current gamepads once they are available
    ref.listen(connectedGamepadsProvider, (previous, next) {
      if (next.hasValue) {
        _restoreAssignments(next.value!);
      }
    }, fireImmediately: true);

    return state;
  }

  void saveAssignment(String gamepadName, int port) {
    if (state[gamepadName] == port) return;
    state = {...state, gamepadName: port};
    _persist();
  }

  void removeAssignment(String gamepadName) {
    if (!state.containsKey(gamepadName)) return;
    final newState = Map<String, int>.from(state);
    newState.remove(gamepadName);
    state = newState;
    _persist();
  }

  Future<void> _restoreAssignments(
    List<nes_gamepad.GamepadInfo> gamepads,
  ) async {
    for (final gp in gamepads) {
      // If gamepad is not assigned to any port but we have a saved assignment
      if (gp.port == null && state.containsKey(gp.name)) {
        final targetPort = state[gp.name]!;

        // check if target port is already occupied (to avoid stealing someone else's port)
        final occupied = gamepads.any((g) => g.port == targetPort);
        if (!occupied) {
          unawaitedLogged(
            nes_gamepad.bindGamepad(id: gp.id, port: targetPort),
            message:
                'Auto-restoring assignment for ${gp.name} to port $targetPort',
            logger: 'gamepad_assignment',
          );
        }
      }
    }
  }

  void _persist() {
    unawaitedLogged(
      Future<void>.sync(
        () => ref
            .read(appStorageProvider)
            .put(StorageKeys.settingsGamepadAssignments, state),
      ),
      message: 'Persist gamepad assignments',
      logger: 'gamepad_assignment',
    );
  }

  Map<String, int> _fromStorage(Object? value) {
    if (value is! Map) return {};
    try {
      return value.map((key, val) => MapEntry(key as String, val as int));
    } catch (e) {
      logError(e, message: 'Failed to load gamepad assignments');
      return {};
    }
  }
}

final gamepadAssignmentProvider =
    NotifierProvider<GamepadAssignmentController, Map<String, int>>(
      GamepadAssignmentController.new,
    );
