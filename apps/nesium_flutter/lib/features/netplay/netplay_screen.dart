import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../domain/nes_controller.dart';
import '../../l10n/app_localizations.dart';
import '../../bridge/api/netplay.dart';
import 'netplay_constants.dart';

class NetplayScreen extends ConsumerStatefulWidget {
  const NetplayScreen({super.key});

  @override
  ConsumerState<NetplayScreen> createState() => _NetplayScreenState();
}

class _NetplayScreenState extends ConsumerState<NetplayScreen> {
  final _serverAddrController = TextEditingController(text: '127.0.0.1:5233');
  final _playerNameController = TextEditingController(text: 'Player');
  final _roomCodeController = TextEditingController();

  Stream<NetplayStatus>? _statusStream;

  @override
  void initState() {
    super.initState();
    _statusStream = netplayStatusStream();
  }

  @override
  void dispose() {
    _serverAddrController.dispose();
    _playerNameController.dispose();
    _roomCodeController.dispose();
    super.dispose();
  }

  Future<void> _connect() async {
    try {
      await netplayConnect(
        serverAddr: _serverAddrController.text,
        playerName: _playerNameController.text,
      );
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text('Connect failed: $e')));
      }
    }
  }

  Future<void> _disconnect() async {
    try {
      await netplayDisconnect();
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text('Disconnect failed: $e')));
      }
    }
  }

  Future<void> _createRoom() async {
    try {
      await netplayCreateRoom();

      // If we have a ROM loaded, send it to the server so late joiners can sync
      final nesState = ref.read(nesControllerProvider);
      if (nesState.romBytes != null) {
        await netplaySendRom(data: nesState.romBytes!);
        await netplaySendRomLoaded();

        if (mounted) {
          ScaffoldMessenger.of(context).showSnackBar(
            const SnackBar(content: Text('Netplay: ROM broadcasted to room')),
          );
        }
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text('Create room failed: $e')));
      }
    }
  }

  Future<void> _joinRoom() async {
    final code = int.tryParse(_roomCodeController.text);
    if (code == null) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(const SnackBar(content: Text('Invalid room code')));
      return;
    }
    try {
      await netplayJoinRoom(roomCode: code);
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text('Join room failed: $e')));
      }
    }
  }

  Future<void> _switchRole(int role) async {
    try {
      await netplaySwitchRole(role: role);
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(
          context,
        ).showSnackBar(SnackBar(content: Text('Switch role failed: $e')));
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    return StreamBuilder<NetplayStatus>(
      stream: _statusStream,
      builder: (context, snapshot) {
        final status = snapshot.data;
        final state = status?.state ?? NetplayState.disconnected;

        return SingleChildScrollView(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              _buildStatusCard(l10n, status),
              const SizedBox(height: 16),
              if (state == NetplayState.disconnected)
                _buildConnectForm(l10n)
              else if (state == NetplayState.connected)
                _buildRoomForm(l10n)
              else if (state == NetplayState.inRoom)
                _buildInRoomInfo(l10n, status!)
              else if (state == NetplayState.connecting)
                const Center(child: CircularProgressIndicator()),
              if (state != NetplayState.disconnected) ...[
                const SizedBox(height: 16),
                ElevatedButton(
                  onPressed: _disconnect,
                  style: ElevatedButton.styleFrom(
                    backgroundColor: Colors.red.shade100,
                    foregroundColor: Colors.red.shade900,
                  ),
                  child: Text(l10n.netplayDisconnect),
                ),
              ],
            ],
          ),
        );
      },
    );
  }

  Widget _buildStatusCard(AppLocalizations l10n, NetplayStatus? status) {
    final state = status?.state ?? NetplayState.disconnected;
    Color statusColor;
    String statusText;

    switch (state) {
      case NetplayState.disconnected:
        statusColor = Colors.grey;
        statusText = l10n.netplayStatusDisconnected;
        break;
      case NetplayState.connecting:
        statusColor = Colors.orange;
        statusText = l10n.netplayStatusConnecting;
        break;
      case NetplayState.connected:
        statusColor = Colors.blue;
        statusText = l10n.netplayStatusConnected;
        break;
      case NetplayState.inRoom:
        statusColor = Colors.green;
        statusText = l10n.netplayStatusInRoom;
        break;
    }

    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          children: [
            Container(
              width: 12,
              height: 12,
              decoration: BoxDecoration(
                color: statusColor,
                shape: BoxShape.circle,
              ),
            ),
            const SizedBox(width: 12),
            Text(
              statusText,
              style: const TextStyle(fontWeight: FontWeight.bold),
            ),
            if (status?.error != null) ...[
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  status!.error!,
                  style: const TextStyle(color: Colors.red, fontSize: 12),
                  overflow: TextOverflow.ellipsis,
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }

  Widget _buildConnectForm(AppLocalizations l10n) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        TextField(
          controller: _serverAddrController,
          decoration: InputDecoration(
            labelText: l10n.netplayServerAddress,
            hintText: '127.0.0.1:5233',
          ),
        ),
        const SizedBox(height: 8),
        TextField(
          controller: _playerNameController,
          decoration: InputDecoration(labelText: l10n.netplayPlayerName),
        ),
        const SizedBox(height: 16),
        ElevatedButton(onPressed: _connect, child: Text(l10n.netplayConnect)),
      ],
    );
  }

  Widget _buildRoomForm(AppLocalizations l10n) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        ElevatedButton(
          onPressed: _createRoom,
          child: Text(l10n.netplayCreateRoom),
        ),
        const SizedBox(height: 16),
        const Divider(),
        const SizedBox(height: 16),
        TextField(
          controller: _roomCodeController,
          decoration: InputDecoration(
            labelText: l10n.netplayRoomCode,
            hintText: '123456',
          ),
          keyboardType: TextInputType.number,
        ),
        const SizedBox(height: 8),
        ElevatedButton(onPressed: _joinRoom, child: Text(l10n.netplayJoinRoom)),
      ],
    );
  }

  Widget _buildInRoomInfo(AppLocalizations l10n, NetplayStatus status) {
    return Card(
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Column(
          children: [
            _buildInfoRow(l10n.netplayRoomCode, status.roomId.toString()),
            const Divider(),
            Row(
              mainAxisAlignment: MainAxisAlignment.spaceBetween,
              children: [
                const Text('Role', style: TextStyle(color: Colors.grey)),
                DropdownButton<int>(
                  value: status.playerIndex,
                  underline: const SizedBox(),
                  items: const [
                    DropdownMenuItem(value: 0, child: Text('Player 1')),
                    DropdownMenuItem(value: 1, child: Text('Player 2')),
                    DropdownMenuItem(value: 2, child: Text('Player 3')),
                    DropdownMenuItem(value: 3, child: Text('Player 4')),
                    DropdownMenuItem(
                      value: spectatorPlayerIndex,
                      child: Text('Spectator'),
                    ),
                  ],
                  onChanged: (value) {
                    if (value != null && value != status.playerIndex) {
                      _switchRole(value);
                    }
                  },
                ),
              ],
            ),
            const Divider(),
            _buildInfoRow('Client ID', status.clientId.toString()),
          ],
        ),
      ),
    );
  }

  Widget _buildInfoRow(String label, String value) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(label, style: const TextStyle(color: Colors.grey)),
          Text(value, style: const TextStyle(fontWeight: FontWeight.bold)),
        ],
      ),
    );
  }
}
