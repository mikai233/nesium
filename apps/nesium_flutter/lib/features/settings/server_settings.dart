import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../logging/app_logger.dart';
import '../../persistence/app_storage.dart';
import '../../persistence/keys.dart';
import '../../bridge/api/server.dart';

/// Server configuration settings.
class ServerSettings {
  const ServerSettings({this.port = 5233});

  final int port;

  ServerSettings copyWith({int? port}) {
    return ServerSettings(port: port ?? this.port);
  }
}

class ServerSettingsController extends Notifier<ServerSettings> {
  bool _initialized = false;

  @override
  ServerSettings build() {
    if (!_initialized) {
      _initialized = true;
      scheduleMicrotask(_load);
    }
    return const ServerSettings();
  }

  Future<void> _load() async {
    try {
      final storage = ref.read(appStorageProvider);
      final port = storage.get(StorageKeys.serverPort) as int?;
      if (port != null) {
        state = state.copyWith(port: port);
      }
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'Failed to load server settings',
        logger: 'server_settings',
      );
    }
  }

  Future<void> setPort(int port) async {
    if (port == state.port) return;
    state = state.copyWith(port: port);
    try {
      await ref.read(appStorageProvider).put(StorageKeys.serverPort, port);
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'Failed to persist server port',
        logger: 'server_settings',
      );
    }
  }

  Future<int> startServer() async {
    try {
      final actualPort = await netserverStart(port: state.port);
      return actualPort;
    } catch (e) {
      rethrow;
    }
  }

  Future<void> stopServer() async {
    try {
      await netserverStop();
    } catch (e) {
      rethrow;
    }
  }
}

final serverSettingsProvider =
    NotifierProvider<ServerSettingsController, ServerSettings>(
      ServerSettingsController.new,
    );

/// Stream of server status updates.
final serverStatusStreamProvider = StreamProvider<ServerStatus>((ref) {
  return netserverStatusStream();
});
