import 'package:flutter/material.dart';

import '../l10n/app_localizations.dart';

enum NesMenuItemId {
  openRom,
  saveState,
  loadState,
  autoSave,
  saveStateSlot,
  loadStateSlot,
  autoSaveSlot,
  saveStateFile,
  loadStateFile,
  reset,
  powerReset,
  eject,
  togglePause,
  loadTasMovie,
  settings,
  about,
  debugger,
  tools,
  tilemapViewer,
  tileViewer,
  spriteViewer,
}

class NesMenuItemSpec {
  const NesMenuItemSpec({
    required this.id,
    required this.icon,
    this.children,
    this.slotIndex,
  });

  final NesMenuItemId id;
  final IconData icon;
  final List<NesMenuItemSpec>? children;
  final int? slotIndex;

  String label(AppLocalizations l10n, {DateTime? timestamp}) {
    if ((id == NesMenuItemId.saveStateSlot ||
            id == NesMenuItemId.loadStateSlot ||
            id == NesMenuItemId.autoSaveSlot) &&
        slotIndex != null) {
      final bool isAuto = id == NesMenuItemId.autoSaveSlot || (slotIndex! > 10);
      final displayIndex = isAuto ? slotIndex! - 10 : slotIndex;
      final base =
          '${isAuto ? l10n.autoSlotLabel : l10n.slotLabel} $displayIndex';
      if (timestamp != null) {
        final timeStr = timestamp.toLocal().toString().split('.')[0];
        return '$base ($timeStr)';
      }
      return '$base (${l10n.slotEmpty})';
    }

    return switch (id) {
      NesMenuItemId.openRom => l10n.menuOpenRom,
      NesMenuItemId.saveState => l10n.menuSaveState,
      NesMenuItemId.loadState => l10n.menuLoadState,
      NesMenuItemId.autoSave => l10n.menuAutoSave,
      NesMenuItemId.saveStateSlot ||
      NesMenuItemId.loadStateSlot ||
      NesMenuItemId.autoSaveSlot => 'Slot $slotIndex', // Fallback
      NesMenuItemId.saveStateFile => l10n.saveToExternalFile,
      NesMenuItemId.loadStateFile => l10n.loadFromExternalFile,
      NesMenuItemId.reset => l10n.menuReset,
      NesMenuItemId.powerReset => l10n.menuPowerReset,
      NesMenuItemId.eject => l10n.menuEject,
      NesMenuItemId.togglePause => l10n.menuPauseResume,
      NesMenuItemId.loadTasMovie => l10n.menuLoadTasMovie,
      NesMenuItemId.settings => l10n.menuPreferences,
      NesMenuItemId.about => l10n.menuAbout,
      NesMenuItemId.debugger => l10n.menuDebugger,
      NesMenuItemId.tools => l10n.menuTools,
      NesMenuItemId.tilemapViewer => l10n.menuTilemapViewer,
      NesMenuItemId.tileViewer => l10n.menuTileViewer,
      NesMenuItemId.spriteViewer => l10n.menuSpriteViewer,
    };
  }
}

enum NesMenuSectionId { file, emulation, settings, windows, help }

class NesMenuSectionSpec {
  const NesMenuSectionSpec({required this.id, required this.items});

  final NesMenuSectionId id;
  final List<NesMenuItemSpec> items;

  String title(AppLocalizations l10n) => switch (id) {
    NesMenuSectionId.file => l10n.menuSectionFile,
    NesMenuSectionId.emulation => l10n.menuSectionEmulation,
    NesMenuSectionId.settings => l10n.menuSectionSettings,
    NesMenuSectionId.windows => l10n.menuSectionWindows,
    NesMenuSectionId.help => l10n.menuSectionHelp,
  };
}

