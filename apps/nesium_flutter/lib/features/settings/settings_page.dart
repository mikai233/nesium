import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter/foundation.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:file_picker/file_picker.dart';

import '../../l10n/app_localizations.dart';
import '../../logging/app_logger.dart';
import '../../platform/platform_capabilities.dart';
import '../../widgets/animated_dropdown_menu.dart';
import '../../widgets/animated_settings_widgets.dart';
import '../../widgets/binding_pill.dart';
import '../controls/input_settings.dart';
import '../controls/turbo_settings.dart';
import '../controls/virtual_controls_editor.dart';
import '../controls/virtual_controls_settings.dart';
import 'android_video_backend_settings.dart';
import 'android_shader_settings.dart';
import 'android_performance_settings.dart';
import 'emulation_settings.dart';
import 'gamepad_settings.dart';
import 'gamepad_assignment_controller.dart';
import 'language_settings.dart';
import 'theme_settings.dart';
import 'video_settings.dart';
import 'windows_video_backend_settings.dart';
import 'windows_performance_settings.dart';
import '../shaders/shader_browser_page.dart';

import 'server_settings.dart';
import '../../platform/nes_palette.dart' as nes_palette;
import '../../domain/connected_gamepads_provider.dart';
import '../../platform/nes_gamepad.dart' as nes_gamepad;
import '../../platform/nes_video.dart' as nes_video;
import '../../domain/nes_controller.dart';
import '../../windows/current_window_kind.dart';
import '../../windows/window_types.dart';

class SettingsPage extends ConsumerStatefulWidget {
  const SettingsPage({super.key});

  @override
  ConsumerState<SettingsPage> createState() => _SettingsPageState();
}

class _SettingsPageState extends ConsumerState<SettingsPage>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;
  StreamSubscription<InputCollision>? _collisionSubscription;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: supportsTcp ? 5 : 4, vsync: this);

    _collisionSubscription = ref
        .read(inputSettingsProvider.notifier)
        .collisionStream
        .listen((collision) {
          if (!mounted) return;
          final l10n = AppLocalizations.of(context)!;
          final player = switch (collision.port) {
            0 => l10n.player1,
            1 => l10n.player2,
            2 => l10n.player3,
            3 => l10n.player4,
            _ => 'Player ${collision.port + 1}',
          };

          final action = _SettingsPageState._actionLabel(
            l10n,
            collision.action,
          );

          ScaffoldMessenger.of(context).showSnackBar(
            SnackBar(
              content: Text(l10n.inputBindingConflictCleared(player, action)),
              duration: const Duration(seconds: 2),
            ),
          );
        });
  }

  @override
  void dispose() {
    _collisionSubscription?.cancel();
    _tabController.dispose();
    super.dispose();
  }

  Future<void> _pickAndApplyCustomPalette(
    BuildContext context,
    VideoSettingsController controller,
  ) async {
    final result = await FilePicker.platform.pickFiles(
      type: FileType.custom,
      allowedExtensions: ['pal'],
      withData: true,
      withReadStream: false,
    );
    final file = result?.files.single;
    if (file == null) return;

    final bytes = file.bytes;
    if (bytes == null) return;

    try {
      await controller.setCustomPalette(bytes, name: file.name);
    } catch (e) {
      if (!context.mounted) return;
      final l10n = AppLocalizations.of(context)!;
      ScaffoldMessenger.of(context).showSnackBar(
        SnackBar(
          content: Text('${l10n.commandFailed(l10n.actionLoadPalette)}: $e'),
        ),
      );
    }
  }

  static String _presetLabel(AppLocalizations l10n, KeyboardPreset preset) =>
      switch (preset) {
        KeyboardPreset.none => l10n.keyboardPresetNone,
        KeyboardPreset.nesStandard => l10n.keyboardPresetNesStandard,
        KeyboardPreset.fightStick => l10n.keyboardPresetFightStick,
        KeyboardPreset.arcadeLayout => l10n.keyboardPresetArcadeLayout,
        KeyboardPreset.custom => l10n.keyboardPresetCustom,
      };

  static String _actionLabel(AppLocalizations l10n, dynamic action) {
    if (action is NesButtonAction) {
      return switch (action) {
        NesButtonAction.a => l10n.inputButtonA,
        NesButtonAction.b => l10n.inputButtonB,
        NesButtonAction.select => l10n.inputButtonSelect,
        NesButtonAction.start => l10n.inputButtonStart,
        NesButtonAction.up => l10n.inputButtonUp,
        NesButtonAction.down => l10n.inputButtonDown,
        NesButtonAction.left => l10n.inputButtonLeft,
        NesButtonAction.right => l10n.inputButtonRight,
        NesButtonAction.turboA => l10n.inputButtonTurboA,
        NesButtonAction.turboB => l10n.inputButtonTurboB,
        NesButtonAction.rewind => l10n.inputButtonRewind,
        NesButtonAction.fastForward => l10n.inputButtonFastForward,
        NesButtonAction.saveState => l10n.inputButtonSaveState,
        NesButtonAction.loadState => l10n.inputButtonLoadState,
        NesButtonAction.pause => l10n.inputButtonPause,
      };
    } else if (action is KeyboardBindingAction) {
      return switch (action) {
        KeyboardBindingAction.up => l10n.keyboardActionUp,
        KeyboardBindingAction.down => l10n.keyboardActionDown,
        KeyboardBindingAction.left => l10n.keyboardActionLeft,
        KeyboardBindingAction.right => l10n.keyboardActionRight,
        KeyboardBindingAction.a => l10n.keyboardActionA,
        KeyboardBindingAction.b => l10n.keyboardActionB,
        KeyboardBindingAction.select => l10n.keyboardActionSelect,
        KeyboardBindingAction.start => l10n.keyboardActionStart,
        KeyboardBindingAction.turboA => l10n.keyboardActionTurboA,
        KeyboardBindingAction.turboB => l10n.keyboardActionTurboB,
        KeyboardBindingAction.rewind => l10n.keyboardActionRewind,
        KeyboardBindingAction.fastForward => l10n.keyboardActionFastForward,
        KeyboardBindingAction.saveState => l10n.keyboardActionSaveState,
        KeyboardBindingAction.loadState => l10n.keyboardActionLoadState,
        KeyboardBindingAction.pause => l10n.keyboardActionPause,
      };
    }
    return 'Unknown';
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    return Scaffold(
      appBar: AppBar(
        title: Text(l10n.settingsTitle),
        bottom: TabBar(
          controller: _tabController,
          tabs: [
            Tab(icon: const Icon(Icons.public), text: l10n.settingsTabGeneral),
            Tab(
              icon: const Icon(Icons.videogame_asset),
              text: l10n.settingsTabInput,
            ),
            Tab(icon: const Icon(Icons.palette), text: l10n.settingsTabVideo),
            Tab(
              icon: const Icon(Icons.settings_applications),
              text: l10n.settingsTabEmulation,
            ),
            if (supportsTcp)
              Tab(
                icon: const Icon(Icons.dns_rounded),
                text: l10n.settingsTabServer,
              ),
          ],
        ),
      ),
      body: TabBarView(
        controller: _tabController,
        children: [
          _GeneralTab(),
          _InputTab(),
          _VideoTab(pickAndApplyCustomPalette: _pickAndApplyCustomPalette),
          _EmulationTab(),
          if (supportsTcp) _ServerTab(),
        ],
      ),
    );
  }
}

// ============================================================================
// General Tab
// ============================================================================

