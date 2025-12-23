import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:nesium_flutter/platform/rust_runtime.dart';
import 'app.dart';

Future<void> main(List<String> args) async {
  WidgetsFlutterBinding.ensureInitialized();
  await initRustRuntime();
  runApp(const ProviderScope(child: NesiumApp()));
}
