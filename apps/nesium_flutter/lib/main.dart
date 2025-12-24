import 'dart:io' show Platform;

import 'package:flutter/foundation.dart' show kIsWeb;
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/platform/rust_runtime.dart';

import 'app.dart';

Future<void> main(List<String> args) async {
  WidgetsFlutterBinding.ensureInitialized();
  await initRustRuntime();

  runApp(const ProviderScope(child: NesiumApp()));

  // macOS-only: hide the native splash overlay after Flutter renders the first frame.
  // Other platforms do not register this channel.
  if (!kIsWeb && Platform.isMacOS) {
    const splash = MethodChannel('app/splash');
    WidgetsBinding.instance.addPostFrameCallback((_) async {
      try {
        await splash.invokeMethod('hideSplash');
      } catch (_) {
        // Ignore if the channel is not available.
      }
    });
  }
}
