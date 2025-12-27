import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../domain/nes_state.dart';
import '../features/controls/input_settings.dart';
import '../features/controls/virtual_controls_editor.dart';
import '../features/controls/virtual_controls_overlay.dart';
import '../features/debugger/debugger_panel.dart';
import '../features/screen/nes_screen_view.dart';
import '../features/tools/tools_panel.dart';
import '../l10n/app_localizations.dart';
import 'nes_actions.dart';
import 'nes_menu_model.dart';

class MobileShell extends StatelessWidget {
  const MobileShell({super.key, required this.state, required this.actions});

  final NesState state;
  final NesActions actions;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    final isLandscape =
        MediaQuery.orientationOf(context) == Orientation.landscape;

    return Scaffold(
      appBar: isLandscape ? null : AppBar(title: Text(l10n.appName)),
      drawer: _MobileDrawer(actions: actions),
      body: Stack(
        fit: StackFit.expand,
        children: [
          Positioned.fill(
            child: NesScreenView(
              error: state.error,
              textureId: state.textureId,
            ),
          ),
          if (isLandscape)
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
                        tooltip: l10n.menuTooltip,
                      ),
                    ),
                  ),
                ),
              ),
            ),
          VirtualControlsOverlay(isLandscape: isLandscape),
        ],
      ),
    );
  }
}

class _MobileDrawer extends StatelessWidget {
  const _MobileDrawer({required this.actions});

  final NesActions actions;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context)!;
    void closeDrawer() => Navigator.of(context).pop();

    Future<void> openPage(Widget page) async {
      closeDrawer();
      await Navigator.of(
        context,
      ).push(MaterialPageRoute<void>(builder: (_) => page));
    }

    return Consumer(
      builder: (context, ref, _) {
        final inputSettings = ref.watch(inputSettingsProvider);
        final inputCtrl = ref.read(inputSettingsProvider.notifier);

        final editor = ref.watch(virtualControlsEditorProvider);
        final editorCtrl = ref.read(virtualControlsEditorProvider.notifier);

        return Drawer(
          child: SafeArea(
            child: ListView(
              children: [
                DrawerHeader(
                  margin: EdgeInsets.zero,
                  child: Align(
                    alignment: Alignment.bottomLeft,
                    child: Text(
                      l10n.appName,
                      style: const TextStyle(fontSize: 24),
                    ),
                  ),
                ),
                for (final item in NesMenus.mobileDrawerItems) ...[
                  if (item.id == NesMenuItemId.settings ||
                      item.id == NesMenuItemId.debugger) ...[
                    const Divider(),
                  ],
                  ListTile(
                    leading: Icon(item.icon),
                    title: Text(item.label(l10n)),
                    onTap: () => _dispatch(
                      context,
                      item.id,
                      closeDrawer: closeDrawer,
                      openPage: openPage,
                    ),
                  ),
                ],
                const Divider(),
                ListTile(
                  leading: const Icon(Icons.tune),
                  title: Text(l10n.virtualControlsEditTitle),
                  subtitle: Text(
                    editor.enabled
                        ? l10n.virtualControlsEditSubtitleEnabled
                        : l10n.virtualControlsEditSubtitleDisabled,
                  ),
                  trailing: Switch(
                    value: editor.enabled,
                    onChanged: (enabled) {
                      if (enabled &&
                          inputSettings.device !=
                              InputDevice.virtualController) {
                        inputCtrl.setDevice(InputDevice.virtualController);
                      }
                      editorCtrl.setEnabled(enabled);
                      closeDrawer();
                    },
                  ),
                ),
                if (editor.enabled) ...[
                  SwitchListTile(
                    secondary: const Icon(Icons.grid_4x4),
                    title: Text(l10n.gridSnappingTitle),
                    value: editor.gridSnapEnabled,
                    onChanged: editorCtrl.setGridSnapEnabled,
                  ),
                  if (editor.gridSnapEnabled)
                    Padding(
                      padding: const EdgeInsets.symmetric(
                        horizontal: 16,
                        vertical: 4,
                      ),
                      child: Column(
                        crossAxisAlignment: CrossAxisAlignment.start,
                        children: [
                          Row(
                            children: [
                              Expanded(child: Text(l10n.gridSpacingLabel)),
                              Text(
                                '${editor.gridSpacing.toStringAsFixed(0)} px',
                              ),
                            ],
                          ),
                          Slider(
                            value: editor.gridSpacing.clamp(4, 64),
                            min: 4,
                            max: 64,
                            divisions: 60,
                            onChanged: editorCtrl.setGridSpacing,
                          ),
                        ],
                      ),
                    ),
                ],
              ],
            ),
          ),
        );
      },
    );
  }

  void _dispatch(
    BuildContext context,
    NesMenuItemId id, {
    required VoidCallback closeDrawer,
    required Future<void> Function(Widget page) openPage,
  }) {
    final l10n = AppLocalizations.of(context)!;
    switch (id) {
      case NesMenuItemId.openRom:
        closeDrawer();
        unawaited(actions.openRom());
        break;
      case NesMenuItemId.reset:
        closeDrawer();
        unawaited(actions.reset());
        break;
      case NesMenuItemId.powerReset:
        closeDrawer();
        unawaited(actions.powerReset());
        break;
      case NesMenuItemId.eject:
        closeDrawer();
        unawaited(actions.eject());
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
            _MobilePage(
              title: l10n.windowDebuggerTitle,
              child: const DebuggerPanel(),
            ),
          ),
        );
        break;
      case NesMenuItemId.tools:
        unawaited(
          openPage(
            _MobilePage(
              title: l10n.windowToolsTitle,
              child: const ToolsPanel(),
            ),
          ),
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
