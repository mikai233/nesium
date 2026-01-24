import 'dart:async';

import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../logging/app_logger.dart';
import '../../persistence/app_storage.dart';
import '../../persistence/keys.dart';
import '../../bridge/api/netplay.dart';
import '../../bridge/api/server.dart';
import '../netplay/netplay_models.dart';

/// Server configuration settings.
class ServerSettings {
  const ServerSettings({
    this.port = 5233,
    this.playerName = 'Player',
    this.p2pServerAddr = 'nesium.mikai.link:5233',
    this.p2pEnabled = false,
    this.p2pHostRoomCode,
    this.transport = NetplayTransportOption.auto,
    this.sni = 'localhost',
    this.fingerprint = '',
    this.directAddr = 'localhost',
  });

  final int port;
  final String playerName;
  final String p2pServerAddr;
  final bool p2pEnabled;
  final int? p2pHostRoomCode;
  final NetplayTransportOption transport;
  final String sni;
  final String fingerprint;
  final String directAddr;

  ServerSettings copyWith({
    int? port,
    String? playerName,
    String? p2pServerAddr,
    bool? p2pEnabled,
    int? Function()? p2pHostRoomCode,
    NetplayTransportOption? transport,
    String? sni,
    String? fingerprint,
    String? directAddr,
  }) {
    return ServerSettings(
      port: port ?? this.port,
      playerName: playerName ?? this.playerName,
      p2pServerAddr: p2pServerAddr ?? this.p2pServerAddr,
      p2pEnabled: p2pEnabled ?? this.p2pEnabled,
      p2pHostRoomCode: p2pHostRoomCode != null
          ? p2pHostRoomCode()
          : this.p2pHostRoomCode,
      transport: transport ?? this.transport,
      sni: sni ?? this.sni,
      fingerprint: fingerprint ?? this.fingerprint,
      directAddr: directAddr ?? this.directAddr,
    );
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
      final playerName =
          storage.get(StorageKeys.settingsNetplayPlayerName) as String?;
      if (playerName != null) {
        state = state.copyWith(playerName: playerName);
      }
      final p2pServerAddr =
          storage.get(StorageKeys.settingsNetplayP2PServerAddr) as String?;
      if (p2pServerAddr != null) {
        state = state.copyWith(p2pServerAddr: p2pServerAddr);
      } else {
        // Migration: Check old signaling/relay keys
        final oldSignaling =
            storage.get('settings.netplay.signaling_addr.v1') as String?;
        final oldRelay =
            storage.get('settings.netplay.relay_addr.v1') as String?;
        final migrationAddr = oldSignaling ?? oldRelay;
        if (migrationAddr != null) {
          state = state.copyWith(p2pServerAddr: migrationAddr);
          // Auto-persist new key
          unawaited(setP2PServerAddr(migrationAddr));
        }
      }

      final p2pEnabled =
          storage.get(StorageKeys.settingsNetplayP2PEnabled) as bool?;
      if (p2pEnabled != null) {
        state = state.copyWith(p2pEnabled: p2pEnabled);
      }

      final transportIndex =
          storage.get(StorageKeys.settingsNetplayTransport) as int?;
      if (transportIndex != null &&
          transportIndex < NetplayTransportOption.values.length) {
        state = state.copyWith(
          transport: NetplayTransportOption.values[transportIndex],
        );
      }

      final sni = storage.get(StorageKeys.settingsNetplaySni) as String?;
      if (sni != null) {
        state = state.copyWith(sni: sni);
      }

      final fingerprint =
          storage.get(StorageKeys.settingsNetplayFingerprint) as String?;
      if (fingerprint != null) {
        state = state.copyWith(fingerprint: fingerprint);
      }

      final directAddr =
          storage.get(StorageKeys.settingsNetplayDirectAddr) as String?;
      if (directAddr != null) {
        state = state.copyWith(directAddr: directAddr);
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

  Future<void> setPlayerName(String name) async {
    if (name == state.playerName) return;
    state = state.copyWith(playerName: name);
    try {
      await ref
          .read(appStorageProvider)
          .put(StorageKeys.settingsNetplayPlayerName, name);
    } catch (e, st) {
      logWarning(e, stackTrace: st, message: 'Failed to persist player name');
    }
  }

  Future<void> setP2PServerAddr(String addr) async {
    if (addr == state.p2pServerAddr) return;
    state = state.copyWith(p2pServerAddr: addr);
    try {
      await ref
          .read(appStorageProvider)
          .put(StorageKeys.settingsNetplayP2PServerAddr, addr);
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'Failed to persist p2p server addr',
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

  Future<void> setP2PEnabled(bool enabled) async {
    if (enabled == state.p2pEnabled) return;
    state = state.copyWith(
      p2pEnabled: enabled,
      p2pHostRoomCode: () => enabled ? state.p2pHostRoomCode : null,
    );
    try {
      await ref
          .read(appStorageProvider)
          .put(StorageKeys.settingsNetplayP2PEnabled, enabled);
    } catch (e, st) {
      logWarning(e, stackTrace: st, message: 'Failed to persist p2p enabled');
    }
  }

  Future<void> setTransport(NetplayTransportOption option) async {
    if (option == state.transport) return;
    state = state.copyWith(transport: option);
    try {
      await ref
          .read(appStorageProvider)
          .put(StorageKeys.settingsNetplayTransport, option.index);
    } catch (e, st) {
      logWarning(e, stackTrace: st, message: 'Failed to persist transport');
    }
  }

  Future<void> setSni(String sni) async {
    if (sni == state.sni) return;
    state = state.copyWith(sni: sni);
    try {
      await ref
          .read(appStorageProvider)
          .put(StorageKeys.settingsNetplaySni, sni);
    } catch (e, st) {
      logWarning(e, stackTrace: st, message: 'Failed to persist sni');
    }
  }

  Future<void> setFingerprint(String fingerprint) async {
    if (fingerprint == state.fingerprint) return;
    state = state.copyWith(fingerprint: fingerprint);
    try {
      await ref
          .read(appStorageProvider)
          .put(StorageKeys.settingsNetplayFingerprint, fingerprint);
    } catch (e, st) {
      logWarning(e, stackTrace: st, message: 'Failed to persist fingerprint');
    }
  }

  Future<void> setDirectAddr(String addr) async {
    if (addr == state.directAddr) return;
    state = state.copyWith(directAddr: addr);
    try {
      await ref
          .read(appStorageProvider)
          .put(StorageKeys.settingsNetplayDirectAddr, addr);
    } catch (e, st) {
      logWarning(e, stackTrace: st, message: 'Failed to persist direct addr');
    }
  }

  Future<void> stopServer() async {
    try {
      await netserverStop();
      state = state.copyWith(p2pHostRoomCode: () => null);
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
      state = state.copyWith(p2pHostRoomCode: () => roomCode);
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
