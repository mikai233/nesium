import 'dart:convert';
import 'dart:io';

import 'package:desktop_multi_window/desktop_multi_window.dart';

import '../logging/app_logger.dart';
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
    final routeOnlyArgs = encodeWindowArguments(kind);
    final targetRoute = _routeFromArgs(routeOnlyArgs);
    final args = encodeWindowArguments(kind, languageCode: languageCode);

    // First, try to find an existing window with the same arguments.
    final existingWindows = await WindowController.getAll();
    for (final window in existingWindows) {
      if (targetRoute != null &&
          _routeFromArgs(window.arguments) == targetRoute) {
        // Ensure it is visible; show() is idempotent.
        await window.show();
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
    final window = await _openOrCreate(
      WindowKind.debugger,
      languageCode: languageCode,
    );
    await window.invokeMethod<void>('setLanguage', languageCode);
  }

  Future<void> openSettingsWindow({String? languageCode}) async {
    if (!isSupported) return;
    final window = await _openOrCreate(
      WindowKind.settings,
      languageCode: languageCode,
    );
    await window.invokeMethod<void>('setLanguage', languageCode);
  }

  Future<void> openToolsWindow({String? languageCode}) async {
    if (!isSupported) return;
    final window = await _openOrCreate(
      WindowKind.tools,
      languageCode: languageCode,
    );
    await window.invokeMethod<void>('setLanguage', languageCode);
  }

  Future<void> openTilemapWindow({String? languageCode}) async {
    if (!isSupported) return;
    final window = await _openOrCreate(
      WindowKind.tilemap,
      languageCode: languageCode,
    );
    await window.invokeMethod<void>('setLanguage', languageCode);
  }

  Future<void> openTileViewerWindow({String? languageCode}) async {
    if (!isSupported) return;
    final window = await _openOrCreate(
      WindowKind.tileViewer,
      languageCode: languageCode,
    );
    await window.invokeMethod<void>('setLanguage', languageCode);
  }

  Future<void> openSpriteViewerWindow({String? languageCode}) async {
    if (!isSupported) return;
    final window = await _openOrCreate(
      WindowKind.spriteViewer,
      languageCode: languageCode,
    );
    await window.invokeMethod<void>('setLanguage', languageCode);
  }

  Future<void> openPaletteViewerWindow({String? languageCode}) async {
    if (!isSupported) return;
    final window = await _openOrCreate(
      WindowKind.paletteViewer,
      languageCode: languageCode,
    );
    await window.invokeMethod<void>('setLanguage', languageCode);
  }
}
