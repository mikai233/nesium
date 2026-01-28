import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../platform/nes_gamepad.dart' as nes_gamepad;
import '../../persistence/app_storage.dart';
import '../../persistence/keys.dart';
import '../../logging/app_logger.dart';
import '../../persistence/storage_codec.dart';
import '../../persistence/storage_key.dart';
import '../../domain/connected_gamepads_provider.dart';

final StorageKey<JsonMap> _gamepadAssignmentsKey = StorageKey(
  StorageKeys.settingsGamepadAssignments,
  jsonMapStringCodec(
    fallback: <String, dynamic>{},
    storageKey: StorageKeys.settingsGamepadAssignments,
  ),
);

class GamepadAssignmentController extends Notifier<Map<String, int>> {
  @override
  Map<String, int> build() {
    final storage = ref.read(appStorageProvider);
    final saved = storage.read(_gamepadAssignmentsKey);
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
    final payload = <String, dynamic>{
      for (final entry in state.entries) entry.key: entry.value,
    };
    unawaitedLogged(
      Future<void>.sync(
        () =>
            ref.read(appStorageProvider).write(_gamepadAssignmentsKey, payload),
      ),
      message: 'Persist gamepad assignments',
      logger: 'gamepad_assignment',
    );
  }

  Map<String, int> _fromStorage(JsonMap? value) {
    if (value == null) return {};
    try {
      return value.map((key, val) {
        final port = val is num ? val.toInt() : null;
        if (port == null) {
          throw StateError('Invalid port type for $key: ${val.runtimeType}');
        }
        return MapEntry(key, port);
      });
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
