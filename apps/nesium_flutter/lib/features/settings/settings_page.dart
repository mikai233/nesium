import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../controls/virtual_controls_settings.dart';

class SettingsPage extends ConsumerWidget {
  const SettingsPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final settings = ref.watch(virtualControlsSettingsProvider);
    final controller = ref.read(virtualControlsSettingsProvider.notifier);

    return Scaffold(
      appBar: AppBar(title: const Text('Settings')),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Text(
            'Virtual Controls',
            style: Theme.of(context).textTheme.titleLarge,
          ),
          const SizedBox(height: 8),
          SwitchListTile(
            value: settings.enabled,
            title: const Text('Enable virtual controls'),
            onChanged: controller.setEnabled,
          ),
          const Divider(),
          _SliderTile(
            label: 'Button size',
            value: settings.buttonSize,
            min: 40,
            max: 120,
            onChanged: controller.setButtonSize,
            valueLabel: '${settings.buttonSize.toStringAsFixed(0)} px',
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
            onChanged: (v) => controller.setTurboFramesPerToggle(v.round()),
            valueLabel: '${settings.turboFramesPerToggle} frames',
          ),
          const Divider(),
          Text(
            'Position Offsets (Portrait)',
            style: Theme.of(context).textTheme.titleMedium,
          ),
          const SizedBox(height: 8),
          _OffsetEditor(
            label: 'D-pad',
            value: settings.portraitDpadOffset,
            onChanged: controller.setPortraitDpadOffset,
          ),
          const SizedBox(height: 12),
          _OffsetEditor(
            label: 'Buttons',
            value: settings.portraitButtonsOffset,
            onChanged: controller.setPortraitButtonsOffset,
          ),
          const Divider(),
          Text(
            'Position Offsets (Landscape)',
            style: Theme.of(context).textTheme.titleMedium,
          ),
          const SizedBox(height: 8),
          _OffsetEditor(
            label: 'D-pad',
            value: settings.landscapeDpadOffset,
            onChanged: controller.setLandscapeDpadOffset,
          ),
          const SizedBox(height: 12),
          _OffsetEditor(
            label: 'Buttons',
            value: settings.landscapeButtonsOffset,
            onChanged: controller.setLandscapeButtonsOffset,
          ),
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

class _OffsetEditor extends StatelessWidget {
  const _OffsetEditor({
    required this.label,
    required this.value,
    required this.onChanged,
  });

  final String label;
  final Offset value;
  final ValueChanged<Offset> onChanged;

  @override
  Widget build(BuildContext context) {
    return Card(
      elevation: 0,
      color: Theme.of(context).colorScheme.surfaceContainerHighest,
      child: Padding(
        padding: const EdgeInsets.all(12),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Text(label, style: Theme.of(context).textTheme.titleSmall),
            const SizedBox(height: 8),
            _SliderTile(
              label: 'X offset',
              value: value.dx,
              min: -250,
              max: 250,
              divisions: 200,
              onChanged: (v) => onChanged(Offset(v, value.dy)),
              valueLabel: '${value.dx.toStringAsFixed(0)} px',
            ),
            _SliderTile(
              label: 'Y offset',
              value: value.dy,
              min: -250,
              max: 250,
              divisions: 200,
              onChanged: (v) => onChanged(Offset(value.dx, v)),
              valueLabel: '${value.dy.toStringAsFixed(0)} px',
            ),
          ],
        ),
      ),
    );
  }
}
