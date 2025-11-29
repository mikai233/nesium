import 'package:flutter/material.dart';

import '../domain/nes_state.dart';
import 'pane_switcher.dart';
import 'panes.dart';

class MobileShell extends StatelessWidget {
  const MobileShell({
    super.key,
    required this.state,
    required this.selectedPane,
    required this.onSelectPane,
    required this.onOpenRom,
    required this.onTogglePause,
    required this.onReset,
    required this.onOpenSettings,
  });

  final NesState state;
  final NesPane selectedPane;
  final ValueChanged<NesPane> onSelectPane;
  final VoidCallback onOpenRom;
  final VoidCallback onTogglePause;
  final VoidCallback onReset;
  final VoidCallback onOpenSettings;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(
        title: const Text('Nesium'),
        actions: [
          _OverflowMenu(
            onOpenRom: onOpenRom,
            onReset: onReset,
            onTogglePause: onTogglePause,
            onOpenSettings: onOpenSettings,
          ),
        ],
      ),
      floatingActionButton: FloatingActionButton.extended(
        onPressed: onOpenRom,
        icon: const Icon(Icons.upload_file),
        label: const Text('Open ROM'),
      ),
      bottomNavigationBar: NavigationBar(
        selectedIndex: selectedPane.index,
        onDestinationSelected: (index) {
          onSelectPane(NesPane.values[index]);
        },
        destinations: const [
          NavigationDestination(
            icon: Icon(Icons.videogame_asset_outlined),
            selectedIcon: Icon(Icons.videogame_asset),
            label: 'Console',
          ),
          NavigationDestination(
            icon: Icon(Icons.bug_report_outlined),
            selectedIcon: Icon(Icons.bug_report),
            label: 'Debugger',
          ),
          NavigationDestination(
            icon: Icon(Icons.analytics_outlined),
            selectedIcon: Icon(Icons.analytics),
            label: 'Tools',
          ),
        ],
      ),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: PaneSwitcher(state: state, selectedPane: selectedPane),
        ),
      ),
    );
  }
}

class _OverflowMenu extends StatelessWidget {
  const _OverflowMenu({
    required this.onOpenRom,
    required this.onReset,
    required this.onTogglePause,
    required this.onOpenSettings,
  });

  final VoidCallback onOpenRom;
  final VoidCallback onReset;
  final VoidCallback onTogglePause;
  final VoidCallback onOpenSettings;

  @override
  Widget build(BuildContext context) {
    return PopupMenuButton<_OverflowAction>(
      onSelected: (action) {
        switch (action) {
          case _OverflowAction.openRom:
            onOpenRom();
            break;
          case _OverflowAction.reset:
            onReset();
            break;
          case _OverflowAction.togglePause:
            onTogglePause();
            break;
          case _OverflowAction.settings:
            onOpenSettings();
            break;
        }
      },
      itemBuilder: (context) => const [
        PopupMenuItem(
          value: _OverflowAction.openRom,
          child: Text('Open ROM...'),
        ),
        PopupMenuItem(value: _OverflowAction.reset, child: Text('Reset')),
        PopupMenuItem(
          value: _OverflowAction.togglePause,
          child: Text('Pause / Resume'),
        ),
        PopupMenuDivider(),
        PopupMenuItem(value: _OverflowAction.settings, child: Text('Settings')),
      ],
    );
  }
}

enum _OverflowAction { openRom, reset, togglePause, settings }
