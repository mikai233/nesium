import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../l10n/app_localizations.dart';
import '../../../widgets/animated_dropdown_menu.dart';
import '../../../widgets/animated_settings_widgets.dart';
import '../settings_utils.dart';
import '../../../../features/controls/input_settings.dart';
import '../../../../features/controls/turbo_settings.dart';
import '../../../../features/controls/virtual_controls_editor.dart';
import '../../../../features/controls/virtual_controls_settings.dart';
import '../../../platform/platform_capabilities.dart';
import '../../../domain/connected_gamepads_provider.dart';
import '../../../platform/nes_gamepad.dart' as nes_gamepad;
import '../gamepad_assignment_controller.dart';
import 'input/connected_gamepads_card.dart';
import 'input/gamepad_mapping_card.dart';
import 'input/keyboard_mapping_card.dart';

class InputTab extends ConsumerWidget {
  const InputTab({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final inputState = ref.watch(inputSettingsProvider);
    final inputSettings = inputState.selectedSettings;
    final inputController = ref.read(inputSettingsProvider.notifier);
    final turboSettings = ref.watch(turboSettingsProvider);
    final turboController = ref.read(turboSettingsProvider.notifier);
    final settings = ref.watch(virtualControlsSettingsProvider);
    final controller = ref.read(virtualControlsSettingsProvider.notifier);
    final editor = ref.watch(virtualControlsEditorProvider);
    final editorController = ref.read(virtualControlsEditorProvider.notifier);
    final gamepadsAsync = ref.watch(connectedGamepadsProvider);

    final supportsVirtual = supportsVirtualControls;
    final usingVirtual = inputSettings.device == InputDevice.virtualController;

    return ListView(
      padding: const EdgeInsets.all(20),
      children: [
        AnimatedSectionHeader(
          title: l10n.inputTitle,
          icon: Icons.gamepad,
          delay: const Duration(milliseconds: 50),
        ),
        // Player Selection
        AnimatedSettingsCard(
          index: 0,
          child: ListTile(
            title: Text(l10n.inputPortLabel),
            trailing: SizedBox(
              width: 200,
              child: AnimatedDropdownMenu<int>(
                density: AnimatedDropdownMenuDensity.compact,
                value: inputState.selectedPort,
                entries: [
                  DropdownMenuEntry(value: 0, label: l10n.player1),
                  DropdownMenuEntry(value: 1, label: l10n.player2),
                  DropdownMenuEntry(
                    value: 2,
                    label: l10n.player3,
                    enabled: false,
                  ),
                  DropdownMenuEntry(
                    value: 3,
                    label: l10n.player4,
                    enabled: false,
                  ),
                ],
                onSelected: (val) {
                  inputController.setSelectedPort(val);
                },
              ),
            ),
          ),
        ),
        // Input Device
        AnimatedSettingsCard(
          index: 1,
          child: ListTile(
            title: Text(l10n.inputDeviceLabel),
            subtitle: Text(switch (inputSettings.device) {
              InputDevice.keyboard => l10n.inputDeviceKeyboard,
              InputDevice.gamepad => l10n.inputDeviceGamepad,
              InputDevice.virtualController =>
                l10n.inputDeviceVirtualController,
            }, style: TextStyle(color: Theme.of(context).colorScheme.primary)),
            trailing: SizedBox(
              width: 200,
              child: AnimatedDropdownMenu<InputDevice>(
                density: AnimatedDropdownMenuDensity.compact,
                value: inputSettings.device,
                entries: [
                  DropdownMenuEntry(
                    value: InputDevice.keyboard,
                    label: l10n.inputDeviceKeyboard,
                  ),
                  DropdownMenuEntry(
                    value: InputDevice.gamepad,
                    label: l10n.inputDeviceGamepad,
                  ),
                  if (supportsVirtual ||
                      inputSettings.device == InputDevice.virtualController)
                    DropdownMenuEntry(
                      value: InputDevice.virtualController,
                      label: l10n.inputDeviceVirtualController,
                      enabled: supportsVirtual,
                    ),
                ],
                onSelected: inputController.setDevice,
              ),
            ),
          ),
        ),
        AnimatedSize(
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
            child: inputSettings.device == InputDevice.gamepad
                ? Column(
                    key: const ValueKey('gamepad_sections'),
                    children: [
                      const ConnectedGamepadsCard(),
                      gamepadsAsync.maybeWhen(
                        data: (gamepads) {
                          if (gamepads.isEmpty) return const SizedBox.shrink();

                          final assignedGamepad = gamepads
                              .cast<nes_gamepad.GamepadInfo?>()
                              .firstWhere(
                                (g) => g?.port == inputState.selectedPort,
                                orElse: () => null,
                              );

                          return Column(
                            key: const ValueKey('gamepad_assignment_group'),
                            children: [
                              AnimatedSettingsCard(
                                key: const ValueKey('gamepad_assignment'),
                                index: 2,
                                child: ListTile(
                                  title: Text(l10n.inputGamepadAssignmentLabel),
                                  subtitle: Text(
                                    assignedGamepad?.name ??
                                        l10n.inputGamepadNone,
                                  ),
                                  trailing: SizedBox(
                                    width: 200,
                                    child: AnimatedDropdownMenu<int?>(
                                      density:
                                          AnimatedDropdownMenuDensity.compact,
                                      value: assignedGamepad?.id,
                                      entries: [
                                        DropdownMenuEntry(
                                          value: null,
                                          label: l10n.inputGamepadNone,
                                        ),
                                        ...gamepads.map(
                                          (g) => DropdownMenuEntry(
                                            value: g.id,
                                            label: g.name,
                                          ),
                                        ),
                                      ],
                                      onSelected: (id) async {
                                        if (id == null &&
                                            assignedGamepad != null) {
                                          await nes_gamepad.bindGamepad(
                                            id: assignedGamepad.id,
                                            port: null,
                                          );
                                          ref
                                              .read(
                                                gamepadAssignmentProvider
                                                    .notifier,
                                              )
                                              .removeAssignment(
                                                assignedGamepad.name,
                                              );
                                        } else if (id != null) {
                                          await nes_gamepad.bindGamepad(
                                            id: id,
                                            port: inputState.selectedPort,
                                          );
                                          final name = gamepads
                                              .firstWhere((g) => g.id == id)
                                              .name;
                                          ref
                                              .read(
                                                gamepadAssignmentProvider
                                                    .notifier,
                                              )
                                              .saveAssignment(
                                                name,
                                                inputState.selectedPort,
                                              );
                                        }
                                        ref.invalidate(
                                          connectedGamepadsProvider,
                                        );
                                      },
                                    ),
                                  ),
                                ),
                              ),
                              if (assignedGamepad != null)
                                GamepadMappingInfoCard(
                                  port: inputState.selectedPort,
                                ),
                            ],
                          );
                        },
                        orElse: () => const SizedBox.shrink(),
                      ),
                    ],
                  )
                : const SizedBox.shrink(),
          ),
        ),
        // Turbo Settings
        AnimatedSettingsCard(
          animateSize: false,
          index: 1,
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Padding(
                padding: const EdgeInsets.fromLTRB(16, 12, 16, 4),
                child: Text(
                  l10n.turboTitle,
                  style: Theme.of(
                    context,
                  ).textTheme.titleSmall?.copyWith(fontWeight: FontWeight.w600),
                ),
              ),
              CheckboxListTile(
                value: turboSettings.linked,
                title: Text(l10n.turboLinkPressRelease),
                onChanged: (value) {
                  if (value == null) return;
                  turboController.setLinked(value);
                },
              ),
              Padding(
                padding: const EdgeInsets.symmetric(horizontal: 16),
                child: Column(
                  children: [
                    AnimatedSliderTile(
                      label: turboSettings.linked
                          ? l10n.virtualControlsTurboFramesPerToggle
                          : l10n.virtualControlsTurboOnFrames,
                      value: turboSettings.onFrames.toDouble(),
                      min: 1,
                      max: 30,
                      divisions: 29,
                      onChanged: (v) => turboController.setOnFrames(v.round()),
                      valueLabel: l10n.framesValue(turboSettings.onFrames),
                    ),
                    AnimatedSwitcher(
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
                      child: turboSettings.linked
                          ? const SizedBox(width: double.infinity, height: 0)
                          : AnimatedSliderTile(
                              key: const ValueKey('turbo_off_frames'),
                              label: l10n.virtualControlsTurboOffFrames,
                              value: turboSettings.offFrames.toDouble(),
                              min: 1,
                              max: 30,
                              divisions: 29,
                              onChanged: (v) =>
                                  turboController.setOffFrames(v.round()),
                              valueLabel: l10n.framesValue(
                                turboSettings.offFrames,
                              ),
                            ),
                    ),
                  ],
                ),
              ),
              const SizedBox(height: 8),
            ],
          ),
        ),
        // Keyboard Settings (shown when keyboard is selected)
        if (inputSettings.device == InputDevice.keyboard) ...[
          AnimatedSettingsCard(
            key: const ValueKey('keyboard_preset'),
            index: 2,
            child: ListTile(
              title: Text(l10n.keyboardPresetLabel),
              subtitle: Text(
                SettingsUtils.presetLabel(l10n, inputSettings.keyboardPreset),
                style: TextStyle(color: Theme.of(context).colorScheme.primary),
              ),
              trailing: SizedBox(
                width: 200,
                child: AnimatedDropdownMenu<KeyboardPreset>(
                  density: AnimatedDropdownMenuDensity.compact,
                  value: inputSettings.keyboardPreset,
                  entries: [
                    for (final preset in KeyboardPreset.values)
                      DropdownMenuEntry(
                        value: preset,
                        label: SettingsUtils.presetLabel(l10n, preset),
                      ),
                  ],
                  onSelected: inputController.setKeyboardPreset,
                ),
              ),
            ),
          ),
          KeyboardMappingInfoCard(port: inputState.selectedPort),
        ],
        // Virtual Controls
        if (supportsVirtualControls) ...[
          AnimatedSize(
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
              child: usingVirtual
                  ? Column(
                      key: const ValueKey('virtual_controls_edit_group'),
                      children: [
                        const SizedBox(height: 12),
                        AnimatedSettingsCard(
                          index: 5,
                          child: ListTile(
                            leading: const Icon(Icons.tune),
                            title: Text(l10n.virtualControlsEditTitle),
                            subtitle: Text(
                              editor.enabled
                                  ? l10n.virtualControlsEditSubtitleEnabled
                                  : l10n.virtualControlsEditSubtitleDisabled,
                            ),
                            trailing: Switch(
                              value: editor.enabled,
                              onChanged: (enabled) {
                                if (enabled &&
                                    inputSettings.device !=
                                        InputDevice.virtualController) {
                                  inputController.setDevice(
                                    InputDevice.virtualController,
                                  );
                                }
                                editorController.setEnabled(enabled);
                                if (enabled) {
                                  Navigator.of(context).maybePop();
                                }
                              },
                            ),
                          ),
                        ),
                        if (editor.enabled) ...[
                          const SizedBox(height: 12),
                          AnimatedSettingsCard(
                            index: 6,
                            child: Column(
                              children: [
                                SwitchListTile(
                                  secondary: const Icon(Icons.grid_4x4),
                                  title: Text(l10n.gridSnappingTitle),
                                  value: editor.gridSnapEnabled,
                                  onChanged:
                                      editorController.setGridSnapEnabled,
                                ),
                                if (editor.gridSnapEnabled)
                                  Padding(
                                    padding: const EdgeInsets.fromLTRB(
                                      16,
                                      0,
                                      16,
                                      12,
                                    ),
                                    child: AnimatedSliderTile(
                                      label: l10n.gridSpacingLabel,
                                      value: editor.gridSpacing.clamp(4, 64),
                                      min: 4,
                                      max: 64,
                                      divisions: 60,
                                      onChanged:
                                          editorController.setGridSpacing,
                                      valueLabel:
                                          '${editor.gridSpacing.toStringAsFixed(0)} px',
                                    ),
                                  ),
                              ],
                            ),
                          ),
                        ],
                        const SizedBox(height: 8),
                      ],
                    )
                  : const SizedBox.shrink(
                      key: ValueKey('virtual_controls_edit_hidden'),
                    ),
            ),
          ),
          AnimatedSize(
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
              child: usingVirtual
                  ? Column(
                      key: const ValueKey('virtual_controls_settings_group'),
                      children: [
                        AnimatedSettingsCard(
                          index: 7,
                          child: Column(
                            crossAxisAlignment: CrossAxisAlignment.start,
                            children: [
                              Padding(
                                padding: const EdgeInsets.fromLTRB(16, 4, 4, 0),
                                child: Row(
                                  mainAxisAlignment:
                                      MainAxisAlignment.spaceBetween,
                                  children: [
                                    Text(
                                      l10n.virtualControlsTitle,
                                      style: Theme.of(context)
                                          .textTheme
                                          .titleSmall
                                          ?.copyWith(
                                            fontWeight: FontWeight.w600,
                                          ),
                                    ),
                                    IconButton(
                                      visualDensity: VisualDensity.compact,
                                      onPressed: controller.resetToDefault,
                                      icon: const Icon(Icons.restore, size: 20),
                                      tooltip: l10n.virtualControlsReset,
                                    ),
                                  ],
                                ),
                              ),
                              Padding(
                                padding: const EdgeInsets.symmetric(
                                  horizontal: 16,
                                ),
                                child: Column(
                                  children: [
                                    AnimatedSliderTile(
                                      label: l10n.virtualControlsButtonSize,
                                      value: settings.buttonSize,
                                      min: 40,
                                      max: 120,
                                      onChanged: controller.setButtonSize,
                                      valueLabel:
                                          '${settings.buttonSize.toStringAsFixed(0)} px',
                                    ),
                                    AnimatedSliderTile(
                                      label: l10n.virtualControlsGap,
                                      value: settings.gap,
                                      min: 4,
                                      max: 24,
                                      onChanged: controller.setGap,
                                      valueLabel:
                                          '${settings.gap.toStringAsFixed(0)} px',
                                    ),
                                    AnimatedSliderTile(
                                      label: l10n.virtualControlsOpacity,
                                      value: settings.opacity,
                                      min: 0.2,
                                      max: 0.8,
                                      onChanged: controller.setOpacity,
                                      valueLabel: settings.opacity
                                          .toStringAsFixed(2),
                                    ),
                                    AnimatedSliderTile(
                                      label: l10n.virtualControlsHitboxScale,
                                      value: settings.hitboxScale,
                                      min: 1.0,
                                      max: 1.4,
                                      divisions: 40,
                                      onChanged: controller.setHitboxScale,
                                      valueLabel: settings.hitboxScale
                                          .toStringAsFixed(2),
                                    ),
                                    AnimatedSwitchTile(
                                      value: settings.hapticsEnabled,
                                      title: Text(
                                        l10n.virtualControlsHapticFeedback,
                                      ),
                                      onChanged: controller.setHapticsEnabled,
                                    ),
                                    AnimatedSliderTile(
                                      label: l10n.virtualControlsDpadDeadzone,
                                      value: settings.dpadDeadzoneRatio,
                                      min: 0.06,
                                      max: 0.30,
                                      divisions: 48,
                                      onChanged:
                                          controller.setDpadDeadzoneRatio,
                                      valueLabel: settings.dpadDeadzoneRatio
                                          .toStringAsFixed(2),
                                    ),
                                    Padding(
                                      padding: const EdgeInsets.symmetric(
                                        vertical: 8,
                                      ),
                                      child: Text(
                                        l10n.virtualControlsDpadDeadzoneHelp,
                                        style: Theme.of(context)
                                            .textTheme
                                            .bodySmall
                                            ?.copyWith(
                                              color: Theme.of(context)
                                                  .colorScheme
                                                  .onSurface
                                                  .withValues(alpha: 0.75),
                                            ),
                                      ),
                                    ),
                                    AnimatedSliderTile(
                                      label: l10n
                                          .virtualControlsDpadBoundaryDeadzone,
                                      value: settings.dpadBoundaryDeadzoneRatio,
                                      min: 0.35,
                                      max: 0.90,
                                      divisions: 55,
                                      onChanged: controller
                                          .setDpadBoundaryDeadzoneRatio,
                                      valueLabel: settings
                                          .dpadBoundaryDeadzoneRatio
                                          .toStringAsFixed(2),
                                    ),
                                    Padding(
                                      padding: const EdgeInsets.symmetric(
                                        vertical: 8,
                                      ),
                                      child: Text(
                                        l10n.virtualControlsDpadBoundaryDeadzoneHelp,
                                        style: Theme.of(context)
                                            .textTheme
                                            .bodySmall
                                            ?.copyWith(
                                              color: Theme.of(context)
                                                  .colorScheme
                                                  .onSurface
                                                  .withValues(alpha: 0.75),
                                            ),
                                      ),
                                    ),
                                    const SizedBox(height: 8),
                                    Text(
                                      l10n.tipAdjustButtonsInDrawer,
                                      style: Theme.of(
                                        context,
                                      ).textTheme.bodySmall,
                                    ),
                                  ],
                                ),
                              ),
                              const SizedBox(height: 8),
                            ],
                          ),
                        ),
                      ],
                    )
                  : const SizedBox.shrink(
                      key: ValueKey('virtual_controls_hidden_placeholder'),
                    ),
            ),
          ),
        ],
      ],
    );
  }
}