class _GeneralTab extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final language = ref.watch(appLanguageProvider);
    final languageController = ref.read(appLanguageProvider.notifier);
    final themeSettings = ref.watch(themeSettingsProvider);
    final themeController = ref.read(themeSettingsProvider.notifier);

    return ListView(
      padding: const EdgeInsets.all(20),
      children: [
        AnimatedSectionHeader(
          title: l10n.generalTitle,
          icon: Icons.settings,
          delay: const Duration(milliseconds: 50),
        ),
        AnimatedSettingsCard(
          index: 0,
          child: Column(
            children: [
              ListTile(
                title: Text(l10n.languageLabel),
                subtitle: Text(
                  switch (language) {
                    AppLanguage.system => l10n.languageSystem,
                    AppLanguage.english => l10n.languageEnglish,
                    AppLanguage.chineseSimplified =>
                      l10n.languageChineseSimplified,
                  },
                  style: TextStyle(
                    color: Theme.of(context).colorScheme.primary,
                  ),
                ),
                trailing: SizedBox(
                  width: 180,
                  child: AnimatedDropdownMenu<AppLanguage>(
                    density: AnimatedDropdownMenuDensity.compact,
                    value: language,
                    entries: [
                      DropdownMenuEntry(
                        value: AppLanguage.system,
                        label: l10n.languageSystem,
                      ),
                      DropdownMenuEntry(
                        value: AppLanguage.english,
                        label: l10n.languageEnglish,
                      ),
                      DropdownMenuEntry(
                        value: AppLanguage.chineseSimplified,
                        label: l10n.languageChineseSimplified,
                      ),
                    ],
                    onSelected: (value) {
                      languageController.setLanguage(value);
                    },
                  ),
                ),
              ),
              const Divider(height: 1),
              ListTile(
                title: Text(l10n.themeLabel),
                subtitle: Text(
                  switch (themeSettings.mode) {
                    AppThemeMode.system => l10n.themeSystem,
                    AppThemeMode.light => l10n.themeLight,
                    AppThemeMode.dark => l10n.themeDark,
                  },
                  style: TextStyle(
                    color: Theme.of(context).colorScheme.primary,
                  ),
                ),
                trailing: SizedBox(
                  width: 180,
                  child: AnimatedDropdownMenu<AppThemeMode>(
                    density: AnimatedDropdownMenuDensity.compact,
                    value: themeSettings.mode,
                    entries: [
                      DropdownMenuEntry(
                        value: AppThemeMode.system,
                        label: l10n.themeSystem,
                      ),
                      DropdownMenuEntry(
                        value: AppThemeMode.light,
                        label: l10n.themeLight,
                      ),
                      DropdownMenuEntry(
                        value: AppThemeMode.dark,
                        label: l10n.themeDark,
                      ),
                    ],
                    onSelected: themeController.setThemeMode,
                  ),
                ),
              ),
            ],
          ),
        ),
      ],
    );
  }
}

// Continue in next part due to length...

// ============================================================================
// Input Tab
// ============================================================================

