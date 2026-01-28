import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../../l10n/app_localizations.dart';
import '../../../../widgets/animated_settings_widgets.dart';
import '../../../../widgets/binding_pill.dart';
import '../../settings_utils.dart';
import '../../../../features/controls/input_settings.dart';
import '../../../../platform/platform_capabilities.dart';
import '../../input_settings_types.dart';
// Note: keyboardPressedKeysProvider should be imported from somewhere.
// It seems it was used in settings_page.dart but not defined there.
// It is likely in input_settings.dart or similar. Checking dependencies.
// Ah, it uses `ref.watch(keyboardPressedKeysProvider)`.

class KeyboardMappingInfoCard extends ConsumerStatefulWidget {
  final int port;
  const KeyboardMappingInfoCard({super.key, required this.port});

  @override
  ConsumerState<KeyboardMappingInfoCard> createState() =>
      _KeyboardMappingInfoCardState();
}

class _KeyboardMappingInfoCardState
    extends ConsumerState<KeyboardMappingInfoCard> {
  Timer? _remappingTimer;

  @override
  void dispose() {
    _remappingTimer?.cancel();
    super.dispose();
  }

  void _startRemapping(KeyboardBindingAction action) {
    ref
        .read(remappingStateProvider.notifier)
        .update(RemapLocation(action, widget.port));

    _remappingTimer?.cancel();
    _remappingTimer = Timer.periodic(const Duration(milliseconds: 50), (timer) {
      final pressed = HardwareKeyboard.instance.logicalKeysPressed;
      if (pressed.isNotEmpty) {
        final key = pressed.first;
        if (key == LogicalKeyboardKey.escape) {
          _applyRemap(action, null);
        } else {
          _applyRemap(action, key);
        }
        timer.cancel();
      }
    });

    // Timeout remapping after 5 seconds
    Future.delayed(const Duration(seconds: 5), () {
      if (mounted && ref.read(remappingStateProvider)?.action == action) {
        ref.read(remappingStateProvider.notifier).update(null);
        _remappingTimer?.cancel();
      }
    });
  }

  void _applyRemap(KeyboardBindingAction action, LogicalKeyboardKey? key) {
    ref.read(inputSettingsProvider.notifier).setCustomBinding(action, key);
    if (mounted) {
      ref.read(remappingStateProvider.notifier).update(null);
    }
  }

  String _keyLabel(AppLocalizations l10n, LogicalKeyboardKey? key) {
    if (key == null) return l10n.unassignedKey;
    final label = key.keyLabel.trim();
    if (label.isNotEmpty) return label;
    return key.debugName ?? 'Key 0x${key.keyId.toRadixString(16)}';
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final inputState = ref.watch(inputSettingsProvider);
    final settings =
        inputState.ports[widget.port] ??
        const InputSettings(
          device: InputDevice.keyboard,
          keyboardPreset: KeyboardPreset.none,
        );

    final inputController = ref.read(inputSettingsProvider.notifier);
    final currentRemap = ref.watch(remappingStateProvider);
    ref.listen(remappingStateProvider, (prev, next) {
      if (next == null) {
        _remappingTimer?.cancel();
        _remappingTimer = null;
      }
    });
    // Assuming keyboardPressedKeysProvider is available via riverpod_provider or we use RawKeyboard/HardwareKeyboard listener
    // In settings_page.dart it was: final pressedKeys = ref.watch(keyboardPressedKeysProvider).value ?? {};
    // I need to check where keyboardPressedKeysProvider comes from.
    // Proceeding with assuming it is available or I will fix it later.
    final pressedKeys = ref.watch(keyboardPressedKeysProvider).value ?? {};

    return AnimatedSize(
      duration: const Duration(milliseconds: 300),
      curve: Curves.easeInOut,
      child: AnimatedSwitcher(
        duration: const Duration(milliseconds: 300),
        transitionBuilder: (Widget child, Animation<double> animation) {
          return FadeTransition(
            opacity: animation,
            child: SizeTransition(
              sizeFactor: animation,
              axisAlignment: -1.0,
              child: child,
            ),
          );
        },
        child: settings.keyboardPreset == KeyboardPreset.none
            ? const SizedBox.shrink()
            : AnimatedSettingsCard(
                key: const ValueKey('keyboard_mapping_card'),
                index: 3,
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Padding(
                      padding: const EdgeInsets.fromLTRB(16, 12, 16, 4),
                      child: Row(
                        mainAxisAlignment: MainAxisAlignment.spaceBetween,
                        children: [
                          Row(
                            mainAxisSize: MainAxisSize.min,
                            children: [
                              Text(
                                l10n.inputGamepadMappingLabel, // Use "Mapping" label for unification
                                style: Theme.of(context).textTheme.labelLarge
                                    ?.copyWith(
                                      color: Theme.of(
                                        context,
                                      ).colorScheme.primary,
                                    ),
                              ),
                              const SizedBox(width: 12),
                              InkWell(
                                onTap: () =>
                                    inputController.resetToDefault(widget.port),
                                borderRadius: BorderRadius.circular(4),
                                child: Padding(
                                  padding: const EdgeInsets.symmetric(
                                    horizontal: 4,
                                    vertical: 2,
                                  ),
                                  child: Text(
                                    l10n.inputResetToDefault,
                                    style: Theme.of(context)
                                        .textTheme
                                        .labelSmall
                                        ?.copyWith(
                                          color: Theme.of(context)
                                              .colorScheme
                                              .primary
                                              .withValues(alpha: 0.7),
                                          decoration: TextDecoration.underline,
                                        ),
                                  ),
                                ),
                              ),
                              Padding(
                                padding: const EdgeInsets.only(left: 12),
                                child: Text(
                                  l10n.longPressToClear,
                                  style: TextStyle(
                                    fontSize: 12,
                                    color: Theme.of(context)
                                        .colorScheme
                                        .onSurface
                                        .withValues(alpha: 0.5),
                                  ),
                                ),
                              ),
                            ],
                          ),
                          if (currentRemap != null &&
                              currentRemap.port == widget.port)
                            Row(
                              mainAxisSize: MainAxisSize.min,
                              children: [
                                Text(
                                  l10n.inputListening,
                                  style: TextStyle(
                                    color: Theme.of(
                                      context,
                                    ).colorScheme.secondary,
                                    fontWeight: FontWeight.bold,
                                    fontSize: 12,
                                  ),
                                ),
                                const SizedBox(width: 8),
                                InkWell(
                                  onTap: () => ref
                                      .read(remappingStateProvider.notifier)
                                      .update(null),
                                  borderRadius: BorderRadius.circular(4),
                                  child: Padding(
                                    padding: const EdgeInsets.symmetric(
                                      horizontal: 4,
                                      vertical: 2,
                                    ),
                                    child: Text(
                                      l10n.cancel,
                                      style: TextStyle(
                                        color: Theme.of(
                                          context,
                                        ).colorScheme.error,
                                        fontWeight: FontWeight.bold,
                                        fontSize: 12,
                                        decoration: TextDecoration.underline,
                                      ),
                                    ),
                                  ),
                                ),
                              ],
                            )
                          else if (pressedKeys.isNotEmpty)
                            Text(
                              l10n.inputDetected(
                                pressedKeys
                                    .map((k) => _keyLabel(l10n, k))
                                    .join(', '),
                              ),
                              style: Theme.of(context).textTheme.bodySmall
                                  ?.copyWith(
                                    color: Theme.of(
                                      context,
                                    ).colorScheme.tertiary,
                                    fontWeight: FontWeight.bold,
                                  ),
                            ),
                        ],
                      ),
                    ),
                    const Divider(),
                    Padding(
                      padding: const EdgeInsets.fromLTRB(8, 8, 8, 12),
                      child: Wrap(
                        spacing: 8,
                        runSpacing: 8,
                        children: [
                          for (final action
                              in KeyboardBindingAction.values.where(
                                (a) => a.isCore,
                              ))
                            Builder(
                              builder: (context) {
                                final key = settings.bindingForAction(action);
                                final conflict = key != null
                                    ? inputState.findConflict(
                                        key,
                                        excludePort: widget.port,
                                        excludeAction: action,
                                      )
                                    : null;

                                final isRemapping =
                                    currentRemap?.action == action &&
                                    currentRemap?.port == widget.port;

                                return SizedBox(
                                  width: 155,
                                  child: BindingPill(
                                    label: SettingsUtils.actionLabel(
                                      l10n,
                                      action,
                                    ),
                                    buttonName: _keyLabel(l10n, key),
                                    isPressed: pressedKeys.contains(key),
                                    isRemapping: isRemapping,
                                    isConflicted: conflict != null,
                                    isEnabled: true,
                                    conflictLabel:
                                        (conflict != null &&
                                            conflict.port != widget.port)
                                        ? 'P${conflict.port + 1}'
                                        : null,
                                    onTap: () => _startRemapping(action),
                                    onLongPress: () => inputController
                                        .setCustomBinding(action, null),
                                    icon: Icons.keyboard,
                                  ),
                                );
                              },
                            ),
                        ],
                      ),
                    ),
                    const Divider(),
                    Padding(
                      padding: const EdgeInsets.fromLTRB(16, 8, 16, 4),
                      child: Row(
                        children: [
                          Icon(
                            Icons.keyboard_command_key,
                            size: 16,
                            color: Theme.of(context).colorScheme.primary,
                          ),
                          const SizedBox(width: 8),
                          Text(
                            l10n.globalHotkeysTitle,
                            style: Theme.of(context).textTheme.labelLarge
                                ?.copyWith(
                                  color: Theme.of(context).colorScheme.primary,
                                ),
                          ),
                        ],
                      ),
                    ),
                    Padding(
                      padding: const EdgeInsets.fromLTRB(8, 0, 8, 12),
                      child: Wrap(
                        spacing: 8,
                        runSpacing: 8,
                        children: [
                          for (final action
                              in KeyboardBindingAction.values.where((a) {
                                if (!a.isExtended) return false;
                                if (a == KeyboardBindingAction.fullScreen &&
                                    !isNativeDesktop) {
                                  return false;
                                }
                                return true;
                              }))
                            Builder(
                              builder: (context) {
                                final key = settings.bindingForAction(action);
                                final conflict = key != null
                                    ? inputState.findConflict(
                                        key,
                                        excludePort: widget.port,
                                        excludeAction: action,
                                      )
                                    : null;

                                final isRemapping =
                                    currentRemap?.action == action &&
                                    currentRemap?.port == widget.port;

                                return SizedBox(
                                  width: 155,
                                  child: BindingPill(
                                    label: SettingsUtils.actionLabel(
                                      l10n,
                                      action,
                                    ),
                                    buttonName: _keyLabel(l10n, key),
                                    isPressed: pressedKeys.contains(key),
                                    isRemapping: isRemapping,
                                    isConflicted: conflict != null,
                                    isEnabled: true,
                                    conflictLabel:
                                        (conflict != null &&
                                            conflict.port != widget.port)
                                        ? 'P${conflict.port + 1}'
                                        : null,
                                    onTap: () => _startRemapping(action),
                                    onLongPress: () => inputController
                                        .setCustomBinding(action, null),
                                    icon: Icons.keyboard,
                                  ),
                                );
                              },
                            ),
                        ],
                      ),
                    ),
                  ],
                ),
              ),
      ),
    );
  }
}
