import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../l10n/app_localizations.dart';
import '../../../widgets/animated_settings_widgets.dart';
import '../server_settings.dart';

class ServerTab extends ConsumerStatefulWidget {
  const ServerTab({super.key});

  @override
  ConsumerState<ServerTab> createState() => _ServerTabState();
}

class _ServerTabState extends ConsumerState<ServerTab> {
  late final TextEditingController _playerNameController;
  late final TextEditingController _portController;
  late final TextEditingController _p2pServerAddrController;

  final FocusNode _playerNameFocus = FocusNode();
  final FocusNode _portFocus = FocusNode();
  final FocusNode _p2pServerAddrFocus = FocusNode();

  ProviderSubscription<ServerSettings>? _settingsSub;

  @override
  void initState() {
    super.initState();
    final settings = ref.read(serverSettingsProvider);
    _playerNameController = TextEditingController(text: settings.playerName);
    _portController = TextEditingController(text: settings.port.toString());
    _p2pServerAddrController = TextEditingController(
      text: settings.p2pServerAddr,
    );

    _settingsSub = ref.listenManual(serverSettingsProvider, (prev, next) {
      _syncControllerIfUnfocused(
        controller: _playerNameController,
        focusNode: _playerNameFocus,
        nextText: next.playerName,
      );
      _syncControllerIfUnfocused(
        controller: _portController,
        focusNode: _portFocus,
        nextText: next.port.toString(),
      );
      _syncControllerIfUnfocused(
        controller: _p2pServerAddrController,
        focusNode: _p2pServerAddrFocus,
        nextText: next.p2pServerAddr,
      );
    });
  }

