import 'dart:async';
import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/logging/app_logger.dart';
import 'package:nesium_flutter/persistence/app_storage.dart';
import 'package:nesium_flutter/platform/platform_capabilities.dart';
import 'package:nesium_flutter/platform/rust_runtime.dart';
import 'package:nesium_flutter/platform/window_manager_shim.dart';
import 'package:nesium_flutter/startup/macos_splash.dart';
import 'package:nesium_flutter/windows/window_routing.dart';

import 'app.dart';

Future<void> main(List<String> args) async {
  initLogging();

  await runZonedGuarded(
    () async {
      WidgetsFlutterBinding.ensureInitialized();

      FlutterError.onError = (details) {
        FlutterError.presentError(details);
        logError(
          details.exception,
          stackTrace: details.stack,
          message: 'FlutterError',
        );
      };

      PlatformDispatcher.instance.onError = (error, stack) {
        logError(error, stackTrace: stack, message: 'Uncaught error');
        return true;
      };

      if (isNativeDesktop) {
        await windowManager.ensureInitialized();
      }

      await initAppStorage();
      await initRustRuntime();
      final kind = await resolveWindowKind();

      runApp(ProviderScope(child: NesiumApp(windowKind: kind)));
      unawaited(hideMacOsSplashAfterFirstFrame(args: args));
    },
    (error, stack) {
      logError(error, stackTrace: stack, message: 'Uncaught zone error');
    },
  );
}
