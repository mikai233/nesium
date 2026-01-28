import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../platform/nes_gamepad.dart';
import '../../persistence/app_storage.dart';
import '../../persistence/keys.dart';
import '../../logging/app_logger.dart';
import '../../persistence/storage_codec.dart';
import '../../persistence/storage_key.dart';

final StorageKey<JsonMap> _gamepadSettingsKey = StorageKey(
  StorageKeys.settingsGamepad,
  jsonMapStringCodec(
    fallback: <String, dynamic>{},
    storageKey: StorageKeys.settingsGamepad,
  ),
);

class GamepadSettingsController extends Notifier<Map<String, GamepadMapping>> {
  @override
  Map<String, GamepadMapping> build() {
    final storage = ref.read(appStorageProvider);
    final subscription = storage.onKeyChanged.listen((event) {
      if (event.key == StorageKeys.settingsGamepad) {
        unawaitedLogged(
          _reloadFromStorage(),
          logger: 'gamepad_settings',
          message: 'Reloading gamepad settings from stream',
        );
      }
    });
    ref.onDispose(() => subscription.cancel());

    return _fromStorage(storage.read(_gamepadSettingsKey));
  }

  Future<void> _reloadFromStorage() async {
    final storage = ref.read(appStorageProvider);
    final value = storage.read(_gamepadSettingsKey);
    final newState = _fromStorage(value);
    // Compare and update if different
    if (!mapEquals(state, newState)) {
      state = newState;
    }
  }

  /// Restores saved mappings for any assigned gamepads in the list.
  /// Should be called whenever the connected gamepads list is updated.
  void restoreMappings(List<GamepadInfo> gamepads) {
    for (final gp in gamepads) {
      if (gp.port != null) {
        final savedMapping = state[gp.name];
        if (savedMapping != null) {
          unawaitedLogged(
            setGamepadMapping(gp.port!, savedMapping),
            message: 'Restoring mapping for ${gp.name} on port ${gp.port}',
            logger: 'gamepad_settings',
          );
        }
      }
    }
  }

  /// Saves the mapping for a specific gamepad and applies it to the port.
  void saveMapping(String gamepadName, int port, GamepadMapping mapping) {
    if (state[gamepadName] == mapping) return;

    // Update state and persist
    state = {...state, gamepadName: mapping};
    unawaitedLogged(
      setGamepadMapping(port, mapping),
      message: 'setGamepadMapping for $gamepadName',
      logger: 'gamepad_settings',
    );
    _persist(state);
  }

  void _persist(Map<String, GamepadMapping> value) {
    unawaitedLogged(
      Future<void>.sync(
        () => ref
            .read(appStorageProvider)
            .write(
              _gamepadSettingsKey,
              Map<String, dynamic>.from(_toStorage(value)),
            ),
      ),
      message: 'Persist gamepad settings',
      logger: 'gamepad_settings',
    );
  }
}

final gamepadSettingsProvider =
    NotifierProvider<GamepadSettingsController, Map<String, GamepadMapping>>(
      GamepadSettingsController.new,
    );

Map<String, Object?> _toStorage(Map<String, GamepadMapping> value) {
  return value.map((key, value) => MapEntry(key, value.toJson()));
}

Map<String, GamepadMapping> _fromStorage(JsonMap? value) {
  if (value == null) return {};
  try {
    final entries = <MapEntry<String, GamepadMapping>>[];
    for (final entry in value.entries) {
      final name = entry.key;

      final val = entry.value;
      if (val is Map) {
        try {
          final mapping = GamepadMapping.fromJson(val.cast<String, dynamic>());
          entries.add(MapEntry(name, mapping));
        } catch (e, st) {
          logWarning(
            e,
            stackTrace: st,
            message: 'Ignoring malformed gamepad mapping entry: $name',
            logger: 'gamepad_settings',
          );
        }
      }
    }
    return Map.fromEntries(entries);
  } catch (e, stack) {
    logError(e, stackTrace: stack, message: 'Failed to load gamepad settings');
    return {};
  }
}
