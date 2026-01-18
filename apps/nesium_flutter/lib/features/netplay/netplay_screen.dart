import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/widgets/animated_dropdown_menu.dart';

import '../../domain/nes_controller.dart';
import '../../l10n/app_localizations.dart';
import '../../bridge/api/netplay.dart';
import 'netplay_constants.dart';
import '../../persistence/app_storage.dart';
import '../../persistence/keys.dart';

enum _NetplayTransportOption { auto, tcp, quic }

class NetplayScreen extends ConsumerStatefulWidget {
  const NetplayScreen({super.key});

  @override
  ConsumerState<NetplayScreen> createState() => _NetplayScreenState();
}

class _NetplayScreenState extends ConsumerState<NetplayScreen> {
  final _serverAddrController = TextEditingController(text: '127.0.0.1:5233');
  final _pinnedFingerprintController = TextEditingController();
  final _playerNameController = TextEditingController(text: 'Player');
  final _roomCodeController = TextEditingController();
  final _p2pRoomCodeController = TextEditingController();
  final _p2pServerAddrController = TextEditingController(
    text: 'nesium.mikai.link:5233',
  );

  bool _p2pJoinEnabled = false;
  SyncMode _selectedSyncMode = SyncMode.lockstep;
  int _joinDesiredRole = spectatorPlayerIndex;

  Stream<NetplayStatus>? _statusStream;
  _NetplayTransportOption _transport = _NetplayTransportOption.auto;

  Timer? _roomQueryDebounce;
  int? _queriedRoomCode;
  int _queriedOccupiedMask = 0;

  @override
  void initState() {
    super.initState();
    _statusStream = netplayStatusStream();
    _loadJoinPrefs();
    _roomCodeController.addListener(_scheduleRoomQuery);
  }

  @override
  void dispose() {
    _serverAddrController.dispose();
    _pinnedFingerprintController.dispose();
    _playerNameController.dispose();
    _roomCodeController.dispose();
    _p2pRoomCodeController.dispose();
    _p2pServerAddrController.dispose();
    _roomQueryDebounce?.cancel();
    super.dispose();
  }

  bool _isSlotOccupied(int slot) {
    if (slot < 0 || slot > 3) return false;
    final code = int.tryParse(_roomCodeController.text.trim());
    if (code == null || code <= 0) return false;
    if (_queriedRoomCode != code) return false;
    return (_queriedOccupiedMask & (1 << slot)) != 0;
  }

  void _scheduleRoomQuery() {
    _roomQueryDebounce?.cancel();
    _roomQueryDebounce = Timer(const Duration(milliseconds: 220), () async {
      final code = int.tryParse(_roomCodeController.text.trim());
      if (!mounted) return;

      if (code == null || code <= 0) {
        setState(() {
          _queriedRoomCode = null;
          _queriedOccupiedMask = 0;
        });
        return;
      }

      // Avoid spamming the server on the same code.
      if (_queriedRoomCode == code) return;

      try {
        final info = await netplayQueryRoom(roomCode: code);
        if (!mounted) return;
        setState(() {
          _queriedRoomCode = code;
          _queriedOccupiedMask = info.ok ? info.occupiedMask : 0;
          if (_joinDesiredRole >= 0 &&
              _joinDesiredRole <= 3 &&
              _isSlotOccupied(_joinDesiredRole)) {
            _joinDesiredRole = spectatorPlayerIndex;
          }
        });
      } catch (_) {
        if (!mounted) return;
        setState(() {
          _queriedRoomCode = code;
          _queriedOccupiedMask = 0;
        });
      }
    });
  }

  Future<void> _loadJoinPrefs() async {
    try {
      final storage = ref.read(appStorageProvider);
      final enabled =
          storage.get(StorageKeys.settingsNetplayJoinP2PEnabled) as bool?;
      final addr =
          storage.get(StorageKeys.settingsNetplayJoinP2PServerAddr) as String?;
      if (!mounted) return;
      setState(() {
        _p2pJoinEnabled = enabled ?? false;
        if (addr != null && addr.trim().isNotEmpty) {
          _p2pServerAddrController.text = addr.trim();
        }
      });
    } catch (_) {}
  }

  Future<void> _persistJoinEnabled(bool enabled) async {
    try {
      await ref
          .read(appStorageProvider)
          .put(StorageKeys.settingsNetplayJoinP2PEnabled, enabled);
    } catch (_) {}
  }

