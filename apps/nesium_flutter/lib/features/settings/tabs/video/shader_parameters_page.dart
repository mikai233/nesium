import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../../bridge/api/video.dart' as video;
import '../../../../l10n/app_localizations.dart';
import '../../../../widgets/animated_settings_widgets.dart';
import '../../shader_parameter_provider.dart';

class ShaderParametersPage extends ConsumerWidget {
  const ShaderParametersPage({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final paramsAsync = ref.watch(shaderParametersProvider);

    return Scaffold(
      appBar: AppBar(
        title: Text(l10n.videoShaderParametersTitle),
        actions: [
          IconButton(
            icon: const Icon(Icons.refresh),
            tooltip: l10n.videoShaderParametersReset,
            onPressed: () =>
                ref.read(shaderParametersProvider.notifier).resetParameters(),
          ),
        ],
      ),
      body: paramsAsync.when(
        data: (params) {
          if (params.isEmpty) {
            return Center(
              child: Text(
                l10n.videoShaderPresetNotSet, // Or generic "No parameters"
                style: Theme.of(context).textTheme.bodyLarge,
              ),
            );
          }

          return ListView.builder(
            padding: const EdgeInsets.symmetric(vertical: 12),
            itemCount: params.length,
            itemBuilder: (context, index) {
              final p = params[index];
              // Detect separators/headings: parameters where min == max are usually
              // used as labels in librashader/RetroArch shader presets.
              final bool isSeparator = (p.minimum - p.maximum).abs() < 0.0001;

              if (isSeparator) {
                return Padding(
                  padding: const EdgeInsets.only(
                    left: 16,
                    right: 16,
                    top: 16,
                    bottom: 8,
                  ),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      Text(
                        p.description.isNotEmpty ? p.description : p.name,
                        style: Theme.of(context).textTheme.titleSmall?.copyWith(
                          color: Theme.of(context).colorScheme.primary,
                          fontWeight: FontWeight.bold,
                        ),
                      ),
                      const Divider(thickness: 1),
                    ],
                  ),
                );
              }

              return Padding(
                padding: const EdgeInsets.symmetric(
                  horizontal: 16,
                  vertical: 4,
                ),
                child: _ShaderParameterSlider(parameter: p),
              );
            },
          );
        },
        loading: () => const Center(child: CircularProgressIndicator()),
        error: (err, stack) => Center(child: Text('Error: $err')),
      ),
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
