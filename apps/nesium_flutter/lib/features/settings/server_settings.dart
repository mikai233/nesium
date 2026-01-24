import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:freezed_annotation/freezed_annotation.dart';

import '../../logging/app_logger.dart';
import '../../persistence/app_storage.dart';
import '../../persistence/keys.dart';
import '../../bridge/api/netplay.dart';
import '../../bridge/api/server.dart';
import '../netplay/netplay_models.dart';

part 'server_settings.freezed.dart';
part 'server_settings.g.dart';

@freezed
sealed class ServerSettings with _$ServerSettings {
  const factory ServerSettings({
    @Default(5233) int port,
    @Default('Player') String playerName,
    @Default('nesium.mikai.link:5233') String p2pServerAddr,
    @Default(false) bool p2pEnabled,
    int? p2pHostRoomCode,
    @Default(NetplayTransportOption.auto) NetplayTransportOption transport,
    @Default('localhost') String sni,
    @Default('') String fingerprint,
    @Default('localhost') String directAddr,
  }) = _ServerSettings;

  factory ServerSettings.fromJson(Map<String, dynamic> json) =>
      _$ServerSettingsFromJson(json);
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
      final stored = storage.get(StorageKeys.settingsServer);

      if (stored is Map) {
        state = ServerSettings.fromJson(Map<String, dynamic>.from(stored));
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
    await _persist();
  }

  Future<void> setPlayerName(String name) async {
    if (name == state.playerName) return;
    state = state.copyWith(playerName: name);
    await _persist();
  }

  Future<void> setP2PServerAddr(String addr) async {
    if (addr == state.p2pServerAddr) return;
    state = state.copyWith(p2pServerAddr: addr);
    await _persist();
  }

  Future<int> startServer() async {
    try {
      final actualPort = await netserverStart(port: state.port);
      return actualPort;
    } catch (e) {
      rethrow;
    }
  }

  Future<void> setP2PEnabled(bool enabled) async {
    if (enabled == state.p2pEnabled) return;
    state = state.copyWith(p2pEnabled: enabled);
    await _persist();
  }

  Future<void> setTransport(NetplayTransportOption option) async {
    if (option == state.transport) return;
    state = state.copyWith(transport: option);
    await _persist();
  }

  Future<void> setSni(String sni) async {
    if (sni == state.sni) return;
    state = state.copyWith(sni: sni);
    await _persist();
  }

  Future<void> setFingerprint(String fingerprint) async {
    if (fingerprint == state.fingerprint) return;
    state = state.copyWith(fingerprint: fingerprint);
    await _persist();
  }

  Future<void> setDirectAddr(String addr) async {
    if (addr == state.directAddr) return;
    state = state.copyWith(directAddr: addr);
    await _persist();
  }

  Future<void> _persist() async {
    try {
      await ref
          .read(appStorageProvider)
          .put(StorageKeys.settingsServer, state.toJson());
    } catch (e, st) {
      logError(
        e,
        stackTrace: st,
        message: 'Failed to persist server settings',
        logger: 'server_settings',
      );
    }
  }

  Future<void> stopServer() async {
    try {
      await netserverStop();
      state = state.copyWith(p2pHostRoomCode: null);
    } catch (e) {
      rethrow;
    }
  }

  Future<int> startP2PHost() async {
    final addr = state.p2pServerAddr.trim();
    if (addr.isEmpty) {
      throw ArgumentError('Invalid P2P server address');
    }

    final wasRunning = await netserverIsRunning();
    try {
      final roomCode = await netplayP2PHostStart(
        signalingAddr: addr,
        relayAddr: addr,
        playerName: state.playerName,
      );
      state = state.copyWith(p2pHostRoomCode: roomCode);
      return roomCode;
    } catch (e) {
      if (!wasRunning) {
        try {
          await netserverStop();
        } catch (_) {}
      }
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
