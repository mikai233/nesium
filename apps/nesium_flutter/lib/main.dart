import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:nesium_flutter/src/rust/frb_generated.dart';

import 'app.dart';

Future<void> main(List<String> args) async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init(
    externalLibrary: ExternalLibrary.process(iKnowHowToUseIt: true),
  );

  runApp(ProviderScope(child: NesiumApp()));
}