class _InputTab extends ConsumerWidget {
  const _InputTab();

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
                      _ConnectedGamepadsCard(),
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
                                _GamepadMappingInfoCard(
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
                _SettingsPageState._presetLabel(
                  l10n,
                  inputSettings.keyboardPreset,
                ),
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
                        label: _SettingsPageState._presetLabel(l10n, preset),
                      ),
                  ],
                  onSelected: inputController.setKeyboardPreset,
                ),
              ),
            ),
          ),
          _KeyboardMappingInfoCard(port: inputState.selectedPort),
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
                                padding: const EdgeInsets.fromLTRB(
                                  16,
                                  12,
                                  16,
                                  4,
                                ),
                                child: Text(
                                  l10n.virtualControlsTitle,
                                  style: Theme.of(context).textTheme.titleSmall
                                      ?.copyWith(fontWeight: FontWeight.w600),
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
                        AnimatedSettingsCard(
                          index: 8,
                          child: ListTile(
                            leading: const Icon(Icons.restore),
                            title: Text(l10n.virtualControlsReset),
                            onTap: controller.resetToDefault,
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

// ============================================================================
// Video Tab
// ============================================================================

class _VideoTab extends ConsumerStatefulWidget {
  const _VideoTab({required this.pickAndApplyCustomPalette});

  final Future<void> Function(BuildContext, VideoSettingsController)
  pickAndApplyCustomPalette;

  @override
  ConsumerState<_VideoTab> createState() => _VideoTabState();
}

class _VideoTabState extends ConsumerState<_VideoTab> {
  Timer? _ntscApplyTimer;
  Timer? _ntscBisqwitApplyTimer;

  @override
  void dispose() {
    _ntscApplyTimer?.cancel();
    _ntscBisqwitApplyTimer?.cancel();
    super.dispose();
  }

  void _scheduleApplyNtscOptions(nes_video.NtscOptions options) {
    _ntscApplyTimer?.cancel();
    _ntscApplyTimer = Timer(const Duration(milliseconds: 120), () async {
      try {
        await nes_video.setNtscOptions(options: options);
      } catch (e, st) {
        logWarning(
          e,
          stackTrace: st,
          message: 'setNtscOptions failed',
          logger: 'settings_page',
        );
      }
    });
  }

  void _scheduleApplyNtscBisqwitOptions(nes_video.NtscBisqwitOptions options) {
    _ntscBisqwitApplyTimer?.cancel();
    _ntscBisqwitApplyTimer = Timer(const Duration(milliseconds: 120), () async {
      try {
        await nes_video.setNtscBisqwitOptions(options: options);
      } catch (e, st) {
        logWarning(
          e,
          stackTrace: st,
          message: 'setNtscBisqwitOptions failed',
          logger: 'settings_page',
        );
      }
    });
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final videoSettings = ref.watch(videoSettingsProvider);
    final videoController = ref.read(videoSettingsProvider.notifier);
    final isAndroid =
        !kIsWeb && defaultTargetPlatform == TargetPlatform.android;
    final isWindows =
        !kIsWeb && defaultTargetPlatform == TargetPlatform.windows;
    final androidBackend = ref.watch(androidVideoBackendSettingsProvider);
    final androidBackendController = ref.read(
      androidVideoBackendSettingsProvider.notifier,
    );
    final androidShaderSettings = isAndroid
        ? ref.watch(androidShaderSettingsProvider)
        : null;
    final androidShaderController = isAndroid
        ? ref.read(androidShaderSettingsProvider.notifier)
        : null;

    final windowsBackend = isWindows
        ? ref.watch(windowsVideoBackendSettingsProvider)
        : const WindowsVideoBackendSettings(
            backend: WindowsVideoBackend.d3d11Gpu,
          );
    final windowsBackendController = isWindows
        ? ref.read(windowsVideoBackendSettingsProvider.notifier)
        : null;

    final windowsPerformance = isWindows
        ? ref.watch(windowsPerformanceSettingsControllerProvider)
        : WindowsPerformanceSettings(highPerformance: false);
    final windowsPerformanceController = isWindows
        ? ref.read(windowsPerformanceSettingsControllerProvider.notifier)
        : null;

    final androidPerformance = isAndroid
        ? ref.watch(androidPerformanceSettingsControllerProvider)
        : AndroidPerformanceSettings(highPerformance: false);
    final androidPerformanceController = isAndroid
        ? ref.read(androidPerformanceSettingsControllerProvider.notifier)
        : null;

    Widget dropdown<T>({
      required String labelText,
      required T value,
      required List<DropdownMenuEntry<T>> entries,
      required Future<void> Function(T) onSelected,
      String? helperText,
    }) {
      return AnimatedDropdownMenu<T>(
        labelText: labelText,
        helperText: helperText,
        value: value,
        entries: entries,
        onSelected: onSelected,
      );
    }

    Future<void> onPaletteModeSelected(PaletteMode value) async {
      if (value == PaletteMode.builtin) {
        try {
          await videoController.setBuiltinPreset(videoSettings.builtinPreset);
        } catch (e, st) {
          logWarning(
            e,
            stackTrace: st,
            message: 'setBuiltinPreset failed',
            logger: 'settings_page',
          );
        }
        return;
      }
      final hasCustom = videoSettings.customPaletteName != null;
      try {
        await videoController.setPaletteMode(PaletteMode.custom);
      } catch (e, st) {
        logWarning(
          e,
          stackTrace: st,
          message: 'setPaletteMode failed',
          logger: 'settings_page',
        );
      }
      if (hasCustom) return;
      if (!context.mounted) return;
      await widget.pickAndApplyCustomPalette(context, videoController);
    }

    Future<void> setBuiltinPalette(nes_palette.PaletteKind value) async {
      try {
        await videoController.setBuiltinPreset(value);
      } catch (e, st) {
        logWarning(
          e,
          stackTrace: st,
          message: 'setBuiltinPreset failed',
          logger: 'settings_page',
        );
      }
    }

    Future<void> setAspectRatio(NesAspectRatio value) async {
      try {
        await videoController.setAspectRatio(value);
      } catch (e, st) {
        logWarning(
          e,
          stackTrace: st,
          message: 'setAspectRatio failed',
          logger: 'settings_page',
        );
      }
    }

    Future<void> setVideoFilter(nes_video.VideoFilter value) async {
      try {
        await videoController.setVideoFilter(value);

        final isNtsc =
            value == nes_video.VideoFilter.ntscComposite ||
            value == nes_video.VideoFilter.ntscSVideo ||
            value == nes_video.VideoFilter.ntscRgb ||
            value == nes_video.VideoFilter.ntscMonochrome;
        if (isNtsc) {
          final options = ref.read(videoSettingsProvider).ntscOptions;
          await nes_video.setNtscOptions(options: options);
        }

        final isNtscBisqwit =
            value == nes_video.VideoFilter.ntscBisqwit2X ||
            value == nes_video.VideoFilter.ntscBisqwit4X ||
            value == nes_video.VideoFilter.ntscBisqwit8X;
        if (isNtscBisqwit) {
          final options = ref.read(videoSettingsProvider).ntscBisqwitOptions;
          await nes_video.setNtscBisqwitOptions(options: options);
        }

        if (value == nes_video.VideoFilter.lcdGrid) {
          final strength = ref.read(videoSettingsProvider).lcdGridStrength;
          await nes_video.setLcdGridOptions(
            options: nes_video.LcdGridOptions(strength: strength),
          );
        }

        if (value == nes_video.VideoFilter.scanlines) {
          final intensity = ref.read(videoSettingsProvider).scanlineIntensity;
          await nes_video.setScanlineOptions(
            options: nes_video.ScanlineOptions(intensity: intensity),
          );
        }

        final kind = ref.read(currentWindowKindProvider);
        final applyInThisEngine = !isNativeDesktop || kind == WindowKind.main;

        // In multi-window mode, the settings window runs in a separate engine
        // and does not own the game texture. Texture resize must happen in the
        // main window engine, which reacts to `settingsChanged(video)`.
        if (applyInThisEngine) {
          await ref.read(nesControllerProvider.notifier).setVideoFilter(value);
        }
      } catch (e, st) {
        logWarning(
          e,
          stackTrace: st,
          message: 'setVideoFilter failed',
          logger: 'settings_page',
        );
      }
    }

    Future<void> setAndroidBackend(AndroidVideoBackend value) async {
      try {
        await androidBackendController.setBackend(value);
      } catch (e, st) {
        logWarning(
          e,
          stackTrace: st,
          message: 'setBackend failed',
          logger: 'settings_page',
        );
      }
    }

    Future<void> setWindowsBackend(WindowsVideoBackend value) async {
      if (windowsBackendController == null) return;
      try {
        await windowsBackendController.setBackend(value);
      } catch (e, st) {
        logWarning(
          e,
          stackTrace: st,
          message: 'setWindowsBackend failed',
          logger: 'settings_page',
        );
      }
    }

    Future<void> pickAndSetShaderPreset() async {
      if (!isAndroid) return;
      if (androidShaderController == null) return;
      if (androidBackend.backend != AndroidVideoBackend.hardware) return;

      if (!context.mounted) return;
      Navigator.of(context).push(
        MaterialPageRoute(builder: (context) => const ShaderBrowserPage()),
      );
    }

    return ListView(
      padding: const EdgeInsets.all(20),
      children: [
        AnimatedSectionHeader(
          title: l10n.videoTitle,
          icon: Icons.videocam,
          delay: const Duration(milliseconds: 100),
        ),
        const SizedBox(height: 8),
        AnimatedSettingsCard(
          index: 0,
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                dropdown<nes_video.VideoFilter>(
                  labelText: l10n.videoFilterLabel,
                  value: videoSettings.videoFilter,
                  entries: [
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.none,
                      label: l10n.videoFilterNone,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.prescale2X,
                      label: l10n.videoFilterPrescale2x,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.prescale3X,
                      label: l10n.videoFilterPrescale3x,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.prescale4X,
                      label: l10n.videoFilterPrescale4x,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.hq2X,
                      label: l10n.videoFilterHq2x,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.hq3X,
                      label: l10n.videoFilterHq3x,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.hq4X,
                      label: l10n.videoFilterHq4x,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.sai2X,
                      label: l10n.videoFilter2xSai,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.super2XSai,
                      label: l10n.videoFilterSuper2xSai,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.superEagle,
                      label: l10n.videoFilterSuperEagle,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.lcdGrid,
                      label: l10n.videoFilterLcdGrid,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.scanlines,
                      label: l10n.videoFilterScanlines,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.xbrz2X,
                      label: l10n.videoFilterXbrz2x,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.xbrz3X,
                      label: l10n.videoFilterXbrz3x,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.xbrz4X,
                      label: l10n.videoFilterXbrz4x,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.xbrz5X,
                      label: l10n.videoFilterXbrz5x,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.xbrz6X,
                      label: l10n.videoFilterXbrz6x,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.ntscComposite,
                      label: l10n.videoFilterNtscComposite,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.ntscSVideo,
                      label: l10n.videoFilterNtscSvideo,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.ntscRgb,
                      label: l10n.videoFilterNtscRgb,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.ntscMonochrome,
                      label: l10n.videoFilterNtscMonochrome,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.ntscBisqwit2X,
                      label: l10n.videoFilterNtscBisqwit2x,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.ntscBisqwit4X,
                      label: l10n.videoFilterNtscBisqwit4x,
                    ),
                    DropdownMenuEntry(
                      value: nes_video.VideoFilter.ntscBisqwit8X,
                      label: l10n.videoFilterNtscBisqwit8x,
                    ),
                  ],
                  onSelected: setVideoFilter,
                ),
                const SizedBox(height: 12),
                if (videoSettings.videoFilter == nes_video.VideoFilter.lcdGrid)
                  AnimatedSliderTile(
                    label: l10n.videoLcdGridStrengthLabel,
                    value: videoSettings.lcdGridStrength,
                    min: 0,
                    max: 1,
                    divisions: 100,
                    valueLabel:
                        '${(videoSettings.lcdGridStrength * 100).round()}%',
                    onChanged: (value) {
                      unawaited(videoController.setLcdGridStrength(value));
                    },
                  ),
                if (videoSettings.videoFilter == nes_video.VideoFilter.lcdGrid)
                  const SizedBox(height: 12),
                if (videoSettings.videoFilter ==
                    nes_video.VideoFilter.scanlines)
                  AnimatedSliderTile(
                    label: l10n.videoScanlinesIntensityLabel,
                    value: videoSettings.scanlineIntensity,
                    min: 0,
                    max: 1,
                    divisions: 100,
                    valueLabel:
                        '${(videoSettings.scanlineIntensity * 100).round()}%',
                    onChanged: (value) {
                      unawaited(videoController.setScanlineIntensity(value));
                    },
                  ),
                if (videoSettings.videoFilter ==
                    nes_video.VideoFilter.scanlines)
                  const SizedBox(height: 12),
                if (videoSettings.videoFilter ==
                        nes_video.VideoFilter.ntscBisqwit2X ||
                    videoSettings.videoFilter ==
                        nes_video.VideoFilter.ntscBisqwit4X ||
                    videoSettings.videoFilter ==
                        nes_video.VideoFilter.ntscBisqwit8X)
                  AnimatedExpansionTile(
                    labelText: l10n.videoNtscBisqwitSettingsTitle,
                    title: Text(l10n.keyboardPresetCustom),
                    initiallyExpanded: false,
                    children: [
                      Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16),
                        child: AnimatedSliderTile(
                          label: l10n.videoNtscHueLabel,
                          value: videoSettings.ntscBisqwitOptions.hue,
                          min: -1,
                          max: 1,
                          divisions: 200,
                          valueLabel: videoSettings.ntscBisqwitOptions.hue
                              .toStringAsFixed(2),
                          onChanged: (value) {
                            final o = videoSettings.ntscBisqwitOptions;
                            final next = nes_video.NtscBisqwitOptions(
                              brightness: o.brightness,
                              contrast: o.contrast,
                              hue: value,
                              saturation: o.saturation,
                              yFilterLength: o.yFilterLength,
                              iFilterLength: o.iFilterLength,
                              qFilterLength: o.qFilterLength,
                            );
                            unawaited(
                              videoController.setNtscBisqwitOptions(next),
                            );
                            _scheduleApplyNtscBisqwitOptions(next);
                          },
                        ),
                      ),
                      Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16),
                        child: AnimatedSliderTile(
                          label: l10n.videoNtscSaturationLabel,
                          value: videoSettings.ntscBisqwitOptions.saturation,
                          min: -1,
                          max: 1,
                          divisions: 200,
                          valueLabel: videoSettings
                              .ntscBisqwitOptions
                              .saturation
                              .toStringAsFixed(2),
                          onChanged: (value) {
                            final o = videoSettings.ntscBisqwitOptions;
                            final next = nes_video.NtscBisqwitOptions(
                              brightness: o.brightness,
                              contrast: o.contrast,
                              hue: o.hue,
                              saturation: value,
                              yFilterLength: o.yFilterLength,
                              iFilterLength: o.iFilterLength,
                              qFilterLength: o.qFilterLength,
                            );
                            unawaited(
                              videoController.setNtscBisqwitOptions(next),
                            );
                            _scheduleApplyNtscBisqwitOptions(next);
                          },
                        ),
                      ),
                      Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16),
                        child: AnimatedSliderTile(
                          label: l10n.videoNtscContrastLabel,
                          value: videoSettings.ntscBisqwitOptions.contrast,
                          min: -1,
                          max: 1,
                          divisions: 200,
                          valueLabel: videoSettings.ntscBisqwitOptions.contrast
                              .toStringAsFixed(2),
                          onChanged: (value) {
                            final o = videoSettings.ntscBisqwitOptions;
                            final next = nes_video.NtscBisqwitOptions(
                              brightness: o.brightness,
                              contrast: value,
                              hue: o.hue,
                              saturation: o.saturation,
                              yFilterLength: o.yFilterLength,
                              iFilterLength: o.iFilterLength,
                              qFilterLength: o.qFilterLength,
                            );
                            unawaited(
                              videoController.setNtscBisqwitOptions(next),
                            );
                            _scheduleApplyNtscBisqwitOptions(next);
                          },
                        ),
                      ),
                      Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16),
                        child: AnimatedSliderTile(
                          label: l10n.videoNtscBrightnessLabel,
                          value: videoSettings.ntscBisqwitOptions.brightness,
                          min: -1,
                          max: 1,
                          divisions: 200,
                          valueLabel: videoSettings
                              .ntscBisqwitOptions
                              .brightness
                              .toStringAsFixed(2),
                          onChanged: (value) {
                            final o = videoSettings.ntscBisqwitOptions;
                            final next = nes_video.NtscBisqwitOptions(
                              brightness: value,
                              contrast: o.contrast,
                              hue: o.hue,
                              saturation: o.saturation,
                              yFilterLength: o.yFilterLength,
                              iFilterLength: o.iFilterLength,
                              qFilterLength: o.qFilterLength,
                            );
                            unawaited(
                              videoController.setNtscBisqwitOptions(next),
                            );
                            _scheduleApplyNtscBisqwitOptions(next);
                          },
                        ),
                      ),
                      Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16),
                        child: AnimatedSliderTile(
                          label: l10n.videoNtscBisqwitYFilterLengthLabel,
                          value: videoSettings.ntscBisqwitOptions.yFilterLength,
                          min: -0.46,
                          max: 4,
                          divisions: 446,
                          valueLabel:
                              (videoSettings.ntscBisqwitOptions.yFilterLength *
                                      100)
                                  .round()
                                  .toString(),
                          onChanged: (value) {
                            final o = videoSettings.ntscBisqwitOptions;
                            final next = nes_video.NtscBisqwitOptions(
                              brightness: o.brightness,
                              contrast: o.contrast,
                              hue: o.hue,
                              saturation: o.saturation,
                              yFilterLength: value,
                              iFilterLength: o.iFilterLength,
                              qFilterLength: o.qFilterLength,
                            );
                            unawaited(
                              videoController.setNtscBisqwitOptions(next),
                            );
                            _scheduleApplyNtscBisqwitOptions(next);
                          },
                        ),
                      ),
                      Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16),
                        child: AnimatedSliderTile(
                          label: l10n.videoNtscBisqwitIFilterLengthLabel,
                          value: videoSettings.ntscBisqwitOptions.iFilterLength,
                          min: 0,
                          max: 4,
                          divisions: 400,
                          valueLabel:
                              (videoSettings.ntscBisqwitOptions.iFilterLength *
                                      100)
                                  .round()
                                  .toString(),
                          onChanged: (value) {
                            final o = videoSettings.ntscBisqwitOptions;
                            final next = nes_video.NtscBisqwitOptions(
                              brightness: o.brightness,
                              contrast: o.contrast,
                              hue: o.hue,
                              saturation: o.saturation,
                              yFilterLength: o.yFilterLength,
                              iFilterLength: value,
                              qFilterLength: o.qFilterLength,
                            );
                            unawaited(
                              videoController.setNtscBisqwitOptions(next),
                            );
                            _scheduleApplyNtscBisqwitOptions(next);
                          },
                        ),
                      ),
                      Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16),
                        child: AnimatedSliderTile(
                          label: l10n.videoNtscBisqwitQFilterLengthLabel,
                          value: videoSettings.ntscBisqwitOptions.qFilterLength,
                          min: 0,
                          max: 4,
                          divisions: 400,
                          valueLabel:
                              (videoSettings.ntscBisqwitOptions.qFilterLength *
                                      100)
                                  .round()
                                  .toString(),
                          onChanged: (value) {
                            final o = videoSettings.ntscBisqwitOptions;
                            final next = nes_video.NtscBisqwitOptions(
                              brightness: o.brightness,
                              contrast: o.contrast,
                              hue: o.hue,
                              saturation: o.saturation,
                              yFilterLength: o.yFilterLength,
                              iFilterLength: o.iFilterLength,
                              qFilterLength: value,
                            );
                            unawaited(
                              videoController.setNtscBisqwitOptions(next),
                            );
                            _scheduleApplyNtscBisqwitOptions(next);
                          },
                        ),
                      ),
                    ],
                  ),
                if (videoSettings.videoFilter ==
                        nes_video.VideoFilter.ntscBisqwit2X ||
                    videoSettings.videoFilter ==
                        nes_video.VideoFilter.ntscBisqwit4X ||
                    videoSettings.videoFilter ==
                        nes_video.VideoFilter.ntscBisqwit8X)
                  const SizedBox(height: 12),
                if (videoSettings.videoFilter ==
                        nes_video.VideoFilter.ntscComposite ||
                    videoSettings.videoFilter ==
                        nes_video.VideoFilter.ntscSVideo ||
                    videoSettings.videoFilter ==
                        nes_video.VideoFilter.ntscRgb ||
                    videoSettings.videoFilter ==
                        nes_video.VideoFilter.ntscMonochrome)
                  AnimatedExpansionTile(
                    labelText: l10n.videoNtscAdvancedTitle,
                    title: Text(l10n.keyboardPresetCustom),
                    initiallyExpanded: false,
                    children: [
                      Padding(
                        padding: const EdgeInsets.symmetric(
                          horizontal: 16,
                          vertical: 8,
                        ),
                        child: SwitchListTile(
                          contentPadding: EdgeInsets.zero,
                          title: Text(l10n.videoNtscMergeFieldsLabel),
                          value: videoSettings.ntscOptions.mergeFields,
                          onChanged: (value) async {
                            final o = videoSettings.ntscOptions;
                            final next = nes_video.NtscOptions(
                              hue: o.hue,
                              saturation: o.saturation,
                              contrast: o.contrast,
                              brightness: o.brightness,
                              sharpness: o.sharpness,
                              gamma: o.gamma,
                              resolution: o.resolution,
                              artifacts: o.artifacts,
                              fringing: o.fringing,
                              bleed: o.bleed,
                              mergeFields: value,
                            );
                            unawaited(videoController.setNtscOptions(next));
                            _scheduleApplyNtscOptions(next);
                          },
                        ),
                      ),
                      Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16),
                        child: AnimatedSliderTile(
                          label: l10n.videoNtscHueLabel,
                          value: videoSettings.ntscOptions.hue,
                          min: -1,
                          max: 1,
                          divisions: 200,
                          valueLabel: videoSettings.ntscOptions.hue
                              .toStringAsFixed(2),
                          onChanged: (value) async {
                            final o = videoSettings.ntscOptions;
                            final next = nes_video.NtscOptions(
                              hue: value,
                              saturation: o.saturation,
                              contrast: o.contrast,
                              brightness: o.brightness,
                              sharpness: o.sharpness,
                              gamma: o.gamma,
                              resolution: o.resolution,
                              artifacts: o.artifacts,
                              fringing: o.fringing,
                              bleed: o.bleed,
                              mergeFields: o.mergeFields,
                            );
                            unawaited(videoController.setNtscOptions(next));
                            _scheduleApplyNtscOptions(next);
                          },
                        ),
                      ),
                      Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16),
                        child: AnimatedSliderTile(
                          label: l10n.videoNtscSaturationLabel,
                          value: videoSettings.ntscOptions.saturation,
                          min: -1,
                          max: 1,
                          divisions: 200,
                          valueLabel: videoSettings.ntscOptions.saturation
                              .toStringAsFixed(2),
                          onChanged: (value) async {
                            final o = videoSettings.ntscOptions;
                            final next = nes_video.NtscOptions(
                              hue: o.hue,
                              saturation: value,
                              contrast: o.contrast,
                              brightness: o.brightness,
                              sharpness: o.sharpness,
                              gamma: o.gamma,
                              resolution: o.resolution,
                              artifacts: o.artifacts,
                              fringing: o.fringing,
                              bleed: o.bleed,
                              mergeFields: o.mergeFields,
                            );
                            unawaited(videoController.setNtscOptions(next));
                            _scheduleApplyNtscOptions(next);
                          },
                        ),
                      ),
                      Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16),
                        child: AnimatedSliderTile(
                          label: l10n.videoNtscContrastLabel,
                          value: videoSettings.ntscOptions.contrast,
                          min: -1,
                          max: 1,
                          divisions: 200,
                          valueLabel: videoSettings.ntscOptions.contrast
                              .toStringAsFixed(2),
                          onChanged: (value) async {
                            final o = videoSettings.ntscOptions;
                            final next = nes_video.NtscOptions(
                              hue: o.hue,
                              saturation: o.saturation,
                              contrast: value,
                              brightness: o.brightness,
                              sharpness: o.sharpness,
                              gamma: o.gamma,
                              resolution: o.resolution,
                              artifacts: o.artifacts,
                              fringing: o.fringing,
                              bleed: o.bleed,
                              mergeFields: o.mergeFields,
                            );
                            unawaited(videoController.setNtscOptions(next));
                            _scheduleApplyNtscOptions(next);
                          },
                        ),
                      ),
                      Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16),
                        child: AnimatedSliderTile(
                          label: l10n.videoNtscBrightnessLabel,
                          value: videoSettings.ntscOptions.brightness,
                          min: -1,
                          max: 1,
                          divisions: 200,
                          valueLabel: videoSettings.ntscOptions.brightness
                              .toStringAsFixed(2),
                          onChanged: (value) async {
                            final o = videoSettings.ntscOptions;
                            final next = nes_video.NtscOptions(
                              hue: o.hue,
                              saturation: o.saturation,
                              contrast: o.contrast,
                              brightness: value,
                              sharpness: o.sharpness,
                              gamma: o.gamma,
                              resolution: o.resolution,
                              artifacts: o.artifacts,
                              fringing: o.fringing,
                              bleed: o.bleed,
                              mergeFields: o.mergeFields,
                            );
                            unawaited(videoController.setNtscOptions(next));
                            _scheduleApplyNtscOptions(next);
                          },
                        ),
                      ),
                      Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16),
                        child: AnimatedSliderTile(
                          label: l10n.videoNtscSharpnessLabel,
                          value: videoSettings.ntscOptions.sharpness,
                          min: -1,
                          max: 1,
                          divisions: 200,
                          valueLabel: videoSettings.ntscOptions.sharpness
                              .toStringAsFixed(2),
                          onChanged: (value) async {
                            final o = videoSettings.ntscOptions;
                            final next = nes_video.NtscOptions(
                              hue: o.hue,
                              saturation: o.saturation,
                              contrast: o.contrast,
                              brightness: o.brightness,
                              sharpness: value,
                              gamma: o.gamma,
                              resolution: o.resolution,
                              artifacts: o.artifacts,
                              fringing: o.fringing,
                              bleed: o.bleed,
                              mergeFields: o.mergeFields,
                            );
                            unawaited(videoController.setNtscOptions(next));
                            _scheduleApplyNtscOptions(next);
                          },
                        ),
                      ),
                      Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16),
                        child: AnimatedSliderTile(
                          label: l10n.videoNtscGammaLabel,
                          value: videoSettings.ntscOptions.gamma,
                          min: -1,
                          max: 1,
                          divisions: 200,
                          valueLabel: videoSettings.ntscOptions.gamma
                              .toStringAsFixed(2),
                          onChanged: (value) async {
                            final o = videoSettings.ntscOptions;
                            final next = nes_video.NtscOptions(
                              hue: o.hue,
                              saturation: o.saturation,
                              contrast: o.contrast,
                              brightness: o.brightness,
                              sharpness: o.sharpness,
                              gamma: value,
                              resolution: o.resolution,
                              artifacts: o.artifacts,
                              fringing: o.fringing,
                              bleed: o.bleed,
                              mergeFields: o.mergeFields,
                            );
                            unawaited(videoController.setNtscOptions(next));
                            _scheduleApplyNtscOptions(next);
                          },
                        ),
                      ),
                      Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16),
                        child: AnimatedSliderTile(
                          label: l10n.videoNtscResolutionLabel,
                          value: videoSettings.ntscOptions.resolution,
                          min: -1,
                          max: 1,
                          divisions: 200,
                          valueLabel: videoSettings.ntscOptions.resolution
                              .toStringAsFixed(2),
                          onChanged: (value) async {
                            final o = videoSettings.ntscOptions;
                            final next = nes_video.NtscOptions(
                              hue: o.hue,
                              saturation: o.saturation,
                              contrast: o.contrast,
                              brightness: o.brightness,
                              sharpness: o.sharpness,
                              gamma: o.gamma,
                              resolution: value,
                              artifacts: o.artifacts,
                              fringing: o.fringing,
                              bleed: o.bleed,
                              mergeFields: o.mergeFields,
                            );
                            unawaited(videoController.setNtscOptions(next));
                            _scheduleApplyNtscOptions(next);
                          },
                        ),
                      ),
                      Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16),
                        child: AnimatedSliderTile(
                          label: l10n.videoNtscArtifactsLabel,
                          value: videoSettings.ntscOptions.artifacts,
                          min: -1,
                          max: 1,
                          divisions: 200,
                          valueLabel: videoSettings.ntscOptions.artifacts
                              .toStringAsFixed(2),
                          onChanged: (value) async {
                            final o = videoSettings.ntscOptions;
                            final next = nes_video.NtscOptions(
                              hue: o.hue,
                              saturation: o.saturation,
                              contrast: o.contrast,
                              brightness: o.brightness,
                              sharpness: o.sharpness,
                              gamma: o.gamma,
                              resolution: o.resolution,
                              artifacts: value,
                              fringing: o.fringing,
                              bleed: o.bleed,
                              mergeFields: o.mergeFields,
                            );
                            unawaited(videoController.setNtscOptions(next));
                            _scheduleApplyNtscOptions(next);
                          },
                        ),
                      ),
                      Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16),
                        child: AnimatedSliderTile(
                          label: l10n.videoNtscFringingLabel,
                          value: videoSettings.ntscOptions.fringing,
                          min: -1,
                          max: 1,
                          divisions: 200,
                          valueLabel: videoSettings.ntscOptions.fringing
                              .toStringAsFixed(2),
                          onChanged: (value) async {
                            final o = videoSettings.ntscOptions;
                            final next = nes_video.NtscOptions(
                              hue: o.hue,
                              saturation: o.saturation,
                              contrast: o.contrast,
                              brightness: o.brightness,
                              sharpness: o.sharpness,
                              gamma: o.gamma,
                              resolution: o.resolution,
                              artifacts: o.artifacts,
                              fringing: value,
                              bleed: o.bleed,
                              mergeFields: o.mergeFields,
                            );
                            unawaited(videoController.setNtscOptions(next));
                            _scheduleApplyNtscOptions(next);
                          },
                        ),
                      ),
                      Padding(
                        padding: const EdgeInsets.symmetric(horizontal: 16),
                        child: AnimatedSliderTile(
                          label: l10n.videoNtscBleedLabel,
                          value: videoSettings.ntscOptions.bleed,
                          min: -1,
                          max: 1,
                          divisions: 200,
                          valueLabel: videoSettings.ntscOptions.bleed
                              .toStringAsFixed(2),
                          onChanged: (value) async {
                            final o = videoSettings.ntscOptions;
                            final next = nes_video.NtscOptions(
                              hue: o.hue,
                              saturation: o.saturation,
                              contrast: o.contrast,
                              brightness: o.brightness,
                              sharpness: o.sharpness,
                              gamma: o.gamma,
                              resolution: o.resolution,
                              artifacts: o.artifacts,
                              fringing: o.fringing,
                              bleed: value,
                              mergeFields: o.mergeFields,
                            );
                            unawaited(videoController.setNtscOptions(next));
                            _scheduleApplyNtscOptions(next);
                          },
                        ),
                      ),
                      const SizedBox(height: 8),
                    ],
                  ),
                const SizedBox(height: 12),
                dropdown<PaletteMode>(
                  labelText: l10n.paletteModeLabel,
                  value: videoSettings.paletteMode,
                  entries: [
                    DropdownMenuEntry(
                      value: PaletteMode.builtin,
                      label: l10n.paletteModeBuiltin,
                    ),
                    DropdownMenuEntry(
                      value: PaletteMode.custom,
                      label: videoSettings.customPaletteName == null
                          ? l10n.paletteModeCustom
                          : l10n.paletteModeCustomActive(
                              videoSettings.customPaletteName!,
                            ),
                    ),
                  ],
                  onSelected: onPaletteModeSelected,
                ),
                const SizedBox(height: 12),
                if (videoSettings.paletteMode == PaletteMode.builtin)
                  dropdown<nes_palette.PaletteKind>(
                    labelText: l10n.builtinPaletteLabel,
                    value: videoSettings.builtinPreset,
                    entries: const [
                      DropdownMenuEntry(
                        value: nes_palette.PaletteKind.nesdevNtsc,
                        label: 'Nesdev (NTSC)',
                      ),
                      DropdownMenuEntry(
                        value: nes_palette.PaletteKind.fbxCompositeDirect,
                        label: 'FirebrandX (Composite Direct)',
                      ),
                      DropdownMenuEntry(
                        value: nes_palette.PaletteKind.sonyCxa2025AsUs,
                        label: 'Sony CXA2025AS (US)',
                      ),
                      DropdownMenuEntry(
                        value: nes_palette.PaletteKind.pal2C07,
                        label: 'RP2C07 (PAL)',
                      ),
                      DropdownMenuEntry(
                        value: nes_palette.PaletteKind.rawLinear,
                        label: 'Raw linear',
                      ),
                    ],
                    onSelected: setBuiltinPalette,
                  )
                else
                  ListTile(
                    contentPadding: EdgeInsets.zero,
                    title: Text(l10n.customPaletteLoadTitle),
                    subtitle: Text(
                      videoSettings.customPaletteName == null
                          ? l10n.customPaletteLoadSubtitle
                          : l10n.paletteModeCustomActive(
                              videoSettings.customPaletteName!,
                            ),
                    ),
                    trailing: IconButton.filledTonal(
                      tooltip: l10n.actionLoadPalette,
                      icon: const Icon(Icons.folder_open),
                      onPressed: () => widget.pickAndApplyCustomPalette(
                        context,
                        videoController,
                      ),
                    ),
                    onTap: () => widget.pickAndApplyCustomPalette(
                      context,
                      videoController,
                    ),
                  ),
              ],
            ),
          ),
        ),
        const SizedBox(height: 12),
        AnimatedSettingsCard(
          index: 1,
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: Column(
              children: [
                SwitchListTile(
                  contentPadding: EdgeInsets.zero,
                  value: videoSettings.integerScaling,
                  title: Text(l10n.videoIntegerScalingTitle),
                  subtitle: Text(l10n.videoIntegerScalingSubtitle),
                  secondary: const Icon(Icons.grid_on),
                  onChanged: (value) async {
                    try {
                      await videoController.setIntegerScaling(value);
                    } catch (e, st) {
                      logWarning(
                        e,
                        stackTrace: st,
                        message: 'setIntegerScaling failed',
                        logger: 'settings_page',
                      );
                    }
                  },
                ),
                const SizedBox(height: 12),
                dropdown<NesAspectRatio>(
                  labelText: l10n.videoAspectRatio,
                  value: videoSettings.aspectRatio,
                  entries: [
                    DropdownMenuEntry(
                      value: NesAspectRatio.square,
                      label: l10n.videoAspectRatioSquare,
                    ),
                    DropdownMenuEntry(
                      value: NesAspectRatio.ntsc,
                      label: l10n.videoAspectRatioNtsc,
                    ),
                    DropdownMenuEntry(
                      value: NesAspectRatio.stretch,
                      label: l10n.videoAspectRatioStretch,
                    ),
                  ],
                  onSelected: setAspectRatio,
                ),
                AnimatedSliderTile(
                  label: l10n.videoScreenVerticalOffset,
                  value: videoSettings.screenVerticalOffset,
                  min: -240,
                  max: 240,
                  divisions: 96,
                  onChanged: (v) => videoController.setScreenVerticalOffset(
                    v.roundToDouble(),
                  ),
                  valueLabel:
                      '${videoSettings.screenVerticalOffset.toStringAsFixed(0)} px',
                ),
                if (isAndroid) ...[
                  const SizedBox(height: 12),
                  dropdown<AndroidVideoBackend>(
                    labelText: l10n.videoBackendAndroidLabel,
                    helperText: l10n.videoBackendRestartHint,
                    value: androidBackend.backend,
                    entries: [
                      DropdownMenuEntry(
                        value: AndroidVideoBackend.hardware,
                        label: l10n.videoBackendHardware,
                      ),
                      DropdownMenuEntry(
                        value: AndroidVideoBackend.upload,
                        label: l10n.videoBackendUpload,
                      ),
                    ],
                    onSelected: setAndroidBackend,
                  ),
                  const SizedBox(height: 16),
                  SwitchListTile(
                    contentPadding: EdgeInsets.zero,
                    secondary: const Icon(Icons.rocket_launch),
                    title: Text(l10n.highPerformanceModeLabel),
                    subtitle: Text(l10n.highPerformanceModeDescription),
                    value: androidPerformance.highPerformance,
                    onChanged: androidPerformanceController == null
                        ? null
                        : (value) => androidPerformanceController
                              .setHighPerformance(value),
                  ),
                ],
                if (isWindows) ...[
                  const SizedBox(height: 12),
                  dropdown<WindowsVideoBackend>(
                    labelText: l10n.videoBackendWindowsLabel,
                    value: windowsBackend.backend,
                    entries: [
                      DropdownMenuEntry(
                        value: WindowsVideoBackend.d3d11Gpu,
                        label: 'D3D11 GPU (Zero-Copy)',
                      ),
                      DropdownMenuEntry(
                        value: WindowsVideoBackend.softwareCpu,
                        label: 'Software CPU (Fallback)',
                      ),
                    ],
                    onSelected: setWindowsBackend,
                  ),
                  const SizedBox(height: 16),
                  SwitchListTile(
                    contentPadding: EdgeInsets.zero,
                    secondary: const Icon(Icons.rocket_launch),
                    title: Text(l10n.highPerformanceModeLabel),
                    subtitle: Text(l10n.highPerformanceModeDescription),
                    value: windowsPerformance.highPerformance,
                    onChanged: windowsPerformanceController == null
                        ? null
                        : (value) => windowsPerformanceController
                              .setHighPerformance(value),
                  ),
                ],
              ],
            ),
          ),
        ),
        if (isAndroid && androidShaderSettings != null) ...[
          const SizedBox(height: 12),
          AnimatedSettingsCard(
            index: 2,
            child: Padding(
              padding: const EdgeInsets.all(12),
              child: Column(
                children: [
                  SwitchListTile(
                    contentPadding: EdgeInsets.zero,
                    secondary: const Icon(Icons.auto_fix_high),
                    title: Text(l10n.videoShaderLibrashaderTitle),
                    subtitle: Text(
                      androidBackend.backend == AndroidVideoBackend.hardware
                          ? l10n.videoShaderLibrashaderSubtitle
                          : l10n.videoShaderLibrashaderSubtitleDisabled,
                    ),
                    value: androidShaderSettings.enabled,
                    onChanged:
                        androidBackend.backend == AndroidVideoBackend.hardware
                        ? (value) async {
                            try {
                              await androidShaderController?.setEnabled(value);
                            } catch (e, st) {
                              logWarning(
                                e,
                                stackTrace: st,
                                message: 'setEnabled failed',
                                logger: 'settings_page',
                              );
                            }
                          }
                        : null,
                  ),
                  const Divider(height: 1),
                  ListTile(
                    contentPadding: EdgeInsets.zero,
                    leading: const Icon(Icons.description_outlined),
                    title: Text(l10n.videoShaderPresetLabel),
                    subtitle: Text(
                      androidShaderSettings.presetPath ??
                          l10n.videoShaderPresetNotSet,
                      maxLines: 2,
                      overflow: TextOverflow.ellipsis,
                    ),
                    trailing: const Icon(Icons.folder_open),
                    onTap:
                        androidBackend.backend == AndroidVideoBackend.hardware
                        ? pickAndSetShaderPreset
                        : null,
                    onLongPress: androidShaderSettings.presetPath == null
                        ? null
                        : () async {
                            try {
                              await androidShaderController?.setPresetPath(
                                null,
                              );
                            } catch (e, st) {
                              logWarning(
                                e,
                                stackTrace: st,
                                message: 'clear preset failed',
                                logger: 'settings_page',
                              );
                            }
                          },
                  ),
                ],
              ),
            ),
          ),
        ],
      ],
    );
  }
}

