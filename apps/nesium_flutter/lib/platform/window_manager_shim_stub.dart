mixin class WindowListener {
  void onWindowClose() {}
  void onWindowEvent(String eventName) {}
}

class WindowManager {
  WindowManager._();

  static final WindowManager instance = WindowManager._();

  Future<void> ensureInitialized() async {}

  Future<void> setTitle(String title) async {}

  void addListener(WindowListener listener) {}

  void removeListener(WindowListener listener) {}
}

final windowManager = WindowManager.instance;
