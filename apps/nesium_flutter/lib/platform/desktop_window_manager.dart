import 'dart:convert';

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
    await WindowController.create(
      WindowConfiguration(
        arguments: jsonEncode({'route': 'debugger'}),
        hiddenAtLaunch: false,
      ),
    );
  }

  Future<void> openToolsWindow() async {
    if (!isSupported) return;
    await WindowController.create(
      WindowConfiguration(
        arguments: jsonEncode({'route': 'tools'}),
        hiddenAtLaunch: false,
      ),
    );
  }
}