// ============================================================================
// Emulation Tab
// ============================================================================

class _EmulationTab extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final colorScheme = Theme.of(context).colorScheme;
    final emulationSettings = ref.watch(emulationSettingsProvider);
    final emulationController = ref.read(emulationSettingsProvider.notifier);

    return ListView(
      padding: const EdgeInsets.all(20),
      children: [
        AnimatedSectionHeader(
          title: l10n.emulationTitle,
          icon: Icons.developer_board,
          delay: const Duration(milliseconds: 100),
        ),
        const SizedBox(height: 8),
        AnimatedSettingsCard(
          index: 0,
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: Column(
              children: [
                SwitchListTile(
                  contentPadding: EdgeInsets.zero,
                  secondary: const Icon(Icons.speed),
                  value: emulationSettings.integerFpsMode,
                  title: Text(l10n.integerFpsTitle),
                  subtitle: Text(l10n.integerFpsSubtitle),
                  onChanged: emulationController.setIntegerFpsMode,
                ),
                const Divider(height: 1),
                SwitchListTile(
                  contentPadding: EdgeInsets.zero,
                  secondary: const Icon(Icons.pause_circle_outline),
                  title: Text(l10n.pauseInBackgroundTitle),
                  subtitle: Text(l10n.pauseInBackgroundSubtitle),
                  value: emulationSettings.pauseInBackground,
                  onChanged: emulationController.setPauseInBackground,
                ),
                const Divider(height: 1),
                SwitchListTile(
                  contentPadding: EdgeInsets.zero,
                  secondary: const Icon(Icons.visibility),
                  title: Text(l10n.showOverlayTitle),
                  subtitle: Text(l10n.showOverlaySubtitle),
                  value: emulationSettings.showEmulationStatusOverlay,
                  onChanged: emulationController.setShowEmulationStatusOverlay,
                ),
              ],
            ),
          ),
        ),
        const SizedBox(height: 12),
        AnimatedSettingsCard(
          index: 1,
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: Column(
              children: [
                SwitchListTile(
                  contentPadding: EdgeInsets.zero,
                  secondary: const Icon(Icons.save_outlined),
                  title: Text(l10n.autoSaveEnabledTitle),
                  subtitle: Text(l10n.autoSaveEnabledSubtitle),
                  value: emulationSettings.autoSaveEnabled,
                  onChanged: emulationController.setAutoSaveEnabled,
                ),
                ClipRect(
                  child: AnimatedSwitcher(
                    duration: const Duration(milliseconds: 220),
                    reverseDuration: const Duration(milliseconds: 180),
                    switchInCurve: Curves.easeOutCubic,
                    switchOutCurve: Curves.easeInCubic,
                    transitionBuilder: (child, animation) {
                      return FadeTransition(
                        opacity: animation,
                        child: SizeTransition(
                          sizeFactor: animation,
                          axisAlignment: -1,
                          child: child,
                        ),
                      );
                    },
                    child: emulationSettings.autoSaveEnabled
                        ? Padding(
                            key: const ValueKey('autoSaveInterval'),
                            padding: const EdgeInsets.fromLTRB(56, 0, 0, 4),
                            child: AnimatedSliderTile(
                              label: l10n.autoSaveIntervalTitle,
                              value: emulationSettings.autoSaveIntervalInMinutes
                                  .toDouble(),
                              min: 1,
                              max: 60,
                              divisions: 59,
                              onChanged: (v) => emulationController
                                  .setAutoSaveIntervalInMinutes(v.toInt()),
                              valueLabel: l10n.autoSaveIntervalValue(
                                emulationSettings.autoSaveIntervalInMinutes,
                              ),
                            ),
                          )
                        : const SizedBox.shrink(
                            key: ValueKey('autoSaveIntervalEmpty'),
                          ),
                  ),
                ),
              ],
            ),
          ),
        ),
        const SizedBox(height: 12),
        AnimatedSettingsCard(
          index: 2,
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: AnimatedDropdownMenu<int>(
              labelText: l10n.quickSaveSlotTitle,
              helperText: l10n.quickSaveSlotSubtitle,
              value: emulationSettings.quickSaveSlot,
              entries: [
                for (int i = 1; i <= 10; i++)
                  DropdownMenuEntry(
                    value: i,
                    label: l10n.quickSaveSlotValue(i),
                  ),
              ],
              onSelected: (value) =>
                  emulationController.setQuickSaveSlot(value),
            ),
          ),
        ),
        const SizedBox(height: 12),
        AnimatedSettingsCard(
          index: 3,
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                AnimatedSliderTile(
                  label: l10n.fastForwardSpeedTitle,
                  value: emulationSettings.fastForwardSpeedPercent.toDouble(),
                  min: 100,
                  max: 1000,
                  divisions: 9,
                  onChanged: (v) =>
                      emulationController.setFastForwardSpeedPercent(v.round()),
                  valueLabel: l10n.fastForwardSpeedValue(
                    emulationSettings.fastForwardSpeedPercent,
                  ),
                ),
                const SizedBox(height: 4),
                Text(
                  l10n.fastForwardSpeedSubtitle,
                  style: Theme.of(context).textTheme.bodySmall?.copyWith(
                    color: colorScheme.onSurfaceVariant,
                  ),
                ),
              ],
            ),
          ),
        ),
        const SizedBox(height: 12),
        AnimatedSettingsCard(
          index: 4,
          child: Padding(
            padding: const EdgeInsets.all(12),
            child: Column(
              children: [
                SwitchListTile(
                  contentPadding: EdgeInsets.zero,
                  secondary: const Icon(Icons.history_toggle_off),
                  title: Text(l10n.rewindEnabledTitle),
                  subtitle: Text(l10n.rewindEnabledSubtitle),
                  value: emulationSettings.rewindEnabled,
                  onChanged: emulationController.setRewindEnabled,
                ),
                ClipRect(
                  child: AnimatedSwitcher(
                    duration: const Duration(milliseconds: 220),
                    reverseDuration: const Duration(milliseconds: 180),
                    switchInCurve: Curves.easeOutCubic,
                    switchOutCurve: Curves.easeInCubic,
                    transitionBuilder: (child, animation) {
                      return FadeTransition(
                        opacity: animation,
                        child: SizeTransition(
                          sizeFactor: animation,
                          axisAlignment: -1,
                          child: child,
                        ),
                      );
                    },
                    child: emulationSettings.rewindEnabled
                        ? Padding(
                            key: const ValueKey('rewindSettings'),
                            padding: const EdgeInsets.fromLTRB(56, 0, 0, 4),
                            child: Column(
                              children: [
                                AnimatedSliderTile(
                                  label: l10n.rewindMinutesTitle,
                                  value: emulationSettings.rewindSeconds
                                      .toDouble(),
                                  min: 60,
                                  max: 3600,
                                  divisions: 59,
                                  onChanged: (v) => emulationController
                                      .setRewindSeconds(v.toInt()),
                                  valueLabel: l10n.rewindMinutesValue(
                                    emulationSettings.rewindSeconds ~/ 60,
                                  ),
                                ),
                                const SizedBox(height: 12),
                                AnimatedSliderTile(
                                  label: l10n.rewindSpeedTitle,
                                  value: emulationSettings.rewindSpeedPercent
                                      .toDouble(),
                                  min: 100,
                                  max: 1000,
                                  divisions: 9,
                                  onChanged: (v) => emulationController
                                      .setRewindSpeedPercent(v.round()),
                                  valueLabel: l10n.rewindSpeedValue(
                                    emulationSettings.rewindSpeedPercent,
                                  ),
                                ),
                                const SizedBox(height: 4),
                                Align(
                                  alignment: Alignment.centerLeft,
                                  child: Text(
                                    l10n.rewindSpeedSubtitle,
                                    style: Theme.of(context).textTheme.bodySmall
                                        ?.copyWith(
                                          color: colorScheme.onSurfaceVariant,
                                        ),
                                  ),
                                ),
                              ],
                            ),
                          )
                        : const SizedBox.shrink(
                            key: ValueKey('rewindSettingsEmpty'),
                          ),
                  ),
                ),
              ],
            ),
          ),
        ),
      ],
    );
  }
}

