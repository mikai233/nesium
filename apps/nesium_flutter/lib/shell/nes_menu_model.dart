import 'package:flutter/material.dart';

enum NesMenuItemId { openRom, reset, togglePause, settings, debugger, tools }

class NesMenuItemSpec {
  const NesMenuItemSpec({
    required this.id,
    required this.label,
    required this.icon,
  });

  final NesMenuItemId id;
  final String label;
  final IconData icon;
}

class NesMenuSectionSpec {
  const NesMenuSectionSpec({required this.title, required this.items});

  final String title;
  final List<NesMenuItemSpec> items;
}

class NesMenus {
  static const NesMenuItemSpec openRom = NesMenuItemSpec(
    id: NesMenuItemId.openRom,
    label: 'Open ROM...',
    icon: Icons.upload_file,
  );

  static const NesMenuItemSpec reset = NesMenuItemSpec(
    id: NesMenuItemId.reset,
    label: 'Reset',
    icon: Icons.restart_alt,
  );

  static const NesMenuItemSpec togglePause = NesMenuItemSpec(
    id: NesMenuItemId.togglePause,
    label: 'Pause / Resume',
    icon: Icons.pause_circle_outline,
  );

  static const NesMenuItemSpec settings = NesMenuItemSpec(
    id: NesMenuItemId.settings,
    label: 'Settings',
    icon: Icons.settings_outlined,
  );

  static const NesMenuItemSpec debugger = NesMenuItemSpec(
    id: NesMenuItemId.debugger,
    label: 'Debugger',
    icon: Icons.bug_report_outlined,
  );

  static const NesMenuItemSpec tools = NesMenuItemSpec(
    id: NesMenuItemId.tools,
    label: 'Tools',
    icon: Icons.analytics_outlined,
  );

  static const List<NesMenuItemSpec> mobileDrawerItems = [
    openRom,
    reset,
    togglePause,
    debugger,
    tools,
    settings,
  ];

  static const List<NesMenuSectionSpec> desktopMenuSections = [
    NesMenuSectionSpec(title: 'File', items: [openRom, reset]),
    NesMenuSectionSpec(title: 'Emulation', items: [togglePause, reset]),
    NesMenuSectionSpec(title: 'Settings', items: [settings]),
    NesMenuSectionSpec(title: 'Windows', items: [debugger, tools]),
  ];
}