  Future<void> _persistJoinServerAddr(String value) async {
    try {
      await ref
          .read(appStorageProvider)
          .put(StorageKeys.settingsNetplayJoinP2PServerAddr, value.trim());
    } catch (_) {}
  }

  String _deriveServerNameFromAddr(String serverAddr) {
    final trimmed = serverAddr.trim();
    if (trimmed.isEmpty) return 'localhost';

    try {
      final uri = trimmed.contains('://')
          ? Uri.parse(trimmed)
          : Uri.parse('dummy://$trimmed');
      final host = uri.host.trim();
      if (host.isNotEmpty) return host;
    } catch (_) {}

    return 'localhost';
  }

  InputDecoration _roundedInputDecoration({
    required String labelText,
    String? hintText,
    required Widget prefixIcon,
    Widget? suffixIcon,
  }) {
    final colorScheme = Theme.of(context).colorScheme;
    const radius = 12.0;
    final enabledBorder = OutlineInputBorder(
      borderRadius: BorderRadius.circular(radius),
      borderSide: BorderSide(
        color: colorScheme.outlineVariant.withValues(alpha: 0.7),
      ),
    );
    final focusedBorder = OutlineInputBorder(
      borderRadius: BorderRadius.circular(radius),
      borderSide: BorderSide(
        color: colorScheme.primary.withValues(alpha: 0.9),
        width: 1.2,
      ),
    );
    return InputDecoration(
      labelText: labelText,
      hintText: hintText,
      prefixIcon: prefixIcon,
      suffixIcon: suffixIcon,
      filled: true,
      fillColor: colorScheme.surface,
      isDense: true,
      contentPadding: const EdgeInsets.fromLTRB(14, 14, 12, 14),
      border: enabledBorder,
      enabledBorder: enabledBorder,
      focusedBorder: focusedBorder,
    );
  }