// ============================================================================
// Server Tab
// ============================================================================

class _ServerTab extends ConsumerStatefulWidget {
  const _ServerTab();

  @override
  ConsumerState<_ServerTab> createState() => _ServerTabState();
}

class _ServerTabState extends ConsumerState<_ServerTab> {
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
                              children: [
                                ...previousChildren,
                                if (currentChild != null) currentChild,
                              ],
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

// ============================================================================
// Helper Functions and Classes
// ============================================================================

String _keyLabel(AppLocalizations l10n, LogicalKeyboardKey? key) {
  if (key == null) return l10n.unassignedKey;
  final label = key.keyLabel.trim();
  if (label.isNotEmpty) return label;
  return key.debugName ?? 'Key 0x${key.keyId.toRadixString(16)}';
}

@immutable
class _KeyCaptureResult {
  const _KeyCaptureResult({required this.key});
  final LogicalKeyboardKey? key;
}

class _KeyCaptureDialog extends ConsumerStatefulWidget {
  const _KeyCaptureDialog({
    required this.title,
    required this.current,
    required this.selectedPort,
    required this.selectedAction,
  });

  final String title;
  final LogicalKeyboardKey? current;
  final int selectedPort;
  final KeyboardBindingAction selectedAction;

  @override
  ConsumerState<_KeyCaptureDialog> createState() => _KeyCaptureDialogState();
}

class _KeyCaptureDialogState extends ConsumerState<_KeyCaptureDialog> {
  final FocusNode _focusNode = FocusNode();
  LogicalKeyboardKey? _last;

