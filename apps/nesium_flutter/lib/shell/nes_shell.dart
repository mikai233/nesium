import 'dart:async';
import 'dart:io';

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
import 'package:nesium_flutter/bridge/api/netplay.dart' as nes_netplay;
import 'package:nesium_flutter/bridge/senders/replay.dart';

import '../domain/nes_controller.dart';
import '../domain/nes_input_masks.dart';
import '../domain/nes_state.dart';
import '../domain/pad_button.dart';
import '../domain/emulation_status.dart';
import '../features/controls/input_settings.dart';
import '../domain/gamepad_service.dart';
import '../features/controls/turbo_settings.dart';
import '../features/save_state/auto_save_service.dart';
import '../features/save_state/save_state_dialog.dart';
import '../features/save_state/save_state_repository.dart';
import '../features/settings/emulation_settings.dart';
import '../features/settings/language_settings.dart';
import '../features/settings/settings_page.dart';
import '../features/about/about_page.dart';
import '../features/netplay/netplay_screen.dart';
import '../features/netplay/netplay_state.dart';
import '../l10n/app_localizations.dart';
import '../logging/app_logger.dart';
import '../platform/desktop_window_manager.dart';
import '../platform/platform_capabilities.dart';
import 'desktop_shell.dart';
import 'nes_actions.dart';
import 'mobile_shell.dart';
import '../features/debugger/debugger_panel.dart';
import '../features/debugger/tilemap_viewer.dart';
import '../features/debugger/tile_viewer.dart';
import '../features/debugger/sprite_viewer.dart';
import '../features/debugger/palette_viewer.dart';
import '../features/tools/tools_panel.dart';

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
  StreamSubscription<nes_netplay.NetplayGameEvent>? _netplayEventsSub;
  StreamSubscription<nes_events.EmulationStatusNotification>?
  _emulationStatusSub;
  StreamSubscription<ReplayEventNotification>? _replayEventsSub;
  Future<void> _netplayEventChain = Future.value();

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
      _startEmulationStatusEvents();
      _startReplayEvents();
      _startNetplayEvents();
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
    _netplayEventsSub?.cancel();
    _emulationStatusSub?.cancel();
    _replayEventsSub?.cancel();
    _focusNode.dispose();
    super.dispose();
  }

  void _startRewinding() {
    final emulationSettings = ref.read(emulationSettingsProvider);
    if (!emulationSettings.rewindEnabled) return;

    ref.read(emulationStatusProvider.notifier).setRewinding(true);
    unawaitedLogged(
      nes_emulation.setRewinding(rewinding: true),
      message: 'setRewinding(true)',
      logger: 'nes_shell',
    );
  }

  void _stopRewinding() {
    ref.read(emulationStatusProvider.notifier).setRewinding(false);
    unawaitedLogged(
      nes_emulation.setRewinding(rewinding: false),
      message: 'setRewinding(false)',
      logger: 'nes_shell',
    );
  }

  void _startFastForwarding() {
    ref.read(emulationStatusProvider.notifier).setFastForwarding(true);
    unawaitedLogged(
      nes_emulation.setFastForwarding(fastForwarding: true),
      message: 'setFastForwarding(true)',
      logger: 'nes_shell',
    );
  }

  void _stopFastForwarding() {
    ref.read(emulationStatusProvider.notifier).setFastForwarding(false);
    unawaitedLogged(
      nes_emulation.setFastForwarding(fastForwarding: false),
      message: 'setFastForwarding(false)',
      logger: 'nes_shell',
    );
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

  void _startEmulationStatusEvents() {
    if (!mounted) return;
    if (_emulationStatusSub != null) return;

    _emulationStatusSub = nes_events.emulationStatusStream().listen((event) {
      final controller = ref.read(emulationStatusProvider.notifier);
      controller.setPaused(event.paused);
      controller.setRewinding(event.rewinding);
      controller.setFastForwarding(event.fastForwarding);
    }, onError: (_) {});
  }

  void _startReplayEvents() {
    if (!mounted) return;
    if (_replayEventsSub != null) return;

    _replayEventsSub = nes_events.replayEventStream().listen(
      (event) {
        if (!mounted) return;
        if (event == ReplayEventNotification.quickSave) {
          _quickSaveState();
        } else if (event == ReplayEventNotification.quickLoad) {
          _quickLoadState();
        }
      },
      onError: (e, st) {
        logError(e, stackTrace: st, message: 'Replay event stream error');
      },
    );
  }

  void _startNetplayEvents() {
    if (!mounted) return;
    if (_netplayEventsSub != null) return;

    _netplayEventsSub = nes_netplay.netplayGameEventStream().listen((event) {
      // Serialize netplay event handling to preserve ordering.
      // Freezed `when` callbacks are async but the stream listener does not await them,
      // which can race `SyncState` vs `StartGame` and cause 1–2 frame drift.
      _netplayEventChain = _netplayEventChain
          .then((_) async {
            if (!mounted) return;
            await event.when(
              loadRom: (data) async {
                try {
                  await nes_api.loadRomFromBytes(bytes: data);
                  _pausedByLifecycle = true;
                  // setPaused(true) might fail if emulation not started?
                  // But we just loaded ROM, so it should be fine.
                  await nes_pause.setPaused(paused: true);
                  ref.read(emulationStatusProvider.notifier).setPaused(true);
                  await nes_netplay.netplaySendRomLoaded();
                  if (mounted) _showSnack('Netplay: ROM loaded');
                } catch (e) {
                  if (mounted) _showSnack('Netplay load failed: $e');
                }
              },
              startGame: () async {
                _pausedByLifecycle = false;
                await nes_pause.setPaused(paused: false);
                ref.read(emulationStatusProvider.notifier).setPaused(false);
                if (mounted) _showSnack('Netplay: Game Started');
              },
              pauseSync: (paused) async {
                _pausedByLifecycle = paused;
                await nes_pause.setPaused(paused: paused);
                ref.read(emulationStatusProvider.notifier).setPaused(paused);
                if (mounted) {
                  _showSnack('Netplay: ${paused ? "Paused" : "Resumed"}');
                }
              },
              resetSync: (kind) async {
                if (kind == 1) {
                  await nes_api.powerResetConsole();
                  if (mounted) {
                    _showSnack('Netplay: Power Reset');
                  }
                } else {
                  await nes_api.resetConsole();
                  if (mounted) _showSnack('Netplay: Console Reset');
                }
              },
              syncState: (frame, data) async {
                await nes_emulation.loadStateFromMemory(data: data);
                if (mounted) {
                  _showSnack(
                    'Netplay: State received (Frame $frame, ${data.length} bytes)',
                  );
                }
              },
              playerLeft: (playerIndex) async {
                if (mounted) {
                  _showSnack('Netplay: Player ${playerIndex + 1} left');
                }
              },
              error: (errorCode) async {
                if (mounted) {
                  final msg = _netplayErrorMessage(errorCode);
                  _showSnack('Netplay error: $msg');
                }
              },
              fallbackToRelay: (relayAddr, relayRoomCode, reason) async {
                try {
                  _pausedByLifecycle = true;
                  await nes_pause.setPaused(paused: true);
                  ref.read(emulationStatusProvider.notifier).setPaused(true);
                } catch (_) {}

                if (mounted) {
                  _showSnack(
                    'Netplay: fallback to relay $relayAddr (room $relayRoomCode): $reason',
                  );
                }
              },
            );
          })
          .catchError((Object e, StackTrace st) {
            logWarning(
              e,
              stackTrace: st,
              message: 'netplay event handling failed',
              logger: 'nes_shell',
            );
          });
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
          ref.read(emulationStatusProvider.notifier).setPaused(false);
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
      ref.read(emulationStatusProvider.notifier).setPaused(true);
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

  /// Translate netplay error code to user-friendly message.
  String _netplayErrorMessage(int errorCode) {
    // Error codes from ErrorCode enum in nesium-netproto/src/messages/session.rs
    switch (errorCode) {
      case 0:
        return 'Unknown error';
      case 1:
        return 'Invalid message';
      case 2:
        return 'Room not found';
      case 3:
        return 'Room is full';
      case 4:
        return 'Already in a room';
      case 5:
        return 'Not in a room';
      case 6:
        return 'Permission denied';
      case 7:
        return 'Game already started';
      case 8:
        return 'Invalid state';
      default:
        return 'Error code $errorCode';
    }
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
      withData: true,
      withReadStream: false,
    );
    final file = result?.files.single;
    final path = file?.path;
    var bytes = file?.bytes;

    if (path == null && bytes == null) {
      return;
    }

    // Determine name from path if available, or just fallback
    final name = path != null ? p.basenameWithoutExtension(path) : 'rom';

    if (!mounted) return;
    final l10n = AppLocalizations.of(context)!;
    await _runRustCommand(l10n.actionLoadRom, () async {
      final isNetplay = await nes_netplay.netplayIsConnected();

      if (isNetplay && bytes != null) {
        await nes_api.loadRom(
          path: path ?? '',
        ); // Use path if available for normal load

        // Cache ROM bytes for late joiner sync
        ref
            .read(nesControllerProvider.notifier)
            .updateRomBytes(Uint8List.fromList(bytes));

        // Pause immediately to wait for sync
        _pausedByLifecycle = true;
        await nes_pause.setPaused(paused: true);
        ref.read(emulationStatusProvider.notifier).setPaused(true);

        try {
          // In netplay mode, any player (non-spectator) may broadcast the ROM.
          // Spectators will be rejected and should wait for the server to push `LoadRom`.
          await nes_netplay.netplaySendRom(data: bytes);
        } catch (e) {
          _pausedByLifecycle = false;
          await nes_pause.setPaused(paused: false);
          ref.read(emulationStatusProvider.notifier).setPaused(false);
          rethrow;
        }

        // Host already loaded the ROM locally; confirm immediately so the server can StartGame.
        try {
          await nes_netplay.netplaySendRomLoaded();
        } catch (e) {
          _pausedByLifecycle = false;
          await nes_pause.setPaused(paused: false);
          ref.read(emulationStatusProvider.notifier).setPaused(false);
          rethrow;
        }
        // Host waits for StartGame too.
      } else if (isNetplay && bytes == null && path != null) {
        // Fallback: read bytes from path if FilePicker didn't give them (shouldn't happen with withData: true)
        // Use dart:io if necessary, but withData: true is standard.
      } else {
        await nes_api.loadRom(
          path: path ?? '',
        ); // Use path if available for normal load

        // Cache ROM bytes for potential netplay late joiner sync
        // If bytes not available from picker, try to read from file
        if (bytes != null) {
          ref
              .read(nesControllerProvider.notifier)
              .updateRomBytes(Uint8List.fromList(bytes));
        } else if (path != null) {
          // Read file bytes for caching (non-web platforms)
          try {
            final file = File(path);
            final fileBytes = await file.readAsBytes();
            ref.read(nesControllerProvider.notifier).updateRomBytes(fileBytes);
          } catch (_) {
            // Ignore read errors - just won't have cached bytes
          }
        }
      }

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
    await _runRustCommand(l10n.actionResetNes, () async {
      final isNetplay = await nes_netplay.netplayIsConnected();
      if (isNetplay) {
        await nes_netplay.netplaySendReset(kind: 0);
      }
      await nes_api.resetConsole();
    });
  }

  Future<void> _powerResetConsole() async {
    final l10n = AppLocalizations.of(context)!;
    await _runRustCommand(l10n.actionPowerResetNes, () async {
      final isNetplay = await nes_netplay.netplayIsConnected();
      if (isNetplay) {
        await nes_netplay.netplaySendReset(kind: 1);
      }
      await nes_api.powerResetConsole();
    });
  }

  Future<void> _powerOffConsole() async {
    if (!mounted) return;
    final l10n = AppLocalizations.of(context)!;
    await _runRustCommand(l10n.actionEjectNes, () async {
      // Disconnect from netplay if connected
      final isNetplay = await nes_netplay.netplayIsConnected();
      if (isNetplay) {
        await nes_netplay.netplayDisconnect();
      }
      await nes_api.powerOffConsole();
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

  int _quickSaveSlot() => ref.read(emulationSettingsProvider).quickSaveSlot;

  void _quickSaveState() {
    final slot = _quickSaveSlot();
    unawaited(_saveToSlot(slot));
  }

  void _quickLoadState() {
    final slot = _quickSaveSlot();
    unawaited(_loadFromSlot(slot));
  }

  Future<void> _openAutoSaveDialog() async {
    await showDialog<void>(
      context: context,
      builder: (_) => const SaveStateDialog(isSaving: false, isAutoSave: true),
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

  Future<void> _loadTasMovie() async {
    final result = await FilePicker.platform.pickFiles(
      type: FileType.custom,
      allowedExtensions: ['fm2'],
      withData: true,
      withReadStream: false,
    );
    final file = result?.files.single;
    if (file == null) return;

    final bytes = file.bytes;
    if (bytes == null) return;

    final data = String.fromCharCodes(bytes);

    if (!mounted) return;
    await _runRustCommand('Load TAS Movie', () async {
      await nes_emulation.loadTasMovie(data: data);
    });
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

    if (key == LogicalKeyboardKey.backspace) {
      if (pressed) {
        _startRewinding();
      } else {
        _stopRewinding();
      }
      return KeyEventResult.handled;
    }

    final inputState = ref.read(inputSettingsProvider);
    final netplay = ref.read(netplayProvider);

    if (netplay.isInRoom) {
      // Netplay: route input through the assigned player index
      final pad = netplay.status.playerIndex;
      if (pad >= 4) return KeyEventResult.ignored; // Spectator or invalid

      // Use Port 1 settings for Netplay local player
      final settings = inputState.ports[0]!;
      if (settings.device != InputDevice.keyboard) {
        return KeyEventResult.ignored;
      }

      final action = settings.resolveKeyboardBindings()[key];
      if (action == null) return KeyEventResult.ignored;

      if (pressed) _stopRewinding();

      final input = ref.read(nesInputMasksProvider.notifier);
      _dispatchAction(input, action, pressed, pad: pad);
      return KeyEventResult.handled;
    }

    // Local play: route inputs for all configured players
    var handled = false;
    for (var i = 0; i < 4; i++) {
      final settings = inputState.ports[i]!;
      if (settings.device != InputDevice.keyboard) continue;

      final action = settings.resolveKeyboardBindings()[key];
      if (action == null) continue;

      if (pressed) _stopRewinding();

      final input = ref.read(nesInputMasksProvider.notifier);
      _dispatchAction(input, action, pressed, pad: i);
      handled = true;
    }

    return handled ? KeyEventResult.handled : KeyEventResult.ignored;
  }

  void _dispatchAction(
    NesInputMasksController input,
    KeyboardBindingAction action,
    bool pressed, {
    required int pad,
  }) {
    ref.read(lastInputMethodProvider.notifier).set(InputMethod.keyboard);

    switch (action) {
      case KeyboardBindingAction.up:
        input.setPressed(PadButton.up, pressed, pad: pad);
        break;
      case KeyboardBindingAction.down:
        input.setPressed(PadButton.down, pressed, pad: pad);
        break;
      case KeyboardBindingAction.left:
        input.setPressed(PadButton.left, pressed, pad: pad);
        break;
      case KeyboardBindingAction.right:
        input.setPressed(PadButton.right, pressed, pad: pad);
        break;
      case KeyboardBindingAction.a:
        input.setPressed(PadButton.a, pressed, pad: pad);
        break;
      case KeyboardBindingAction.b:
        input.setPressed(PadButton.b, pressed, pad: pad);
        break;
      case KeyboardBindingAction.select:
        input.setPressed(PadButton.select, pressed, pad: pad);
        break;
      case KeyboardBindingAction.start:
        input.setPressed(PadButton.start, pressed, pad: pad);
        break;
      case KeyboardBindingAction.turboA:
        input.setTurboEnabled(PadButton.a, pressed, pad: pad);
        break;
      case KeyboardBindingAction.turboB:
        input.setTurboEnabled(PadButton.b, pressed, pad: pad);
        break;
      case KeyboardBindingAction.rewind:
        if (pressed) {
          _startRewinding();
        } else {
          _stopRewinding();
        }
        break;
      case KeyboardBindingAction.fastForward:
        if (pressed) {
          _startFastForwarding();
        } else {
          _stopFastForwarding();
        }
        break;
      case KeyboardBindingAction.saveState:
        if (pressed) _quickSaveState();
        break;
      case KeyboardBindingAction.loadState:
        if (pressed) _quickLoadState();
        break;
      case KeyboardBindingAction.pause:
        if (pressed) _togglePause();
        break;
    }
  }

  Future<void> _togglePause() async {
    try {
      _pausedByLifecycle = false;
      await nes_pause.togglePause();
      final paused = await nes_pause.isPaused();
      ref.read(emulationStatusProvider.notifier).setPaused(paused);
      // Send pause sync to other players if connected
      final isNetplay = await nes_netplay.netplayIsConnected();
      if (isNetplay) {
        await nes_netplay.netplaySendPause(paused: paused);
      }
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

  NesActions _buildActions() {
    return NesActions(
      openRom: _promptAndLoadRom,
      saveState: _saveState,
      loadState: _loadState,
      openAutoSave: _openAutoSaveDialog,
      saveStateSlot: _saveToSlot,
      loadStateSlot: _loadFromSlot,
      saveStateFile: _saveToFile,
      loadStateFile: _loadFromFile,
      loadTasMovie: _loadTasMovie,
      reset: _resetConsole,
      powerReset: _powerResetConsole,
      powerOff: _powerOffConsole,
      togglePause: _togglePause,
      setRewinding: (active) => active ? _startRewinding() : _stopRewinding(),
      setFastForwarding: (active) =>
          active ? _startFastForwarding() : _stopFastForwarding(),
      openSettings: _openSettings,
      openAbout: _openAbout,
      openDebugger: _openDebugger,
      openTools: _openTools,
      openTilemapViewer: _openTilemapViewer,
      openTileViewer: _openTileViewer,
      openSpriteViewer: _openSpriteViewer,
      openPaletteViewer: _openPaletteViewer,
      openNetplay: _openNetplay,
    );
  }

  Future<void> _openDebugger() async {
    if (_desktopWindowManager.isSupported) {
      final languageCode = ref.read(appLanguageProvider).languageCode;
      await _desktopWindowManager.openDebuggerWindow(
        languageCode: languageCode,
      );
    } else {
      // Mobile: always use in-app navigation
      if (!mounted) return;
      final l10n = AppLocalizations.of(context)!;
      await Navigator.of(context).push(
        MaterialPageRoute<void>(
          builder: (_) => ProviderScope(
            overrides: [nesActionsProvider.overrideWithValue(_buildActions())],
            child: Scaffold(
              appBar: AppBar(title: Text(l10n.windowDebuggerTitle)),
              body: const DebuggerPanel(),
            ),
          ),
        ),
      );
    }
  }

  Future<void> _openTools() async {
    if (_desktopWindowManager.isSupported) {
      final languageCode = ref.read(appLanguageProvider).languageCode;
      await _desktopWindowManager.openToolsWindow(languageCode: languageCode);
    } else {
      // Mobile: always use in-app navigation (or fallback for desktop)
      if (!mounted) return;
      final l10n = AppLocalizations.of(context)!;
      await Navigator.of(context).push(
        MaterialPageRoute<void>(
          builder: (_) => ProviderScope(
            overrides: [nesActionsProvider.overrideWithValue(_buildActions())],
            child: Scaffold(
              appBar: AppBar(title: Text(l10n.windowToolsTitle)),
              body: const ToolsPanel(),
            ),
          ),
        ),
      );
    }
  }

  Future<void> _openTilemapViewer() async {
    if (_isDesktop) {
      if (_desktopWindowManager.isSupported) {
        final languageCode = ref.read(appLanguageProvider).languageCode;
        await _desktopWindowManager.openTilemapWindow(
          languageCode: languageCode,
        );
      } else {
        // Fallback to in-app navigation for platforms where multi-window is not supported
        if (!mounted) return;
        final l10n = AppLocalizations.of(context)!;
        await Navigator.of(context).push(
          MaterialPageRoute<void>(
            builder: (_) => Scaffold(
              appBar: AppBar(title: Text(l10n.menuTilemapViewer)),
              body: const TilemapViewer(),
            ),
          ),
        );
      }
    } else {
      // Mobile: always use in-app navigation
      if (!mounted) return;
      final l10n = AppLocalizations.of(context)!;
      await Navigator.of(context).push(
        MaterialPageRoute<void>(
          builder: (_) => ProviderScope(
            overrides: [nesActionsProvider.overrideWithValue(_buildActions())],
            child: Scaffold(
              appBar: AppBar(title: Text(l10n.menuTilemapViewer)),
              body: const TilemapViewer(),
            ),
          ),
        ),
      );
    }
  }

  Future<void> _openTileViewer() async {
    if (_isDesktop) {
      if (_desktopWindowManager.isSupported) {
        final languageCode = ref.read(appLanguageProvider).languageCode;
        await _desktopWindowManager.openTileViewerWindow(
          languageCode: languageCode,
        );
      } else {
        // Fallback to in-app navigation for platforms where multi-window is not supported
        if (!mounted) return;
        final l10n = AppLocalizations.of(context)!;
        await Navigator.of(context).push(
          MaterialPageRoute<void>(
            builder: (_) => Scaffold(
              appBar: AppBar(title: Text(l10n.menuTileViewer)),
              body: const TileViewer(),
            ),
          ),
        );
      }
    } else {
      // Mobile: always use in-app navigation
      if (!mounted) return;
      final l10n = AppLocalizations.of(context)!;
      await Navigator.of(context).push(
        MaterialPageRoute<void>(
          builder: (_) => ProviderScope(
            overrides: [nesActionsProvider.overrideWithValue(_buildActions())],
            child: Scaffold(
              appBar: AppBar(title: Text(l10n.menuTileViewer)),
              body: const TileViewer(),
            ),
          ),
        ),
      );
    }
  }

  Future<void> _openSpriteViewer() async {
    if (_isDesktop) {
      if (_desktopWindowManager.isSupported) {
        final languageCode = ref.read(appLanguageProvider).languageCode;
        await _desktopWindowManager.openSpriteViewerWindow(
          languageCode: languageCode,
        );
      } else {
        // Fallback to in-app navigation for platforms where multi-window is not supported
        if (!mounted) return;
        final l10n = AppLocalizations.of(context)!;
        await Navigator.of(context).push(
          MaterialPageRoute<void>(
            builder: (_) => Scaffold(
              appBar: AppBar(title: Text(l10n.menuSpriteViewer)),
              body: const SpriteViewer(),
            ),
          ),
        );
      }
    } else {
      // Mobile: always use in-app navigation
      if (!mounted) return;
      final l10n = AppLocalizations.of(context)!;
      await Navigator.of(context).push(
        MaterialPageRoute<void>(
          builder: (_) => ProviderScope(
            overrides: [nesActionsProvider.overrideWithValue(_buildActions())],
            child: Scaffold(
              appBar: AppBar(title: Text(l10n.menuSpriteViewer)),
              body: const SpriteViewer(),
            ),
          ),
        ),
      );
    }
  }

  Future<void> _openPaletteViewer() async {
    if (_isDesktop) {
      if (_desktopWindowManager.isSupported) {
        final languageCode = ref.read(appLanguageProvider).languageCode;
        await _desktopWindowManager.openPaletteViewerWindow(
          languageCode: languageCode,
        );
      } else {
        if (!mounted) return;
        final l10n = AppLocalizations.of(context)!;
        await Navigator.of(context).push(
          MaterialPageRoute<void>(
            builder: (_) => Scaffold(
              appBar: AppBar(title: Text(l10n.menuPaletteViewer)),
              body: const PaletteViewer(),
            ),
          ),
        );
      }
    } else {
      if (!mounted) return;
      final l10n = AppLocalizations.of(context)!;
      await Navigator.of(context).push(
        MaterialPageRoute<void>(
          builder: (_) => ProviderScope(
            overrides: [nesActionsProvider.overrideWithValue(_buildActions())],
            child: Scaffold(
              appBar: AppBar(title: Text(l10n.menuPaletteViewer)),
              body: const PaletteViewer(),
            ),
          ),
        ),
      );
    }
  }

  @override
  Widget build(BuildContext context) {
    final NesState state = ref.watch(nesControllerProvider);
    ref.watch(autoSaveServiceProvider); // Keep auto-save timer running
    ref.watch(gamepadServiceProvider); // Keep gamepad polling running

    final actions = _buildActions();

    final shell = _isDesktop
        ? DesktopShell(state: state, actions: actions)
        : MobileShell(state: state, actions: actions);

    return ProviderScope(
      overrides: [nesActionsProvider.overrideWithValue(actions)],
      child: Focus(
        focusNode: _focusNode,
        autofocus: true,
        onKeyEvent: _handleKeyEvent,
        child: shell,
      ),
    );
  }

  Future<void> _openNetplay() async {
    if (!mounted) return;
    await Navigator.of(context).push(
      MaterialPageRoute<void>(
        builder: (_) => ProviderScope(
          overrides: [nesActionsProvider.overrideWithValue(_buildActions())],
          child: Scaffold(
            appBar: AppBar(
              title: Text(AppLocalizations.of(context)!.menuNetplay),
            ),
            body: const NetplayScreen(),
          ),
        ),
      ),
    );
  }
}
