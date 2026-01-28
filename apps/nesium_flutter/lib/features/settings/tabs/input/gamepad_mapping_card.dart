import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../../l10n/app_localizations.dart';
import '../../../../widgets/animated_settings_widgets.dart';
import '../../../../widgets/binding_pill.dart';
import '../../gamepad_settings.dart';
import '../../settings_utils.dart';
import '../../../../domain/connected_gamepads_provider.dart';
import '../../../../platform/nes_gamepad.dart' as nes_gamepad;
import '../../../../features/controls/input_settings.dart';
import '../../input_settings_types.dart';

class GamepadMappingInfoCard extends ConsumerStatefulWidget {
  final int port;

  const GamepadMappingInfoCard({super.key, required this.port});

  @override
  ConsumerState<GamepadMappingInfoCard> createState() =>
      _GamepadMappingInfoCardState();
}

class _GamepadMappingInfoCardState
    extends ConsumerState<GamepadMappingInfoCard> {
  Timer? _remappingTimer;

  @override
  void dispose() {
    _remappingTimer?.cancel();
    super.dispose();
  }

  void _startRemapping(NesButtonAction action) {
    final gamepads = ref.read(connectedGamepadsProvider).value ?? [];
    final assignedId = _getAssignedGamepadId(gamepads);
    if (assignedId == null) return;

    ref
        .read(remappingStateProvider.notifier)
        .update(RemapLocation(action, widget.port));

    _remappingTimer?.cancel();
    _remappingTimer = Timer.periodic(const Duration(milliseconds: 50), (
      timer,
    ) async {
      final pressed = await nes_gamepad.getGamepadPressedButtons(assignedId);
      if (pressed.isNotEmpty) {
        // Find first non-"unknown" button
        final btn = pressed.firstWhere(
          (b) => b != nes_gamepad.GamepadButton.unknown,
          orElse: () => nes_gamepad.GamepadButton.unknown,
        );
        if (btn != nes_gamepad.GamepadButton.unknown) {
          _applyRemap(action, btn);
          timer.cancel();
        }
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

  int? _getAssignedGamepadId(List<nes_gamepad.GamepadInfo> gamepads) {
    for (final gp in gamepads) {
      if (gp.port == widget.port) return gp.id;
    }
    return null;
  }

  Future<void> _applyRemap(
    NesButtonAction action,
    nes_gamepad.GamepadButton button,
  ) async {
    final currentMapping = await nes_gamepad.getGamepadMapping(widget.port);
    if (currentMapping == null) return;

    final newMapping = nes_gamepad.GamepadMapping(
      a: action == NesButtonAction.a ? button : currentMapping.a,
      b: action == NesButtonAction.b ? button : currentMapping.b,
      select: action == NesButtonAction.select ? button : currentMapping.select,
      start: action == NesButtonAction.start ? button : currentMapping.start,
      up: action == NesButtonAction.up ? button : currentMapping.up,
      down: action == NesButtonAction.down ? button : currentMapping.down,
      left: action == NesButtonAction.left ? button : currentMapping.left,
      right: action == NesButtonAction.right ? button : currentMapping.right,
      turboA: action == NesButtonAction.turboA ? button : currentMapping.turboA,
      turboB: action == NesButtonAction.turboB ? button : currentMapping.turboB,
      rewind: action == NesButtonAction.rewind ? button : currentMapping.rewind,
      fastForward: action == NesButtonAction.fastForward
          ? button
          : currentMapping.fastForward,
      saveState: action == NesButtonAction.saveState
          ? button
          : currentMapping.saveState,
      loadState: action == NesButtonAction.loadState
          ? button
          : currentMapping.loadState,
      pause: action == NesButtonAction.pause ? button : currentMapping.pause,
    );

    final gamepads = ref.read(connectedGamepadsProvider).value ?? [];
    final gp = gamepads.where((g) => g.port == widget.port).firstOrNull;

    if (gp != null) {
      ref
          .read(gamepadSettingsProvider.notifier)
          .saveMapping(gp.name, widget.port, newMapping);
    } else {
      // Fallback: just apply to port if no gamepad info found (shouldn't happen)
      await nes_gamepad.setGamepadMapping(widget.port, newMapping);
    }
    if (mounted) {
      ref.invalidate(nes_gamepad.gamepadMappingProvider(widget.port));
      ref.read(remappingStateProvider.notifier).update(null);
    }
  }

  NesButtonAction? _toNesButtonAction(KeyboardBindingAction action) {
    return switch (action) {
      KeyboardBindingAction.rewind => NesButtonAction.rewind,
      KeyboardBindingAction.fastForward => NesButtonAction.fastForward,
      KeyboardBindingAction.saveState => NesButtonAction.saveState,
      KeyboardBindingAction.loadState => NesButtonAction.loadState,
      KeyboardBindingAction.pause => NesButtonAction.pause,
      _ => null,
    };
  }

  nes_gamepad.GamepadButton? _getGamepadButton(
    nes_gamepad.GamepadMapping mapping,
    NesButtonAction action,
  ) {
    return switch (action) {
      NesButtonAction.a => mapping.a,
      NesButtonAction.b => mapping.b,
      NesButtonAction.select => mapping.select,
      NesButtonAction.start => mapping.start,
      NesButtonAction.up => mapping.up,
      NesButtonAction.down => mapping.down,
      NesButtonAction.left => mapping.left,
      NesButtonAction.right => mapping.right,
      NesButtonAction.turboA => mapping.turboA,
      NesButtonAction.turboB => mapping.turboB,
      NesButtonAction.rewind => mapping.rewind,
      NesButtonAction.fastForward => mapping.fastForward,
      NesButtonAction.saveState => mapping.saveState,
      NesButtonAction.loadState => mapping.loadState,
      NesButtonAction.pause => mapping.pause,
    };
  }

  Future<void> _clearBinding(NesButtonAction action) async {
    final gamepads = ref.read(connectedGamepadsProvider).value ?? [];
    final gp = gamepads.where((g) => g.port == widget.port).firstOrNull;
    if (gp == null && !kIsWeb) return;

    final currentMapping =
        ref.read(gamepadSettingsProvider)[gp?.name] ??
        (await nes_gamepad.getGamepadMapping(widget.port)) ??
        nes_gamepad.GamepadMapping.standard();

    final newMapping = switch (action) {
      NesButtonAction.a => currentMapping.copyWith(a: null),
      NesButtonAction.b => currentMapping.copyWith(b: null),
      NesButtonAction.select => currentMapping.copyWith(select: null),
      NesButtonAction.start => currentMapping.copyWith(start: null),
      NesButtonAction.up => currentMapping.copyWith(up: null),
      NesButtonAction.down => currentMapping.copyWith(down: null),
      NesButtonAction.left => currentMapping.copyWith(left: null),
      NesButtonAction.right => currentMapping.copyWith(right: null),
      NesButtonAction.turboA => currentMapping.copyWith(turboA: null),
      NesButtonAction.turboB => currentMapping.copyWith(turboB: null),
      NesButtonAction.rewind => currentMapping.copyWith(rewind: null),
      NesButtonAction.fastForward => currentMapping.copyWith(fastForward: null),
      NesButtonAction.saveState => currentMapping.copyWith(saveState: null),
      NesButtonAction.loadState => currentMapping.copyWith(loadState: null),
      NesButtonAction.pause => currentMapping.copyWith(pause: null),
    };

    if (gp != null) {
      ref
          .read(gamepadSettingsProvider.notifier)
          .saveMapping(gp.name, widget.port, newMapping);
    } else {
      await nes_gamepad.setGamepadMapping(widget.port, newMapping);
    }

    if (mounted) {
      ref.invalidate(nes_gamepad.gamepadMappingProvider(widget.port));
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text(
            AppLocalizations.of(context)!.inputBindingConflictCleared(
              "Gamepad",
              SettingsUtils.actionLabel(AppLocalizations.of(context)!, action),
            ),
          ),
          duration: const Duration(seconds: 2),
        ),
      );
    }
  }

  Future<void> _resetToDefault() async {
    final gamepads = ref.read(connectedGamepadsProvider).value ?? [];
    final gp = gamepads.where((g) => g.port == widget.port).firstOrNull;
    final standard = nes_gamepad.GamepadMapping.standard();

    if (gp != null) {
      ref
          .read(gamepadSettingsProvider.notifier)
          .saveMapping(gp.name, widget.port, standard);
    } else {
      await nes_gamepad.setGamepadMapping(widget.port, standard);
    }

    if (mounted) {
      ref.invalidate(nes_gamepad.gamepadMappingProvider(widget.port));
    }
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final gamepads = ref.watch(connectedGamepadsProvider).value ?? [];
    final mappingAsync = ref.watch(
      nes_gamepad.gamepadMappingProvider(widget.port),
    );
    final assignedId = _getAssignedGamepadId(gamepads);
    final pressedButtons = assignedId != null
        ? ref
                  .watch(nes_gamepad.gamepadPressedButtonsProvider(assignedId))
                  .value ??
              []
        : <nes_gamepad.GamepadButton>[];

    final currentRemap = ref.watch(remappingStateProvider);
    ref.listen(remappingStateProvider, (prev, next) {
      if (next == null) {
        _remappingTimer?.cancel();
        _remappingTimer = null;
      }
    });
    final inputSettings = ref.watch(inputSettingsProvider);
    final settings = inputSettings.selectedSettings;

    return AnimatedSettingsCard(
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
                      l10n.inputGamepadMappingLabel,
                      style: Theme.of(context).textTheme.labelLarge?.copyWith(
                        color: Theme.of(context).colorScheme.primary,
                      ),
                    ),
                    const SizedBox(width: 12),
                    InkWell(
                      onTap: _resetToDefault,
                      borderRadius: BorderRadius.circular(4),
                      child: Padding(
                        padding: const EdgeInsets.symmetric(
                          horizontal: 4,
                          vertical: 2,
                        ),
                        child: Text(
                          l10n.inputResetToDefault,
                          style: Theme.of(context).textTheme.labelSmall
                              ?.copyWith(
                                color: Theme.of(
                                  context,
                                ).colorScheme.primary.withValues(alpha: 0.7),
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
                          color: Theme.of(
                            context,
                          ).colorScheme.onSurface.withValues(alpha: 0.5),
                        ),
                      ),
                    ),
                  ],
                ),
                if (currentRemap != null && currentRemap.port == widget.port)
                  Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text(
                        l10n.inputListening,
                        style: TextStyle(
                          color: Theme.of(context).colorScheme.secondary,
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
                              color: Theme.of(context).colorScheme.error,
                              fontWeight: FontWeight.bold,
                              fontSize: 12,
                              decoration: TextDecoration.underline,
                            ),
                          ),
                        ),
                      ),
                    ],
                  )
                else if (pressedButtons.isNotEmpty)
                  Text(
                    l10n.inputDetected(
                      pressedButtons.map((b) => b.toFriendlyName()).join(', '),
                    ),
                    style: Theme.of(context).textTheme.bodySmall?.copyWith(
                      color: Theme.of(context).colorScheme.tertiary,
                      fontWeight: FontWeight.bold,
                    ),
                  ),
              ],
            ),
          ),
          mappingAsync.when(
            data: (mapping) {
              return AnimatedSize(
                duration: const Duration(milliseconds: 300),
                curve: Curves.easeInOut,
                child: AnimatedSwitcher(
                  duration: const Duration(milliseconds: 300),
                  transitionBuilder:
                      (Widget child, Animation<double> animation) {
                        return FadeTransition(
                          opacity: animation,
                          child: SizeTransition(
                            sizeFactor: animation,
                            axisAlignment: -1.0,
                            child: child,
                          ),
                        );
                      },
                  child: mapping == null
                      ? const SizedBox.shrink()
                      : Column(
                          key: const ValueKey('gamepad_binding_content'),
                          children: [
                            const Divider(),
                            Builder(
                              builder: (context) {
                                final mappingRows = [
                                  (
                                    l10n.inputButtonA,
                                    NesButtonAction.a,
                                    mapping.a,
                                  ),
                                  (
                                    l10n.inputButtonB,
                                    NesButtonAction.b,
                                    mapping.b,
                                  ),
                                  (
                                    l10n.inputButtonTurboA,
                                    NesButtonAction.turboA,
                                    mapping.turboA,
                                  ),
                                  (
                                    l10n.inputButtonTurboB,
                                    NesButtonAction.turboB,
                                    mapping.turboB,
                                  ),
                                  (
                                    l10n.inputButtonSelect,
                                    NesButtonAction.select,
                                    mapping.select,
                                  ),
                                  (
                                    l10n.inputButtonStart,
                                    NesButtonAction.start,
                                    mapping.start,
                                  ),
                                  (
                                    l10n.inputButtonUp,
                                    NesButtonAction.up,
                                    mapping.up,
                                  ),
                                  (
                                    l10n.inputButtonDown,
                                    NesButtonAction.down,
                                    mapping.down,
                                  ),
                                  (
                                    l10n.inputButtonLeft,
                                    NesButtonAction.left,
                                    mapping.left,
                                  ),
                                  (
                                    l10n.inputButtonRight,
                                    NesButtonAction.right,
                                    mapping.right,
                                  ),
                                ];

                                final buttonCounts =
                                    <nes_gamepad.GamepadButton, int>{};
                                for (final r in mappingRows) {
                                  if (r.$3 != null) {
                                    buttonCounts[r.$3!] =
                                        (buttonCounts[r.$3!] ?? 0) + 1;
                                  }
                                }

                                return Wrap(
                                  spacing: 8,
                                  runSpacing: 8,
                                  children: mappingRows.map((row) {
                                    final isPressed = pressedButtons.contains(
                                      row.$3,
                                    );
                                    final isRemapping =
                                        currentRemap != null &&
                                        currentRemap.action == row.$2 &&
                                        currentRemap.port == widget.port;
                                    final isConflicted =
                                        row.$3 != null &&
                                        (buttonCounts[row.$3!] ?? 0) > 1;

                                    return SizedBox(
                                      width: 155,
                                      child: BindingPill(
                                        label: row.$1,
                                        buttonName: (row.$3 != null)
                                            ? row.$3!.toFriendlyName()
                                            : l10n.unassignedKey,
                                        isPressed: isPressed,
                                        isRemapping: isRemapping,
                                        isConflicted: isConflicted,
                                        onTap: () => _startRemapping(row.$2),
                                        onLongPress: () =>
                                            _clearBinding(row.$2),
                                        icon: Icons.gamepad,
                                      ),
                                    );
                                  }).toList(),
                                );
                              },
                            ),
                            if (settings.keyboardPreset !=
                                    KeyboardPreset.none ||
                                settings.device != InputDevice.keyboard) ...[
                              const Divider(),
                              Padding(
                                padding: const EdgeInsets.fromLTRB(
                                  16,
                                  8,
                                  16,
                                  4,
                                ),
                                child: Row(
                                  children: [
                                    Icon(
                                      settings.device == InputDevice.keyboard
                                          ? Icons.keyboard_command_key
                                          : Icons.gamepad,
                                      size: 16,
                                      color: Theme.of(
                                        context,
                                      ).colorScheme.primary,
                                    ),
                                    const SizedBox(width: 8),
                                    Text(
                                      settings.device == InputDevice.keyboard
                                          ? l10n.globalHotkeysTitle
                                          : l10n.gamepadHotkeysTitle,
                                      style: Theme.of(context)
                                          .textTheme
                                          .labelLarge
                                          ?.copyWith(
                                            color: Theme.of(
                                              context,
                                            ).colorScheme.primary,
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
                                        in KeyboardBindingAction.values.where(
                                          (a) => a.isExtended,
                                        ))
                                      Builder(
                                        builder: (context) {
                                          final nesAction = _toNesButtonAction(
                                            action,
                                          )!;
                                          final isRemapping =
                                              currentRemap != null &&
                                              currentRemap.action ==
                                                  nesAction &&
                                              currentRemap.port == widget.port;
                                          final gamepadButton =
                                              _getGamepadButton(
                                                mapping,
                                                nesAction,
                                              );
                                          final isPressed =
                                              gamepadButton != null &&
                                              pressedButtons.contains(
                                                gamepadButton,
                                              );

                                          return SizedBox(
                                            width: 155,
                                            child: BindingPill(
                                              label: SettingsUtils.actionLabel(
                                                l10n,
                                                action,
                                              ),
                                              buttonName:
                                                  gamepadButton
                                                      ?.toFriendlyName() ??
                                                  '---',
                                              isPressed: isPressed,
                                              isRemapping: isRemapping,
                                              isConflicted: false,
                                              isEnabled: assignedId != null,
                                              onTap: () =>
                                                  _startRemapping(nesAction),
                                              onLongPress: () =>
                                                  _clearBinding(nesAction),
                                              icon: Icons.gamepad,
                                            ),
                                          );
                                        },
                                      ),
                                  ],
                                ),
                              ),
                            ],
                          ],
                        ),
                ),
              );
            },
            loading: () => const Padding(
              padding: EdgeInsets.all(16),
              child: Center(child: CircularProgressIndicator()),
            ),
            error: (e, _) => Padding(
              padding: const EdgeInsets.all(16),
              child: Text(e.toString()),
            ),
          ),
        ],
      ),
    );
  }
}
