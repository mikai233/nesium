import 'package:flutter/material.dart';

import '../l10n/app_localizations.dart';

enum NesMenuItemId {
  openRom,
  reset,
  powerReset,
  eject,
  togglePause,
  settings,
  debugger,
  tools,
}

class NesMenuItemSpec {
  const NesMenuItemSpec({required this.id, required this.icon});

  final NesMenuItemId id;
  final IconData icon;

  String label(AppLocalizations l10n) => switch (id) {
    NesMenuItemId.openRom => l10n.menuOpenRom,
    NesMenuItemId.reset => l10n.menuReset,
    NesMenuItemId.powerReset => l10n.menuPowerReset,
    NesMenuItemId.eject => l10n.menuEject,
    NesMenuItemId.togglePause => l10n.menuPauseResume,
    NesMenuItemId.settings => l10n.menuPreferences,
    NesMenuItemId.debugger => l10n.menuDebugger,
    NesMenuItemId.tools => l10n.menuTools,
  };
}

enum NesMenuSectionId { file, emulation, settings, windows }

class NesMenuSectionSpec {
  const NesMenuSectionSpec({required this.id, required this.items});

  final NesMenuSectionId id;
  final List<NesMenuItemSpec> items;

  String title(AppLocalizations l10n) => switch (id) {
    NesMenuSectionId.file => l10n.menuSectionFile,
    NesMenuSectionId.emulation => l10n.menuSectionEmulation,
    NesMenuSectionId.settings => l10n.menuSectionSettings,
    NesMenuSectionId.windows => l10n.menuSectionWindows,
  };
}

class NesMenus {
  static const NesMenuItemSpec openRom = NesMenuItemSpec(
    id: NesMenuItemId.openRom,
    icon: Icons.upload_file,
  );

  static const NesMenuItemSpec reset = NesMenuItemSpec(
    id: NesMenuItemId.reset,
    icon: Icons.restart_alt,
  );

  static const NesMenuItemSpec powerReset = NesMenuItemSpec(
    id: NesMenuItemId.powerReset,
    icon: Icons.power_settings_new,
  );

  static const NesMenuItemSpec eject = NesMenuItemSpec(
    id: NesMenuItemId.eject,
    icon: Icons.eject,
  );

  static const NesMenuItemSpec togglePause = NesMenuItemSpec(
    id: NesMenuItemId.togglePause,
    icon: Icons.pause_circle_outline,
  );

  static const NesMenuItemSpec settings = NesMenuItemSpec(
    id: NesMenuItemId.settings,
    icon: Icons.settings_outlined,
  );

  static const NesMenuItemSpec debugger = NesMenuItemSpec(
    id: NesMenuItemId.debugger,
    icon: Icons.bug_report_outlined,
  );

  static const NesMenuItemSpec tools = NesMenuItemSpec(
    id: NesMenuItemId.tools,
    icon: Icons.analytics_outlined,
  );

  static const List<NesMenuItemSpec> mobileDrawerItems = [
    openRom,
    reset,
    powerReset,
    eject,
    togglePause,
    debugger,
    tools,
    settings,
  ];

  static const List<NesMenuSectionSpec> desktopMenuSections = [
    NesMenuSectionSpec(id: NesMenuSectionId.file, items: [openRom]),
    NesMenuSectionSpec(
      id: NesMenuSectionId.emulation,
      items: [togglePause, reset, powerReset, eject],
    ),
    NesMenuSectionSpec(id: NesMenuSectionId.settings, items: [settings]),
    NesMenuSectionSpec(id: NesMenuSectionId.windows, items: [debugger, tools]),
  ];
}
