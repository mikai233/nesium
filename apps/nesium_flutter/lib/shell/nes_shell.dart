import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/src/rust/api/load_rom.dart' as nes_api;
import 'package:nesium_flutter/src/rust/api/input.dart' as nes_input;
import 'package:nesium_flutter/src/rust/lib.dart' show PadButton;

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
  final FocusNode _focusNode = FocusNode();

  bool get _isDesktop =>
      !kIsWeb &&
      (defaultTargetPlatform == TargetPlatform.macOS ||
          defaultTargetPlatform == TargetPlatform.linux ||
          defaultTargetPlatform == TargetPlatform.windows);

  @override
  void initState() {
    super.initState();
    Future.microtask(() async {
      // Start the NES runtime thread on the Rust side via FRB,
      // then initialize the texture used for rendering.
      try {
        await nes_api.startNesRuntime();
      } catch (_) {
        // Let UI report errors lazily when commands are used.
      }
      await ref.read(nesControllerProvider.notifier).initTexture();
    });
  }

  void _selectPane(NesPane pane) {
    ref.read(selectedPaneProvider.notifier).state = pane;
  }

  void _showTodo(String label) {
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text('$label: TODO (wire via FRB)')));
  }

  void _showSnack(String message) {
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text(message)));
  }

  Future<void> _runRustCommand(
    String label,
    Future<void> Function() action, {
    bool showSuccessSnack = true,
  }) async {
    try {
      await action();
      if (!mounted) return;
      if (showSuccessSnack) {
        _showSnack('$label succeeded');
      }
    } catch (e) {
      if (!mounted) return;
      await showDialog<void>(
        context: context,
        builder: (context) => AlertDialog(
          title: Text('$label failed'),
          content: Text('$e'),
          actions: [
            TextButton(
              onPressed: () => Navigator.of(context).pop(),
              child: const Text('OK'),
            ),
          ],
        ),
      );
    }
  }

  Future<void> _promptAndLoadRom() async {
    final result = await FilePicker.platform.pickFiles(
      type: FileType.custom,
      allowedExtensions: ['nes'],
      withReadStream: false,
    );
    final path = result?.files.single.path;
    if (path == null || path.isEmpty) {
      return;
    }

    await _runRustCommand('Load ROM', () => nes_api.loadRom(path: path));
  }

  Future<void> _resetConsole() async {
    await _runRustCommand('Reset NES', nes_api.resetConsole);
  }

  KeyEventResult _handleKeyEvent(FocusNode _, KeyEvent event) {
    // Treat key repeat as a continued key down to avoid system beeps.
    if (event is! KeyDownEvent &&
        event is! KeyUpEvent &&
        event is! KeyRepeatEvent) {
      return KeyEventResult.ignored;
    }

    final pressed = event is KeyDownEvent || event is KeyRepeatEvent;
    final key = event.logicalKey;

    // Map a handful of logical keys to NES buttons.
    final mapping = <LogicalKeyboardKey, PadButton>{
      LogicalKeyboardKey.keyZ: PadButton.a,
      LogicalKeyboardKey.keyX: PadButton.b,
      LogicalKeyboardKey.shiftLeft: PadButton.select,
      LogicalKeyboardKey.shiftRight: PadButton.select,
      LogicalKeyboardKey.enter: PadButton.start,
      LogicalKeyboardKey.arrowUp: PadButton.up,
      LogicalKeyboardKey.arrowDown: PadButton.down,
      LogicalKeyboardKey.arrowLeft: PadButton.left,
      LogicalKeyboardKey.arrowRight: PadButton.right,
    };

    final button = mapping[key];
    if (button == null) return KeyEventResult.ignored;

    nes_input
        .setButton(pad: 0, button: button, pressed: pressed)
        .catchError((e) => _showSnack('Input error: $e'));

    return KeyEventResult.handled;
  }

  @override
  Widget build(BuildContext context) {
    final NesState state = ref.watch(nesControllerProvider);
    final NesPane selectedPane = ref.watch(selectedPaneProvider);

    final callbacks = (
      openRom: _promptAndLoadRom,
      togglePause: () => _showTodo('Pause/Resume'),
      reset: _resetConsole,
      openSettings: () => _showTodo('Settings'),
      openDebugger: () => _desktopWindowManager.openDebuggerWindow(),
      openTools: () => _desktopWindowManager.openToolsWindow(),
    );

    if (_isDesktop) {
      return Focus(
        focusNode: _focusNode,
        autofocus: true,
        onKeyEvent: _handleKeyEvent,
        child: DesktopShell(
          state: state,
          onOpenRom: callbacks.openRom,
          onTogglePause: callbacks.togglePause,
          onReset: callbacks.reset,
          onOpenSettings: callbacks.openSettings,
          onOpenDebugger: callbacks.openDebugger,
          onOpenTools: callbacks.openTools,
        ),
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
