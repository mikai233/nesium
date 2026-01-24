import 'dart:async';
import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/logging/app_logger.dart';
import 'package:nesium_flutter/persistence/app_storage.dart';
import 'package:nesium_flutter/platform/platform_capabilities.dart';
import 'package:nesium_flutter/platform/rust_runtime.dart';
import 'package:nesium_flutter/platform/window_manager_shim.dart';
import 'package:nesium_flutter/startup/launch_args.dart';
import 'package:nesium_flutter/startup/macos_splash.dart';
import 'package:nesium_flutter/windows/window_routing.dart';
import 'package:nesium_flutter/windows/current_window_kind.dart';

import 'app.dart';

Future<void> main(List<String> args) async {
  initLogging();
  final launchArgs = parseLaunchArgs(args);

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

        const windowOptions = WindowOptions(
          center: true,
          skipTaskbar: false,
          titleBarStyle: TitleBarStyle.normal,
        );

        await windowManager.waitUntilReadyToShow(windowOptions, () async {
          await windowManager.show();
          await windowManager.focus();
        });
      }

      await initAppStorage();
      await initRustRuntime();
      final kind = await resolveWindowKind();

      runApp(
        ProviderScope(
          overrides: [
            launchArgsProvider.overrideWithValue(launchArgs),
            currentWindowKindProvider.overrideWithValue(kind),
          ],
          child: NesiumApp(windowKind: kind),
        ),
      );
      unawaited(hideMacOsSplashAfterFirstFrame(args: args));
    },
    (error, stack) {
      logError(error, stackTrace: stack, message: 'Uncaught zone error');
    },
  );
}

LaunchArgs parseLaunchArgs(List<String> args) {
  String? romPath;
  String? luaScriptPath;

  for (var i = 0; i < args.length; i++) {
    final arg = args[i];

    String? takeValue() {
      if (i + 1 >= args.length) return null;
      i += 1;
      return args[i];
    }

    if (arg == '-r') {
      romPath = takeValue() ?? romPath;
      continue;
    }
    if (arg == '--rom') {
      romPath = takeValue() ?? romPath;
      continue;
    }
    if (arg.startsWith('-r=')) {
      romPath = arg.substring('-r='.length);
      continue;
    }
    if (arg.startsWith('--rom=')) {
      romPath = arg.substring('--rom='.length);
      continue;
    }

    if (arg == '-l') {
      luaScriptPath = takeValue() ?? luaScriptPath;
      continue;
    }
    if (arg == '--lua' || arg == '--script') {
      luaScriptPath = takeValue() ?? luaScriptPath;
      continue;
    }
    if (arg.startsWith('-l=')) {
      luaScriptPath = arg.substring('-l='.length);
      continue;
    }
    if (arg.startsWith('--lua=')) {
      luaScriptPath = arg.substring('--lua='.length);
      continue;
    }
    if (arg.startsWith('--script=')) {
      luaScriptPath = arg.substring('--script='.length);
      continue;
    }

    if (!arg.startsWith('-')) {
      final hadRom = romPath != null;
      romPath ??= arg;
      if (hadRom) {
        luaScriptPath ??= arg;
      }
    }
  }

  romPath = romPath?.trim();
  luaScriptPath = luaScriptPath?.trim();

  return LaunchArgs(
    romPath: (romPath == null || romPath.isEmpty) ? null : romPath,
    luaScriptPath: (luaScriptPath == null || luaScriptPath.isEmpty)
        ? null
        : luaScriptPath,
  );
}
