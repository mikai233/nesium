import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../controls/input_settings.dart';
import '../controls/virtual_controls_settings.dart';
import 'emulation_settings.dart';

class SettingsPage extends ConsumerWidget {
  const SettingsPage({super.key});

  Future<void> _editCustomBinding(
    BuildContext context,
    InputSettingsController controller,
    InputSettings settings,
    KeyboardBindingAction action,
  ) async {
    final result = await showDialog<_KeyCaptureResult>(
      context: context,
      builder: (context) => _KeyCaptureDialog(
        title: 'Bind ${action.label}',
        current: settings.customBindingFor(action),
      ),
    );
    if (result == null) return;
    controller.setCustomBinding(action, result.key);
  }

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final inputSettings = ref.watch(inputSettingsProvider);
    final inputController = ref.read(inputSettingsProvider.notifier);

    final settings = ref.watch(virtualControlsSettingsProvider);
    final controller = ref.read(virtualControlsSettingsProvider.notifier);

    final emulationSettings = ref.watch(emulationSettingsProvider);
    final emulationController = ref.read(emulationSettingsProvider.notifier);

    final supportsVirtual =
        !kIsWeb &&
        (defaultTargetPlatform == TargetPlatform.android ||
            defaultTargetPlatform == TargetPlatform.iOS);
    final usingVirtual = inputSettings.device == InputDevice.virtualController;

