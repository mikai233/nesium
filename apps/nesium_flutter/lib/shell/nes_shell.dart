import 'dart:async';

import 'package:flutter/foundation.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:file_picker/file_picker.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/bridge/api/load_rom.dart' as nes_api;
import 'package:nesium_flutter/bridge/api/input.dart' as nes_input;
import 'package:nesium_flutter/bridge/api/pause.dart' as nes_pause;
import 'package:nesium_flutter/bridge/lib.dart' show PadButton;

import '../domain/nes_controller.dart';
import '../domain/nes_input_masks.dart';
import '../domain/nes_state.dart';
import '../features/controls/input_settings.dart';
import '../features/controls/virtual_controls_settings.dart';
import '../features/settings/emulation_settings.dart';
import '../features/settings/settings_page.dart';
import '../platform/desktop_window_manager.dart';
import 'desktop_shell.dart';
import 'nes_actions.dart';
import 'mobile_shell.dart';

class NesShell extends ConsumerStatefulWidget {
  const NesShell({super.key});

  @override
  ConsumerState<NesShell> createState() => _NesShellState();
}

class _NesShellState extends ConsumerState<NesShell>
    with WidgetsBindingObserver {
  final DesktopWindowManager _desktopWindowManager =
      const DesktopWindowManager();
  final FocusNode _focusNode = FocusNode();
  bool _pausedByLifecycle = false;

  bool get _isDesktop =>
      !kIsWeb &&
      (defaultTargetPlatform == TargetPlatform.macOS ||
          defaultTargetPlatform == TargetPlatform.linux ||
          defaultTargetPlatform == TargetPlatform.windows);

  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    Future.microtask(() async {
      // Start the NES runtime thread on the Rust side via FRB,
      // then initialize the texture used for rendering.
      try {
        await nes_api.startNesRuntime();
      } catch (_) {
        // Let UI report errors lazily when commands are used.
      }
      await ref.read(nesControllerProvider.notifier).initTexture();
      final frames = ref
          .read(virtualControlsSettingsProvider)
          .turboFramesPerToggle;
      await nes_input
          .setTurboFramesPerToggle(frames: frames)
          .catchError((_) {});
    });
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    _focusNode.dispose();
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    final pauseInBackground = ref
        .read(emulationSettingsProvider)
        .pauseInBackground;
    if (!pauseInBackground) {
      // Never auto-pause; also don't auto-resume.
      _pausedByLifecycle = false;
      return;
    }

    switch (state) {
      case AppLifecycleState.resumed:
        if (_pausedByLifecycle) {
          _pausedByLifecycle = false;
          unawaited(nes_pause.setPaused(paused: false).catchError((_) {}));
        }
        break;
      case AppLifecycleState.inactive:
      case AppLifecycleState.paused:
      case AppLifecycleState.hidden:
      case AppLifecycleState.detached:
        unawaited(_pauseForLifecycle());
        break;
    }
  }

  Future<void> _pauseForLifecycle() async {
    try {
      final wasPaused = await nes_pause.isPaused();
      if (wasPaused) return;
      _pausedByLifecycle = true;
      await nes_pause.setPaused(paused: true);
    } catch (_) {}
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
    // Avoid sending key events to the emulator when a different route (e.g. settings)
    // is on top.
    final route = ModalRoute.of(context);
    if (route != null && !route.isCurrent) {
      return KeyEventResult.ignored;
    }

    // Treat key repeat as a continued key down to avoid system beeps.
    if (event is! KeyDownEvent &&
        event is! KeyUpEvent &&
        event is! KeyRepeatEvent) {
      return KeyEventResult.ignored;
    }

    final pressed = event is KeyDownEvent || event is KeyRepeatEvent;
    final key = event.logicalKey;

    final inputSettings = ref.read(inputSettingsProvider);
    if (inputSettings.device != InputDevice.keyboard) {
      return KeyEventResult.ignored;
    }

    final action = inputSettings.resolveKeyboardBindings()[key];
    if (action == null) return KeyEventResult.ignored;

    final input = ref.read(nesInputMasksProvider.notifier);
    switch (action) {
      case KeyboardBindingAction.up:
        input.setPressed(PadButton.up, pressed);
        break;
      case KeyboardBindingAction.down:
        input.setPressed(PadButton.down, pressed);
        break;
      case KeyboardBindingAction.left:
        input.setPressed(PadButton.left, pressed);
        break;
      case KeyboardBindingAction.right:
        input.setPressed(PadButton.right, pressed);
        break;
      case KeyboardBindingAction.a:
        input.setPressed(PadButton.a, pressed);
        break;
      case KeyboardBindingAction.b:
        input.setPressed(PadButton.b, pressed);
        break;
      case KeyboardBindingAction.select:
        input.setPressed(PadButton.select, pressed);
        break;
      case KeyboardBindingAction.start:
        input.setPressed(PadButton.start, pressed);
        break;
      case KeyboardBindingAction.turboA:
        input.setTurboEnabled(PadButton.a, pressed);
        break;
      case KeyboardBindingAction.turboB:
        input.setTurboEnabled(PadButton.b, pressed);
        break;
    }

    return KeyEventResult.handled;
  }

  Future<void> _togglePause() async {
    try {
      _pausedByLifecycle = false;
      final paused = await nes_pause.togglePause();
      if (!mounted) return;
      _showSnack(paused ? 'Paused' : 'Resumed');
    } catch (e) {
      if (!mounted) return;
      _showSnack('Pause failed: $e');
    }
  }

  Future<void> _openSettings() async {
    if (!mounted) return;
    await Navigator.of(
      context,
    ).push(MaterialPageRoute<void>(builder: (_) => const SettingsPage()));
  }

  Future<void> _openDebugger() async {
    await _desktopWindowManager.openDebuggerWindow();
  }

  Future<void> _openTools() async {
    await _desktopWindowManager.openToolsWindow();
  }

  @override
  Widget build(BuildContext context) {
    final NesState state = ref.watch(nesControllerProvider);

    final actions = NesActions(
      openRom: _promptAndLoadRom,
      reset: _resetConsole,
      togglePause: _togglePause,
      openSettings: _openSettings,
      openDebugger: _openDebugger,
      openTools: _openTools,
    );

    final shell = _isDesktop
        ? DesktopShell(state: state, actions: actions)
        : MobileShell(state: state, actions: actions);

    return Focus(
      focusNode: _focusNode,
      autofocus: true,
      onKeyEvent: _handleKeyEvent,
      child: shell,
    );
  }
}