  @override
  void dispose() {
    _focusNode.dispose();
    super.dispose();
  }

  KeyEventResult _onKeyEvent(FocusNode _, KeyEvent event) {
    if (event is! KeyDownEvent) return KeyEventResult.handled;

    final key = event.logicalKey;
    if (key == LogicalKeyboardKey.escape) {
      Navigator.of(context).pop(const _KeyCaptureResult(key: null));
      return KeyEventResult.handled;
    }
    if (key == LogicalKeyboardKey.unidentified) {
      return KeyEventResult.handled;
    }

    setState(() => _last = key);

    // If there's no conflict, or we want to pop immediately anyway?
    // User said "不阻止玩家设置", so we can just pop.
    // But maybe wait a bit to show the hint?
    // Let's pop immediately for now to keep it snappy.
    Navigator.of(context).pop(_KeyCaptureResult(key: key));
    return KeyEventResult.handled;
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final inputState = ref.watch(inputSettingsProvider);
    final currentConflict = widget.current != null
        ? inputState.findConflict(
            widget.current!,
            excludePort: widget.selectedPort,
            excludeAction: widget.selectedAction,
          )
        : null;

    return AlertDialog(
      title: Text(widget.title),
      content: Focus(
        autofocus: true,
        focusNode: _focusNode,
        onKeyEvent: _onKeyEvent,
        child: ConstrainedBox(
          constraints: const BoxConstraints(minWidth: 280),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(l10n.keyCapturePressKeyToBind),
              const SizedBox(height: 12),
              Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    l10n.keyCaptureCurrent(_keyLabel(l10n, widget.current)),
                    style: const TextStyle(fontWeight: FontWeight.w500),
                  ),
                  if (currentConflict != null) ...[
                    const SizedBox(height: 2),
                    Text(
                      l10n.inputBindingCapturedConflictHint(
                        switch (currentConflict.port) {
                          0 => l10n.player1,
                          1 => l10n.player2,
                          2 => l10n.player3,
                          3 => l10n.player4,
                          _ => 'P${currentConflict.port + 1}',
                        },
                        _SettingsPageState._actionLabel(
                          l10n,
                          currentConflict.action,
                        ),
                      ),
                      style: TextStyle(
                        color: Theme.of(context).colorScheme.secondary,
                        fontSize: 12,
                      ),
                    ),
                  ],
                ],
              ),
              if (_last != null) ...[
                const SizedBox(height: 8),
                Text(l10n.keyCaptureCaptured(_keyLabel(l10n, _last))),
              ],
              const SizedBox(height: 16),
              Text(
                l10n.keyCapturePressEscToClear,
                style: Theme.of(context).textTheme.bodySmall,
              ),
            ],
          ),
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: Text(l10n.cancel),
        ),
      ],
    );
  }
}

