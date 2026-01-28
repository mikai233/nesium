import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:hive_flutter/hive_flutter.dart';
import '../logging/app_logger.dart';

abstract class AppStorage {
  T? get<T>(String key);

  Future<void> put(String key, Object? value);
  Future<void> delete(String key);
}

AppStorage appStorage = _MemoryAppStorage();

final appStorageProvider = Provider<AppStorage>((_) => appStorage);

Future<void> initAppStorage() async {
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

final class _HiveAppStorage implements AppStorage {
  const _HiveAppStorage(this._box);
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
  }

  @override
  Future<void> delete(String key) async {
    await _box.delete(key);
  }
}

final class _MemoryAppStorage implements AppStorage {
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
  }

  @override
  Future<void> delete(String key) async {
    _data.remove(key);
  }
}