  Future<void> _connect() async {
    final l10n = AppLocalizations.of(context)!;
    try {
      final serverAddr = _serverAddrController.text.trim();
      final playerName = _playerNameController.text.trim();

      final serverName = _deriveServerNameFromAddr(serverAddr);

      final pinned = _pinnedFingerprintController.text.trim();
      final usePinned = pinned.isNotEmpty;

      switch (_transport) {
        case _NetplayTransportOption.tcp:
          await netplayConnect(serverAddr: serverAddr, playerName: playerName);
          break;
        case _NetplayTransportOption.quic:
          if (usePinned) {
            await netplayConnectQuicPinned(
              serverAddr: serverAddr,
              serverName: serverName,
              pinnedSha256Fingerprint: pinned,
              playerName: playerName,
            );
          } else {
            await netplayConnectQuic(
              serverAddr: serverAddr,
              serverName: serverName,
              playerName: playerName,
            );
          }
          break;
        case _NetplayTransportOption.auto:
          if (usePinned) {
            await netplayConnectAutoPinned(
              serverAddr: serverAddr,
              serverName: serverName,
              pinnedSha256Fingerprint: pinned,
              playerName: playerName,
            );
          } else {
            await netplayConnectAuto(
              serverAddr: serverAddr,
              serverName: serverName,
              playerName: playerName,
            );
          }
          break;
      }
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(l10n.netplayConnectFailed(e.toString()))),
        );
      }
    }
  }

  Future<void> _connectP2P() async {
    final l10n = AppLocalizations.of(context)!;
    final signalingAddr = _p2pServerAddrController.text.trim();
    if (signalingAddr.isEmpty) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(l10n.netplayInvalidP2PServerAddr)),
        );
      }
      return;
    }

    final code = int.tryParse(_p2pRoomCodeController.text.trim());
    if (code == null) {
      ScaffoldMessenger.of(
        context,
      ).showSnackBar(SnackBar(content: Text(l10n.netplayInvalidRoomCode)));
      return;
    }
    final playerNameRaw = _playerNameController.text.trim();
    final playerName = playerNameRaw.isEmpty ? 'Player' : playerNameRaw;

    try {
      final nesState = ref.read(nesControllerProvider);
      final hasRom = nesState.romBytes != null;
      await netplayP2PConnectJoinAuto(
        signalingAddr: signalingAddr,
        relayAddr: signalingAddr,
        roomCode: code,
        playerName: playerName,
        desiredRole: _joinDesiredRole,
        hasRom: hasRom,
      );
    } catch (e) {
      if (mounted) {
        ScaffoldMessenger.of(context).showSnackBar(
          SnackBar(content: Text(l10n.netplayConnectFailed(e.toString()))),
        );
      }
    }
  }

  Future<void> _pastePinnedFingerprint() async {
    final data = await Clipboard.getData(Clipboard.kTextPlain);
    final text = data?.text;
    if (text == null) return;

    setState(() {
      _pinnedFingerprintController.text = text.trim();
    });
  }

  String _transportName(AppLocalizations l10n, NetplayTransport transport) {
    switch (transport) {
      case NetplayTransport.unknown:
        return l10n.netplayTransportUnknown;
      case NetplayTransport.tcp:
        return l10n.netplayTransportTcp;
      case NetplayTransport.quic:
        return l10n.netplayTransportQuic;
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
      // Set sync mode preference before creating the room.
      await netplaySetSyncMode(mode: _selectedSyncMode);

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
      // Best-effort: refresh occupancy before joining to avoid "stale enabled slot" races.
      if (_joinDesiredRole >= 0 && _joinDesiredRole <= 3) {
        try {
          final info = await netplayQueryRoom(roomCode: code);
          if (info.ok && ((info.occupiedMask & (1 << _joinDesiredRole)) != 0)) {
            if (mounted) {
              setState(() => _joinDesiredRole = spectatorPlayerIndex);
            }
          }
        } catch (_) {}
      }

      final nesState = ref.read(nesControllerProvider);
      final hasRom = nesState.romBytes != null;
      await netplayJoinRoom(
        roomCode: code,
        desiredRole: _joinDesiredRole,
        hasRom: hasRom,
      );
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
              _StateTransitionSwitcher(
                state: state,
                child: _buildContent(l10n, state, status),
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

    return _buildCard(state, statusColor, statusText, statusIcon, l10n, status);
  }

  Widget _buildStatusBadge(String label, IconData icon) {
    final theme = Theme.of(context);
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 4),
      decoration: BoxDecoration(
        color: theme.colorScheme.surfaceContainerHighest,
        borderRadius: BorderRadius.circular(999),
        border: Border.all(color: theme.colorScheme.outlineVariant),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 14, color: theme.colorScheme.onSurfaceVariant),
          const SizedBox(width: 4),
          Text(
            label,
            style: theme.textTheme.labelMedium?.copyWith(
              fontWeight: FontWeight.w600,
              color: theme.colorScheme.onSurfaceVariant,
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildCard(
    NetplayState state,
    Color statusColor,
    String statusText,
    IconData statusIcon,
    AppLocalizations l10n,
    NetplayStatus? status,
  ) {
    final theme = Theme.of(context);
    return AnimatedContainer(
      duration: const Duration(milliseconds: 300),
      curve: Curves.easeInOut,
      decoration: BoxDecoration(
        color: statusColor.withAlpha(25),
        borderRadius: BorderRadius.circular(16),
        border: Border.all(color: statusColor.withAlpha(51)),
      ),
      child: Padding(
        padding: const EdgeInsets.all(16),
        child: Row(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Icon(statusIcon, color: statusColor),
            const SizedBox(width: 16),
            Expanded(
              child: AnimatedSize(
                duration: const Duration(milliseconds: 300),
                curve: Curves.easeInOut,
                alignment: Alignment.topLeft,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Flexible(
                          child: Text(
                            statusText,
                            style: theme.textTheme.titleMedium?.copyWith(
                              color: statusColor,
                              fontWeight: FontWeight.bold,
                            ),
                            overflow: TextOverflow.ellipsis,
                          ),
                        ),
                        if (status != null &&
                            state != NetplayState.disconnected &&
                            state != NetplayState.connecting) ...[
                          if (status.transport != NetplayTransport.unknown) ...[
                            const SizedBox(width: 8),
                            _buildStatusBadge(
                              _transportName(l10n, status.transport),
                              Icons.compare_arrows_rounded,
                            ),
                          ],
                          if (state == NetplayState.inRoom) ...[
                            const SizedBox(width: 8),
                            _buildStatusBadge(
                              status.syncMode == SyncMode.lockstep
                                  ? l10n.netplaySyncModeLockstep
                                  : l10n.netplaySyncModeRollback,
                              status.syncMode == SyncMode.lockstep
                                  ? Icons.lock_clock_rounded
                                  : Icons.history_rounded,
                            ),
                          ],
                        ],
                      ],
                    ),
                    if (status?.tcpFallbackFromQuic == true) ...[
                      const SizedBox(height: 4),
                      Text(
                        l10n.netplayUsingTcpFallback,
                        style: theme.textTheme.bodySmall?.copyWith(
                          color: theme.colorScheme.onSurfaceVariant,
                        ),
                      ),
                    ],
                    if (status?.error != null) ...[
                      const SizedBox(height: 4),
                      Text(
                        status!.error!,
                        style: theme.textTheme.bodySmall?.copyWith(
                          color: theme.colorScheme.error,
                        ),
                        overflow: TextOverflow.ellipsis,
                      ),
                    ],
                  ],
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }

  Widget _buildConnectForm(AppLocalizations l10n) {
    final theme = Theme.of(context);
    final directConnectDisabled = _p2pJoinEnabled;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        AbsorbPointer(
          absorbing: directConnectDisabled,
          child: Opacity(
            opacity: directConnectDisabled ? 0.5 : 1,
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.stretch,
              children: [
                AnimatedDropdownMenu<_NetplayTransportOption>(
                  labelText: l10n.netplayTransportLabel,
                  value: _transport,
                  entries: [
                    DropdownMenuEntry(
                      value: _NetplayTransportOption.auto,
                      label: l10n.netplayTransportAuto,
                    ),
                    DropdownMenuEntry(
                      value: _NetplayTransportOption.tcp,
                      label: l10n.netplayTransportTcp,
                    ),
                    DropdownMenuEntry(
                      value: _NetplayTransportOption.quic,
                      label: l10n.netplayTransportQuic,
                    ),
                  ],
                  enabled: !directConnectDisabled,
                  onSelected: (value) => setState(() => _transport = value),
                ),
                const SizedBox(height: 16),
                TextField(
                  controller: _serverAddrController,
                  enabled: !directConnectDisabled,
                  decoration: _roundedInputDecoration(
                    labelText: l10n.netplayServerAddress,
                    hintText: '127.0.0.1:5233',
                    prefixIcon: const Icon(Icons.dns_rounded),
                  ),
                ),
                if (_transport != _NetplayTransportOption.tcp) ...[
                  const SizedBox(height: 16),
                  TextField(
                    controller: _pinnedFingerprintController,
                    enabled: !directConnectDisabled,
                    decoration: _roundedInputDecoration(
                      labelText: l10n.netplayQuicFingerprintLabel,
                      hintText: l10n.netplayQuicFingerprintHint,
                      prefixIcon: const Icon(Icons.key_rounded),
                      suffixIcon: IconButton(
                        onPressed: _pastePinnedFingerprint,
                        tooltip: l10n.paste,
                        icon: const Icon(Icons.content_paste_rounded),
                      ),
                    ),
                  ),
                  const SizedBox(height: 8),
                  Text(
                    l10n.netplayQuicFingerprintHelper,
                    style: Theme.of(context).textTheme.bodySmall,
                  ),
                ],
              ],
            ),
          ),
        ),
        const SizedBox(height: 16),
        TextField(
          controller: _playerNameController,
          decoration: _roundedInputDecoration(
            labelText: l10n.netplayPlayerName,
            prefixIcon: const Icon(Icons.person_rounded),
          ),
        ),
        const SizedBox(height: 16),
        Card(
          elevation: 0,
          color: theme.colorScheme.surfaceContainerLow,
          shape: RoundedRectangleBorder(
            borderRadius: BorderRadius.circular(12),
            side: BorderSide(color: theme.colorScheme.outlineVariant),
          ),
          child: Column(
            children: [
              SwitchListTile(
                value: _p2pJoinEnabled,
                onChanged: (enabled) {
                  FocusManager.instance.primaryFocus?.unfocus();
                  setState(() => _p2pJoinEnabled = enabled);
                  unawaited(_persistJoinEnabled(enabled));
                },
                secondary: Icon(
                  _p2pJoinEnabled
                      ? Icons.wifi_tethering_rounded
                      : Icons.lan_rounded,
                  color: theme.colorScheme.primary,
                ),
                title: Text(l10n.netplayJoinViaP2P),
              ),
              AnimatedCrossFade(
                duration: const Duration(milliseconds: 250),
                sizeCurve: Curves.easeInOut,
                firstCurve: Curves.easeOut,
                secondCurve: Curves.easeIn,
                crossFadeState: _p2pJoinEnabled
                    ? CrossFadeState.showSecond
                    : CrossFadeState.showFirst,
                firstChild: const SizedBox(width: double.infinity, height: 0),
                secondChild: Column(
                  children: [
                    const Divider(height: 1),
                    Padding(
                      padding: const EdgeInsets.all(16),
                      child: Column(
                        children: [
                          TextField(
                            controller: _p2pServerAddrController,
                            decoration: _roundedInputDecoration(
                              labelText: l10n.netplayP2PServerLabel,
                              prefixIcon: const Icon(Icons.hub_rounded),
                            ),
                            onChanged: (value) =>
                                unawaited(_persistJoinServerAddr(value)),
                          ),
                          const SizedBox(height: 16),
                          TextField(
                            controller: _p2pRoomCodeController,
                            decoration: _roundedInputDecoration(
                              labelText: l10n.netplayP2PRoomCode,
                              prefixIcon: const Icon(Icons.numbers_rounded),
                            ),
                            keyboardType: TextInputType.number,
                          ),
                          const SizedBox(height: 16),
                          AnimatedDropdownMenu<int>(
                            labelText: l10n.netplayRoleLabel,
                            value: _joinDesiredRole,
                            entries: [
                              DropdownMenuEntry<int>(
                                value: spectatorPlayerIndex,
                                label: l10n.netplaySpectator,
                                leadingIcon: const Icon(
                                  Icons.visibility_rounded,
                                ),
                              ),
                              for (var idx = 0; idx < 4; idx++)
                                DropdownMenuEntry<int>(
                                  value: idx,
                                  label: l10n.netplayPlayerIndex(idx + 1),
                                  leadingIcon: const Icon(
                                    Icons.sports_esports_rounded,
                                  ),
                                ),
                            ],
                            onSelected: (value) {
                              setState(() => _joinDesiredRole = value);
                            },
                          ),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
        const SizedBox(height: 24),
        FilledButton.icon(
          onPressed: _p2pJoinEnabled ? _connectP2P : _connect,
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
        // Sync mode selector
        AnimatedDropdownMenu<SyncMode>(
          labelText: l10n.netplaySyncModeLabel,
          value: _selectedSyncMode,
          entries: [
            DropdownMenuEntry(
              value: SyncMode.lockstep,
              label: l10n.netplaySyncModeLockstep,
              leadingIcon: const Icon(Icons.lock_clock_rounded),
            ),
            DropdownMenuEntry(
              value: SyncMode.rollback,
              label: l10n.netplaySyncModeRollback,
              leadingIcon: const Icon(Icons.history_rounded),
            ),
          ],
          onSelected: (value) {
            setState(() => _selectedSyncMode = value);
          },
        ),
        const SizedBox(height: 16),
        FilledButton.icon(
          onPressed: () {
            _createRoom();
          },
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
          decoration: _roundedInputDecoration(
            labelText: l10n.netplayRoomCode,
            prefixIcon: const Icon(Icons.numbers_rounded),
          ),
          keyboardType: TextInputType.number,
        ),
        const SizedBox(height: 16),
        AnimatedDropdownMenu<int>(
          labelText: l10n.netplayRoleLabel,
          value: _joinDesiredRole,
          entries: [
            DropdownMenuEntry<int>(
              value: spectatorPlayerIndex,
              label: l10n.netplaySpectator,
              leadingIcon: const Icon(Icons.visibility_rounded),
            ),
            for (var idx = 0; idx < 4; idx++)
              DropdownMenuEntry<int>(
                value: idx,
                label: l10n.netplayPlayerIndex(idx + 1),
                leadingIcon: const Icon(Icons.sports_esports_rounded),
                enabled: !_isSlotOccupied(idx),
              ),
          ],
          onSelected: (value) {
            setState(() => _joinDesiredRole = value);
          },
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
    bool roleOccupiedByOther(int role) {
      if (role == spectatorPlayerIndex) return false;
      return status.players.any(
        (p) => p.playerIndex == role && p.clientId != status.clientId,
      );
    }

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
                DropdownMenuEntry(
                  value: 0,
                  label: l10n.netplayPlayerIndex(1),
                  enabled: status.playerIndex == 0 || !roleOccupiedByOther(0),
                ),
                DropdownMenuEntry(
                  value: 1,
                  label: l10n.netplayPlayerIndex(2),
                  enabled: status.playerIndex == 1 || !roleOccupiedByOther(1),
                ),
                DropdownMenuEntry(
                  value: 2,
                  label: l10n.netplayPlayerIndex(3),
                  enabled: status.playerIndex == 2 || !roleOccupiedByOther(2),
                ),
                DropdownMenuEntry(
                  value: 3,
                  label: l10n.netplayPlayerIndex(4),
                  enabled: status.playerIndex == 3 || !roleOccupiedByOther(3),
                ),
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
            const Padding(
              padding: EdgeInsets.symmetric(vertical: 12),
              child: Divider(),
            ),
            _buildInfoRow(
              l10n.netplayTransportLabel,
              _transportName(l10n, status.transport),
              icon: Icons.swap_horiz_rounded,
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

/// Custom animated switcher that performs sequential transitions:
/// 1. Fade out old content
/// 2. Fade in new content with slide-up effect
class _StateTransitionSwitcher extends StatefulWidget {
  final NetplayState state;
  final Widget child;

  const _StateTransitionSwitcher({required this.state, required this.child});

  @override
  State<_StateTransitionSwitcher> createState() =>
      _StateTransitionSwitcherState();
}

class _StateTransitionSwitcherState extends State<_StateTransitionSwitcher>
    with SingleTickerProviderStateMixin {
  late AnimationController _controller;
  late Animation<double> _fadeOutAnimation;
  late Animation<double> _fadeInAnimation;
  late Animation<Offset> _slideAnimation;

  Widget? _oldChild;
  Widget? _currentChild;
  NetplayState? _previousState;
  bool _isAnimating = false;

  @override
  void initState() {
    super.initState();
    _controller = AnimationController(
      duration: const Duration(milliseconds: 600),
      vsync: this,
    );

    // Fade out happens in first half (0.0 - 0.4)
    _fadeOutAnimation = Tween<double>(begin: 1.0, end: 0.0).animate(
      CurvedAnimation(
        parent: _controller,
        curve: const Interval(0.0, 0.4, curve: Curves.easeIn),
      ),
    );

    // Fade in happens in second half (0.4 - 1.0)
    _fadeInAnimation = Tween<double>(begin: 0.0, end: 1.0).animate(
      CurvedAnimation(
        parent: _controller,
        curve: const Interval(0.4, 1.0, curve: Curves.easeOut),
      ),
    );

    // Slide up happens during fade in (0.4 - 1.0)
    _slideAnimation =
        Tween<Offset>(begin: const Offset(0, 0.08), end: Offset.zero).animate(
          CurvedAnimation(
            parent: _controller,
            curve: const Interval(0.4, 1.0, curve: Curves.easeOutCubic),
          ),
        );

    _currentChild = widget.child;
    _previousState = widget.state;

    _controller.addStatusListener((status) {
      if (status == AnimationStatus.completed) {
        setState(() {
          _isAnimating = false;
          _oldChild = null;
        });
      }
    });
  }

  @override
  void didUpdateWidget(_StateTransitionSwitcher oldWidget) {
    super.didUpdateWidget(oldWidget);

    // Check if state actually changed
    if (widget.state != _previousState) {
      _previousState = widget.state;
      _startTransition();
    } else {
      // State unchanged, just update the current child
      _currentChild = widget.child;
    }
  }

  void _startTransition() {
    if (_isAnimating) {
      // Already animating, just capture the new target
      _controller.reset();
    }

    setState(() {
      _oldChild = _currentChild;
      _currentChild = widget.child;
      _isAnimating = true;
    });

    _controller.forward(from: 0.0);
  }

  @override
  void dispose() {
    _controller.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    if (!_isAnimating) {
      // Not animating, show current child directly
      return _currentChild ?? const SizedBox.shrink();
    }

    // During animation: show either old (fading out) or new (fading in)
    return AnimatedBuilder(
      animation: _controller,
      builder: (context, _) {
        // First half: show old child fading out
        if (_controller.value < 0.4 && _oldChild != null) {
          return Opacity(opacity: _fadeOutAnimation.value, child: _oldChild);
        }

        // Second half: show new child fading in with slide
        return SlideTransition(
          position: _slideAnimation,
          child: Opacity(opacity: _fadeInAnimation.value, child: _currentChild),
        );
      },
    );
  }
}
