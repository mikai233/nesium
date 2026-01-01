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
    this.slotStates = const {},
    this.hasRom = false,
    this.trailing,
  });

  final NesActions actions;
  final List<NesMenuSectionSpec> sections;
  final Map<int, DateTime?> slotStates;
  final bool hasRom;
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
                    (section) => _buildSubmenu(
                      section.title(l10n),
                      section.items
                          .map((item) => _buildMenuItem(item, l10n))
                          .toList(),
                      textStyle,
                    ),
                  )
                  .toList(growable: false),
            ),
          ),
          if (trailing != null) trailing!,
        ],
      ),
    );
  }

  Widget _buildSubmenu(String title, List<Widget> items, TextStyle? textStyle) {
    return SubmenuButton(
      menuChildren: items,
      child: Text(title, style: textStyle, textAlign: TextAlign.center),
    );
  }

  Widget _buildMenuItem(NesMenuItemSpec item, AppLocalizations l10n) {
    final timestamp = item.slotIndex != null
        ? slotStates[item.slotIndex]
        : null;
    final hasData = timestamp != null;

    if (item.children != null && item.children!.isNotEmpty) {
      final bool isSaveLoad =
          item.id == NesMenuItemId.saveState ||
          item.id == NesMenuItemId.loadState ||
          item.id == NesMenuItemId.autoSave;
      return SubmenuButton(
        menuChildren: isSaveLoad && !hasRom
            ? []
            : item.children!
                  .map((child) => _buildMenuItem(child, l10n))
                  .toList(),
        child: Text(item.label(l10n)),
      );
    }

    Widget? leading;
    bool enabled = true;

    if (item.id == NesMenuItemId.saveStateSlot ||
        item.id == NesMenuItemId.loadStateSlot ||
        item.id == NesMenuItemId.autoSaveSlot) {
      leading = Icon(
        hasData ? Icons.save : Icons.check_box_outline_blank,
        size: 16,
      );

      if (!hasRom) {
        enabled = false;
      } else if ((item.id == NesMenuItemId.loadStateSlot ||
              item.id == NesMenuItemId.autoSaveSlot) &&
          !hasData) {
        enabled = false;
      }
    } else if (item.id == NesMenuItemId.saveStateFile ||
        item.id == NesMenuItemId.loadStateFile) {
      if (!hasRom) {
        enabled = false;
      }
    } else if (item.id == NesMenuItemId.loadTasMovie) {
      enabled = false;
    }

    return MenuItemButton(
      onPressed: enabled ? () => _dispatch(item) : null,
      leadingIcon: leading,
      child: Text(item.label(l10n, timestamp: timestamp)),
    );
  }

  void _dispatch(NesMenuItemSpec item) {
    final id = item.id;
    switch (id) {
      case NesMenuItemId.openRom:
        unawaited(actions.openRom?.call());
        break;
      case NesMenuItemId.saveState:
        break;
      case NesMenuItemId.loadState:
        break;
      case NesMenuItemId.autoSave:
        unawaited(actions.openAutoSave?.call());
        break;
      case NesMenuItemId.saveStateSlot:
        if (item.slotIndex != null) {
          unawaited(actions.saveStateSlot?.call(item.slotIndex!));
        }
        break;
      case NesMenuItemId.loadStateSlot:
      case NesMenuItemId.autoSaveSlot:
        if (item.slotIndex != null) {
          unawaited(actions.loadStateSlot?.call(item.slotIndex!));
        }
        break;
      case NesMenuItemId.saveStateFile:
        unawaited(actions.saveStateFile?.call());
        break;
      case NesMenuItemId.loadStateFile:
        unawaited(actions.loadStateFile?.call());
        break;
      case NesMenuItemId.reset:
        unawaited(actions.reset?.call());
        break;
      case NesMenuItemId.powerReset:
        unawaited(actions.powerReset?.call());
        break;
      case NesMenuItemId.eject:
        unawaited(actions.eject?.call());
        break;
      case NesMenuItemId.togglePause:
        unawaited(actions.togglePause?.call());
        break;
      case NesMenuItemId.loadTasMovie:
        unawaited(actions.loadTasMovie?.call());
        break;
      case NesMenuItemId.settings:
        unawaited(actions.openSettings?.call());
        break;
      case NesMenuItemId.about:
        unawaited(actions.openAbout?.call());
        break;
      case NesMenuItemId.debugger:
        unawaited(actions.openDebugger?.call());
        break;
      case NesMenuItemId.tools:
        unawaited(actions.openTools?.call());
        break;
      case NesMenuItemId.tilemapViewer:
        unawaited(actions.openTilemapViewer?.call());
        break;
    }
  }
}
