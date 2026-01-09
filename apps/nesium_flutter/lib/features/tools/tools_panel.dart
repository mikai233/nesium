import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../domain/nes_controller.dart';
import '../../l10n/app_localizations.dart';
import '../../shell/nes_actions.dart';
import '../../shell/nes_menu_model.dart';
import '../../logging/app_logger.dart';

class ToolsPanel extends ConsumerWidget {
  const ToolsPanel({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final actions = _getActions(ref);
    final hasRom = ref.watch(
      nesControllerProvider.select((s) => s.romHash != null),
    );

    final tools = NesMenus.tools.children ?? [];

    if (tools.isEmpty) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(32),
          child: Text(
            l10n.toolsPlaceholderBody,
            style: Theme.of(context).textTheme.bodyMedium?.copyWith(
              color: Theme.of(context).colorScheme.outline,
            ),
            textAlign: TextAlign.center,
          ),
        ),
      );
    }

    return ListView.builder(
      itemCount: tools.length,
      itemBuilder: (context, index) {
        final tool = tools[index];
        final enabled = !_requiresRom(tool.id) || hasRom;
        return ListTile(
          leading: Icon(tool.icon),
          title: Text(tool.label(l10n)),
          trailing: const Icon(Icons.chevron_right),
          enabled: enabled,
          onTap: enabled ? () => _onToolTap(tool.id, actions) : null,
        );
      },
    );
  }

  bool _requiresRom(NesMenuItemId id) => switch (id) {
    NesMenuItemId.tilemapViewer ||
    NesMenuItemId.tileViewer ||
    NesMenuItemId.spriteViewer ||
    NesMenuItemId.paletteViewer ||
    NesMenuItemId.historyViewer => true,
    _ => false,
  };

  NesActions? _getActions(WidgetRef ref) {
    try {
      return ref.watch(nesActionsProvider);
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'Failed to watch nesActionsProvider',
        logger: 'tools_panel',
      );
      return null;
    }
  }

  void _onToolTap(NesMenuItemId id, NesActions? actions) {
    switch (id) {
      case NesMenuItemId.tilemapViewer:
        actions?.openTilemapViewer?.call();
        break;
      case NesMenuItemId.tileViewer:
        actions?.openTileViewer?.call();
        break;
      case NesMenuItemId.spriteViewer:
        actions?.openSpriteViewer?.call();
        break;
      case NesMenuItemId.paletteViewer:
        actions?.openPaletteViewer?.call();
        break;
      case NesMenuItemId.historyViewer:
        actions?.openHistoryViewer?.call();
        break;
      default:
        break;
    }
  }
}
