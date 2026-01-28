import 'dart:async';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:hive_flutter/hive_flutter.dart';
import '../logging/app_logger.dart';
import '../windows/app_data_sync.dart';
import '../windows/window_types.dart';

abstract class AppStorage {
  T? get<T>(String key);

  Future<void> put(String key, Object? value);
  Future<void> delete(String key);

  /// Called when a sync message is received from another window.
  Future<void> handleSyncUpdate(String key, Object? value);

  /// Returns whether a key should be synced across windows.
  bool shouldSync(String key);

  /// Stream of key-value changes.
  Stream<({String key, Object? value})> get onKeyChanged;

  /// Returns all data that should be synced across windows.
  Map<String, dynamic> exportSyncableData();
}

AppStorage appStorage = _MemoryAppStorage();

final appStorageProvider = Provider<AppStorage>((_) => appStorage);

Future<void> initAppStorage(
  WindowKind kind, {
  Map<String, dynamic>? initialData,
}) async {
  if (kind != WindowKind.main) {
    appStorage = _RemoteAppStorage(initialData: initialData);
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
    if (shouldSync(key)) {
      await AppDataSync.broadcast(
        group: AppDataSync.methodSyncKV,
        payload: {'key': key, 'value': value},
      );
    }
  }

  @override
  Future<void> delete(String key) async {
    await _box.delete(key);
    _notify(key, null);
    if (shouldSync(key)) {
      await AppDataSync.broadcast(
        group: AppDataSync.methodSyncKV,
        payload: {'key': key, 'value': null},
      );
    }
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
    await AppDataSync.broadcast(
      group: AppDataSync.methodSyncKV,
      payload: {'key': key, 'value': value},
    );
  }

  @override
  bool shouldSync(String key) => key.startsWith('settings.');

  @override
  Map<String, dynamic> exportSyncableData() {
    final Map<String, dynamic> data = {};
    for (final key in _box.keys) {
      final k = key.toString();
      if (shouldSync(k)) {
        data[k] = _box.get(key);
      }
    }
    return data;
  }
}

final class _RemoteAppStorage
    with _AppStorageStreamMixin
    implements AppStorage {
  late final Map<String, Object?> _cache;

  _RemoteAppStorage({Map<String, dynamic>? initialData}) {
    _cache = initialData != null
        ? Map<String, Object?>.from(initialData)
        : <String, Object?>{};
  }

  @override
  Map<String, dynamic> exportSyncableData() {
    return Map<String, dynamic>.from(_cache);
  }

  @override
  bool shouldSync(String key) => key.startsWith('settings.');

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
    // Send to Main Window
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

  Future<void> _sendToMain(String key, Object? value) async {
    // Only broadcast to Main if the key should be synced.
    if (shouldSync(key)) {
      await AppDataSync.broadcast(
        group: AppDataSync.methodSyncKV,
        payload: {'key': key, 'value': value},
      );
    }
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
  bool shouldSync(String key) => false;

  @override
  Map<String, dynamic> exportSyncableData() {
    return {};
  }
}