    return Scaffold(
      appBar: AppBar(title: const Text('Settings')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Text('Input', style: Theme.of(context).textTheme.titleLarge),
          const SizedBox(height: 8),
          Card(
            elevation: 0,
            color: Theme.of(context).colorScheme.surfaceContainerHighest,
            child: Padding(
              padding: const EdgeInsets.all(12),
              child: InputDecorator(
                decoration: const InputDecoration(
                  labelText: 'Input device',
                  border: OutlineInputBorder(),
                ),
                child: DropdownButtonHideUnderline(
                  child: DropdownButton<InputDevice>(
                    value: inputSettings.device,
                    isExpanded: true,
                    items: [
                      const DropdownMenuItem(
                        value: InputDevice.keyboard,
                        child: Text('Keyboard'),
                      ),
                      if (supportsVirtual ||
                          inputSettings.device == InputDevice.virtualController)
                        DropdownMenuItem(
                          value: InputDevice.virtualController,
                          enabled: supportsVirtual,
                          child: const Text('Virtual controller'),
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
                  decoration: const InputDecoration(
                    labelText: 'Keyboard preset',
                    border: OutlineInputBorder(),
                  ),
                  child: DropdownButtonHideUnderline(
                    child: DropdownButton<KeyboardPreset>(
                      value: inputSettings.keyboardPreset,
                      isExpanded: true,
                      items: [
                        for (final preset in KeyboardPreset.values)
                          DropdownMenuItem(
                            value: preset,
                            child: Text(preset.label),
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
            if (inputSettings.keyboardPreset == KeyboardPreset.custom) ...[
              const SizedBox(height: 12),
              Text(
                'Custom key bindings',
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const SizedBox(height: 8),
              Card(
                elevation: 0,
                color: Theme.of(context).colorScheme.surfaceContainerHighest,
                child: Column(
                  children: [
                    for (final action in KeyboardBindingAction.values)
                      ListTile(
                        title: Text(action.label),
                        subtitle: Text(
                          _keyLabel(inputSettings.customBindingFor(action)),
                        ),
                        trailing: const Icon(Icons.edit),
                        onTap: () => _editCustomBinding(
                          context,
                          inputController,
                          inputSettings,
                          action,
                        ),
                      ),
                  ],
                ),
              ),
              Text(
                'Tip: press Escape to clear a binding.',
                style: Theme.of(context).textTheme.bodySmall,
              ),
            ],
          ],
          const Divider(),
          Text('Emulation', style: Theme.of(context).textTheme.titleLarge),
          const SizedBox(height: 8),
          Card(
            elevation: 0,
            color: Theme.of(context).colorScheme.surfaceContainerHighest,
            child: Column(
              children: [
                SwitchListTile(
                  value: emulationSettings.integerFpsMode,
                  title: const Text('Integer FPS mode (60Hz, NTSC)'),
                  subtitle: const Text(
                    'Reduces scrolling judder on 60Hz displays. PAL will be added later.',
                  ),
                  onChanged: emulationController.setIntegerFpsMode,
                ),
                SwitchListTile(
                  value: emulationSettings.pauseInBackground,
                  title: const Text('Pause in background'),
                  subtitle: const Text(
                    'Automatically pauses the emulator when the app is not active.',
                  ),
                  onChanged: emulationController.setPauseInBackground,
                ),
              ],
            ),
          ),
          if (supportsVirtual) ...[
            const Divider(),
            Text(
              'Virtual Controls',
              style: Theme.of(context).textTheme.titleLarge,
            ),
            const SizedBox(height: 8),
            if (!usingVirtual)
              Text(
                'Switch input to "Virtual controller" to use these settings.',
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
                      label: 'Button size',
                      value: settings.buttonSize,
                      min: 40,
                      max: 120,
                      onChanged: controller.setButtonSize,
                      valueLabel:
                          '${settings.buttonSize.toStringAsFixed(0)} px',
                    ),
                    _SliderTile(
                      label: 'Gap',
                      value: settings.gap,
                      min: 4,
                      max: 24,
                      onChanged: controller.setGap,
                      valueLabel: '${settings.gap.toStringAsFixed(0)} px',
                    ),
                    _SliderTile(
                      label: 'Opacity',
                      value: settings.opacity,
                      min: 0.2,
                      max: 0.8,
                      onChanged: controller.setOpacity,
                      valueLabel: settings.opacity.toStringAsFixed(2),
                    ),
                    _SliderTile(
                      label: 'Hitbox scale',
                      value: settings.hitboxScale,
                      min: 1.0,
                      max: 1.4,
                      divisions: 40,
                      onChanged: controller.setHitboxScale,
                      valueLabel: settings.hitboxScale.toStringAsFixed(2),
                    ),
                    SwitchListTile(
                      value: settings.hapticsEnabled,
                      title: const Text('Haptic feedback'),
                      onChanged: controller.setHapticsEnabled,
                    ),
                    _SliderTile(
                      label: 'D-pad deadzone',
                      value: settings.dpadDeadzoneRatio,
                      min: 0.06,
                      max: 0.30,
                      divisions: 48,
                      onChanged: controller.setDpadDeadzoneRatio,
                      valueLabel: settings.dpadDeadzoneRatio.toStringAsFixed(2),
                    ),
                    _SliderTile(
                      label: 'Turbo frames per toggle',
                      value: settings.turboFramesPerToggle.toDouble(),
                      min: 1,
                      max: 8,
                      divisions: 7,
                      onChanged: (v) =>
                          controller.setTurboFramesPerToggle(v.round()),
                      valueLabel: '${settings.turboFramesPerToggle} frames',
                    ),
                    const SizedBox(height: 8),
                    Text(
                      'Tip: adjust button position/size from the in-game drawer.',
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

String _keyLabel(LogicalKeyboardKey? key) {
  if (key == null) return 'Unassigned';
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
              Text('Press a key to bind.'),
              const SizedBox(height: 8),
              Text('Current: ${_keyLabel(widget.current)}'),
              if (_last != null) Text('Captured: ${_keyLabel(_last)}'),
              const SizedBox(height: 8),
              Text(
                'Press Escape to clear.',
                style: Theme.of(context).textTheme.bodySmall,
              ),
            ],
          ),
        ),
      ),
      actions: [
        TextButton(
          onPressed: () => Navigator.of(context).pop(),
          child: const Text('Cancel'),
        ),
      ],
    );
  }
}
