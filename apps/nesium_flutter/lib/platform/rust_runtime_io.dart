import 'dart:io';

import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

import '../src/rust/frb_generated.dart';

import 'package:path/path.dart' as p;

String _libFileName() {
  if (Platform.isWindows) return 'nesium_flutter.dll';
  if (Platform.isMacOS) return 'libnesium_flutter.dylib';
  if (Platform.isLinux) return 'libnesium_flutter.so';
  if (Platform.isAndroid) return 'libnesium_flutter.so';
  if (Platform.isIOS) return 'libnesium_flutter.a';
  throw UnsupportedError('Unsupported platform');
}

ExternalLibrary _openRustLibrary() {
  final name = _libFileName();

  if (Platform.isAndroid) {
    return ExternalLibrary.open(name);
  }

  if (Platform.isMacOS || Platform.isIOS || Platform.isLinux) {
    return ExternalLibrary.process(iKnowHowToUseIt: true);
  }

  final exeDir = File(Platform.resolvedExecutable).parent.path;
  final libPath = p.join(exeDir, name);
  return ExternalLibrary.open(libPath);
}

Future<void> initRustRuntime() async {
  await RustLib.init(externalLibrary: _openRustLibrary());
}
