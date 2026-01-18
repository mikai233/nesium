import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../../bridge/api/netplay.dart';

final netplayStatusProvider = StreamProvider<NetplayStatus>((ref) {
  return netplayStatusStream();
});

class NetplayAppState {
  NetplayAppState({required this.status});
  final NetplayStatus status;

  bool get isInRoom => status.state == NetplayState.inRoom;
}

final netplayProvider = Provider<NetplayAppState>((ref) {
  final status =
      ref.watch(netplayStatusProvider).value ??
      const NetplayStatus(
        state: NetplayState.disconnected,
        transport: NetplayTransport.unknown,
        tcpFallbackFromQuic: false,
        clientId: 0,
        roomId: 0,
        playerIndex: 100, // spectator placeholder
        players: [],
        syncMode: SyncMode.lockstep,
      );
  return NetplayAppState(status: status);
});
