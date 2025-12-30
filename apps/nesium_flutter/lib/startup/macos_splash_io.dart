import 'dart:convert';
import 'dart:io' show Platform;

import 'package:desktop_multi_window/desktop_multi_window.dart';
import 'package:flutter/foundation.dart' show kDebugMode, kProfileMode;
import 'package:flutter/services.dart';
import 'package:flutter/widgets.dart';

import '../logging/app_logger.dart';

/// macOS-only: hide the native splash overlay after Flutter renders the first frame.
///
/// If this fails and we silently ignore it, the splash may stay forever and the app
/// becomes unusable. We retry briefly and fail fast in debug/profile builds.
Future<void> hideMacOsSplashAfterFirstFrame({
  required List<String> args,
}) async {
  if (!Platform.isMacOS) return;

  // Only the main window has the native splash overlay + channel handler.
  // Secondary windows (desktop_multi_window) should not call this.
  if (!await _isMainWindow(args)) return;

  const splash = MethodChannel('app/splash');

  WidgetsBinding.instance.addPostFrameCallback((_) async {
    const int maxAttempts = 8;
    const Duration retryDelay = Duration(milliseconds: 50);

    for (var attempt = 1; attempt <= maxAttempts; attempt++) {
      try {
        await splash.invokeMethod('hideSplash');
        return;
      } catch (e, st) {
        if (attempt == maxAttempts) {
          if (kDebugMode || kProfileMode) {
            Error.throwWithStackTrace(e, st);
          }
          // Release: give up quietly. Native side has a timeout fallback.
          return;
        }
        await Future<void>.delayed(retryDelay);
      }
    }
  });
}

Future<bool> _isMainWindow(List<String> args) async {
  // desktop_multi_window uses: ["multi_window", windowId, jsonArgs]
  if (args.length >= 3 && args.first == 'multi_window') {
    final route = _parseRoute(args[2]);
    if (route != null) return route == 'main';
    return false;
  }

  // First try the args passed to `main()` (some platforms may pass JSON here).
  for (final arg in args) {
    final route = _parseRoute(arg);
    if (route != null) return route == 'main';
  }

  // Fallback to the desktop_multi_window controller arguments.
  try {
    final controller = await WindowController.fromCurrentEngine();
    final route = _parseRoute(controller.arguments);
    if (route != null) {
      return route == 'main';
    }
  } catch (e, st) {
    logWarning(
      e,
      stackTrace: st,
      message: 'Failed to determine window kind',
      logger: 'macos_splash',
    );
  }
  return true;
}

String? _parseRoute(String? payload) {
  if (payload == null || payload.isEmpty) return null;
  try {
    final data = jsonDecode(payload);
    if (data is Map && data['route'] is String) {
      return data['route'] as String;
    }
  } catch (e, st) {
    logWarning(
      e,
      stackTrace: st,
      message: 'Failed to parse route from payload',
      logger: 'macos_splash',
    );
  }
  return null;
}
