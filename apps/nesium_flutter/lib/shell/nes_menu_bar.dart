import 'dart:async';

import 'package:flutter/material.dart';

import '../l10n/app_localizations.dart';
import 'nes_actions.dart';
import 'nes_menu_model.dart';

class NesMenuBar extends StatelessWidget {
  const NesMenuBar({
    super.key,
    required this.actions,
    required this.sections,
    this.trailing,
  });

  final NesActions actions;
  final List<NesMenuSectionSpec> sections;
  final Widget? trailing;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final textStyle = Theme.of(context).textTheme.titleSmall;

    return Container(
      height: 28,
      decoration: const BoxDecoration(
        color: Color(0xFFF7F7F7),
        border: Border(
          bottom: BorderSide(color: Color(0xFFE0E0E0), width: 0.5),
        ),
      ),
      alignment: Alignment.centerLeft,
      child: Row(
        children: [
          Expanded(
            child: MenuBar(
              style: MenuStyle(
                padding: WidgetStateProperty.all(EdgeInsets.zero),
              ),
              children: sections
                  .map(
                    (section) => _buildMenu(section.title(l10n), [
                      ...section.items.map(
                        (item) => MenuItemButton(
                          onPressed: () => _dispatch(item.id),
                          child: Text(item.label(l10n)),
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
          ),
          if (trailing != null) trailing!,
        ],
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
      case NesMenuItemId.powerReset:
        unawaited(actions.powerReset());
        break;
      case NesMenuItemId.eject:
        unawaited(actions.eject());
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
}
