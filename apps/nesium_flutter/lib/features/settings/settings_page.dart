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
import '../controls/input_settings.dart';
import '../controls/turbo_settings.dart';
import '../controls/virtual_controls_editor.dart';
import '../controls/virtual_controls_settings.dart';
import 'android_video_backend_settings.dart';
import 'emulation_settings.dart';
import 'language_settings.dart';
import 'theme_settings.dart';
import 'video_settings.dart';
import 'server_settings.dart';
import '../../platform/nes_palette.dart' as nes_palette;

class SettingsPage extends ConsumerStatefulWidget {
  const SettingsPage({super.key});

  @override
  ConsumerState<SettingsPage> createState() => _SettingsPageState();
}

class _SettingsPageState extends ConsumerState<SettingsPage>
    with SingleTickerProviderStateMixin {
  late TabController _tabController;

  @override
  void initState() {
    super.initState();
    _tabController = TabController(length: 5, vsync: this);
  }

  @override
  void dispose() {
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

  Future<void> _editCustomBinding(
    BuildContext context,
    InputSettingsController controller,
    InputSettings settings,
    KeyboardBindingAction action,
  ) async {
    final l10n = AppLocalizations.of(context)!;
    final result = await showDialog<_KeyCaptureResult>(
      context: context,
      builder: (context) => _KeyCaptureDialog(
        title: l10n.bindKeyTitle(_actionLabel(l10n, action)),
        current: settings.customBindingFor(action),
      ),
    );
    if (result == null) return;
    controller.setCustomBinding(action, result.key);
  }

  static String _presetLabel(AppLocalizations l10n, KeyboardPreset preset) =>
      switch (preset) {
        KeyboardPreset.nesStandard => l10n.keyboardPresetNesStandard,
        KeyboardPreset.fightStick => l10n.keyboardPresetFightStick,
        KeyboardPreset.arcadeLayout => l10n.keyboardPresetArcadeLayout,
        KeyboardPreset.custom => l10n.keyboardPresetCustom,
      };

  static String _actionLabel(
    AppLocalizations l10n,
    KeyboardBindingAction action,
  ) => switch (action) {
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
  };

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
          _InputTab(editCustomBinding: _editCustomBinding),
          _VideoTab(pickAndApplyCustomPalette: _pickAndApplyCustomPalette),
          _EmulationTab(),
          _ServerTab(),
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
  const _InputTab({required this.editCustomBinding});

  final Future<void> Function(
    BuildContext,
    InputSettingsController,
    InputSettings,
    KeyboardBindingAction,
  )
  editCustomBinding;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final inputSettings = ref.watch(inputSettingsProvider);
    final inputController = ref.read(inputSettingsProvider.notifier);
    final turboSettings = ref.watch(turboSettingsProvider);
    final turboController = ref.read(turboSettingsProvider.notifier);
    final settings = ref.watch(virtualControlsSettingsProvider);
    final controller = ref.read(virtualControlsSettingsProvider.notifier);
    final editor = ref.watch(virtualControlsEditorProvider);
    final editorController = ref.read(virtualControlsEditorProvider.notifier);

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
        // Input Device
        AnimatedSettingsCard(
          index: 0,
          child: ListTile(
            title: Text(l10n.inputDeviceLabel),
            subtitle: Text(switch (inputSettings.device) {
              InputDevice.keyboard => l10n.inputDeviceKeyboard,
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
        // Turbo Settings
        AnimatedSettingsCard(
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
                      label: l10n.virtualControlsTurboOnFrames,
                      value: turboSettings.onFrames.toDouble(),
                      min: 1,
                      max: 30,
                      divisions: 29,
                      onChanged: (v) => turboController.setOnFrames(v.round()),
                      valueLabel: l10n.framesValue(turboSettings.onFrames),
                    ),
                    AnimatedSliderTile(
                      label: l10n.virtualControlsTurboOffFrames,
                      value: turboSettings.offFrames.toDouble(),
                      min: 1,
                      max: 30,
                      divisions: 29,
                      onChanged: (v) => turboController.setOffFrames(v.round()),
                      valueLabel: l10n.framesValue(turboSettings.offFrames),
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
          AnimatedSettingsCard(
            index: 3,
            child: AnimatedExpansionTile(
              title: Text(
                inputSettings.keyboardPreset == KeyboardPreset.custom
                    ? l10n.customKeyBindingsTitle
                    : l10n.keyBindingsTitle,
                style: Theme.of(context).textTheme.titleMedium,
              ),
              children: [
                for (final action in KeyboardBindingAction.values)
                  ListTile(
                    title: Text(_SettingsPageState._actionLabel(l10n, action)),
                    subtitle: Text(
                      _keyLabel(l10n, inputSettings.bindingForAction(action)),
                    ),
                    trailing:
                        inputSettings.keyboardPreset == KeyboardPreset.custom
                        ? const Icon(Icons.edit)
                        : null,
                    onTap: inputSettings.keyboardPreset == KeyboardPreset.custom
                        ? () => editCustomBinding(
                            context,
                            inputController,
                            inputSettings,
                            action,
                          )
                        : null,
                  ),
                if (inputSettings.keyboardPreset == KeyboardPreset.custom)
                  Padding(
                    padding: const EdgeInsets.fromLTRB(16, 0, 16, 12),
                    child: Text(
                      l10n.tipPressEscapeToClearBinding,
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                  ),
              ],
            ),
          ),
        ],
        // Virtual Controls
        if (supportsVirtualControls) ...[
          const SizedBox(height: 12),
          AnimatedSettingsCard(
            index: 4,
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
                      inputSettings.device != InputDevice.virtualController) {
                    inputController.setDevice(InputDevice.virtualController);
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
              index: 5,
              child: Column(
                children: [
                  SwitchListTile(
                    secondary: const Icon(Icons.grid_4x4),
                    title: Text(l10n.gridSnappingTitle),
                    value: editor.gridSnapEnabled,
                    onChanged: editorController.setGridSnapEnabled,
                  ),
                  if (editor.gridSnapEnabled)
                    Padding(
                      padding: const EdgeInsets.fromLTRB(16, 0, 16, 12),
                      child: AnimatedSliderTile(
                        label: l10n.gridSpacingLabel,
                        value: editor.gridSpacing.clamp(4, 64),
                        min: 4,
                        max: 64,
                        divisions: 60,
                        onChanged: editorController.setGridSpacing,
                        valueLabel:
                            '${editor.gridSpacing.toStringAsFixed(0)} px',
                      ),
                    ),
                ],
              ),
            ),
          ],
          if (!usingVirtual)
            Padding(
              padding: const EdgeInsets.only(top: 8),
              child: Text(
                l10n.virtualControlsSwitchInputTip,
                style: Theme.of(context).textTheme.bodyMedium,
              ),
            ),
          const SizedBox(height: 8),
          Opacity(
            opacity: usingVirtual ? 1 : 0.5,
            child: IgnorePointer(
              ignoring: !usingVirtual,
              child: Column(
                children: [
                  AnimatedSliderTile(
                    label: l10n.virtualControlsButtonSize,
                    value: settings.buttonSize,
                    min: 40,
                    max: 120,
                    onChanged: controller.setButtonSize,
                    valueLabel: '${settings.buttonSize.toStringAsFixed(0)} px',
                  ),
                  AnimatedSliderTile(
                    label: l10n.virtualControlsGap,
                    value: settings.gap,
                    min: 4,
                    max: 24,
                    onChanged: controller.setGap,
                    valueLabel: '${settings.gap.toStringAsFixed(0)} px',
                  ),
                  AnimatedSliderTile(
                    label: l10n.virtualControlsOpacity,
                    value: settings.opacity,
                    min: 0.2,
                    max: 0.8,
                    onChanged: controller.setOpacity,
                    valueLabel: settings.opacity.toStringAsFixed(2),
                  ),
                  AnimatedSliderTile(
                    label: l10n.virtualControlsHitboxScale,
                    value: settings.hitboxScale,
                    min: 1.0,
                    max: 1.4,
                    divisions: 40,
                    onChanged: controller.setHitboxScale,
                    valueLabel: settings.hitboxScale.toStringAsFixed(2),
                  ),
                  AnimatedSwitchTile(
                    value: settings.hapticsEnabled,
                    title: Text(l10n.virtualControlsHapticFeedback),
                    onChanged: controller.setHapticsEnabled,
                  ),
                  AnimatedSliderTile(
                    label: l10n.virtualControlsDpadDeadzone,
                    value: settings.dpadDeadzoneRatio,
                    min: 0.06,
                    max: 0.30,
                    divisions: 48,
                    onChanged: controller.setDpadDeadzoneRatio,
                    valueLabel: settings.dpadDeadzoneRatio.toStringAsFixed(2),
                  ),
                  Padding(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 16,
                      vertical: 8,
                    ),
                    child: Text(
                      l10n.virtualControlsDpadDeadzoneHelp,
                      style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        color: Theme.of(
                          context,
                        ).colorScheme.onSurface.withValues(alpha: 0.75),
                      ),
                    ),
                  ),
                  AnimatedSliderTile(
                    label: l10n.virtualControlsDpadBoundaryDeadzone,
                    value: settings.dpadBoundaryDeadzoneRatio,
                    min: 0.35,
                    max: 0.90,
                    divisions: 55,
                    onChanged: controller.setDpadBoundaryDeadzoneRatio,
                    valueLabel: settings.dpadBoundaryDeadzoneRatio
                        .toStringAsFixed(2),
                  ),
                  Padding(
                    padding: const EdgeInsets.symmetric(
                      horizontal: 16,
                      vertical: 8,
                    ),
                    child: Text(
                      l10n.virtualControlsDpadBoundaryDeadzoneHelp,
                      style: Theme.of(context).textTheme.bodySmall?.copyWith(
                        color: Theme.of(
                          context,
                        ).colorScheme.onSurface.withValues(alpha: 0.75),
                      ),
                    ),
                  ),
                  const SizedBox(height: 8),
                  Padding(
                    padding: const EdgeInsets.symmetric(horizontal: 16),
                    child: Text(
                      l10n.tipAdjustButtonsInDrawer,
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                  ),
                  const Divider(),
                  AnimatedSettingsCard(
                    index: 6,
                    child: ListTile(
                      leading: const Icon(Icons.restore),
                      title: Text(l10n.virtualControlsReset),
                      onTap: controller.resetToDefault,
                    ),
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
// Video Tab
// ============================================================================

class _VideoTab extends ConsumerWidget {
  const _VideoTab({required this.pickAndApplyCustomPalette});

  final Future<void> Function(BuildContext, VideoSettingsController)
  pickAndApplyCustomPalette;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final videoSettings = ref.watch(videoSettingsProvider);
    final videoController = ref.read(videoSettingsProvider.notifier);
    final androidBackend = ref.watch(androidVideoBackendSettingsProvider);
    final androidBackendController = ref.read(
      androidVideoBackendSettingsProvider.notifier,
    );
    final isAndroid =
        !kIsWeb && defaultTargetPlatform == TargetPlatform.android;

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
      await pickAndApplyCustomPalette(context, videoController);
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
                      onPressed: () =>
                          pickAndApplyCustomPalette(context, videoController),
                    ),
                    onTap: () =>
                        pickAndApplyCustomPalette(context, videoController),
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
                    labelText: l10n.videoBackendLabel,
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
                ],
              ],
            ),
          ),
        ),
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
                            key: const ValueKey('rewindSeconds'),
                            padding: const EdgeInsets.fromLTRB(56, 0, 0, 4),
                            child: AnimatedSliderTile(
                              label: l10n.rewindSecondsTitle,
                              value: emulationSettings.rewindSeconds.toDouble(),
                              min: 10,
                              max: 300,
                              divisions: 29,
                              onChanged: (v) => emulationController
                                  .setRewindSeconds(v.toInt()),
                              valueLabel: l10n.rewindSecondsValue(
                                emulationSettings.rewindSeconds,
                              ),
                            ),
                          )
                        : const SizedBox.shrink(
                            key: ValueKey('rewindSecondsEmpty'),
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

class _ServerTab extends ConsumerWidget {
  @override
  Widget build(BuildContext context, WidgetRef ref) {
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
        // Port configuration
        AnimatedSettingsCard(
          index: 1,
          child: Padding(
            padding: const EdgeInsets.all(20),
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                TextField(
                  controller: TextEditingController(
                    text: settings.port.toString(),
                  ),
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
              ],
            ),
          ),
        ),
        const SizedBox(height: 16),
        // Start/Stop button
        AnimatedSettingsCard(
          index: 2,
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
                            await controller.startServer();
                          } catch (e) {
                            if (context.mounted) {
                              ScaffoldMessenger.of(context).showSnackBar(
                                SnackBar(
                                  content: Text(
                                    l10n.serverStartFailed(e.toString()),
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

class _KeyCaptureDialog extends StatefulWidget {
  const _KeyCaptureDialog({required this.title, required this.current});

  final String title;
  final LogicalKeyboardKey? current;

  @override
  State<_KeyCaptureDialog> createState() => _KeyCaptureDialogState();
}

class _KeyCaptureDialogState extends State<_KeyCaptureDialog> {
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
    Navigator.of(context).pop(_KeyCaptureResult(key: key));
    return KeyEventResult.handled;
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
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
              const SizedBox(height: 8),
              Text(l10n.keyCaptureCurrent(_keyLabel(l10n, widget.current))),
              if (_last != null)
                Text(l10n.keyCaptureCaptured(_keyLabel(l10n, _last))),
              const SizedBox(height: 8),
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
