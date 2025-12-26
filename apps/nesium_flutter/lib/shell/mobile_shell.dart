import 'dart:async';

import 'package:flutter/material.dart';

import '../domain/nes_state.dart';
import '../features/debugger/debugger_panel.dart';
import '../features/screen/nes_screen_view.dart';
import '../features/tools/tools_panel.dart';
import 'nes_actions.dart';
import 'nes_menu_model.dart';

class MobileShell extends StatelessWidget {
  const MobileShell({super.key, required this.state, required this.actions});

  final NesState state;
  final NesActions actions;

  @override
  Widget build(BuildContext context) {
    final isLandscape =
        MediaQuery.orientationOf(context) == Orientation.landscape;

    return Scaffold(
      appBar: isLandscape ? null : AppBar(title: const Text('Nesium')),
      drawer: _MobileDrawer(actions: actions),
      body: isLandscape
          ? Stack(
              fit: StackFit.expand,
              children: [
                Positioned.fill(
                  child: NesScreenView(
                    error: state.error,
                    textureId: state.textureId,
                  ),
                ),
                Positioned(
                  left: 0,
                  top: 0,
                  child: SafeArea(
                    child: Padding(
                      padding: const EdgeInsets.all(8),
                      child: Builder(
                        builder: (context) => Material(
                          color: Colors.black54,
                          borderRadius: BorderRadius.circular(12),
                          clipBehavior: Clip.antiAlias,
                          child: IconButton(
                            onPressed: () => Scaffold.of(context).openDrawer(),
                            icon: const Icon(Icons.menu),
                            color: Colors.white,
                            tooltip: 'Menu',
                          ),
                        ),
                      ),
                    ),
                  ),
                ),
              ],
            )
          : NesScreenView(error: state.error, textureId: state.textureId),
    );
  }
}

class _MobileDrawer extends StatelessWidget {
  const _MobileDrawer({required this.actions});

  final NesActions actions;

  @override
  Widget build(BuildContext context) {
    void closeDrawer() => Navigator.of(context).pop();

    Future<void> openPage(Widget page) async {
      closeDrawer();
      await Navigator.of(
        context,
      ).push(MaterialPageRoute<void>(builder: (_) => page));
    }

    return Drawer(
      child: SafeArea(
        child: ListView(
          children: [
            const DrawerHeader(
              margin: EdgeInsets.zero,
              child: Align(
                alignment: Alignment.bottomLeft,
                child: Text('Nesium', style: TextStyle(fontSize: 24)),
              ),
            ),
            for (final item in NesMenus.mobileDrawerItems) ...[
              if (item.id == NesMenuItemId.settings ||
                  item.id == NesMenuItemId.debugger) ...[
                const Divider(),
              ],
              ListTile(
                leading: Icon(item.icon),
                title: Text(item.label),
                onTap: () => _dispatch(
                  context,
                  item.id,
                  closeDrawer: closeDrawer,
                  openPage: openPage,
                ),
              ),
            ],
          ],
        ),
      ),
    );
  }

  void _dispatch(
    BuildContext context,
    NesMenuItemId id, {
    required VoidCallback closeDrawer,
    required Future<void> Function(Widget page) openPage,
  }) {
    switch (id) {
      case NesMenuItemId.openRom:
        closeDrawer();
        unawaited(actions.openRom());
        break;
      case NesMenuItemId.reset:
        closeDrawer();
        unawaited(actions.reset());
        break;
      case NesMenuItemId.togglePause:
        closeDrawer();
        unawaited(actions.togglePause());
        break;
      case NesMenuItemId.settings:
        closeDrawer();
        unawaited(actions.openSettings());
        break;
      case NesMenuItemId.debugger:
        unawaited(
          openPage(
            const _MobilePage(title: 'Debugger', child: DebuggerPanel()),
          ),
        );
        break;
      case NesMenuItemId.tools:
        unawaited(
          openPage(const _MobilePage(title: 'Tools', child: ToolsPanel())),
        );
        break;
    }
  }
}

class _MobilePage extends StatelessWidget {
  const _MobilePage({required this.title, required this.child});

  final String title;
  final Widget child;

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: Text(title)),
      body: child,
    );
  }
}
