import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:file_picker/file_picker.dart';

import '../../l10n/app_localizations.dart';
import '../../logging/app_logger.dart';
import '../../platform/platform_capabilities.dart';
import '../controls/input_settings.dart';
import '../controls/virtual_controls_settings.dart';
import 'emulation_settings.dart';
import 'language_settings.dart';
import 'video_settings.dart';
import '../../bridge/api/palette.dart' as nes_palette;

class SettingsPage extends ConsumerWidget {
  const SettingsPage({super.key});

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
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final inputSettings = ref.watch(inputSettingsProvider);
    final inputController = ref.read(inputSettingsProvider.notifier);

    final settings = ref.watch(virtualControlsSettingsProvider);
    final controller = ref.read(virtualControlsSettingsProvider.notifier);

    final emulationSettings = ref.watch(emulationSettingsProvider);
    final emulationController = ref.read(emulationSettingsProvider.notifier);

    final videoSettings = ref.watch(videoSettingsProvider);
    final videoController = ref.read(videoSettingsProvider.notifier);

    final language = ref.watch(appLanguageProvider);
    final languageController = ref.read(appLanguageProvider.notifier);

    final supportsVirtual = isNativeMobile;
    final usingVirtual = inputSettings.device == InputDevice.virtualController;

    return Scaffold(
      appBar: AppBar(title: Text(l10n.settingsTitle)),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Text(
            l10n.generalTitle,
            style: Theme.of(context).textTheme.titleLarge,
          ),
          const SizedBox(height: 8),
          Card(
            elevation: 0,
            color: Theme.of(context).colorScheme.surfaceContainerHighest,
            child: Padding(
              padding: const EdgeInsets.all(12),
              child: InputDecorator(
                decoration: InputDecoration(
                  labelText: l10n.languageLabel,
                  border: const OutlineInputBorder(),
                ),
                child: DropdownButtonHideUnderline(
                  child: DropdownButton<AppLanguage>(
                    value: language,
                    isExpanded: true,
                    items: [
                      DropdownMenuItem(
                        value: AppLanguage.system,
                        child: Text(l10n.languageSystem),
                      ),
                      DropdownMenuItem(
                        value: AppLanguage.english,
                        child: Text(l10n.languageEnglish),
                      ),
                      DropdownMenuItem(
                        value: AppLanguage.chineseSimplified,
                        child: Text(l10n.languageChineseSimplified),
                      ),
                    ],
                    onChanged: (value) {
                      if (value == null) return;
                      languageController.setLanguage(value);
                    },
                  ),
                ),
              ),
            ),
          ),
          const Divider(),
          Text(l10n.inputTitle, style: Theme.of(context).textTheme.titleLarge),
          const SizedBox(height: 8),
          Card(
            elevation: 0,
            color: Theme.of(context).colorScheme.surfaceContainerHighest,
            child: Padding(
              padding: const EdgeInsets.all(12),
              child: InputDecorator(
                decoration: InputDecoration(
                  labelText: l10n.inputDeviceLabel,
                  border: const OutlineInputBorder(),
                ),
                child: DropdownButtonHideUnderline(
                  child: DropdownButton<InputDevice>(
                    value: inputSettings.device,
                    isExpanded: true,
                    items: [
                      DropdownMenuItem(
                        value: InputDevice.keyboard,
                        child: Text(l10n.inputDeviceKeyboard),
                      ),
                      if (supportsVirtual ||
                          inputSettings.device == InputDevice.virtualController)
                        DropdownMenuItem(
                          value: InputDevice.virtualController,
                          enabled: supportsVirtual,
                          child: Text(l10n.inputDeviceVirtualController),
                        ),
                    ],
                    onChanged: (value) {
                      if (value == null) return;
                      inputController.setDevice(value);
                    },
                  ),
                ),
              ),
            ),
          ),
          if (inputSettings.device == InputDevice.keyboard) ...[
            const SizedBox(height: 12),
            Card(
              elevation: 0,
              color: Theme.of(context).colorScheme.surfaceContainerHighest,
              child: Padding(
                padding: const EdgeInsets.all(12),
                child: InputDecorator(
                  decoration: InputDecoration(
                    labelText: l10n.keyboardPresetLabel,
                    border: const OutlineInputBorder(),
                  ),
                  child: DropdownButtonHideUnderline(
                    child: DropdownButton<KeyboardPreset>(
                      value: inputSettings.keyboardPreset,
                      isExpanded: true,
                      items: [
                        for (final preset in KeyboardPreset.values)
                          DropdownMenuItem(
                            value: preset,
                            child: Text(_presetLabel(l10n, preset)),
                          ),
                      ],
                      onChanged: (value) {
                        if (value == null) return;
                        inputController.setKeyboardPreset(value);
                      },
                    ),
                  ),
                ),
              ),
            ),
            const SizedBox(height: 12),
            Card(
              elevation: 0,
              color: Theme.of(context).colorScheme.surfaceContainerHighest,
              child: Theme(
                data: Theme.of(
                  context,
                ).copyWith(dividerColor: Colors.transparent),
                child: ExpansionTile(
                  initiallyExpanded: false,
                  title: Text(
                    inputSettings.keyboardPreset == KeyboardPreset.custom
                        ? l10n.customKeyBindingsTitle
                        : l10n.keyBindingsTitle,
                    style: Theme.of(context).textTheme.titleMedium,
                  ),
                  children: [
                    for (final action in KeyboardBindingAction.values)
                      ListTile(
                        title: Text(_actionLabel(l10n, action)),
                        subtitle: Text(
                          _keyLabel(
                            l10n,
                            inputSettings.bindingForAction(action),
                          ),
                        ),
                        trailing:
                            inputSettings.keyboardPreset ==
                                KeyboardPreset.custom
                            ? const Icon(Icons.edit)
                            : null,
                        onTap:
                            inputSettings.keyboardPreset ==
                                KeyboardPreset.custom
                            ? () => _editCustomBinding(
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
            ),
          ],
          const Divider(),
          Text(
            l10n.emulationTitle,
            style: Theme.of(context).textTheme.titleLarge,
          ),
          const SizedBox(height: 8),
          Card(
            elevation: 0,
            color: Theme.of(context).colorScheme.surfaceContainerHighest,
            child: Column(
              children: [
                SwitchListTile(
                  value: emulationSettings.integerFpsMode,
                  title: Text(l10n.integerFpsTitle),
                  subtitle: Text(l10n.integerFpsSubtitle),
                  onChanged: emulationController.setIntegerFpsMode,
                ),
                SwitchListTile(
                  value: emulationSettings.pauseInBackground,
                  title: Text(l10n.pauseInBackgroundTitle),
                  subtitle: Text(l10n.pauseInBackgroundSubtitle),
                  onChanged: emulationController.setPauseInBackground,
                ),
              ],
            ),
          ),
          const Divider(),
          Text(l10n.videoTitle, style: Theme.of(context).textTheme.titleLarge),
          const SizedBox(height: 8),
          Card(
            elevation: 0,
            color: Theme.of(context).colorScheme.surfaceContainerHighest,
            child: Padding(
              padding: const EdgeInsets.all(12),
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  InputDecorator(
                    decoration: InputDecoration(
                      labelText: l10n.paletteModeLabel,
                      border: const OutlineInputBorder(),
                    ),
                    child: DropdownButtonHideUnderline(
                      child: DropdownButton<PaletteMode>(
                        value: videoSettings.paletteMode,
                        isExpanded: true,
                        items: [
                          DropdownMenuItem(
                            value: PaletteMode.builtin,
                            child: Text(l10n.paletteModeBuiltin),
                          ),
                          DropdownMenuItem(
                            value: PaletteMode.custom,
                            child: Text(
                              videoSettings.customPaletteName == null
                                  ? l10n.paletteModeCustom
                                  : l10n.paletteModeCustomActive(
                                      videoSettings.customPaletteName!,
                                    ),
                            ),
                          ),
                        ],
                        onChanged: (value) async {
                          if (value == null) return;
                          if (value == PaletteMode.builtin) {
                            try {
                              await videoController.setBuiltinPreset(
                                videoSettings.builtinPreset,
                              );
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

                          if (videoSettings.customPaletteName != null) {
                            videoController.useCustomIfAvailable();
                            return;
                          }
                          await _pickAndApplyCustomPalette(
                            context,
                            videoController,
                          );
                        },
                      ),
                    ),
                  ),
                  const SizedBox(height: 12),
                  if (videoSettings.paletteMode == PaletteMode.builtin)
                    InputDecorator(
                      decoration: InputDecoration(
                        labelText: l10n.builtinPaletteLabel,
                        border: const OutlineInputBorder(),
                      ),
                      child: DropdownButtonHideUnderline(
                        child: DropdownButton<nes_palette.PaletteKind>(
                          value: videoSettings.builtinPreset,
                          isExpanded: true,
                          items: const [
                            DropdownMenuItem(
                              value: nes_palette.PaletteKind.nesdevNtsc,
                              child: Text('Nesdev (NTSC)'),
                            ),
                            DropdownMenuItem(
                              value: nes_palette.PaletteKind.fbxCompositeDirect,
                              child: Text('FirebrandX (Composite Direct)'),
                            ),
                            DropdownMenuItem(
                              value: nes_palette.PaletteKind.sonyCxa2025AsUs,
                              child: Text('Sony CXA2025AS (US)'),
                            ),
                            DropdownMenuItem(
                              value: nes_palette.PaletteKind.pal2C07,
                              child: Text('RP2C07 (PAL)'),
                            ),
                            DropdownMenuItem(
                              value: nes_palette.PaletteKind.rawLinear,
                              child: Text('Raw linear'),
                            ),
                          ],
                          onChanged: (value) async {
                            if (value == null) return;
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
                          },
                        ),
                      ),
                    )
                  else
                    ListTile(
                      contentPadding: EdgeInsets.zero,
                      title: Text(l10n.customPaletteLoadTitle),
                      subtitle: Text(l10n.customPaletteLoadSubtitle),
                      trailing: const Icon(Icons.folder_open),
                      onTap: () =>
                          _pickAndApplyCustomPalette(context, videoController),
                    ),
                ],
              ),
            ),
          ),
          if (supportsVirtual) ...[
            const Divider(),
            Text(
              l10n.virtualControlsTitle,
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            if (!usingVirtual)
              Text(
                l10n.virtualControlsSwitchInputTip,
                style: Theme.of(context).textTheme.bodyMedium,
              ),
            const SizedBox(height: 8),
            Opacity(
              opacity: usingVirtual ? 1 : 0.5,
              child: IgnorePointer(
                ignoring: !usingVirtual,
                child: Column(
                  children: [
                    _SliderTile(
                      label: l10n.virtualControlsButtonSize,
                      value: settings.buttonSize,
                      min: 40,
                      max: 120,
                      onChanged: controller.setButtonSize,
                      valueLabel:
                          '${settings.buttonSize.toStringAsFixed(0)} px',
                    ),
                    _SliderTile(
                      label: l10n.virtualControlsGap,
                      value: settings.gap,
                      min: 4,
                      max: 24,
                      onChanged: controller.setGap,
                      valueLabel: '${settings.gap.toStringAsFixed(0)} px',
                    ),
                    _SliderTile(
                      label: l10n.virtualControlsOpacity,
                      value: settings.opacity,
                      min: 0.2,
                      max: 0.8,
                      onChanged: controller.setOpacity,
                      valueLabel: settings.opacity.toStringAsFixed(2),
                    ),
                    _SliderTile(
                      label: l10n.virtualControlsHitboxScale,
                      value: settings.hitboxScale,
                      min: 1.0,
                      max: 1.4,
                      divisions: 40,
                      onChanged: controller.setHitboxScale,
                      valueLabel: settings.hitboxScale.toStringAsFixed(2),
                    ),
                    SwitchListTile(
                      value: settings.hapticsEnabled,
                      title: Text(l10n.virtualControlsHapticFeedback),
                      onChanged: controller.setHapticsEnabled,
                    ),
                    _SliderTile(
                      label: l10n.virtualControlsDpadDeadzone,
                      value: settings.dpadDeadzoneRatio,
                      min: 0.06,
                      max: 0.30,
                      divisions: 48,
                      onChanged: controller.setDpadDeadzoneRatio,
                      valueLabel: settings.dpadDeadzoneRatio.toStringAsFixed(2),
                    ),
                    _SliderTile(
                      label: l10n.virtualControlsTurboFramesPerToggle,
                      value: settings.turboFramesPerToggle.toDouble(),
                      min: 1,
                      max: 8,
                      divisions: 7,
                      onChanged: (v) =>
                          controller.setTurboFramesPerToggle(v.round()),
                      valueLabel: l10n.framesValue(
                        settings.turboFramesPerToggle,
                      ),
                    ),
                    const SizedBox(height: 8),
                    Text(
                      l10n.tipAdjustButtonsInDrawer,
                      style: Theme.of(context).textTheme.bodySmall,
                    ),
                  ],
                ),
              ),
            ),
          ],
        ],
      ),
    );
  }
}

class _SliderTile extends StatelessWidget {
  const _SliderTile({
    required this.label,
    required this.value,
    required this.min,
    required this.max,
    required this.onChanged,
    required this.valueLabel,
    this.divisions,
  });

  final String label;
  final double value;
  final double min;
  final double max;
  final int? divisions;
  final ValueChanged<double> onChanged;
  final String valueLabel;

  @override
  Widget build(BuildContext context) {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            Expanded(child: Text(label)),
            Text(valueLabel, style: Theme.of(context).textTheme.labelMedium),
          ],
        ),
        Slider(
          value: value.clamp(min, max),
          min: min,
          max: max,
          divisions: divisions,
          onChanged: onChanged,
        ),
      ],
    );
  }
}

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
