import 'dart:convert';
import 'dart:ui';

import 'package:desktop_multi_window/desktop_multi_window.dart';
import 'package:flutter/foundation.dart';

class DesktopWindowManager {
  const DesktopWindowManager();

  bool get isSupported =>
      !kIsWeb &&
      (defaultTargetPlatform == TargetPlatform.macOS ||
          defaultTargetPlatform == TargetPlatform.linux ||
          defaultTargetPlatform == TargetPlatform.windows);

  Future<void> openDebuggerWindow() async {
    if (!isSupported) return;
    final controller = await DesktopMultiWindow.createWindow(
      jsonEncode({'route': 'debugger'}),
    );
    await _configureAndShow(controller, title: 'Nesium Debugger');
  }

  Future<void> openToolsWindow() async {
    if (!isSupported) return;
    final controller = await DesktopMultiWindow.createWindow(
      jsonEncode({'route': 'tools'}),
    );
    await _configureAndShow(controller, title: 'Nesium Tools');
  }

  Future<void> _configureAndShow(
    WindowController controller, {
    required String title,
  }) async {
    // Provide a reasonable default size/position so the window is visible.
    const frame = Rect.fromLTWH(200, 200, 1100, 800);
    await controller.setFrame(frame);
    await controller.center();
    await controller.setTitle(title);
    await controller.show();
  }
}
