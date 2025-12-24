import 'package:desktop_multi_window/desktop_multi_window.dart';
import 'package:flutter/foundation.dart';

import '../windows/window_routing.dart';

class DesktopWindowManager {
  const DesktopWindowManager();

  bool get isSupported =>
      !kIsWeb &&
      (defaultTargetPlatform == TargetPlatform.macOS ||
          defaultTargetPlatform == TargetPlatform.linux ||
          defaultTargetPlatform == TargetPlatform.windows);

  Future<WindowController> _openOrCreate(WindowKind kind) async {
    final args = encodeWindowArguments(kind);

    // First, try to find an existing window with the same arguments.
    final existingWindows = await WindowController.getAll();
    for (final window in existingWindows) {
      if (window.arguments == args) {
        // Ensure it is visible; show() is idempotent.
        await window.show();
        return window;
      }
    }

    // Otherwise, create a new window.
    final controller = await WindowController.create(
      WindowConfiguration(arguments: args, hiddenAtLaunch: false),
    );
    return controller;
  }

  Future<void> openDebuggerWindow() async {
    if (!isSupported) return;
    await _openOrCreate(WindowKind.debugger);
  }

  Future<void> openToolsWindow() async {
    if (!isSupported) return;
    await _openOrCreate(WindowKind.tools);
  }
}
