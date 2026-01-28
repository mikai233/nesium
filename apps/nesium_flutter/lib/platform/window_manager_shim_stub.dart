import 'dart:ui';

mixin class WindowListener {
  void onWindowClose() {}
  void onWindowEvent(String eventName) {}
  void onWindowEnterFullScreen() {}
  void onWindowLeaveFullScreen() {}
}

enum TitleBarStyle { normal, hidden, hiddenInTitleBar }

class WindowOptions {
  final Size? size;
  final bool? center;
  final bool? skipTaskbar;
  final TitleBarStyle? titleBarStyle;
  final bool? titleBarShown;
  final Color? backgroundColor;

  const WindowOptions({
    this.size,
    this.center,
    this.skipTaskbar,
    this.titleBarStyle,
    this.titleBarShown,
    this.backgroundColor,
  });
}

class WindowManager {
  WindowManager._();

  static final WindowManager instance = WindowManager._();

  Future<void> ensureInitialized() async {}

  Future<void> setTitle(String title) async {}

  void addListener(WindowListener listener) {}

  void removeListener(WindowListener listener) {}

  Future<void> waitUntilReadyToShow(
    WindowOptions options,
    VoidCallback callback,
  ) async {
    callback();
  }

  Future<void> show() async {}

  Future<void> focus() async {}

  Future<void> setFullScreen(bool fullScreen) async {}
}

final windowManager = WindowManager.instance;
