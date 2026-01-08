import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:animations/animations.dart';
import 'package:nesium_flutter/widgets/animated_dropdown_menu.dart';

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
    final l10n = AppLocalizations.of(context)!;
    try {
      await netplayConnect(
        serverAddr: _serverAddrController.text,
        playerName: _playerNameController.text,
      );
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(l10n.netplayConnectFailed(e.toString()))),
        );
      }
    }
  }

  Future<void> _disconnect() async {
    final l10n = AppLocalizations.of(context)!;
    try {
      await netplayDisconnect();
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(l10n.netplayDisconnectFailed(e.toString()))),
        );
      }
    }
  }

  Future<void> _createRoom() async {
    final l10n = AppLocalizations.of(context)!;
    try {
      await netplayCreateRoom();

      // If we have a ROM loaded, send it to the server so late joiners can sync
      final nesState = ref.read(nesControllerProvider);
      if (nesState.romBytes != null) {
        await netplaySendRom(data: nesState.romBytes!);
        await netplaySendRomLoaded();

        if (mounted) {
          ScaffoldMessenger.of(
            context,
          ).showSnackBar(SnackBar(content: Text(l10n.netplayRomBroadcasted)));
        }
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(l10n.netplayCreateRoomFailed(e.toString()))),
        );
      }
    }
  }

  Future<void> _joinRoom() async {
    final l10n = AppLocalizations.of(context)!;
    final code = int.tryParse(_roomCodeController.text);
    if (code == null) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text(l10n.netplayInvalidRoomCode)));
      return;
    }
    try {
      await netplayJoinRoom(roomCode: code);
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(l10n.netplayJoinRoomFailed(e.toString()))),
        );
      }
    }
  }

  Future<void> _switchRole(int role) async {
    final l10n = AppLocalizations.of(context)!;
    try {
      await netplaySwitchRole(role: role);
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(l10n.netplaySwitchRoleFailed(e.toString()))),
        );
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
          padding: const EdgeInsets.all(24),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              _buildStatusCard(l10n, status),
              const SizedBox(height: 24),
              AnimatedSize(
                duration: const Duration(milliseconds: 300),
                curve: Curves.easeInOut,
                child: PageTransitionSwitcher(
                  duration: const Duration(milliseconds: 300),
                  transitionBuilder:
                      (
                        Widget child,
                        Animation<double> animation,
                        Animation<double> secondaryAnimation,
                      ) {
                        return FadeThroughTransition(
                          animation: animation,
                          secondaryAnimation: secondaryAnimation,
                          fillColor: Colors.transparent,
                          child: child,
                        );
                      },
                  child: _buildContent(l10n, state, status),
                ),
              ),
            ],
          ),
        );
      },
    );
  }

  Widget _buildContent(
    AppLocalizations l10n,
    NetplayState state,
    NetplayStatus? status,
  ) {
    switch (state) {
      case NetplayState.disconnected:
        return KeyedSubtree(
          key: const ValueKey('disconnected'),
          child: _buildConnectForm(l10n),
        );
      case NetplayState.connected:
        return KeyedSubtree(
          key: const ValueKey('connected'),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              _buildRoomForm(l10n),
              const SizedBox(height: 24),
              _buildDisconnectButton(l10n),
            ],
          ),
        );
      case NetplayState.inRoom:
        return KeyedSubtree(
          key: const ValueKey('inRoom'),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              _buildInRoomInfo(l10n, status!),
              const SizedBox(height: 24),
              _buildDisconnectButton(l10n),
            ],
          ),
        );
      case NetplayState.connecting:
        return KeyedSubtree(
          key: const ValueKey('connecting'),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              const SizedBox(height: 48),
              const Center(child: CircularProgressIndicator()),
              const SizedBox(height: 48),
              _buildDisconnectButton(l10n),
            ],
          ),
        );
    }
  }

  Widget _buildDisconnectButton(AppLocalizations l10n) {
    return FilledButton.tonal(
      onPressed: _disconnect,
      style: FilledButton.styleFrom(
        foregroundColor: Theme.of(context).colorScheme.error,
      ),
      child: Text(l10n.netplayDisconnect),
    );
  }

  Widget _buildStatusCard(AppLocalizations l10n, NetplayStatus? status) {
    final state = status?.state ?? NetplayState.disconnected;
    final theme = Theme.of(context);
    final colorScheme = theme.colorScheme;

    Color statusColor;
    String statusText;
    IconData statusIcon;

    switch (state) {
      case NetplayState.disconnected:
        statusColor = colorScheme.outline;
        statusText = l10n.netplayStatusDisconnected;
        statusIcon = Icons.link_off_rounded;
        break;
      case NetplayState.connecting:
        statusColor = colorScheme.tertiary;
        statusText = l10n.netplayStatusConnecting;
        statusIcon = Icons.sync_rounded;
        break;
      case NetplayState.connected:
        statusColor = colorScheme.primary;
        statusText = l10n.netplayStatusConnected;
        statusIcon = Icons.hub_rounded;
        break;
      case NetplayState.inRoom:
        statusColor = colorScheme.primary;
        statusText = l10n.netplayStatusInRoom;
        statusIcon = Icons.videogame_asset_rounded;
        break;
    }

    return Card(
      elevation: 0,
      color: statusColor.withAlpha(25),
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(16),
        side: BorderSide(color: statusColor.withAlpha(51)),
      ),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          children: [
            Icon(statusIcon, color: statusColor),
            const SizedBox(width: 16),
            Text(
              statusText,
              style: theme.textTheme.titleMedium?.copyWith(
                color: statusColor,
                fontWeight: FontWeight.bold,
              ),
            ),
            if (status?.error != null) ...[
              const SizedBox(width: 12),
              Expanded(
                child: Text(
                  status!.error!,
                  style: theme.textTheme.bodySmall?.copyWith(
                    color: colorScheme.error,
                  ),
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
            prefixIcon: const Icon(Icons.dns_rounded),
            border: const OutlineInputBorder(),
          ),
        ),
        const SizedBox(height: 16),
        TextField(
          controller: _playerNameController,
          decoration: InputDecoration(
            labelText: l10n.netplayPlayerName,
            prefixIcon: const Icon(Icons.person_rounded),
            border: const OutlineInputBorder(),
          ),
        ),
        const SizedBox(height: 24),
        FilledButton.icon(
          onPressed: _connect,
          icon: const Icon(Icons.login_rounded),
          label: Text(l10n.netplayConnect),
        ),
      ],
    );
  }

  Widget _buildRoomForm(AppLocalizations l10n) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        FilledButton.icon(
          onPressed: _createRoom,
          icon: const Icon(Icons.add_circle_outline_rounded),
          label: Text(l10n.netplayCreateRoom),
        ),
        const SizedBox(height: 24),
        Row(
          children: [
            const Expanded(child: Divider()),
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Text(
                l10n.netplayOrSeparator,
                style: Theme.of(context).textTheme.labelLarge?.copyWith(
                  color: Theme.of(context).colorScheme.outline,
                ),
              ),
            ),
            const Expanded(child: Divider()),
          ],
        ),
        const SizedBox(height: 24),
        TextField(
          controller: _roomCodeController,
          decoration: InputDecoration(
            labelText: l10n.netplayRoomCode,
            prefixIcon: const Icon(Icons.numbers_rounded),
            border: const OutlineInputBorder(),
          ),
          keyboardType: TextInputType.number,
        ),
        const SizedBox(height: 16),
        FilledButton.tonalIcon(
          onPressed: _joinRoom,
          icon: const Icon(Icons.meeting_room_rounded),
          label: Text(l10n.netplayJoinRoom),
        ),
      ],
    );
  }

  Widget _buildInRoomInfo(AppLocalizations l10n, NetplayStatus status) {
    return Card(
      elevation: 0,
      color: Theme.of(context).colorScheme.surfaceContainerLow,
      shape: RoundedRectangleBorder(
        borderRadius: BorderRadius.circular(16),
        side: BorderSide(color: Theme.of(context).colorScheme.outlineVariant),
      ),
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            _buildInfoRow(
              l10n.netplayRoomCode,
              status.roomId.toString(),
              icon: Icons.tag_rounded,
            ),
            const Padding(
              padding: EdgeInsets.symmetric(vertical: 12),
              child: Divider(),
            ),
            Text(
              l10n.netplayPlayerListHeader,
              style: Theme.of(context).textTheme.labelLarge?.copyWith(
                color: Theme.of(context).colorScheme.primary,
                fontWeight: FontWeight.bold,
              ),
            ),
            const SizedBox(height: 8),
            ...status.players.map(
              (p) => _buildPlayerRow(l10n, p, localClientId: status.clientId),
            ),
            const Padding(
              padding: EdgeInsets.symmetric(vertical: 12),
              child: Divider(),
            ),
            AnimatedDropdownMenu<int>(
              labelText: l10n.netplayRoleLabel,
              value: status.playerIndex,
              entries: [
                DropdownMenuEntry(value: 0, label: l10n.netplayPlayerIndex(1)),
                DropdownMenuEntry(value: 1, label: l10n.netplayPlayerIndex(2)),
                DropdownMenuEntry(value: 2, label: l10n.netplayPlayerIndex(3)),
                DropdownMenuEntry(value: 3, label: l10n.netplayPlayerIndex(4)),
                DropdownMenuEntry(
                  value: spectatorPlayerIndex,
                  label: l10n.netplaySpectator,
                ),
              ],
              onSelected: (value) => _switchRole(value),
            ),
            const Padding(
              padding: EdgeInsets.symmetric(vertical: 12),
              child: Divider(),
            ),
            _buildInfoRow(
              l10n.netplayClientId,
              status.clientId.toString(),
              icon: Icons.fingerprint_rounded,
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildPlayerRow(
    AppLocalizations l10n,
    NetplayPlayer player, {
    required int localClientId,
  }) {
    final theme = Theme.of(context);
    final isSpectator = player.playerIndex == spectatorPlayerIndex;
    final isSelf = player.clientId == localClientId;

    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 6),
      child: Row(
        children: [
          Icon(
            isSpectator
                ? Icons.visibility_rounded
                : Icons.videogame_asset_rounded,
            size: 18,
            color: isSelf
                ? theme.colorScheme.primary
                : theme.colorScheme.onSurfaceVariant,
          ),
          const SizedBox(width: 12),
          Expanded(
            child: RichText(
              text: TextSpan(
                style: theme.textTheme.bodyLarge,
                children: [
                  TextSpan(
                    text: player.name,
                    style: isSelf
                        ? const TextStyle(fontWeight: FontWeight.bold)
                        : null,
                  ),
                  if (isSelf)
                    TextSpan(
                      text: ' ${l10n.netplayYouIndicator}',
                      style: theme.textTheme.labelMedium?.copyWith(
                        color: theme.colorScheme.primary,
                        fontWeight: FontWeight.bold,
                      ),
                    ),
                ],
              ),
              overflow: TextOverflow.ellipsis,
            ),
          ),
          Text(
            isSpectator
                ? l10n.netplaySpectator
                : l10n.netplayPlayerIndex(player.playerIndex + 1),
            style: theme.textTheme.labelMedium?.copyWith(
              color: isSelf
                  ? theme.colorScheme.primary
                  : theme.colorScheme.onSurfaceVariant,
              fontWeight: isSelf ? FontWeight.bold : null,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildInfoRow(String label, String value, {IconData? icon}) {
    final theme = Theme.of(context);
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 4),
      child: Row(
        children: [
          if (icon != null) ...[
            Icon(icon, size: 20, color: theme.colorScheme.onSurfaceVariant),
            const SizedBox(width: 12),
          ],
          Text(
            label,
            style: theme.textTheme.bodyMedium?.copyWith(
              color: theme.colorScheme.onSurfaceVariant,
            ),
          ),
          const Spacer(),
          Text(
            value,
            style: theme.textTheme.titleMedium?.copyWith(
              fontWeight: FontWeight.bold,
              fontFamily: 'RobotoMono',
            ),
          ),
        ],
      ),
    );
  }
}
