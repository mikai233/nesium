import 'dart:async';
import 'package:desktop_multi_window/desktop_multi_window.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:hive_flutter/hive_flutter.dart';
import '../logging/app_logger.dart';
import '../windows/settings_sync.dart';
import '../windows/window_types.dart';

abstract class AppStorage {
  T? get<T>(String key);

  Future<void> put(String key, Object? value);
  Future<void> delete(String key);

  /// Called when a sync message is received from another window.
  Future<void> handleSyncUpdate(String key, Object? value);

  /// Called when a child window requests a full sync.
  Future<void> handleRequestFullSync(Object replyToId);

  /// Stream of key-value changes.
  Stream<({String key, Object? value})> get onKeyChanged;
}

AppStorage appStorage = _MemoryAppStorage();

final appStorageProvider = Provider<AppStorage>((_) => appStorage);

Future<void> initAppStorage(WindowKind kind) async {
  if (kind != WindowKind.main) {
    appStorage = _RemoteAppStorage();
    return;
  }

  try {
    await Hive.initFlutter();
    final box = await Hive.openBox<Object?>('nesium');
    appStorage = _HiveAppStorage(box);
  } catch (e, st) {
    logWarning(
      e,
      stackTrace: st,
      message: 'Failed to initialize Hive storage',
      logger: 'app_storage',
    );
    appStorage = _MemoryAppStorage();
  }
}

base mixin _AppStorageStreamMixin implements AppStorage {
  final StreamController<({String key, Object? value})> _controller =
      StreamController.broadcast();

  @override
  Stream<({String key, Object? value})> get onKeyChanged => _controller.stream;

  void _notify(String key, Object? value) {
    _controller.add((key: key, value: value));
  }
}

final class _HiveAppStorage with _AppStorageStreamMixin implements AppStorage {
  _HiveAppStorage(this._box);
  final Box<Object?> _box;

  @override
  T? get<T>(String key) {
    final value = _box.get(key);
    if (value is T) return value;
    return null;
  }

  @override
  Future<void> put(String key, Object? value) async {
    await _box.put(key, value);
    _notify(key, value);
    // Broadcast to children
    await SettingsSync.broadcast(
      group: SettingsSync.methodSyncKV,
      payload: {'key': key, 'value': value},
    );
  }

  @override
  Future<void> delete(String key) async {
    await _box.delete(key);
    _notify(key, null);
    await SettingsSync.broadcast(
      group: SettingsSync.methodSyncKV,
      payload: {'key': key, 'value': null},
    );
  }

  @override
  Future<void> handleSyncUpdate(String key, Object? value) async {
    // Received update from a child window.
    // 1. Commit to Hive.
    if (value == null) {
      await _box.delete(key);
    } else {
      await _box.put(key, value);
    }
    _notify(key, value);

    // 2. Broadcast to *other* children.
    await SettingsSync.broadcast(
      group: SettingsSync.methodSyncKV,
      payload: {'key': key, 'value': value},
    );
  }

  @override
  Future<void> handleRequestFullSync(Object replyToId) async {
    final targetId = replyToId.toString();
    // Iterate all keys and send to the requester.
    for (final key in _box.keys) {
      final k = key.toString();
      final value = _box.get(key);
      try {
        await WindowController.fromWindowId(targetId).invokeMethod(
          SettingsSync.methodSettingsChanged,
          {
            'group': SettingsSync.methodSyncKV,
            'payload': {'key': k, 'value': value},
          },
        );
      } catch (e, st) {
        logWarning(
          e,
          stackTrace: st,
          message: 'HiveAppStorage failed to sync key $k to window $targetId',
          logger: 'app_storage',
        );
      }
    }
  }
}

final class _RemoteAppStorage
    with _AppStorageStreamMixin
    implements AppStorage {
  final Map<String, Object?> _cache = <String, Object?>{};

  _RemoteAppStorage() {
    // Request full sync on startup.
    // Fire and forget, but with internal retry.
    _requestFullSync();
  }

  Future<void> _requestFullSync([int attempt = 0]) async {
    try {
      if (attempt > 0) {
        await Future.delayed(Duration(milliseconds: 500 * attempt));
      }

      // Check if we can get the current engine's controller first.
      final controller = await WindowController.fromCurrentEngine();
      final myId = controller.windowId;

      // Broadcast a "request full sync" to ALL windows.
      // The Main window will respond directly to us.
      await SettingsSync.broadcast(
        group: SettingsSync.methodRequestFullSync,
        payload: myId,
      );
    } catch (e, st) {
      if (attempt < 5) {
        logWarning(
          'RemoteAppStorage failed to request full sync (attempt ${attempt + 1}), retrying... Error: $e',
          logger: 'app_storage',
        );
        return _requestFullSync(attempt + 1);
      }

      logError(
        e,
        stackTrace: st,
        message:
            'RemoteAppStorage failed to request full sync after 6 attempts',
        logger: 'app_storage',
      );
    }
  }

  @override
  T? get<T>(String key) {
    final value = _cache[key];
    if (value is T) return value;
    return null;
  }

  @override
  Future<void> put(String key, Object? value) async {
    _cache[key] = value;
    _notify(key, value);
    // Send to Main Window (and implicitly others via Main's broadcast)
    await _sendToMain(key, value);
  }

  @override
  Future<void> delete(String key) async {
    _cache.remove(key);
    _notify(key, null);
    await _sendToMain(key, null);
  }

  @override
  Future<void> handleSyncUpdate(String key, Object? value) async {
    if (value == null) {
      _cache.remove(key);
    } else {
      _cache[key] = value;
    }
    _notify(key, value);
  }

  @override
  Future<void> handleRequestFullSync(Object replyToId) async {
    // Remote storage doesn't store the source of truth, so it shouldn't be asked for full sync.
  }

  Future<void> _sendToMain(String key, Object? value) async {
    // We broadcast the KV update to ALL windows (including Main).
    // Main will hear it and persist it. Other children will hear it and update their cache.
    await SettingsSync.broadcast(
      group: SettingsSync.methodSyncKV,
      payload: {'key': key, 'value': value},
    );
  }
}

final class _MemoryAppStorage
    with _AppStorageStreamMixin
    implements AppStorage {
  final Map<String, Object?> _data = <String, Object?>{};

  @override
  T? get<T>(String key) {
    final value = _data[key];
    if (value is T) return value;
    return null;
  }

  @override
  Future<void> put(String key, Object? value) async {
    _data[key] = value;
    _notify(key, value);
  }

  @override
  Future<void> delete(String key) async {
    _data.remove(key);
    _notify(key, null);
  }

  @override
  Future<void> handleSyncUpdate(String key, Object? value) async {
    // Memory storage doesn't sync.
  }

  @override
  Future<void> handleRequestFullSync(Object replyToId) async {
    // Memory storage doesn't sync.
  }
}
