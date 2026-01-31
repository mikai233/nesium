import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../../../bridge/api/video.dart' as video;
import '../../../../domain/nes_controller.dart';
import '../../../../l10n/app_localizations.dart';
import '../../../../widgets/animated_settings_widgets.dart';
import '../../../screen/floating_game_preview_state.dart';
import '../../shader_parameter_provider.dart';

class ShaderParametersPage extends ConsumerStatefulWidget {
  const ShaderParametersPage({super.key});

  @override
  ConsumerState<ShaderParametersPage> createState() =>
      _ShaderParametersPageState();
}

class _ShaderParametersPageState extends ConsumerState<ShaderParametersPage> {
  final TextEditingController _searchController = TextEditingController();
  bool _isSearching = false;
  String _searchQuery = '';

  @override
  void dispose() {
    _searchController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final paramsAsync = ref.watch(shaderParametersProvider);
    final preview = ref.watch(floatingGamePreviewProvider);

    return Scaffold(
      appBar: AppBar(
        title: _isSearching
            ? TextField(
                controller: _searchController,
                autofocus: true,
                decoration: InputDecoration(
                  hintText: l10n.searchHint, // reusing search label or similar
                  border: InputBorder.none,
                  hintStyle: TextStyle(
                    color: Theme.of(
                      context,
                    ).colorScheme.onSurface.withValues(alpha: 0.6),
                  ),
                ),
                style: TextStyle(
                  color: Theme.of(context).colorScheme.onSurface,
                ),
                onChanged: (value) {
                  setState(() {
                    _searchQuery = value.toLowerCase();
                  });
                },
              )
            : Text(l10n.videoShaderParametersTitle),
        actions: [
          IconButton(
            icon: Icon(_isSearching ? Icons.close : Icons.search),
            tooltip: _isSearching ? l10n.cancel : l10n.searchTooltip,
            onPressed: () {
              setState(() {
                if (_isSearching) {
                  _isSearching = false;
                  _searchQuery = '';
                  _searchController.clear();
                } else {
                  _isSearching = true;
                }
              });
            },
          ),
          if (!_isSearching)
            IconButton(
              icon: const Icon(Icons.refresh),
              tooltip: l10n.videoShaderParametersReset,
              onPressed: () =>
                  ref.read(shaderParametersProvider.notifier).resetParameters(),
            ),
          if (!_isSearching)
            if (defaultTargetPlatform == TargetPlatform.android ||
                defaultTargetPlatform == TargetPlatform.iOS ||
                defaultTargetPlatform == TargetPlatform.linux)
              if (ref.watch(
                nesControllerProvider.select((s) => s.romHash != null),
              ))
                IconButton(
                  onPressed: () {
                    ref.read(floatingGamePreviewProvider.notifier).toggle();
                  },
                  icon: Icon(
                    preview.visible
                        ? Icons.fullscreen_exit
                        : Icons.picture_in_picture_alt,
                  ),
                  tooltip: l10n.settingsFloatingPreviewTooltip,
                ),
        ],
      ),
      body: paramsAsync.when(
        data: (params) {
          if (params.isEmpty) {
            return Center(
              child: Text(
                l10n.videoShaderPresetNotSet,
                style: Theme.of(context).textTheme.bodyLarge,
              ),
            );
          }

          final filteredParams = params.where((p) {
            // Always show separators if we are not searching,
            // or if we are searching, maybe only show them if they match?
            // Simple approach: Filter strictly by content.
            // If query is empty, show everything.
            if (_searchQuery.isEmpty) return true;

            final name = p.name.toLowerCase();
            final desc = p.description.toLowerCase();
            return name.contains(_searchQuery) || desc.contains(_searchQuery);
          }).toList();

          if (filteredParams.isEmpty) {
            return Center(child: Text(l10n.noResults));
          }

          return ListView.builder(
            padding: const EdgeInsets.symmetric(vertical: 12),
            itemCount: filteredParams.length,
            itemBuilder: (context, index) {
              final p = filteredParams[index];
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
                child: _ShaderParameterSlider(
                  // Key is important if the list changes order/filtering
                  key: ValueKey(p.name),
                  parameter: p,
                ),
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
  const _ShaderParameterSlider({super.key, required this.parameter});

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
