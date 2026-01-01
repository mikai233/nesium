import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../../l10n/app_localizations.dart';
import '../../shell/nes_actions.dart';
import '../../shell/nes_menu_model.dart';

class ToolsPanel extends ConsumerWidget {
  const ToolsPanel({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context)!;
    final actions = _getActions(ref);

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
        return ListTile(
          leading: Icon(tool.icon),
          title: Text(tool.label(l10n)),
          trailing: const Icon(Icons.chevron_right),
          onTap: () => _onToolTap(tool.id, actions),
        );
      },
    );
  }

  NesActions? _getActions(WidgetRef ref) {
    try {
      return ref.watch(nesActionsProvider);
    } catch (_) {
      return null;
    }
  }

  void _onToolTap(NesMenuItemId id, NesActions? actions) {
    switch (id) {
      case NesMenuItemId.tilemapViewer:
        actions?.openTilemapViewer?.call();
        break;
      default:
        break;
    }
  }
}
