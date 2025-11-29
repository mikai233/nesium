import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';
import 'package:nesium_flutter/src/rust/frb_generated.dart';

import 'app.dart';

void main(List<String> args) async {
  WidgetsFlutterBinding.ensureInitialized();
  await RustLib.init(
    externalLibrary: ExternalLibrary.process(iKnowHowToUseIt: true),
  );

  String? windowKind;
  if (args.isNotEmpty) {
    // desktop_multi_window passes: ["multi_window", windowId, payloadJson]
    final payload = (args.length >= 3 && args.first == 'multi_window')
        ? args[2]
        : args.last;

    try {
      final data = jsonDecode(payload);
      if (data is Map && data['route'] is String) {
        windowKind = data['route'] as String;
      }
    } catch (_) {
      windowKind = null;
    }
  }

  runApp(ProviderScope(child: NesiumApp(windowKind: windowKind)));
}
