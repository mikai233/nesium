import 'package:flutter/material.dart';

import '../features/screen/nes_screen_view.dart';
import '../domain/nes_state.dart';

class DesktopShell extends StatelessWidget {
  const DesktopShell({
    super.key,
    required this.state,
    required this.onOpenRom,
    required this.onTogglePause,
    required this.onReset,
    required this.onOpenSettings,
    required this.onOpenDebugger,
    required this.onOpenTools,
  });

  final NesState state;
  final VoidCallback onOpenRom;
  final VoidCallback onTogglePause;
  final VoidCallback onReset;
  final VoidCallback onOpenSettings;
  final VoidCallback onOpenDebugger;
  final VoidCallback onOpenTools;

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
            _DesktopMenuBar(
              onOpenRom: onOpenRom,
              onReset: onReset,
              onTogglePause: onTogglePause,
              onOpenSettings: onOpenSettings,
              onOpenDebugger: onOpenDebugger,
              onOpenTools: onOpenTools,
            ),
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
  const _DesktopMenuBar({
    required this.onOpenRom,
    required this.onReset,
    required this.onTogglePause,
    required this.onOpenSettings,
    required this.onOpenDebugger,
    required this.onOpenTools,
  });

  final VoidCallback onOpenRom;
  final VoidCallback onReset;
  final VoidCallback onTogglePause;
  final VoidCallback onOpenSettings;
  final VoidCallback onOpenDebugger;
  final VoidCallback onOpenTools;

  @override
  Widget build(BuildContext context) {
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
        children: [
          _buildMenu('File', [
            MenuItemButton(
              onPressed: onOpenRom,
              child: const Text('Open ROM...'),
            ),
            MenuItemButton(onPressed: onReset, child: const Text('Reset')),
          ], textStyle),
          _buildMenu('Emulation', [
            MenuItemButton(
              onPressed: onTogglePause,
              child: const Text('Pause / Resume'),
            ),
            MenuItemButton(onPressed: onReset, child: const Text('Soft Reset')),
          ], textStyle),
          _buildMenu('Settings', [
            MenuItemButton(
              onPressed: onOpenSettings,
              child: const Text('Preferences...'),
            ),
            const MenuItemButton(
              onPressed: null,
              child: Text('Input Mapping (coming soon)'),
            ),
          ], textStyle),
          _buildMenu('Windows', [
            MenuItemButton(
              onPressed: onOpenDebugger,
              child: const Text('Open Debugger Window'),
            ),
            MenuItemButton(
              onPressed: onOpenTools,
              child: const Text('Open Tools Window'),
            ),
          ], textStyle),
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
}
