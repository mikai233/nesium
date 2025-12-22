import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:path/path.dart' as p;

import 'package:nesium_flutter/src/rust/frb_generated.dart';
import 'app.dart';

String _libFileName() {
  if (Platform.isWindows) return 'nesium_flutter.dll';
  if (Platform.isMacOS) return 'libnesium_flutter.dylib';
  if (Platform.isLinux) return 'libnesium_flutter.so';
  if (Platform.isAndroid) return 'libnesium_flutter.so';
  throw UnsupportedError('Unsupported platform');
}

ExternalLibrary _openRustLibrary() {
  final name = _libFileName();

  if (Platform.isAndroid) {
    return ExternalLibrary.open(name);
  }

  if (Platform.isMacOS) {
    final exePath =
        Platform.resolvedExecutable; // .../MyApp.app/Contents/MacOS/MyApp
    final contentsDir = p.dirname(p.dirname(exePath)); // .../MyApp.app/Contents
    final libPath = p.join(contentsDir, 'Frameworks', name);
    return ExternalLibrary.open(libPath);
  }

  final exeDir = File(Platform.resolvedExecutable).parent.path;
  final libPath = p.join(exeDir, name);
  return ExternalLibrary.open(libPath);
}

Future<void> main(List<String> args) async {
  WidgetsFlutterBinding.ensureInitialized();

  await RustLib.init(externalLibrary: _openRustLibrary());

  runApp(const ProviderScope(child: NesiumApp()));
}
