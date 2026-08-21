import 'package:flutter/material.dart';
import 'package:nesium_flutter/features/debugger/tile/tile_viewer_models.dart';
import 'package:nesium_flutter/l10n/app_localizations.dart';

class TilePresetButtons extends StatelessWidget {
  const TilePresetButtons({
    required this.presets,
    required this.selectedPreset,
    required this.onSelected,
    this.onChanged,
    super.key,
  });

  final List<TilePreset> presets;
  final TilePreset? selectedPreset;
  final ValueChanged<TilePreset> onSelected;
  final VoidCallback? onChanged;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;

    String presetLabel(TilePreset preset) => switch (preset) {
      TilePreset.ppu => l10n.tileViewerPresetPpu,
      TilePreset.chr => l10n.tileViewerPresetChr,
      TilePreset.rom => l10n.tileViewerPresetRom,
      TilePreset.bg => l10n.tileViewerPresetBg,
      TilePreset.oam => l10n.tileViewerPresetOam,
    };

    return Wrap(
      spacing: 4,
      runSpacing: 4,
      children: presets.map((preset) {
        final isSelected = selectedPreset == preset;
        return FilterChip(
          label: Text(presetLabel(preset)),
          selected: isSelected,
          onSelected: (_) {
            onSelected(preset);
            onChanged?.call();
          },
          showCheckmark: false,
          visualDensity: VisualDensity.compact,
          labelPadding: const EdgeInsets.symmetric(horizontal: 4),
        );
      }).toList(),
    );
  }
}
