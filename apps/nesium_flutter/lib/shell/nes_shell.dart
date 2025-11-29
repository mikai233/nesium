import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../domain/nes_controller.dart';
import '../domain/nes_state.dart';
import '../platform/desktop_window_manager.dart';
import 'desktop_shell.dart';
import 'mobile_shell.dart';
import 'panes.dart';

class NesShell extends ConsumerStatefulWidget {
  const NesShell({super.key});

  @override
  ConsumerState<NesShell> createState() => _NesShellState();
}

class _NesShellState extends ConsumerState<NesShell> {
  final DesktopWindowManager _desktopWindowManager =
      const DesktopWindowManager();

  bool get _isDesktop =>
      !kIsWeb &&
      (defaultTargetPlatform == TargetPlatform.macOS ||
          defaultTargetPlatform == TargetPlatform.linux ||
          defaultTargetPlatform == TargetPlatform.windows);

  @override
  void initState() {
    super.initState();
    Future.microtask(
      () => ref.read(nesControllerProvider.notifier).initTexture(),
    );
  }

  void _selectPane(NesPane pane) {
    ref.read(selectedPaneProvider.notifier).state = pane;
  }

  void _showTodo(String label) {
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text('$label: TODO (wire via FRB)')));
  }

  @override
  Widget build(BuildContext context) {
    final NesState state = ref.watch(nesControllerProvider);
    final NesPane selectedPane = ref.watch(selectedPaneProvider);

    final callbacks = (
      openRom: () => _showTodo('Open ROM'),
      togglePause: () => _showTodo('Pause/Resume'),
      reset: () => _showTodo('Reset'),
      openSettings: () => _showTodo('Settings'),
      openDebugger: () => _desktopWindowManager.openDebuggerWindow(),
      openTools: () => _desktopWindowManager.openToolsWindow(),
    );

    if (_isDesktop) {
      return DesktopShell(
        state: state,
        onOpenRom: callbacks.openRom,
        onTogglePause: callbacks.togglePause,
        onReset: callbacks.reset,
        onOpenSettings: callbacks.openSettings,
        onOpenDebugger: callbacks.openDebugger,
        onOpenTools: callbacks.openTools,
      );
    }

    return MobileShell(
      state: state,
      selectedPane: selectedPane,
      onSelectPane: _selectPane,
      onOpenRom: callbacks.openRom,
      onTogglePause: callbacks.togglePause,
      onReset: callbacks.reset,
      onOpenSettings: callbacks.openSettings,
    );
  }
}