class NesMenus {
  static const NesMenuItemSpec openRom = NesMenuItemSpec(
    id: NesMenuItemId.openRom,
    icon: Icons.upload_file,
  );
  static const NesMenuItemSpec saveState = NesMenuItemSpec(
    id: NesMenuItemId.saveState,
    icon: Icons.save,
  );
  static const NesMenuItemSpec loadState = NesMenuItemSpec(
    id: NesMenuItemId.loadState,
    icon: Icons.file_open,
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
  static const NesMenuItemSpec loadTasMovie = NesMenuItemSpec(
    id: NesMenuItemId.loadTasMovie,
    icon: Icons.movie_outlined,
  );
  static const NesMenuItemSpec settings = NesMenuItemSpec(
    id: NesMenuItemId.settings,
    icon: Icons.settings_outlined,
  );
  static const NesMenuItemSpec about = NesMenuItemSpec(
    id: NesMenuItemId.about,
    icon: Icons.info_outline,
  );
  static const NesMenuItemSpec debugger = NesMenuItemSpec(
    id: NesMenuItemId.debugger,
    icon: Icons.bug_report_outlined,
  );
  static const NesMenuItemSpec tools = NesMenuItemSpec(
    id: NesMenuItemId.tools,
    icon: Icons.analytics_outlined,
    children: [
      NesMenuItemSpec(id: NesMenuItemId.tilemapViewer, icon: Icons.grid_on),
      NesMenuItemSpec(id: NesMenuItemId.tileViewer, icon: Icons.apps),
      NesMenuItemSpec(id: NesMenuItemId.spriteViewer, icon: Icons.animation),
    ],
  );
  static List<NesMenuItemSpec> _buildSaveStateChildren() => [
    for (int i = 1; i <= 10; i++)
      NesMenuItemSpec(
        id: NesMenuItemId.saveStateSlot,
        icon: Icons.save,
        slotIndex: i,
      ),
    const NesMenuItemSpec(
      id: NesMenuItemId.saveStateFile,
      icon: Icons.file_upload,
    ),
  ];
  static List<NesMenuItemSpec> _buildLoadStateChildren() => [
    for (int i = 1; i <= 10; i++)
      NesMenuItemSpec(
        id: NesMenuItemId.loadStateSlot,
        icon: Icons.file_open,
        slotIndex: i,
      ),
    const NesMenuItemSpec(
      id: NesMenuItemId.loadStateFile,
      icon: Icons.file_download,
    ),
  ];
  static List<NesMenuItemSpec> _buildAutoSaveChildren() => [
    for (int i = 11; i <= 20; i++)
      NesMenuItemSpec(
        id: NesMenuItemId.autoSaveSlot,
        icon: Icons.history,
        slotIndex: i,
      ),
  ];
  static const List<NesMenuItemSpec> mobileDrawerItems = [
    openRom,
    saveState,
    loadState,
    NesMenuItemSpec(id: NesMenuItemId.autoSave, icon: Icons.history),
    reset,
    powerReset,
    eject,
    togglePause,
    debugger,
    tools,
    settings,
    about,
  ];
  static List<NesMenuSectionSpec> desktopMenuSections() => [
    NesMenuSectionSpec(
      id: NesMenuSectionId.file,
      items: [
        openRom,
        NesMenuItemSpec(
          id: NesMenuItemId.saveState,
          icon: Icons.save,
          children: _buildSaveStateChildren(),
        ),
        NesMenuItemSpec(
          id: NesMenuItemId.loadState,
          icon: Icons.file_open,
          children: _buildLoadStateChildren(),
        ),
        NesMenuItemSpec(
          id: NesMenuItemId.autoSave,
          icon: Icons.history,
          children: _buildAutoSaveChildren(),
        ),
      ],
    ),
    const NesMenuSectionSpec(
      id: NesMenuSectionId.emulation,
      items: [togglePause, loadTasMovie, reset, powerReset, eject],
    ),
    const NesMenuSectionSpec(id: NesMenuSectionId.settings, items: [settings]),
    const NesMenuSectionSpec(
      id: NesMenuSectionId.windows,
      items: [debugger, tools],
    ),
    const NesMenuSectionSpec(id: NesMenuSectionId.help, items: [about]),
  ];

  /// Minimal menu for Web builds (no debugger/tools).
  static List<NesMenuSectionSpec> webMenuSections() => [
    NesMenuSectionSpec(
      id: NesMenuSectionId.file,
      items: [
        openRom,
        NesMenuItemSpec(
          id: NesMenuItemId.saveState,
          icon: Icons.save,
          children: _buildSaveStateChildren(),
        ),
        NesMenuItemSpec(
          id: NesMenuItemId.loadState,
          icon: Icons.file_open,
          children: _buildLoadStateChildren(),
        ),
        NesMenuItemSpec(
          id: NesMenuItemId.autoSave,
          icon: Icons.history,
          children: _buildAutoSaveChildren(),
        ),
      ],
    ),
    const NesMenuSectionSpec(
      id: NesMenuSectionId.emulation,
      items: [togglePause, loadTasMovie, reset, powerReset],
    ),
    const NesMenuSectionSpec(id: NesMenuSectionId.settings, items: [settings]),
    const NesMenuSectionSpec(id: NesMenuSectionId.help, items: [about]),
  ];
}
