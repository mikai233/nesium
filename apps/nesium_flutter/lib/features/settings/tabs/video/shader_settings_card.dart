import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../../bridge/api/video.dart' as video;
import '../../../../l10n/app_localizations.dart';
import '../../../../widgets/animated_settings_widgets.dart';
import '../../shader_parameter_provider.dart';

class ShaderSettingsCard extends ConsumerWidget {
  const ShaderSettingsCard({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final paramsAsync = ref.watch(shaderParametersProvider);

    return paramsAsync.when(
      data: (params) {
        if (params.isEmpty) return const SizedBox.shrink();

        return Padding(
          padding: const EdgeInsets.symmetric(horizontal: 12),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              const SizedBox(height: 8),
              AnimatedExpansionTile(
                labelText: l10n.videoShaderParametersTitle,
                title: Text(l10n.videoShaderParametersSubtitle),
                initiallyExpanded: false,
                children: [
                  ...params.entries.map(
                    (entry) => Padding(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 16,
                        vertical: 4,
                      ),
                      child: _ShaderParameterSlider(parameter: entry.value),
                    ),
                  ),
                  const SizedBox(height: 12),
                ],
              ),
              const SizedBox(height: 12),
            ],
          ),
        );
      },
      loading: () => const SizedBox.shrink(),
      error: (err, stack) => const SizedBox.shrink(),
    );
  }
}

class _ShaderParameterSlider extends ConsumerStatefulWidget {
  const _ShaderParameterSlider({required this.parameter});

  final video.ShaderParameter parameter;

  @override
  ConsumerState<_ShaderParameterSlider> createState() =>
      _ShaderParameterSliderState();
}

class _ShaderParameterSliderState
    extends ConsumerState<_ShaderParameterSlider> {
  late double _currentValue;

  @override
  void initState() {
    super.initState();
    _currentValue = widget.parameter.current;
  }

  @override
  void didUpdateWidget(_ShaderParameterSlider oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.parameter.current != widget.parameter.current) {
      _currentValue = widget.parameter.current;
    }
  }

  @override
  Widget build(BuildContext context) {
    final p = widget.parameter;
    return AnimatedSliderTile(
      label: p.name,
      helperText: p.description.isNotEmpty ? p.description : null,
      value: _currentValue,
      min: p.minimum,
      max: p.maximum,
      // Calculate divisions based on step
      divisions: (p.step > 0 && (p.maximum - p.minimum) > 0)
          ? ((p.maximum - p.minimum) / p.step).round()
          : null,
      valueLabel: _currentValue.toStringAsFixed(3),
      onChanged: (value) {
        setState(() => _currentValue = value);
        ref
            .read(shaderParametersProvider.notifier)
            .updateParameter(p.name, value);
      },
    );
  }
}
