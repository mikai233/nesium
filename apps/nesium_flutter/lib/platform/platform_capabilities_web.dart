bool get isNativeDesktop => false;

bool get isNativeMobile => false;

/// Web builds may run on phones/tablets, but Flutter's `defaultTargetPlatform`
/// is often the host OS (e.g. macOS/Windows) which makes detection unreliable.
///
/// Expose the option unconditionally; UI/overlay remains opt-in via
/// `InputDevice.virtualController`.
bool get supportsVirtualControls => true;

bool get preferVirtualControlsByDefault => false;

/// Web browsers do not support raw TCP sockets.
bool get supportsTcp => false;

bool get useAndroidNativeGameView => false;
