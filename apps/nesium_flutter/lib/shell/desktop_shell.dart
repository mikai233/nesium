import 'dart:async';

import 'package:flutter/material.dart';

import '../features/screen/nes_screen_view.dart';
import '../domain/nes_state.dart';
import '../l10n/app_localizations.dart';
import 'nes_actions.dart';
import 'nes_menu_model.dart';

class DesktopShell extends StatelessWidget {
  const DesktopShell({super.key, required this.state, required this.actions});

  final NesState state;
  final NesActions actions;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      body: MediaQuery.removePadding(
        context: context,
        removeLeft: true,
        removeRight: true,
        removeBottom: true,
        child: Column(
          children: [
            _DesktopMenuBar(actions: actions),
            Expanded(
              child: NesScreenView(
                error: state.error,
                textureId: state.textureId,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

class _DesktopMenuBar extends StatelessWidget {
  const _DesktopMenuBar({required this.actions});

  final NesActions actions;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final textStyle = Theme.of(context).textTheme.titleSmall;
    return Container(
      height: 28,
      // padding: const EdgeInsets.symmetric(horizontal: 8),
      decoration: const BoxDecoration(
        color: Color(0xFFF7F7F7),
        border: Border(
          bottom: BorderSide(color: Color(0xFFE0E0E0), width: 0.5),
        ),
      ),
      alignment: Alignment.centerLeft,
      child: MenuBar(
        style: MenuStyle(
          padding: WidgetStateProperty.all(EdgeInsets.zero),
          // elevation: WidgetStateProperty.all(0),
          // backgroundColor: WidgetStateProperty.all(Colors.transparent),
        ),
        children: NesMenus.desktopMenuSections
            .map(
              (section) => _buildMenu(section.title(l10n), [
                ...section.items.map(
                  (item) => MenuItemButton(
                    onPressed: () => _dispatch(item.id),
                    child: Text(_desktopLabel(l10n, item.id)),
                  ),
                ),
                if (section.id == NesMenuSectionId.settings)
                  MenuItemButton(
                    onPressed: null,
                    child: Text(l10n.menuInputMappingComingSoon),
                  ),
              ], textStyle),
            )
            .toList(growable: false),
      ),
    );
  }

  Widget _buildMenu(String title, List<Widget> items, TextStyle? textStyle) {
    return SubmenuButton(
      menuChildren: items,
      child: Text(title, style: textStyle, textAlign: TextAlign.center),
    );
  }

  void _dispatch(NesMenuItemId id) {
    switch (id) {
      case NesMenuItemId.openRom:
        unawaited(actions.openRom());
        break;
      case NesMenuItemId.reset:
        unawaited(actions.reset());
        break;
      case NesMenuItemId.togglePause:
        unawaited(actions.togglePause());
        break;
      case NesMenuItemId.settings:
        unawaited(actions.openSettings());
        break;
      case NesMenuItemId.debugger:
        unawaited(actions.openDebugger());
        break;
      case NesMenuItemId.tools:
        unawaited(actions.openTools());
        break;
    }
  }

  String _desktopLabel(AppLocalizations l10n, NesMenuItemId id) {
    switch (id) {
      case NesMenuItemId.openRom:
        return l10n.menuOpenRom;
      case NesMenuItemId.reset:
        return l10n.menuReset;
      case NesMenuItemId.togglePause:
        return l10n.menuPauseResume;
      case NesMenuItemId.settings:
        return l10n.menuPreferences;
      case NesMenuItemId.debugger:
        return l10n.menuOpenDebuggerWindow;
      case NesMenuItemId.tools:
        return l10n.menuOpenToolsWindow;
    }
  }
}
