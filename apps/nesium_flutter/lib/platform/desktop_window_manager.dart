import 'dart:convert';
import 'dart:io';

import 'package:desktop_multi_window/desktop_multi_window.dart';

import '../logging/app_logger.dart';
import '../persistence/app_storage.dart';
import '../windows/window_routing.dart';
import 'platform_capabilities.dart';

class DesktopWindowManager {
  const DesktopWindowManager();

  // We currently only enable multi-window on Windows and macOS.
  // Linux falls back to in-app navigation (EGL stability issues with
  // desktop_multi_window plugin).
  bool get isSupported =>
      isNativeDesktop && (Platform.isWindows || Platform.isMacOS);

  String? _routeFromArgs(String args) {
    if (args.isEmpty) return null;
    try {
      final data = jsonDecode(args);
      if (data is Map && data['route'] is String) {
        return data['route'] as String;
      }
    } catch (e, st) {
      logWarning(
        e,
        stackTrace: st,
        message: 'Failed to parse route from args',
        logger: 'desktop_window_manager',
      );
    }
    return null;
  }

  Future<WindowController> _openOrCreate(
    WindowKind kind, {
    String? languageCode,
  }) async {
    final currentController = await WindowController.fromCurrentEngine();
    final mainId = currentController.windowId;

    final initialData = appStorage.exportSyncableData();

    final routeOnlyArgs = encodeWindowArguments(
      kind,
      mainWindowId: mainId,
      initialData: initialData,
    );
    final targetRoute = _routeFromArgs(routeOnlyArgs);
    final args = encodeWindowArguments(
      kind,
      languageCode: languageCode,
      mainWindowId: mainId,
      initialData: initialData,
    );

    // First, try to find an existing window with the same arguments.
    final existingWindows = await WindowController.getAll();
    for (final window in existingWindows) {
      if (targetRoute != null &&
          _routeFromArgs(window.arguments) == targetRoute) {
        // Ensure it is visible; show() is idempotent.
        await window.show();
        try {
          // Instruct the remote window to bring itself to front.
          // This is handled in window_message_router_io.dart.
          await window.invokeMethod('focusWindow');
        } catch (e, st) {
          logWarning(
            e,
            stackTrace: st,
            message: 'Failed to focus existing window ${window.windowId}',
            logger: 'desktop_window_manager',
          );
        }
        return window;
      }
    }

    // Otherwise, create a new window.
    final controller = await WindowController.create(
      WindowConfiguration(arguments: args, hiddenAtLaunch: true),
    );
    return controller;
  }

  Future<void> openDebuggerWindow({String? languageCode}) async {
    if (!isSupported) return;
    await _openOrCreate(WindowKind.debugger, languageCode: languageCode);
  }

  Future<void> openToolsWindow({String? languageCode}) async {
    if (!isSupported) return;
    await _openOrCreate(WindowKind.tools, languageCode: languageCode);
  }

  Future<void> openTilemapWindow({String? languageCode}) async {
    if (!isSupported) return;
    await _openOrCreate(WindowKind.tilemap, languageCode: languageCode);
  }

  Future<void> openTileViewerWindow({String? languageCode}) async {
    if (!isSupported) return;
    await _openOrCreate(WindowKind.tileViewer, languageCode: languageCode);
  }

  Future<void> openSpriteViewerWindow({String? languageCode}) async {
    if (!isSupported) return;
    await _openOrCreate(WindowKind.spriteViewer, languageCode: languageCode);
  }

  Future<void> openPaletteViewerWindow({String? languageCode}) async {
    if (!isSupported) return;
    await _openOrCreate(WindowKind.paletteViewer, languageCode: languageCode);
  }

  Future<void> openSettingsWindow({String? languageCode}) async {
    if (!isSupported) return;
    await _openOrCreate(WindowKind.settings, languageCode: languageCode);
  }
}
