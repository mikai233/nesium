import 'dart:io' show Platform;

import 'package:flutter/foundation.dart' show kIsWeb, kDebugMode, kProfileMode;
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/platform/rust_runtime.dart';

import 'app.dart';

Future<void> main(List<String> args) async {
  WidgetsFlutterBinding.ensureInitialized();
  await initRustRuntime();

  runApp(const ProviderScope(child: NesiumApp()));

  hideMacOsSplashAfterFirstFrame();
}

/// macOS-only: hide the native splash overlay after Flutter renders the first frame.
///
/// If this fails and we silently ignore it, the splash may stay forever and the app
/// becomes unusable. We retry briefly and fail fast in debug/profile builds.
void hideMacOsSplashAfterFirstFrame() {
  if (kIsWeb || !Platform.isMacOS) return;

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
