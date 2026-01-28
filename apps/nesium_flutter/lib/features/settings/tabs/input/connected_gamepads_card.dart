import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../../l10n/app_localizations.dart';
import '../../../../widgets/animated_settings_widgets.dart';
import '../../gamepad_assignment_controller.dart';
import '../../../../domain/connected_gamepads_provider.dart';
import '../../../../platform/nes_gamepad.dart' as nes_gamepad;

class ConnectedGamepadsCard extends ConsumerWidget {
  const ConnectedGamepadsCard({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final gamepadsAsync = ref.watch(connectedGamepadsProvider);

    return AnimatedSettingsCard(
      index: 2,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Padding(
            padding: const EdgeInsets.fromLTRB(16, 12, 16, 4),
            child: Row(
              children: [
                Icon(
                  Icons.sports_esports,
                  size: 20,
                  color: Theme.of(context).colorScheme.primary,
                ),
                const SizedBox(width: 8),
                Text(
                  l10n.connectedGamepadsTitle,
                  style: Theme.of(
                    context,
                  ).textTheme.titleSmall?.copyWith(fontWeight: FontWeight.w600),
                ),
              ],
            ),
          ),
          gamepadsAsync.when(
            data: (gamepads) {
              if (gamepads.isEmpty) {
                return Padding(
                  padding: const EdgeInsets.fromLTRB(16, 8, 16, 16),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Row(
                        children: [
                          Icon(
                            Icons.info_outline,
                            size: 16,
                            color: Theme.of(context).colorScheme.outline,
                          ),
                          const SizedBox(width: 8),
                          Text(
                            l10n.connectedGamepadsNone,
                            style: TextStyle(
                              color: Theme.of(context).colorScheme.outline,
                            ),
                          ),
                        ],
                      ),
                      if (kIsWeb) ...[
                        const SizedBox(height: 8),
                        Container(
                          padding: const EdgeInsets.all(8),
                          decoration: BoxDecoration(
                            color: Theme.of(context)
                                .colorScheme
                                .primaryContainer
                                .withValues(alpha: 0.3),
                            borderRadius: BorderRadius.circular(8),
                            border: Border.all(
                              color: Theme.of(
                                context,
                              ).colorScheme.primary.withValues(alpha: 0.2),
                            ),
                          ),
                          child: Row(
                            children: [
                              Icon(
                                Icons.touch_app,
                                size: 16,
                                color: Theme.of(context).colorScheme.primary,
                              ),
                              const SizedBox(width: 8),
                              Expanded(
                                child: Text(
                                  l10n.webGamepadActivationHint,
                                  style: TextStyle(
                                    fontSize: 12,
                                    color: Theme.of(
                                      context,
                                    ).colorScheme.primary,
                                    fontWeight: FontWeight.w500,
                                  ),
                                ),
                              ),
                            ],
                          ),
                        ),
                      ],
                    ],
                  ),
                );
              }
              return Column(
                children: gamepads.map((gamepad) {
                  final portLabel = gamepad.port != null
                      ? l10n.connectedGamepadsPort(gamepad.port! + 1)
                      : l10n.connectedGamepadsUnassigned;
                  return ListTile(
                    leading: Icon(
                      Icons.gamepad,
                      color: gamepad.connected
                          ? Theme.of(context).colorScheme.primary
                          : Theme.of(context).colorScheme.outline,
                    ),
                    title: Text(gamepad.name),
                    subtitle: Text(portLabel),
                    dense: true,
                    trailing: gamepad.port != null
                        ? IconButton(
                            icon: const Icon(Icons.link_off),
                            onPressed: () async {
                              await nes_gamepad.bindGamepad(
                                id: gamepad.id,
                                port: null,
                              );
                              ref
                                  .read(gamepadAssignmentProvider.notifier)
                                  .removeAssignment(gamepad.name);
                              ref.invalidate(connectedGamepadsProvider);
                            },
                          )
                        : null,
                  );
                }).toList(),
              );
            },
            loading: () => const Padding(
              padding: EdgeInsets.all(16),
              child: Center(
                child: SizedBox(
                  width: 20,
                  height: 20,
                  child: CircularProgressIndicator(strokeWidth: 2),
                ),
              ),
            ),
            error: (error, stack) => Padding(
              padding: const EdgeInsets.fromLTRB(16, 8, 16, 16),
              child: Text(
                l10n.connectedGamepadsNone,
                style: TextStyle(color: Theme.of(context).colorScheme.outline),
              ),
            ),
          ),
        ],
      ),
    );
  }
}