// ============================================================================
// Connected Gamepads Card
// ============================================================================

class _ConnectedGamepadsCard extends ConsumerWidget {
  const _ConnectedGamepadsCard();

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

class RemapLocation {
  final Object action;
  final int port;
  const RemapLocation(this.action, this.port);
}

class RemappingNotifier extends Notifier<RemapLocation?> {
  @override
  RemapLocation? build() => null;
  void update(RemapLocation? val) => state = val;
}

/// State for the remapping process.
final remappingStateProvider =
    NotifierProvider<RemappingNotifier, RemapLocation?>(RemappingNotifier.new);

enum NesButtonAction {
  a,
  b,
  select,
  start,
  up,
  down,
  left,
  right,
  turboA,
  turboB,
  rewind,
  fastForward,
  saveState,
  loadState,
  pause,
}

extension NesButtonActionExt on NesButtonAction {
  bool get isCore => index <= NesButtonAction.turboB.index;
  bool get isExtended => index > NesButtonAction.turboB.index;
}

class _GamepadMappingInfoCard extends ConsumerStatefulWidget {
  final int port;

  const _GamepadMappingInfoCard({required this.port});

  @override
  ConsumerState<_GamepadMappingInfoCard> createState() =>
      _GamepadMappingInfoCardState();
}

class _GamepadMappingInfoCardState
    extends ConsumerState<_GamepadMappingInfoCard> {
  Timer? _remappingTimer;

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
              _SettingsPageState._actionLabel(
                AppLocalizations.of(context)!,
                action,
              ),
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
                                              label:
                                                  _SettingsPageState._actionLabel(
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

  @override
  void dispose() {
    _remappingTimer?.cancel();
    super.dispose();
  }
}

class _KeyboardMappingInfoCard extends ConsumerStatefulWidget {
  final int port;
  const _KeyboardMappingInfoCard({required this.port});

  @override
  ConsumerState<_KeyboardMappingInfoCard> createState() =>
      _KeyboardMappingInfoCardState();
}

class _KeyboardMappingInfoCardState
    extends ConsumerState<_KeyboardMappingInfoCard> {
  Timer? _remappingTimer;

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

  @override
  void dispose() {
    _remappingTimer?.cancel();
    super.dispose();
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
                                    label: _SettingsPageState._actionLabel(
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
                              in KeyboardBindingAction.values.where(
                                (a) => a.isExtended,
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
                                    label: _SettingsPageState._actionLabel(
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
