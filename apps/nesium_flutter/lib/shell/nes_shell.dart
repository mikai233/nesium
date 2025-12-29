import 'dart:async';

import 'package:file_selector/file_selector.dart';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:file_picker/file_picker.dart';
import 'package:path/path.dart' as p;
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/bridge/api/events.dart' as nes_events;
import 'package:nesium_flutter/bridge/api/load_rom.dart' as nes_api;
import 'package:nesium_flutter/bridge/api/input.dart' as nes_input;
import 'package:nesium_flutter/bridge/api/pause.dart' as nes_pause;
import 'package:nesium_flutter/bridge/api/emulation.dart' as nes_emulation;

import '../domain/nes_controller.dart';
import '../domain/nes_input_masks.dart';
import '../domain/nes_state.dart';
import '../domain/pad_button.dart';
import '../features/controls/input_settings.dart';
import '../features/controls/turbo_settings.dart';
import '../features/save_state/save_state_dialog.dart';
import '../features/save_state/save_state_repository.dart';
import '../features/settings/emulation_settings.dart';
import '../features/settings/language_settings.dart';
import '../features/settings/settings_page.dart';
import '../features/about/about_page.dart';
import '../l10n/app_localizations.dart';
import '../logging/app_logger.dart';
import '../platform/desktop_window_manager.dart';
import '../platform/platform_capabilities.dart';
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
  StreamSubscription<nes_events.RuntimeNotification>? _runtimeNotificationsSub;

  bool get _isDesktop => isNativeDesktop;

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
      _startRuntimeEvents();
      await ref.read(nesControllerProvider.notifier).initTexture();
      final turbo = ref.read(turboSettingsProvider);
      await nes_input
          .setTurboTiming(onFrames: turbo.onFrames, offFrames: turbo.offFrames)
          .catchError((Object e, StackTrace st) {
            logError(
              e,
              stackTrace: st,
              message: 'setTurboTiming (init)',
              logger: 'nes_shell',
            );
          });
    });
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    _runtimeNotificationsSub?.cancel();
    _focusNode.dispose();
    super.dispose();
  }

  void _startRuntimeEvents() {
    if (!mounted) return;
    if (_runtimeNotificationsSub != null) return;

    _runtimeNotificationsSub = nes_events.runtimeNotifications().listen((
      notification,
    ) {
      if (!mounted) return;

      switch (notification.kind) {
        case nes_events.RuntimeNotificationKind.audioInitFailed:
          final error = notification.error ?? 'unknown error';
          _showSnack('Audio init failed: $error');
          break;
      }
    }, onError: (_) {});
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
          unawaitedLogged(
            nes_pause.setPaused(paused: false),
            message: 'setPaused(false) (resume)',
            logger: 'nes_shell',
          );
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
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'pauseForLifecycle failed',
        logger: 'nes_shell',
      );
    }
  }

  void _showSnack(String message) {
    ScaffoldMessenger.of(
      context,
    ).showSnackBar(SnackBar(content: Text(message)));
  }

  Future<void> _runRustCommand(
    String label,
    Future<void> Function() action,
  ) async {
    try {
      await action();
    } catch (e) {
      if (!mounted) return;
      final l10n = AppLocalizations.of(context)!;
      _showSnack('${l10n.commandFailed(label)}: $e');
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

    if (!mounted) return;
    final l10n = AppLocalizations.of(context)!;
    await _runRustCommand(l10n.actionLoadRom, () async {
      await nes_api.loadRom(path: path);
      final name = p.basenameWithoutExtension(path);
      await ref.read(nesControllerProvider.notifier).refreshRomHash();
      ref
          .read(nesControllerProvider.notifier)
          .updateRomInfo(
            hash: ref.read(nesControllerProvider).romHash,
            name: name,
          );
    });
  }

  Future<void> _resetConsole() async {
    final l10n = AppLocalizations.of(context)!;
    await _runRustCommand(l10n.actionResetNes, nes_api.resetConsole);
  }

  Future<void> _powerResetConsole() async {
    final l10n = AppLocalizations.of(context)!;
    await _runRustCommand(l10n.actionPowerResetNes, nes_api.powerResetConsole);
  }

  Future<void> _ejectConsole() async {
    final l10n = AppLocalizations.of(context)!;
    await _runRustCommand(l10n.actionEjectNes, () async {
      await nes_api.ejectConsole();
      await ref.read(nesControllerProvider.notifier).refreshRomHash();
    });
  }

  Future<void> _saveState() async {
    await showDialog<void>(
      context: context,
      builder: (_) => const SaveStateDialog(isSaving: true),
    );
  }

  Future<void> _loadState() async {
    await showDialog<void>(
      context: context,
      builder: (_) => const SaveStateDialog(isSaving: false),
    );
  }

  Future<void> _saveToSlot(int slot) async {
    final l10n = AppLocalizations.of(context)!;
    final repository = ref.read(saveStateRepositoryProvider.notifier);
    try {
      final data = await nes_emulation.saveStateToMemory();
      await repository.saveState(slot, data);
      if (mounted) {
        _showSnack(l10n.stateSavedToSlot(slot));
      }
    } catch (e) {
      if (mounted) {
        _showSnack('${l10n.commandFailed('Save to slot $slot')}: $e');
      }
    }
  }

  Future<void> _loadFromSlot(int slot) async {
    final l10n = AppLocalizations.of(context)!;
    final repository = ref.read(saveStateRepositoryProvider.notifier);
    try {
      if (!repository.hasSave(slot)) return;
      final data = await repository.loadState(slot);
      if (data != null) {
        await nes_emulation.loadStateFromMemory(data: data);
        if (mounted) {
          _showSnack(l10n.stateLoadedFromSlot(slot));
        }
      }
    } catch (e) {
      if (mounted) {
        _showSnack('${l10n.commandFailed('Load from slot $slot')}: $e');
      }
    }
  }

  Future<void> _saveToFile() async {
    final l10n = AppLocalizations.of(context)!;
    const XTypeGroup typeGroup = XTypeGroup(
      label: 'Nesium State',
      extensions: <String>['nesium'],
    );

    try {
      String? path;
      final romName = ref.read(nesControllerProvider).romName ?? 'save';
      final suggestedName = '$romName.nesium';

      if (isNativeMobile) {
        final String? directoryPath = await getDirectoryPath(
          confirmButtonText: 'Save here',
        );
        if (directoryPath != null) {
          path = p.join(directoryPath, suggestedName);
        }
      } else {
        final FileSaveLocation? result = await getSaveLocation(
          acceptedTypeGroups: <XTypeGroup>[typeGroup],
          suggestedName: suggestedName,
        );
        path = result?.path;
      }

      if (path != null) {
        await _runRustCommand(
          'Save to file',
          () => nes_emulation.saveState(path: path!),
        );
      }
    } catch (e) {
      if (mounted) {
        _showSnack('${l10n.commandFailed('Save to file')}: $e');
      }
    }
  }

  Future<void> _loadFromFile() async {
    final l10n = AppLocalizations.of(context)!;
    const XTypeGroup typeGroup = XTypeGroup(
      label: 'Nesium State',
      extensions: <String>['nesium'],
    );

    try {
      final XFile? result = await openFile(
        acceptedTypeGroups: <XTypeGroup>[typeGroup],
      );

      if (result != null) {
        await _runRustCommand(
          'Load from file',
          () => nes_emulation.loadState(path: result.path),
        );
      }
    } catch (e) {
      if (mounted) {
        _showSnack('${l10n.commandFailed('Load from file')}: $e');
      }
    }
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
      await nes_pause.togglePause();
      // Intentionally do not show a snackbar on success to avoid noisy UI.
      // Errors are still surfaced below.
    } catch (e) {
      if (!mounted) return;
      final l10n = AppLocalizations.of(context)!;
      _showSnack(l10n.snackPauseFailed('$e'));
    }
  }

  Future<void> _openSettings() async {
    if (!mounted) return;
    await Navigator.of(
      context,
    ).push(MaterialPageRoute<void>(builder: (_) => const SettingsPage()));
  }

  Future<void> _openAbout() async {
    if (!mounted) return;
    await Navigator.of(
      context,
    ).push(MaterialPageRoute<void>(builder: (_) => const AboutPage()));
  }

  Future<void> _openDebugger() async {
    final languageCode = ref.read(appLanguageProvider).languageCode;
    await _desktopWindowManager.openDebuggerWindow(languageCode: languageCode);
  }

  Future<void> _openTools() async {
    final languageCode = ref.read(appLanguageProvider).languageCode;
    await _desktopWindowManager.openToolsWindow(languageCode: languageCode);
  }

  @override
  Widget build(BuildContext context) {
    final NesState state = ref.watch(nesControllerProvider);

    final actions = NesActions(
      openRom: _promptAndLoadRom,
      saveState: _saveState,
      loadState: _loadState,
      saveStateSlot: _saveToSlot,
      loadStateSlot: _loadFromSlot,
      saveStateFile: _saveToFile,
      loadStateFile: _loadFromFile,
      reset: _resetConsole,
      powerReset: _powerResetConsole,
      eject: _ejectConsole,
      togglePause: _togglePause,
      openSettings: _openSettings,
      openAbout: _openAbout,
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