  void _syncControllerIfUnfocused({
    required TextEditingController controller,
    required FocusNode focusNode,
    required String nextText,
  }) {
    if (focusNode.hasFocus) return;
    if (controller.text == nextText) return;
    controller.value = controller.value.copyWith(
      text: nextText,
      selection: TextSelection.collapsed(offset: nextText.length),
      composing: TextRange.empty,
    );
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final settings = ref.watch(serverSettingsProvider);
    final controller = ref.read(serverSettingsProvider.notifier);
    final statusAsync = ref.watch(serverStatusStreamProvider);
    final theme = Theme.of(context);

    final status = statusAsync.value;
    final isRunning = status?.running ?? false;

    return ListView(
      padding: const EdgeInsets.all(20),
      children: [
        AnimatedSectionHeader(
          title: l10n.serverTitle,
          icon: Icons.dns_rounded,
          delay: const Duration(milliseconds: 50),
        ),
        const SizedBox(height: 8),
        // Server status card
        AnimatedSettingsCard(
          index: 0,
          child: AnimatedContainer(
            duration: const Duration(milliseconds: 300),
            curve: Curves.easeInOut,
            decoration: BoxDecoration(
              color: isRunning
                  ? theme.colorScheme.primary.withAlpha(25)
                  : theme.colorScheme.outline.withAlpha(15),
              borderRadius: BorderRadius.circular(16),
              border: Border.all(
                color: isRunning
                    ? theme.colorScheme.primary.withAlpha(51)
                    : theme.colorScheme.outline.withAlpha(35),
                width: 1.5,
              ),
            ),
            child: Padding(
              padding: const EdgeInsets.all(16),
              child: Row(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  AnimatedSwitcher(
                    duration: const Duration(milliseconds: 300),
                    child: Icon(
                      isRunning
                          ? Icons.check_circle_rounded
                          : Icons.cancel_rounded,
                      key: ValueKey(isRunning),
                      color: isRunning
                          ? theme.colorScheme.primary
                          : theme.colorScheme.outline,
                    ),
                  ),
                  const SizedBox(width: 16),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        AnimatedSwitcher(
                          duration: const Duration(milliseconds: 300),
                          layoutBuilder: (currentChild, previousChildren) {
                            return Stack(
                              alignment: Alignment.centerLeft,
                              children: [...previousChildren, ?currentChild],
                            );
                          },
                          child: Text(
                            isRunning
                                ? l10n.serverStatusRunning
                                : l10n.serverStatusStopped,
                            key: ValueKey(isRunning),
                            style: theme.textTheme.titleMedium?.copyWith(
                              color: isRunning
                                  ? theme.colorScheme.primary
                                  : theme.colorScheme.outline,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                        ),
                        // Wrap content in AnimatedSize to smooth out height changes
                        AnimatedSize(
                          duration: const Duration(milliseconds: 300),
                          curve: Curves.easeInOut,
                          alignment: Alignment.topLeft,
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              if (isRunning && status != null) ...[
                                const SizedBox(height: 8),
                                Text(
                                  l10n.serverBindAddress(status.bindAddress),
                                  style: theme.textTheme.bodySmall?.copyWith(
                                    color: theme.colorScheme.onSurfaceVariant,
                                  ),
                                ),
                                const SizedBox(height: 4),
                                Text(
                                  l10n.serverClientCount(status.clientCount),
                                  style: theme.textTheme.bodySmall?.copyWith(
                                    color: theme.colorScheme.onSurfaceVariant,
                                  ),
                                ),
                                if (status.quicEnabled &&
                                    status.quicCertSha256Fingerprint
                                        .trim()
                                        .isNotEmpty) ...[
                                  const SizedBox(height: 4),
                                  Row(
                                    children: [
                                      Expanded(
                                        child: Text(
                                          l10n.serverQuicFingerprint(
                                            status.quicCertSha256Fingerprint,
                                          ),
                                          style: theme.textTheme.bodySmall
                                              ?.copyWith(
                                                color: theme
                                                    .colorScheme
                                                    .onSurfaceVariant,
                                                fontFamily: 'RobotoMono',
                                              ),
                                        ),
                                      ),
                                      IconButton(
                                        onPressed: () async {
                                          await Clipboard.setData(
                                            ClipboardData(
                                              text: status
                                                  .quicCertSha256Fingerprint,
                                            ),
                                          );
                                          if (context.mounted) {
                                            ScaffoldMessenger.of(
                                              context,
                                            ).showSnackBar(
                                              SnackBar(
                                                content: Text(
                                                  l10n.lastErrorCopied,
                                                ),
                                              ),
                                            );
                                          }
                                        },
                                        icon: const Icon(
                                          Icons.content_copy_rounded,
                                        ),
                                        tooltip: l10n.copy,
                                      ),
                                    ],
                                  ),
                                ],
                                if (settings.p2pEnabled &&
                                    settings.p2pHostRoomCode != null) ...[
                                  const SizedBox(height: 4),
                                  Row(
                                    children: [
                                      Expanded(
                                        child: Text(
                                          '${l10n.netplayP2PRoomCode}: ${settings.p2pHostRoomCode}',
                                          style: theme.textTheme.bodySmall
                                              ?.copyWith(
                                                color: theme
                                                    .colorScheme
                                                    .onSurfaceVariant,
                                                fontFamily: 'RobotoMono',
                                              ),
                                        ),
                                      ),
                                      IconButton(
                                        onPressed: () async {
                                          await Clipboard.setData(
                                            ClipboardData(
                                              text:
                                                  '${settings.p2pHostRoomCode}',
                                            ),
                                          );
                                          if (context.mounted) {
                                            ScaffoldMessenger.of(
                                              context,
                                            ).showSnackBar(
                                              SnackBar(
                                                content: Text(
                                                  l10n.lastErrorCopied,
                                                ),
                                              ),
                                            );
                                          }
                                        },
                                        icon: const Icon(
                                          Icons.content_copy_rounded,
                                        ),
                                        tooltip: l10n.copy,
                                      ),
                                    ],
                                  ),
                                ],
                              ],
                            ],
                          ),
                        ),
                      ],
                    ),
                  ),
                ],
              ),
            ),
          ),
        ),
        const SizedBox(height: 16),
        // Player Name & Port configuration
        AnimatedSettingsCard(
          index: 1,
          child: Column(
            children: [
              Padding(
                padding: const EdgeInsets.all(16),
                child: TextField(
                  controller: _playerNameController,
                  focusNode: _playerNameFocus,
                  decoration: InputDecoration(
                    labelText: l10n.netplayPlayerName,
                    prefixIcon: const Icon(Icons.person_rounded),
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(12),
                    ),
                    filled: true,
                    fillColor: theme.colorScheme.surfaceContainerHighest
                        .withAlpha(50),
                  ),
                  onChanged: controller.setPlayerName,
                ),
              ),
              const Divider(height: 1),
              Padding(
                padding: const EdgeInsets.all(16),
                child: TextField(
                  controller: _portController,
                  focusNode: _portFocus,
                  decoration: InputDecoration(
                    labelText: l10n.serverPortLabel,
                    hintText: '5233',
                    prefixIcon: const Icon(Icons.numbers_rounded),
                    border: OutlineInputBorder(
                      borderRadius: BorderRadius.circular(12),
                    ),
                    filled: true,
                    fillColor: theme.colorScheme.surfaceContainerHighest
                        .withAlpha(50),
                  ),
                  keyboardType: TextInputType.number,
                  enabled: !isRunning,
                  onChanged: (value) {
                    final port = int.tryParse(value);
                    if (port != null && port > 0 && port <= 65535) {
                      controller.setPort(port);
                    }
                  },
                ),
              ),
            ],
          ),
        ),
        const SizedBox(height: 16),
        // P2P Mode Configuration
        AnimatedSettingsCard(
          index: 2,
          child: Column(
            children: [
              SwitchListTile(
                value: settings.p2pEnabled,
                onChanged: isRunning ? null : controller.setP2PEnabled,
                secondary: Icon(
                  settings.p2pEnabled
                      ? Icons.wifi_tethering_rounded
                      : Icons.lan_rounded,
                  color: theme.colorScheme.primary,
                ),
                title: Text(l10n.netplayP2PEnabled),
              ),
              AnimatedCrossFade(
                firstChild: const SizedBox(width: double.infinity, height: 0),
                secondChild: Column(
                  children: [
                    const Divider(height: 1),
                    Padding(
                      padding: const EdgeInsets.all(16),
                      child: TextField(
                        controller: _p2pServerAddrController,
                        focusNode: _p2pServerAddrFocus,
                        enabled: !isRunning,
                        decoration: InputDecoration(
                          labelText: l10n.netplayP2PServerLabel,
                          prefixIcon: const Icon(Icons.hub_rounded),
                          border: OutlineInputBorder(
                            borderRadius: BorderRadius.circular(12),
                          ),
                          filled: true,
                          fillColor: theme.colorScheme.surfaceContainerHighest
                              .withAlpha(50),
                        ),
                        onChanged: controller.setP2PServerAddr,
                      ),
                    ),
                  ],
                ),
                crossFadeState: settings.p2pEnabled
                    ? CrossFadeState.showSecond
                    : CrossFadeState.showFirst,
                duration: const Duration(milliseconds: 300),
                sizeCurve: Curves.easeInOut,
                alignment: Alignment.topCenter,
              ),
            ],
          ),
        ),
        const SizedBox(height: 16),
        // Start/Stop button
        AnimatedSettingsCard(
          index: 3,
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: AnimatedSwitcher(
              duration: const Duration(milliseconds: 300),
              child: SizedBox(
                width: double.infinity,
                key: ValueKey(isRunning),
                child: isRunning
                    ? FilledButton.tonalIcon(
                        onPressed: () async {
                          try {
                            await controller.stopServer();
                          } catch (e) {
                            if (context.mounted) {
                              ScaffoldMessenger.of(context).showSnackBar(
                                SnackBar(
                                  content: Text(
                                    l10n.serverStopFailed(e.toString()),
                                  ),
                                ),
                              );
                            }
                          }
                        },
                        style: FilledButton.styleFrom(
                          foregroundColor: theme.colorScheme.error,
                          padding: const EdgeInsets.symmetric(vertical: 16),
                          shape: RoundedRectangleBorder(
                            borderRadius: BorderRadius.circular(12),
                          ),
                        ),
                        icon: const Icon(Icons.stop_rounded),
                        label: Text(l10n.serverStopButton),
                      )
                    : FilledButton.icon(
                        onPressed: () async {
                          try {
                            if (settings.p2pEnabled) {
                              if (settings.p2pServerAddr.trim().isEmpty) {
                                ScaffoldMessenger.of(context).showSnackBar(
                                  SnackBar(
                                    content: Text(
                                      l10n.netplayInvalidP2PServerAddr,
                                    ),
                                  ),
                                );
                                return;
                              }

                              await controller.startP2PHost();
                            } else {
                              await controller.startServer();
                            }
                          } catch (e) {
                            if (context.mounted) {
                              ScaffoldMessenger.of(context).showSnackBar(
                                SnackBar(
                                  content: Text(
                                    settings.p2pEnabled
                                        ? l10n.netplayConnectFailed(
                                            e.toString(),
                                          )
                                        : l10n.serverStartFailed(e.toString()),
                                  ),
                                ),
                              );
                            }
                          }
                        },
                        style: FilledButton.styleFrom(
                          padding: const EdgeInsets.symmetric(vertical: 16),
                          shape: RoundedRectangleBorder(
                            borderRadius: BorderRadius.circular(12),
                          ),
                        ),
                        icon: const Icon(Icons.play_arrow_rounded),
                        label: Text(l10n.serverStartButton),
                      ),
              ),
            ),
          ),
        ),
      ],
    );
  }

  @override
  void dispose() {
    _settingsSub?.close();
    _playerNameController.dispose();
    _portController.dispose();
    _p2pServerAddrController.dispose();
    _playerNameFocus.dispose();
    _portFocus.dispose();
    _p2pServerAddrFocus.dispose();
    super.dispose();
  }
}
